// Copyright ⓒ 2019 David Kellum
//
// The `Decoder` implemented here was originally derived from
// `tendil::stream::LossyDecoder` (version 0.4.1) source as found:
//
// https://github.com/servo/tendril
// Copyright ⓒ 2015 Keegan McAllister
// Licensed under the Apache license, v2.0, or the MIT license

use std::borrow::Cow;

use encoding_rs::{self as enc, DecoderResult};

use tendril::{Tendril, TendrilSink, Atomicity, NonAtomic};
use tendril::fmt as form;
use tendril::stream::Utf8LossyDecoder;

/// A `TendrilSink` adaptor that takes bytes, decodes them as the given
/// character encoding, lossily replace ill-formed byte sequences with U+FFFD
/// replacement characters, and emits Unicode (`StrTendril`).
///
/// This allocates new tendrils for encodings other than UTF-8.
pub struct Decoder<Sink, A=NonAtomic>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    inner: DecoderInner<Sink, A>,
}

enum DecoderInner<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    Utf8(Utf8LossyDecoder<Sink, A>),
    EncodingRs(enc::Decoder, Sink),
}

impl<Sink, A> Decoder<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    /// Create a new incremental decoder using the encoding_rs crate.
    pub fn new(encoding: &'static enc::Encoding, sink: Sink)
        -> Self
    {
        if encoding == enc::UTF_8 {
            return Self::utf8(sink);
        }
        Self {
            inner: DecoderInner::EncodingRs(encoding.new_decoder(), sink)
        }
    }

    /// Create a new incremental decoder for the UTF-8 encoding.
    ///
    /// This is useful for content that is known at run-time to be UTF-8
    /// (whereas `Utf8LossyDecoder` requires knowning at compile-time.)
    pub fn utf8(sink: Sink) -> Decoder<Sink, A> {
        Decoder {
            inner: DecoderInner::Utf8(Utf8LossyDecoder::new(sink))
        }
    }

    /// Give a reference to the inner sink.
    /// FIXME: unused
    pub fn inner_sink(&self) -> &Sink {
        match self.inner {
            DecoderInner::Utf8(ref utf8) => &utf8.inner_sink,
            DecoderInner::EncodingRs(_, ref inner_sink) => inner_sink,
        }
    }

    /// Give a mutable reference to the inner sink.
    /// FIXME: unused
    pub fn inner_sink_mut(&mut self) -> &mut Sink {
        match self.inner {
            DecoderInner::Utf8(ref mut utf8) => &mut utf8.inner_sink,
            DecoderInner::EncodingRs(_, ref mut inner_sink) => inner_sink,
        }
    }
}

impl<Sink, A> TendrilSink<form::Bytes, A> for Decoder<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    type Output = Sink::Output;

    fn process(&mut self, t: Tendril<form::Bytes, A>) {
        match self.inner {
            DecoderInner::Utf8(ref mut utf8) => return utf8.process(t),
            DecoderInner::EncodingRs(ref mut decoder, ref mut sink) => {
                if t.is_empty() {
                    return;
                }
                decode_to_sink(t, decoder, sink, false);
            },
        }
    }

    fn error(&mut self, desc: Cow<'static, str>) {
        match self.inner {
            DecoderInner::Utf8(ref mut utf8) => utf8.error(desc),
            DecoderInner::EncodingRs(_, ref mut sink) => sink.error(desc),
        }
    }

    fn finish(self) -> Sink::Output {
        match self.inner {
            DecoderInner::Utf8(utf8) => return utf8.finish(),
            DecoderInner::EncodingRs(mut decoder, mut sink) => {
                decode_to_sink(Tendril::new(), &mut decoder, &mut sink, true);
                sink.finish()
            }
        }
    }
}

fn decode_to_sink<Sink, A>(
    mut t: Tendril<form::Bytes, A>,
    decoder: &mut enc::Decoder,
    sink: &mut Sink,
    last: bool)
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    loop {
        let mut outt = <Tendril<form::Bytes, A>>::new();
        let max_len = decoder
            .max_utf8_buffer_length_without_replacement(t.len())
            .unwrap_or(8192);
        unsafe {
            outt.push_uninitialized(std::cmp::min(max_len as u32, 8192));
        }
        let (result, bytes_read, bytes_written) =
            decoder.decode_to_utf8_without_replacement(&t, &mut outt, last);
        if bytes_written > 0 {
            sink.process(unsafe {
                outt.subtendril(0, bytes_written as u32)
                    .reinterpret_without_validating()
            });
        }
        match result {
            DecoderResult::InputEmpty => return,
            DecoderResult::OutputFull => {},
            DecoderResult::Malformed(_, _) => {
                sink.error(Cow::Borrowed("invalid sequence"));
                sink.process("\u{FFFD}".into());
            },
        }
        t.pop_front(bytes_read as u32);
        if t.is_empty() {
            return;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tendril::SliceExt;

    struct Accumulate<A>
        where A: Atomicity
    {
        tendrils: Vec<Tendril<form::UTF8, A>>,
        errors: Vec<String>,
    }

    impl<A> Accumulate<A>
        where A: Atomicity
    {
        fn new() -> Accumulate<A> {
            Accumulate {
                tendrils: vec![],
                errors: vec![],
            }
        }
    }

    impl<A> TendrilSink<form::UTF8, A> for Accumulate<A>
        where A: Atomicity
    {
        type Output = (Vec<Tendril<form::UTF8, A>>, Vec<String>);

        fn process(&mut self, t: Tendril<form::UTF8, A>) {
            self.tendrils.push(t);
        }

        fn error(&mut self, desc: Cow<'static, str>) {
            self.errors.push(desc.into_owned());
        }

        fn finish(self) -> Self::Output {
            (self.tendrils, self.errors)
        }
    }

    fn check_decode(
        mut decoder: Decoder<Accumulate<NonAtomic>>,
        input: &[&[u8]],
        expected: &str,
        errs: usize)
    {
        for x in input {
            decoder.process(x.to_tendril());
        }
        let (tendrils, errors) = decoder.finish();
        let mut tendril: Tendril<form::UTF8> = Tendril::new();
        for t in tendrils {
            tendril.push_tendril(&t);
        }
        assert_eq!(expected, &*tendril);
        assert_eq!(errs, errors.len());
    }

    pub type Tests = &'static [(&'static [&'static [u8]], &'static str, usize)];

    const UTF_8: Tests = &[
        (&[], "", 0),
        (&[b""], "", 0),
        (&[b"xyz"], "xyz", 0),
        (&[b"x", b"y", b"z"], "xyz", 0),

        (&[b"\xEA\x99\xAE"], "\u{a66e}", 0),
        (&[b"\xEA", b"\x99\xAE"], "\u{a66e}", 0),
        (&[b"\xEA\x99", b"\xAE"], "\u{a66e}", 0),
        (&[b"\xEA", b"\x99", b"\xAE"], "\u{a66e}", 0),
        (&[b"\xEA", b"", b"\x99", b"", b"\xAE"], "\u{a66e}", 0),
        (&[b"", b"\xEA", b"", b"\x99", b"", b"\xAE", b""], "\u{a66e}", 0),

        (&[b"xy\xEA", b"\x99\xAEz"], "xy\u{a66e}z", 0),
        (&[b"xy\xEA", b"\xFF", b"\x99\xAEz"], "xy\u{fffd}\u{fffd}\u{fffd}\u{fffd}z", 4),
        (&[b"xy\xEA\x99", b"\xFFz"], "xy\u{fffd}\u{fffd}z", 2),

        // incomplete char at end of input
        (&[b"\xC0"], "\u{fffd}", 1),
        (&[b"\xEA\x99"], "\u{fffd}", 1),
    ];

    #[test]
    fn decode_utf8_encoding_rs() {
        for &(input, expected, errs) in UTF_8 {
            let decoder = Decoder::new(enc::UTF_8, Accumulate::new());
            check_decode(decoder, input, expected, errs);
        }
    }

    const KOI8_U: Tests = &[
        (&[b"\xfc\xce\xc5\xd2\xc7\xc9\xd1"], "Энергия", 0),
        (&[b"\xfc\xce", b"\xc5\xd2\xc7\xc9\xd1"], "Энергия", 0),
        (&[b"\xfc\xce", b"\xc5\xd2\xc7", b"\xc9\xd1"], "Энергия", 0),
        (&[b"\xfc\xce", b"", b"\xc5\xd2\xc7", b"\xc9\xd1", b""], "Энергия", 0),
    ];

    #[test]
    fn decode_koi8_u_encoding_rs() {
        for &(input, expected, errs) in KOI8_U {
            let decoder = Decoder::new(enc::KOI8_U, Accumulate::new());
            check_decode(decoder, input, expected, errs);
        }
    }

    const WINDOWS_949: Tests = &[
        (&[], "", 0),
        (&[b""], "", 0),
        (&[b"\xbe\xc8\xb3\xe7"], "안녕", 0),
        (&[b"\xbe", b"\xc8\xb3\xe7"], "안녕", 0),
        (&[b"\xbe", b"", b"\xc8\xb3\xe7"], "안녕", 0),
        (&[b"\xbe\xc8\xb3\xe7\xc7\xcf\xbc\xbc\xbf\xe4"], "안녕하세요", 0),
        (&[b"\xbe\xc8\xb3\xe7\xc7"], "안녕\u{fffd}", 1),

        (&[b"\xbe", b"", b"\xc8\xb3"], "안\u{fffd}", 1),
        (&[b"\xbe\x28\xb3\xe7"], "\u{fffd}(녕", 1),
    ];

    #[test]
    fn decode_windows_949_encoding_rs() {
        for &(input, expected, errs) in WINDOWS_949 {
            let decoder = Decoder::new(enc::EUC_KR, Accumulate::new());
            check_decode(decoder, input, expected, errs);
        }
    }
}
