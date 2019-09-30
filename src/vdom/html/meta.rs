//! Compile-time static metadata for html elements and attributes

#![allow(unused)]

/// HTML tag constants
pub mod t {
    use crate::vdom::{LocalName, lname};

    pub const A: LocalName = lname!("a");
    pub const BODY: LocalName = lname!("body");
    pub const HEAD: LocalName = lname!("head");
    pub const META: LocalName = lname!("meta");
}

/// HTML attribute constants
pub mod a {
    use crate::vdom::{LocalName, lname};

    pub const CLASS:          LocalName = lname!("class");
    pub const CHARSET:        LocalName = lname!("charset");
    pub const CONTENT:        LocalName = lname!("content");
    pub const HREF:           LocalName = lname!("href");
    pub const HTTP_EQUIV:     LocalName = lname!("http-equiv");
    pub const ID:             LocalName = lname!("id");
    pub const ITEMREF:        LocalName = lname!("itemref");
    pub const REL:            LocalName = lname!("rel");
    pub const STYLE:          LocalName = lname!("style");
    pub const TARGET:         LocalName = lname!("target");
    pub const TITLE:          LocalName = lname!("title");
}
