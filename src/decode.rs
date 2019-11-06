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
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use encoding_rs as enc;
use enc::DecoderResult;

use tendril::{Tendril, TendrilSink, Atomicity, NonAtomic};
use tendril::fmt as form;
use tendril::stream::Utf8LossyDecoder;

pub struct EncodingHint {
    encodings: HashMap<&'static enc::Encoding, f32>,
    top: Option<&'static enc::Encoding>,
    confidence: f32,
    changed: bool,
}

/// Recommended confidence for any default charset
pub const DEFAULT_CONF: f32       = 0.01;

/// Recommended confidence for a hint from an HTTP content-type header with
/// charset.
pub const HTTP_CTYPE_CONF: f32    = 0.09;

/// Recommended confidence for the sum of all hints from within an HTML head,
/// in meta elements.
pub const HTML_META_CONF: f32     = 0.20;

impl EncodingHint {
    /// Construct new, empty EncodingHint.
    pub fn new() -> EncodingHint {
        EncodingHint {
            encodings: HashMap::new(),
            top: None,
            confidence: 0.0,
            changed: false,
        }
    }

    /// Construct a new Encoding hint with the specified encoding at
    /// `DEFAULT_CONF`, wrapped for sharing.
    pub fn shared_default(enc: &'static enc::Encoding)
        -> Rc<RefCell<EncodingHint>>
    {
        let mut eh = EncodingHint::new();
        eh.add_hint(enc, DEFAULT_CONF);
        eh.clear_changed();
        Rc::new(RefCell::new(eh))
    }

    /// Add a hint for an encoding, by label ASCII-intepreted bytes, and some
    /// positive confidence value.  If no encoding (or applicable replacement)
    /// is found for the specified label, returns false.  Return true if an
    /// encoding is found _and_ this hint changes the top confidence encoding.
    pub fn add_label_hint<L>(&mut self, enc: L, confidence: f32)
        -> bool
        where L: AsRef<[u8]>
    {
        if let Some(enc) = enc::Encoding::for_label(enc.as_ref()) {
            self.add_hint(enc, confidence)
        } else {
            false
        }
    }

    /// Add a hint for the specified encoding and some positive confidence
    /// value. Return true if this hint changes the top most confident
    /// encoding.
    pub fn add_hint(&mut self, enc: &'static enc::Encoding, confidence: f32)
        -> bool
    {
        assert!(confidence > 0.0);

        let new_conf = *(
            self.encodings.entry(enc)
                .and_modify(|c| *c += confidence)
                .or_insert(confidence)
        );

        if new_conf > self.confidence {
            self.confidence = new_conf;
            if self.top == Some(enc) {
                false
            } else {
                self.top = Some(enc);
                self.changed = true;
                true
            }
        } else {
            false
        }
    }

    /// Return the top (most confident) encoding, if at least one encoding has
    /// been hinted.
    pub fn top(&self) -> Option<&'static enc::Encoding> {
        self.top
    }

    /// Return the summed confidence value for the top (most confident)
    /// encoding. Returns 0.0 if no hint has been provided.
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// Return the latest top encoding if the top has changed since
    /// construction or the last call to `clear_changed`.
    pub fn changed(&self) -> Option<&'static enc::Encoding> {
        if self.changed {
            self.top
        } else {
            None
        }
    }

    /// Clear changed state.
    pub fn clear_changed(&mut self) {
        self.changed = false
    }
}

impl Default for EncodingHint {
    fn default() -> EncodingHint {
        EncodingHint::new()
    }
}

/// A `TendrilSink` adaptor that takes bytes, decodes them as the given
/// character encoding, lossily replace ill-formed byte sequences with U+FFFD
/// replacement characters, and emits Unicode (`StrTendril`).
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
    Other(enc::Decoder, Sink),
}

impl<Sink, A> Decoder<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    pub fn new(encoding: &'static enc::Encoding, sink: Sink) -> Self {

        let mode = if encoding == enc::UTF_8 {
            Mode::Utf8(Utf8LossyDecoder::new(sink))
        } else {
            Mode::Other(encoding.new_decoder(), sink)
        };

        Decoder { mode }
    }

    /// Give a reference to the inner sink.
    /// FIXME: unused
    pub fn inner_sink(&self) -> &Sink {
        match self.mode {
            Mode::Utf8(ref utf8) => &utf8.inner_sink,
            Mode::Other(_, ref inner_sink) => inner_sink,
        }
    }

    /// Give a mutable reference to the inner sink.
    /// FIXME: unused
    pub fn inner_sink_mut(&mut self) -> &mut Sink {
        match self.mode {
            Mode::Utf8(ref mut utf8) => &mut utf8.inner_sink,
            Mode::Other(_, ref mut inner_sink) => inner_sink,
        }
    }
}

impl<Sink, A> TendrilSink<form::Bytes, A> for Decoder<Sink, A>
    where Sink: TendrilSink<form::UTF8, A>, A: Atomicity
{
    type Output = Sink::Output;

    fn process(&mut self, t: Tendril<form::Bytes, A>) {
        match self.mode {
            Mode::Utf8(ref mut utf8) => utf8.process(t),
            Mode::Other(ref mut decoder, ref mut sink) => {
                if t.is_empty() {
                    return;
                }
                decode_to_sink(t, decoder, sink, false);
            },
        }
    }

    fn error(&mut self, desc: Cow<'static, str>) {
        match self.mode {
            Mode::Utf8(ref mut utf8) => utf8.error(desc),
            Mode::Other(_, ref mut sink) => sink.error(desc),
        }
    }

    fn finish(self) -> Sink::Output {
        match self.mode {
            Mode::Utf8(utf8) => utf8.finish(),
            Mode::Other(mut decoder, mut sink) => {
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

    #[test]
    fn encoding_hint() {
        let mut encs = EncodingHint::new();
        assert!( encs.add_label_hint("LATIN1",     0.3));
        assert!(!encs.add_label_hint("iso-8859-1", 0.4));
        assert!(!encs.add_label_hint("utf-8",      0.5));
        assert_eq!(
            "windows-1252", encs.top().unwrap().name(),
            "desired replacement for first two hints"
        );
        assert_eq!(0.3 + 0.4, encs.confidence());
    }
}
