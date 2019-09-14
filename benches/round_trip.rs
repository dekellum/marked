#![warn(rust_2018_idioms)]

#![feature(test)]
extern crate test; // Still required, see rust-lang/rust#55133

use std::default::Default;
use std::io;
use std::fs::File;

use test::Bencher;

use encoding_rs as enc;
use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::rcdom::RcDom;
use html5ever::serialize as rc_serialize;

use prescan;
use prescan::dom::html::Sink;
use prescan::decode::Decoder;

#[bench]
fn round_trip_vdom(b: &mut Bencher) {
    b.iter(|| {
        let parser_sink = parse_document(Sink::default(), ParseOpts::default());
        let decoder = Decoder::new(enc::UTF_8, parser_sink);
        let mut fin = sample_file("github-dekellum.html").expect("sample");
        let doc = decoder.read_until(&mut fin).expect("parse");
        let mut out = Vec::with_capacity(273108);
        doc.serialize(&mut out).expect("serialization");
        assert_eq!(out.len(), 273108);
    });
}

#[bench]
fn round_trip_rcdom(b: &mut Bencher) {
    b.iter(|| {
        let parser_sink = parse_document(RcDom::default(), ParseOpts::default());
        let decoder = Decoder::new(enc::UTF_8, parser_sink);
        let mut fin = sample_file("github-dekellum.html").expect("sample");
        let doc = decoder.read_until(&mut fin).expect("parse");
        let mut out = Vec::with_capacity(273108);
        rc_serialize(&mut out, &doc.document, Default::default())
            .expect("serialization");
        assert_eq!(out.len(), 272273);
    });
}

fn sample_file(fname: &str) -> Result<File, io::Error> {
    let root = env!("CARGO_MANIFEST_DIR");
    let fpath = format!("{}/samples/{}", root, fname);
    File::open(fpath)
}