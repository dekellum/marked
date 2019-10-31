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
use markup5ever_rcdom::{SerializableHandle, RcDom};
use html5ever::serialize as rc_serialize;

use prescan;
use prescan::vdom::html::Sink;
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
        let ser_handle: SerializableHandle = doc.document.clone().into();
        rc_serialize(&mut out, &ser_handle, Default::default())
            .expect("serialization");
        assert_eq!(out.len(), 272273);
    });
}

#[bench]
fn text_content(b: &mut Bencher) {
    let parser_sink = parse_document(Sink::default(), ParseOpts::default());
    let decoder = Decoder::new(enc::UTF_8, parser_sink);
    let mut fin = sample_file("github-dekellum.html").expect("sample");
    let doc = decoder.read_until(&mut fin).expect("parse");

    b.iter(|| {
        let out = doc.document_node_ref().text();
        assert_eq!(out.unwrap().len32(), 31637);
    });
}

fn sample_file(fname: &str) -> Result<File, io::Error> {
    let root = env!("CARGO_MANIFEST_DIR");
    let fpath = format!("{}/samples/{}", root, fname);
    File::open(fpath)
}
