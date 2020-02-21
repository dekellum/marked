#![warn(rust_2018_idioms)]

#[macro_use] extern crate html5ever;

/// Default parse buffer size used for parsing, including an initial buffer
/// which may be reparsed after new encoding hints are considered.
pub const PARSE_BUFFER_SIZE: u32 = 4 * 1024;

/// Recommended confidence for an initial default encoding.
///
/// Used as a necessary starting encoding, such as UTF-8 (recommended
/// based on current adoption) or `Windows-1252" (for backward compatibility).
pub const DEFAULT_CONF: f32       = 0.01;

/// Recommended confidence for a hint from an HTTP Content-Type header with
/// charset.
pub const HTTP_CTYPE_CONF: f32    = 0.09;

/// Recommended confidence for the sum of all hints from within an HTML head,
/// in meta elements.
pub const HTML_META_CONF: f32     = 0.20;

/// Recommended confidence for hints based on a leading Byte-Order-Mark (BOM)
/// at the start of a document stream.
pub const BOM_CONF: f32           = 0.31;

mod chars;

mod decode;
pub use decode::{
    Decoder, EncodingHint, SharedEncodingHint,
};

mod dom;
pub use dom::{
    filter, html, xml,
    Document, Element, Node, NodeId, NodeRef, Selector,
    Attribute, LocalName, Namespace, QualName, StrTendril,
};

#[cfg(test)]
mod logger;
