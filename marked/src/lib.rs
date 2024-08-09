//! Parsing, filtering, selecting and serializing HTML/XML markup.
//!
//! See the project [../README] for a feature overview.
//!
//! [../README]: https://github.com/dekellum/marked#readme

#![warn(rust_2018_idioms)]

#[macro_use] pub extern crate html5ever;

/// Initial parse buffer size in which encoding hints are considered, possibly
/// triggering reparse.
pub const INITIAL_BUFFER_SIZE: u32 = 4 * 1024;

/// Subsequent parse buffer size used for reading and parsing, after
/// [`INITIAL_BUFFER_SIZE`].
pub const READ_BUFFER_SIZE: u32 = 16 * 1024;

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
    html,
    Document, DocumentType, Element,
    Node, NodeData, NodeId, NodeRef, ProcessingInstruction,
    Descender, Selector,
    Attribute, LocalName, Namespace, QualName, StrTendril,
};

pub use dom::filter;

#[cfg(feature = "xml")]
pub use dom::xml;

#[doc(hidden)]
pub mod logger;
