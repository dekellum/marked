#![warn(rust_2018_idioms)]

use std::default::Default;
use std::io;

use encoding_rs as enc;
use html5ever::driver::ParseOpts;
use html5ever::tree_builder::TreeBuilderOpts;

use prescan::decode::EncodingHint;
use prescan::vdom::html::parse_buffered;

mod logger;
use logger::setup_logger;

fn main() {
    setup_logger(1).unwrap();
    let _opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    }; // FIXME: allow passing to parse_buffered?
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let stdin = io::stdin();
    let doc = parse_buffered(eh, &mut stdin.lock()).expect("parse");

    // check dom.errors?

    doc.serialize(&mut io::stdout())
        .expect("serialization failed");
}
