#![warn(rust_2018_idioms)]

use std::default::Default;
use std::io;

use encoding_rs as enc;

use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::tree_builder::TreeBuilderOpts;

use prescan;

use prescan::dom::html::Sink;
use prescan::decode::Decoder;

fn main() {
   let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let parser_sink = parse_document(Sink::default(), opts);

    // Decoders are "Sink adaptors"—like the Parser, they also impl trait
    // TendrilSink.
    let decoder = Decoder::new(enc::UTF_8, parser_sink);
    // (or enc::WINDOWS_1252, etc.)

    let stdin = io::stdin();
    let doc = decoder.read_until(&mut stdin.lock()).expect("parse");

    // check dom.errors?

    doc.serialize(&mut io::stdout())
        .expect("serialization failed");
}
