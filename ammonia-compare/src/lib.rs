#![warn(rust_2018_idioms)]
#![feature(test)]
#![cfg(test)]

extern crate test; // Still required, see rust-lang/rust#55133

use std::default::Default;
use std::io;
use std::io::Read;
use std::fs::File;

use test::Bencher;

use marked::{
    chain_filters, filter, filter::Action,
    html::{a, t, parse_utf8_fragment},
    NodeData, NodeRef
};

// Detach tags for which content should not be retained
//
// This includes:
// * The Ammonia::Builder::clean_content_tags default: STYLE and
//   SCRIPT, despite conflicting rustdoc.
// * DOMs like *5ever rcdom have specific handling for TEMPLATE,
//   so we detach that as well.
pub fn detach_clean_content_tags(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(ref elm) = data.as_element() {
        match elm.name.local {
            t::STYLE | t::SCRIPT | t::TEMPLATE => return Action::Detach,
            _ => ()
        }
    }
    Action::Continue
}

// Fold elements that are not in the default Ammonia::Builder::tags whitelist.
pub fn fold_non_whitelist_tags(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(ref elm) = data.as_element() {
        match elm.name.local {
            // The default Ammonia::Builder::tags whitelist should be kept
            t::A | t::ABBR | t::ACRONYM | t::AREA | t::ARTICLE | t::ASIDE |
            t::B | t::BDI | t::BDO | t::BLOCKQUOTE | t::BR | t::CAPTION |
            t::CENTER | t::CITE | t::CODE | t::COL | t::COLGROUP | t::DATA |
            t::DD | t::DEL | t::DETAILS | t::DFN | t::DIV | t::DL | t::DT |
            t::EM | t::FIGCAPTION | t::FIGURE | t::FOOTER |
            t::H1 | t::H2 | t::H3 | t::H4 | t::H5 | t::H6 |
            t::HEADER | t::HGROUP | t::HR | t::I | t::IMG | t::INS | t::KBD |
            t::LI | t::MAP | t::MARK | t::NAV | t::OL | t::P | t::PRE | t::Q |
            t::RP | t::RT | t::RTC | t::RUBY | t::S | t::SAMP | t::SMALL |
            t::SPAN | t::STRIKE | t::STRONG | t::SUB | t::SUMMARY | t::SUP |
            t::TABLE | t::TBODY | t::TD | t::TH | t::THEAD | t::TIME | t::TR |
            t::TT | t::U | t::UL | t::VAR | t::WBR
                => (),
            _   => return Action::Fold,
        }
    }
    Action::Continue
}

// Set the `<a>` `rel` attribute based on default Ammonia::Builder::link_rel
//
// Ammonia removes the rel attribute before adding this at end of attributes,
// so do the same here.
fn link_rel(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(elm) = data.as_element_mut() {
        if elm.is_elem(t::A) {
            // Ensure one rel attribute at end by removing first
            elm.remove_attr(a::REL);
            elm.set_attr(a::REL, "noopener noreferrer");
        }
    }
    Action::Continue
}

#[bench]
fn b40_marked_parse_only(b: &mut Bencher) {

    let mut frag = String::new();
    sample_file("github-dekellum-frag.html")
        .expect("sample_file")
        .read_to_string(&mut frag)
        .expect("read_to_string");
    let frag = frag.trim();
    b.iter(|| {
        parse_utf8_fragment(frag.as_bytes());
    });
}

#[bench]
fn b41_marked_clean(b: &mut Bencher) {

    let mut frag = String::new();
    sample_file("github-dekellum-frag.html")
        .expect("sample_file")
        .read_to_string(&mut frag)
        .expect("read_to_string");
    let frag = frag.trim();
    b.iter(|| {
        let mut doc = parse_utf8_fragment(frag.as_bytes());
        doc.filter_breadth(chain_filters!(
            detach_clean_content_tags,
            filter::detach_comments,
            filter::detach_pis,
        ));

        doc.filter(chain_filters!(
            fold_non_whitelist_tags,
            // This is sufficient for this sample (with link_rel) but isn't the
            // exact same config as Ammonia defaults:
            filter::retain_basic_attributes,
            link_rel
        ));

        let out = doc.to_string();
        assert_eq!(out.len(), 52062, "[[[{}]]]", out);
    });
}

#[bench]
fn b42_ammonia_clean(b: &mut Bencher) {
    let mut frag = String::new();
    sample_file("github-dekellum-frag.html")
        .expect("sample_file")
        .read_to_string(&mut frag)
        .expect("read_to_string");
    let frag = frag.trim();
    let amm = ammonia::Builder::default();
    b.iter(|| {
        let doc = amm.clean(&frag);
        let out = doc.to_string();
        assert_eq!(out.len(), 52062, "[[[{}]]]", out);
    });
}

fn sample_file(fname: &str) -> Result<File, io::Error> {
    let root = env!("CARGO_MANIFEST_DIR");
    let fpath = format!("{}/samples/{}", root, fname);
    File::open(fpath)
}
