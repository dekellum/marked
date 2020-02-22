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

use marked;
use marked::{Decoder, EncodingHint};
use marked::html::parse_buffered;

#[bench]
fn b00_round_trip_rcdom(b: &mut Bencher) {
    b.iter(|| {
        let parser_sink =
            parse_document(RcDom::default(), ParseOpts::default());
        let decoder = Decoder::new(enc::UTF_8, parser_sink);
        let mut fin = sample_file("github-dekellum.html")
            .expect("sample_file");
        let doc = decoder.read_to_end(&mut fin).expect("parse");
        let mut out = Vec::with_capacity(273108);
        let ser_handle: SerializableHandle = doc.document.clone().into();
        rc_serialize(&mut out, &ser_handle, Default::default())
            .expect("serialization");
        assert_eq!(out.len(), 272273);
    });
}

#[bench]
fn b01_round_trip_marked(b: &mut Bencher) {
    b.iter(|| {
        let mut fin = sample_file("github-dekellum.html")
            .expect("sample_file");
        let eh = EncodingHint::shared_default(enc::UTF_8);
        let doc = parse_buffered(eh, &mut fin).expect("parse");
        let mut out = Vec::with_capacity(273108);
        doc.serialize(&mut out).expect("serialization");
        assert_eq!(out.len(), 273108);
    });
}

#[bench]
fn b11_decode_eucjp_parse_marked(b: &mut Bencher) {
    b.iter(|| {
        let mut fin = sample_file("matsunami_eucjp_meta.html")
            .expect("sample_file");
        let eh = EncodingHint::shared_default(enc::UTF_8);
        parse_buffered(eh, &mut fin).expect("parse");
    });
}

#[bench]
fn b12_decode_windows1251_parse_marked(b: &mut Bencher) {
    b.iter(|| {
        let mut fin = sample_file("russez_windows1251_meta.html")
            .expect("sample_file");
        let eh = EncodingHint::shared_default(enc::UTF_8);
        parse_buffered(eh, &mut fin).expect("parse");
    });
}

#[bench]
fn b13_utf8_parse_marked(b: &mut Bencher) {
    b.iter(|| {
        let mut fin = sample_file("github-dekellum.html")
            .expect("sample_file");
        let eh = EncodingHint::shared_default(enc::UTF_8);
        parse_buffered(eh, &mut fin).expect("parse");
    });
}

#[bench]
fn b20_text_content(b: &mut Bencher) {
    let mut fin = sample_file("github-dekellum.html")
        .expect("sample_file");
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let doc = parse_buffered(eh, &mut fin).expect("parse");

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
