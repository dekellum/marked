#![warn(rust_2018_idioms)]
#![feature(test)]

extern crate test; // Still required, see rust-lang/rust#55133

use std::default::Default;
use std::io;
use std::fs::File;

use test::Bencher;

use encoding_rs as enc;
use html5ever::{ns, namespace_url};

use marked;
use marked::Attribute;
use marked::QualName;
use marked::EncodingHint;
use marked::NodeData;
use marked::NodeRef;
use marked::chain_filters;
use marked::filter;
use marked::filter::Action;
use marked::html::parse_buffered;
use marked::html::{TAG_META, a, t};

fn link_rel(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(elm) = data.as_element_mut() {
        if elm.is_elem(t::A) {
            let mut found = false;
            for a in elm.attrs.iter_mut() {
                if a.name.local == a::REL {
                    // FIXME: Delete here so we can always add at end of attributes?
                    a.value = "noopener noreferrer".into();
                    found = true;
                }
            }
            if !found {
                elm.attrs.push(Attribute {
                    name: QualName::new(None, ns!(), a::REL),
                    value: "noopener noreferrer".into()
                });
            }
        }
    }
    Action::Continue
}

fn detach_no_content_elements(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if data.is_elem(t::STYLE) || data.is_elem(t::SCRIPT) {
        Action::Detach
    } else {
        Action::Continue
    }
}

/// Fold known banned elements
/// ([`TagMeta::is_banned`](crate::html::TagMeta::is_banned)) and any elements
/// which are unknown. This is more similar to what Ammonia does.
pub fn fold_banned_elements(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(ref mut elm) = data.as_element_mut() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            if tmeta.is_banned() {
                return Action::Fold;
            }
        } else {
            return Action::Fold;
        }
    }
    Action::Continue
}

/// More items folded by Ammonium defaults, which is more fragment oriented.
fn fold_other(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if  data.is_elem(t::FORM) ||
        data.is_elem(t::HEAD) ||
        data.is_elem(t::LINK) ||
        data.is_elem(t::META) ||
        data.is_elem(t::SVG)  ||
        data.is_elem(t::TITLE)
    {
        Action::Fold
    } else {
        Action::Continue
    }
}

#[bench]
fn b40_marked_clean_reader(b: &mut Bencher) {
    b.iter(|| {
        let mut fin = sample_file("github-dekellum.html")
            .expect("sample_file");
        let eh = EncodingHint::shared_default(enc::UTF_8);
        let mut doc = parse_buffered(eh, &mut fin).expect("parse");
        doc.filter(chain_filters!(
            detach_no_content_elements,
            //filter::fold_empty_inline,
            filter::detach_comments,
            filter::detach_pis,
            fold_banned_elements,
            fold_other,
            filter::retain_basic_attributes,
            filter::xmp_to_pre,
            link_rel
        ));
        //doc.filter(filter::text_normalize); // Always use new pass.

        let out = doc.to_string();
        assert_eq!(out.len(), 52493, "[[[{}]]]", out);
    });
}

#[bench]
fn b41_ammonia_clean_reader(b: &mut Bencher) {
    let amm = ammonia::Builder::default();
    b.iter(|| {
        let fin = sample_file("github-dekellum.html")
            .expect("sample_file");
        let doc = amm.clean_from_reader(fin).unwrap();
        let out = doc.to_string();
        assert_eq!(out.len(), 52329);
    });
}

fn sample_file(fname: &str) -> Result<File, io::Error> {
    let root = env!("CARGO_MANIFEST_DIR");
    let fpath = format!("{}/samples/{}", root, fname);
    File::open(fpath)
}
