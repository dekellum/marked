
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use encoding_rs as enc;

use crate::DEFAULT_CONF;

/// A set of confidence-weighted evidence that a text document is in a
/// particular encoding.
#[derive(Debug)]
pub struct EncodingHint {
    encodings: HashMap<&'static enc::Encoding, f32>,
    top: Option<&'static enc::Encoding>,
    confidence: f32,
    errors: u32,
    changed: bool,
}

/// An `EncodingHint` that can be shared between `Decoder` and `Sink`, by
/// reference on the same thread, and internally mutated. The type is neither
/// `Send` nor `Sync`.
pub type SharedEncodingHint = Rc<RefCell<EncodingHint>>;

impl EncodingHint {
    /// Construct new, empty EncodingHint.
    fn new() -> EncodingHint {
        EncodingHint {
            encodings: HashMap::new(),
            top: None,
            confidence: 0.0,
            errors: 0,
            changed: false,
        }
    }

    /// Construct a new Encoding hint with the specified encoding at
    /// [`DEFAULT_CONF`] confidence, wrapped for sharing.
    pub fn shared_default(enc: &'static enc::Encoding) -> SharedEncodingHint {
        let mut eh = EncodingHint::new();
        eh.add_hint(enc, DEFAULT_CONF);
        eh.clear_changed();
        Rc::new(RefCell::new(eh))
    }

    /// Construct a new Encoding hint with the specified encoding and
    /// confidence, wrapped for sharing.
    pub fn shared_with_hint(enc: &'static enc::Encoding, confidence: f32)
        -> SharedEncodingHint
    {
        let mut eh = EncodingHint::new();
        eh.add_hint(enc, confidence);
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

    /// Return true if the given encoding name could be read with _both_ any
    /// current top encoding and from the provided encoding, from the same
    /// source bytes.
    ///
    /// All supported encoding names are ASCII, so any _parsed_ name encoding
    /// hint that would transition from an ASCII-compatible encoding
    /// (e.g. Windows-1252, UTF-8) to an ASCII-incompatible encoding
    /// (e.g. UTF-16, REPLACEMENT), or vice-versa, or to different
    /// ASCII-incompatible encodings (e.g. UTF-16BE to UTF-16LE) is nonsensical
    /// and should be ignored.
    ///
    /// Note that this check should only be applied to text hints in the
    /// document itself, and not applied to hints from Byte-Order-Marks since
    /// they aren't ASCII names, or the HTTP Content-Type header since it isn't
    /// part of the document body.
    pub fn could_read_from(&self, enc: &'static enc::Encoding) -> bool {
        if let Some(t) = self.top {
            if  ( includes_ascii(t) && !includes_ascii(enc)) ||
                (!includes_ascii(t) && t != enc)
            {
                return false;
            }
        }
        true
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

    /// Return the total errors accumulated since construction or the last call
    /// to `clear_errors`.
    pub fn errors(&self) -> u32 {
        self.errors
    }

    /// Increment errors count by one.
    pub fn increment_error(&mut self) {
        self.errors += 1
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

    /// Clear `changed` flag.
    pub fn clear_changed(&mut self) {
        self.changed = false;
    }
    /// Clear `errors` count.
    pub fn clear_errors(&mut self) {
        self.errors = 0;
    }
}

// Could the encoding include an ASCII text encoding name?
// This is slightly more lenient then Encoding::is_ascii_compatible in that it
// grants that ISO-2022-JP _could be_ in ASCII mode.
fn includes_ascii(enc: &'static enc::Encoding) -> bool {
    !(enc == enc::UTF_16BE || enc == enc::UTF_16LE || enc == enc::REPLACEMENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_send<T: Send>() -> bool { true }
    fn is_sync<T: Sync>() -> bool { true }

    #[test]
    fn test_send_sync() {
        assert!(is_send::<EncodingHint>());
        assert!(is_sync::<EncodingHint>());
    }

    // Adapted from static_asserts 1.1.0 `assert_not_impl_any` macro
    // MIT/Apache licensed

    trait AmbiguousIfImpl<A> {
        fn some_f() -> bool { true }
    }
    impl<T: ?Sized> AmbiguousIfImpl<()> for T {}

    #[allow(unused)] struct NotSync;
    impl<T: ?Sized + Sync> AmbiguousIfImpl<NotSync> for T {}

    #[allow(unused)] struct NotSend;
    impl<T: ?Sized + Send> AmbiguousIfImpl<NotSend> for T {}

    #[test]
    fn test_not_send_nor_sync() {
        assert!(<SharedEncodingHint as AmbiguousIfImpl<_>>::some_f());
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

    #[test]
    fn could_read_from() {
        let mut eh = EncodingHint::new();
        eh.add_hint(enc::UTF_8, 0.5);
        assert!( eh.could_read_from(enc::UTF_8));
        assert!( eh.could_read_from(enc::WINDOWS_1252));
        assert!( eh.could_read_from(enc::ISO_2022_JP));
        assert!(!eh.could_read_from(enc::UTF_16LE));
        assert!(!eh.could_read_from(enc::UTF_16BE));
    }

    #[test]
    fn could_read_from_multi_byte() {
        let mut eh = EncodingHint::new();
        eh.add_hint(enc::UTF_16LE, 0.5);
        assert!( eh.could_read_from(enc::UTF_16LE));
        assert!(!eh.could_read_from(enc::UTF_16BE));
        assert!(!eh.could_read_from(enc::ISO_2022_JP));
        assert!(!eh.could_read_from(enc::UTF_8));
    }
}
