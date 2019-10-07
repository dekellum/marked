//! Compile-time static metadata for html elements and attributes

#![allow(unused)]

/// *ML `Namespace` constants
pub mod ns {
    use html5ever::ns;
    use crate::vdom::Namespace;

    pub const HTML:           Namespace = ns!(html);
}

/// HTML tag constants
pub mod t {
    use html5ever::local_name as lname;
    use crate::vdom::LocalName;

    // FIXME: For now this is an incomplete list

    pub const A:              LocalName = lname!("a");
    pub const BODY:           LocalName = lname!("body");
    pub const DIV:            LocalName = lname!("div");
    pub const HEAD:           LocalName = lname!("head");
    pub const HTML:           LocalName = lname!("html");
    pub const META:           LocalName = lname!("meta");
    pub const P:              LocalName = lname!("p");
    pub const STRIKE:         LocalName = lname!("strike");
}

/// HTML attribute constants
pub mod a {
    use html5ever::local_name as lname;
    use crate::vdom::LocalName;

    // FIXME: For now this is an incomplete list and lacks any association with
    // elements

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
