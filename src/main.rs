use std::default::Default;
use std::io;

use encoding_rs::WINDOWS_1252;

use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;
use html5ever::tendril::TendrilSink;
use html5ever::tree_builder::TreeBuilderOpts;
use html5ever::{parse_document, serialize};

use tendril::stream::LossyDecoder;

fn main() {
   let opts = ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let stdin = io::stdin();

    let parser_sink = parse_document(RcDom::default(), opts);

    // Decoders are "Sink adaptors" (e.g. they also implement TendrilSink)
    let decoder = LossyDecoder::new_encoding_rs(WINDOWS_1252, parser_sink);
    let dom = decoder.read_from(&mut stdin.lock())
        .expect("parse");

    serialize(&mut io::stdout(), &dom.document, Default::default())
        .expect("serialization failed");
}