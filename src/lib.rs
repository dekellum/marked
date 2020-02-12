#![warn(rust_2018_idioms)]

#[macro_use] extern crate html5ever;

// Default parse buffer size
const PARSE_BUFFER_SIZE: u32 = 4 * 1024;

mod chars;

mod decode;
pub use decode::{
    Decoder, EncodingHint, SharedEncodingHint,
    DEFAULT_CONF, HTML_META_CONF, HTTP_CTYPE_CONF
};

mod dom;
pub use dom::{
    filter, html, xml,
    Document, Element, Node, NodeId, NodeRef, Selector,
    Attribute, LocalName, Namespace, QualName, StrTendril,
};

#[cfg(test)]
mod logger;
