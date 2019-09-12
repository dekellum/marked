#![warn(rust_2018_idioms)]

use std::default::Default;
use std::io;

use encoding_rs as enc;

use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, serialize};

pub mod decode;

use decode::Decoder;

fn main() {
   let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let parser_sink = parse_document(RcDom::default(), opts);

    // Decoders are "Sink adaptors"—like the Parser, they also impl trait
    // TendrilSink.
    let decoder = Decoder::new(enc::WINDOWS_1252, parser_sink);
    // (or enc::UTF_8, etc.)

    let stdin = io::stdin();
    let dom = decoder.read_until(&mut stdin.lock()).expect("parse");

    // check dom.errors?

    serialize(&mut io::stdout(), &dom.document, Default::default())
        .expect("serialization failed");
}
