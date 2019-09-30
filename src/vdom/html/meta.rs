//! Compile-time static metadata for html elements and attributes

#![allow(unused)]

use std::collections::{HashMap};

use crate::vdom::LocalName;
use lazy_static::lazy_static;

pub struct TagMeta {
    is_empty: bool,
    is_deprecated: bool,
    is_inline: bool,
    is_meta: bool,
    is_banned: bool,
    basic_attrs: Vec<LocalName>,
}

impl TagMeta {
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
    }
    pub fn is_inline(&self) -> bool {
        self.is_inline
    }
    pub fn is_meta(&self) -> bool {
        self.is_meta
    }
    pub fn is_banned(&self) -> bool {
        self.is_banned
    }

    pub fn has_basic_attr(&self, name: &LocalName) -> bool {
        self.basic_attrs.binary_search(name).is_ok()
    }
}

impl Default for TagMeta {
    fn default() -> TagMeta {
        TagMeta {
            is_empty: false,
            is_deprecated: false,
            is_inline: false,
            is_meta: false,
            is_banned: false,
            basic_attrs: vec![],
        }
    }
}

lazy_static! {
    /// A static lookup table for metadata on known HTML tags.
    pub static ref TAG_META: HashMap<LocalName, TagMeta> = init_tag_metadata();
}

fn init_tag_metadata() -> HashMap<LocalName, TagMeta> {
    let mut tag_meta = HashMap::new();

    let mut basic_attrs = vec![
        a::HREF, a::REL, a::ID, a::CLASS, a::STYLE, a::TARGET, a::TITLE,
        a::EXPORTPARTS.clone() // Just for testing
    ];
    basic_attrs.sort();
    basic_attrs.dedup();

    tag_meta.insert(t::A, TagMeta {
        is_inline: true,
        basic_attrs,
        .. TagMeta::default()
    });

    tag_meta
}

pub mod t {
    use crate::vdom::{LocalName, lname};

    /// The <a> anchor tag
    pub const A: LocalName = lname!("a");
}

pub mod a {
    use crate::vdom::{LocalName, lname};

    pub const CLASS:          LocalName = lname!("class");
    pub const HREF:           LocalName = lname!("href");
    pub const ID:             LocalName = lname!("id");
    pub const REL:            LocalName = lname!("rel");
    pub const STYLE:          LocalName = lname!("style");
    pub const TARGET:         LocalName = lname!("target");
    pub const TITLE:          LocalName = lname!("title");

    pub const ITEMREF:        LocalName = lname!("itemref");

    lazy_static::lazy_static! {
        pub static ref EXPORTPARTS: LocalName = "exportparts".into();
    }
}

#[test]
fn lookup_basics() {
    let tmeta = TAG_META.get(&t::A).unwrap();
    assert!(tmeta.is_inline());
    assert!(tmeta.has_basic_attr(&a::TARGET));
    assert!(tmeta.has_basic_attr(&a::EXPORTPARTS));
}
