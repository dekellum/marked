// Copyright © 2019 David Kellum
//
// The `Decoder` implemented here was originally derived from
// `tendril::stream::LossyDecoder` and `tendril::stream::TendrilSink`
// (version 0.4.1) source as found:
//
// https://github.com/servo/tendril
// Copyright © 2015 Keegan McAllister
// Licensed under the Apache license v2.0, or the MIT license

//! Support for streaming charset decoding.

use std::borrow::Cow;
use std::io;

use encoding_rs as enc;
use enc::DecoderResult;

use tendril::{Tendril, TendrilSink, Atomicity, NonAtomic};
use tendril::fmt as form;
use tendril::stream::Utf8LossyDecoder;

mod encoding_hint;

pub use encoding_hint::{
    EncodingHint, SharedEncodingHint,
};

use crate::READ_BUFFER_SIZE;

/// A `TendrilSink` adaptor that takes bytes, decodes them as the given
/// character encoding, while replacing any ill-formed byte sequences with
/// U+FFFD replacement characters, and emits Unicode (`StrTendril`).
///
/// This allocates new tendrils for encodings other than UTF-8.
pub struct Decoder<Sink, A=NonAtomic>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    mode: Mode<Sink, A>,
}

enum Mode<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    Utf8(Utf8LossyDecoder<Sink, A>),
    Other(enc::Decoder, Tendril<form::Bytes, A>, Sink),
}

impl<Sink, A> Decoder<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    pub fn new(encoding: &'static enc::Encoding, sink: Sink) -> Self {

        let mode = if encoding == enc::UTF_8 {
            Mode::Utf8(Utf8LossyDecoder::new(sink))
        } else {
            let mut outt = Tendril::<form::Bytes, A>::new();
            let decoder = encoding.new_decoder();
            unsafe {
                outt.push_uninitialized(READ_BUFFER_SIZE);
            }

            Mode::Other(decoder, outt, sink)
        };

        Decoder { mode }
    }

    /// Return reference to the inner sink.
    pub fn inner_sink(&self) -> &Sink {
        match self.mode {
            Mode::Utf8(ref utf8) => &utf8.inner_sink,
            Mode::Other(_, _, ref inner_sink) => inner_sink,
        }
    }

    /// Read until EOF of stream, processing each buffer, and finish this
    /// decoder. Returns the sink output or any io::Error.
    pub fn read_to_end<R>(mut self, r: &mut R)
        -> Result<Sink::Output, io::Error>
        where Self: Sized, R: io::Read
    {
        let mut buff = Tendril::<form::Bytes, A>::new();
        unsafe {
            buff.push_uninitialized(READ_BUFFER_SIZE);
        }
        loop {
            debug_assert_eq!(buff.len32(), READ_BUFFER_SIZE);
            match r.read(&mut buff) {
                Ok(0) => return Ok(self.finish()),
                Ok(n) => {
                    self.process(buff.subtendril(0, n as u32));
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e)
            }
        } // repeat until EOF of Err
    }
}

impl<Sink, A> TendrilSink<form::Bytes, A> for Decoder<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    type Output = Sink::Output;

    fn process(&mut self, t: Tendril<form::Bytes, A>) {
        match self.mode {
            Mode::Utf8(ref mut utf8) => utf8.process(t),
            Mode::Other(ref mut decoder, ref mut outt, ref mut sink) => {
                if t.is_empty() {
                    return;
                }
                decode_to_sink(t, decoder, outt, sink, false);
            },
        }
    }

    fn error(&mut self, desc: Cow<'static, str>) {
        match self.mode {
            Mode::Utf8(ref mut utf8) => utf8.error(desc),
            Mode::Other(_, _, ref mut sink) => sink.error(desc),
        }
    }

    fn finish(self) -> Sink::Output {
        match self.mode {
            Mode::Utf8(utf8) => utf8.finish(),
            Mode::Other(mut decoder, mut outt, mut sink) => {
                decode_to_sink(
                    Tendril::new(), &mut decoder, &mut outt, &mut sink, true);
                sink.finish()
            }
        }
    }
}

fn decode_to_sink<Sink, A>(
    mut t: Tendril<form::Bytes, A>,
    decoder: &mut enc::Decoder,
    outt: &mut Tendril<form::Bytes, A>,
    sink: &mut Sink,
    last: bool)
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    loop {
        let (result, bytes_read, bytes_written) =
            decoder.decode_to_utf8_without_replacement(&t, outt, last);
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
                // String matched in Sink, don't change
                sink.error(Cow::Borrowed("invalid byte sequence"));
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
mod tests {
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
        (&[b"xy\xEA", b"\xFF", b"\x99\xAEz"],
         "xy\u{fffd}\u{fffd}\u{fffd}\u{fffd}z", 4),
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
