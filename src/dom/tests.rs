use std::{io, io::Read};

use crate::{
    Attribute, Document, Element, Node, QualName, StrTendril,
    dom::NodeData,
    filter, filter::Action,
    html, html::{a, t, TAG_META},
    xml,
};

use crate::chain_filters;
use crate::logger::ensure_logger;
use crate::decode::EncodingHint;

use log::debug;

use encoding_rs as enc;

#[test]
#[cfg(target_pointer_width = "64")]
fn size_of() {
    use std::mem::size_of;
    assert_eq!(size_of::<Node>(), 80);
    assert_eq!(size_of::<NodeData>(), 56);
    assert_eq!(size_of::<Element>(), 48);
    assert_eq!(size_of::<Attribute>(), 40);
    assert_eq!(size_of::<Vec<Attribute>>(), 24);
    assert_eq!(size_of::<QualName>(), 24);
    assert_eq!(size_of::<StrTendril>(), 16);
}

#[test]
fn empty_document() {
    ensure_logger();
    let doc = Document::default();
    assert_eq!(None, doc.root_element_ref(), "no root Element");
    assert_eq!(1, doc.nodes().count(), "one Document node");
}

#[test]
fn one_element() {
    ensure_logger();
    let mut doc = Document::new();
    let element = Node::new_element(
        QualName::new(None, ns!(), "one".into()),
        vec![]
    );
    let id = doc.append_child(Document::DOCUMENT_NODE_ID, element);

    assert!(doc.root_element_ref().is_some(), "pushed root Element");
    assert_eq!(id, doc.root_element_ref().unwrap().id());
    assert_eq!(2, doc.nodes().count(), "root + 1 element");
}

#[test]
fn mixed_text_no_root() {
    ensure_logger();
    let mut doc = Document::new();
    let element = Node::new_element(
        QualName::new(None, ns!(), "one".into()),
        vec![]
    );
    let id = doc.append_child(Document::DOCUMENT_NODE_ID, element);
    doc.append_child(id, Node::new_text("bar"));
    doc.insert_before_sibling(id, Node::new_text("foo"));

    assert!(doc.root_element_ref().is_none());
    assert_eq!(
        doc.text(Document::DOCUMENT_NODE_ID).unwrap().as_ref(),
        "foobar"
    );
}

fn strike_fold_filter(node: &mut Node) -> Action {
    if node.is_elem(t::STRIKE) { Action::Fold } else { Action::Continue }
}

fn strike_remove_filter(node: &mut Node) -> Action {
    if node.is_elem(t::STRIKE) { Action::Detach } else { Action::Continue }
}

#[test]
fn test_fold_filter() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<div>foo <strike><i>bar</i>s</strike> baz</div>"
            .as_bytes()
    );
    doc.filter(strike_fold_filter);
    assert_eq!(
        "<html><head></head><body>\
         <div>foo <i>bar</i>s baz</div>\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_remove_filter() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<div>foo <strike><i>bar</i>s</strike> baz</div>"
            .as_bytes()
    );
    doc.filter(strike_remove_filter);
    assert_eq!(
        "<html><head></head><body>\
         <div>foo  baz</div>\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_filter_chain() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>foo<strike><i>bar</i>s</strike> \n\t baz</div>"
            .as_bytes()
    );

    // just to confirm that closures also work in chain
    let other_filter = |n: &mut Node| {
        if n.is_elem(t::META) { Action::Detach } else { Action::Continue }
    };

    doc.filter(chain_filters!(
        other_filter,
        strike_remove_filter,
        |_n: &mut Node| { Action::Continue }, // in place noop
        filter::text_normalize
    ));

    assert_eq!(
        "<div>foo baz</div>",
        doc.to_string()
    );
}

#[test]
fn test_simple_xml() {
    ensure_logger();
    let doc = xml::parse_utf8(
        "<a>foo <b><c>bar</c></b> baz</a>"
            .as_bytes()
    ).expect("parsed");
    assert_eq!(
        "<a>foo <b><c>bar</c></b> baz</a>",
        doc.to_string()
    );
}

#[test]
fn test_xml_with_decl() {
    ensure_logger();
    let doc = xml::parse_utf8(
r####"
<?xml version="1.0" encoding="UTF-8"?>
<a>foo <b><c>bar</c></b> baz</a>
"####
        .as_bytes()
    ).expect("parsed");
    assert_eq!(
        "<a>foo <b><c>bar</c></b> baz</a>",
        doc.to_string()
    );
}

#[test]
fn test_xmp() {
    ensure_logger();
    let doc = html::parse_utf8_fragment(
        "<div>foo <xmp><i>bar</i></xmp> baz</div>"
            .as_bytes()
    );
    assert_eq!(
        "<div>foo <xmp><i>bar</i></xmp> baz</div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(5, doc.nodes.len() - 2);
}

#[test]
fn test_plaintext() {
    ensure_logger();
    let doc = html::parse_utf8_fragment(
        "<div><plaintext><i>bar baz</div>"
            .as_bytes()
    );
    // Serializer isn't aware that <plaintext> doesn't need end tags, etc.
    assert_eq!(
        "<div><plaintext><i>bar baz</div></plaintext></div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(3, doc.nodes.len() - 2);
}

#[test]
fn test_img_decoding_unknown() {
    ensure_logger();
    // The decoding attribute is unknown to html5ever
    let doc = html::parse_utf8_fragment(
        r##"<img href="foo" decoding="sync"/>"##
            .as_bytes()
    );
    assert_eq!(
        doc.root_element_ref()
            .unwrap()
            .find_child(|n| n.is_elem(t::IMG))
            .expect("<img>")
            .as_element()
            .unwrap()
            .attr(&*a::DECODING) //Note required alt syntax
            .unwrap()
            .as_ref(),
        "sync");

    assert_eq!(
        r##"<div><img href="foo" decoding="sync"></div>"##,
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(2, doc.nodes.len() - 2);
}

#[test]
fn test_text_fragment() {
    ensure_logger();
    let doc = html::parse_utf8_fragment(
        "plain &lt; text".as_bytes()
    );
    assert_eq!(
        "<div>\
         plain &lt; text\
         </div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(2, doc.nodes.len() - 2);

    let text_doc = doc.root_element_ref()
        .unwrap()
        .find_child(|n| n.as_text().is_some())
        .unwrap()
        .deep_clone();

    debug!("text doc nodes:\n{:?}", text_doc);

    assert!(text_doc.root_element().is_none());
}

#[test]
fn test_empty_tag() {
    ensure_logger();
    let doc = html::parse_utf8_fragment(
        "plain<wbr>text".as_bytes()
    );
    assert_eq!(
        "<div>\
         plain<wbr>text\
         </div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(4, doc.nodes.len() - 2);
}

#[test]
fn test_shallow_fragment() {
    ensure_logger();
    let doc = html::parse_utf8_fragment(
        "<b>b</b> text <i>i</i>".as_bytes()
    );
    assert_eq!(
        "<div>\
         <b>b</b> text <i>i</i>\
         </div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(6, doc.nodes.len() - 2);
}

#[test]
fn test_inline_fragment() {
    ensure_logger();

    // An single inline element such as <i> will not be used as the root
    let doc = html::parse_utf8_fragment(
        "<i>text</i>".as_bytes()
    );
    assert_eq!(
        "<div>\
         <i>text</i>\
         </div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(3, doc.nodes.len() - 2);
}

#[test]
fn test_empty_fragment() {
    ensure_logger();
    let doc = html::parse_utf8_fragment("".as_bytes());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!("<div></div>", doc.to_string());
}

#[test]
fn test_deep_clone() {
    ensure_logger();
    let doc = html::parse_utf8(
        "<div>foo <a href=\"link\"><i>bar</i>s</a> baz</div>\
         <div>sibling</div>"
            .as_bytes()
    );

    let doc = doc.deep_clone(doc.root_element().expect("root"));
    assert_eq!(
        "<html><head></head><body>\
           <div>foo <a href=\"link\"><i>bar</i>s</a> baz</div>\
           <div>sibling</div>\
         </body></html>",
        doc.to_string()
    );

    let mut nodes = doc.nodes();
    assert_eq!(Document::DOCUMENT_NODE_ID, nodes.next().unwrap());
    assert!(doc[nodes.next().unwrap()].is_elem(t::HTML));
    assert!(doc[nodes.next().unwrap()].is_elem(t::HEAD));
    assert!(doc[nodes.next().unwrap()].is_elem(t::BODY));
    assert!(doc[nodes.next().unwrap()].is_elem(t::DIV));
    assert_eq!("foo ", doc[nodes.next().unwrap()].as_text().unwrap().as_ref());
    assert!(doc[nodes.next().unwrap()].is_elem(t::A));
    assert!(doc[nodes.next().unwrap()].is_elem(t::I));
    assert_eq!("bar", doc[nodes.next().unwrap()].as_text().unwrap().as_ref());
    assert_eq!("s", doc[nodes.next().unwrap()].as_text().unwrap().as_ref());
    assert_eq!(" baz", doc[nodes.next().unwrap()].as_text().unwrap().as_ref());
    assert!(doc[nodes.next().unwrap()].is_elem(t::DIV));
    assert_eq!("sibling", doc[nodes.next().unwrap()].as_text().unwrap().as_ref());
    assert!(nodes.next().is_none());
}

#[test]
fn test_select_children() {
    ensure_logger();
    let doc = html::parse_utf8(
        "<p>1</p>\
         <div>\
           fill\
           <p>2</p>\
           <p>3</p>\
           <div>\
             <p>4</p>\
             <i>fill</i>\
           </div>\
         </div>"
            .as_bytes()
    );

    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    let f1: Vec<_> = body
        .select_children(|n| n.is_elem(t::P))
        .map(|n| n.text().unwrap().to_string())
        .collect();

    assert_eq!(f1, vec!["1"]);
}

#[test]
fn test_select() {
    ensure_logger();
    let doc = html::parse_utf8_fragment(
        "<p>1</p>\
         <div>\
           fill\
           <p>2</p>\
           <p>3</p>\
           <div>\
             <p>4</p>\
             <i>fill</i>\
           </div>\
         </div>"
            .as_bytes()
    );

    let root = doc.root_element_ref().expect("root");

    assert_eq!("1fill234fill", root.text().unwrap().to_string());

    let f1: Vec<_> = root
        .select(|n| n.is_elem(t::P))
        .map(|n| n.text().unwrap().to_string())
        .collect();

    assert_eq!(f1, vec!["1", "2", "3", "4"]);
}

#[test]
fn test_meta_buffered() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::WINDOWS_1252);
    let mut reader = std::io::Cursor::new(
        r####"
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <meta charset='UTF-8'/>
  <META http-equiv=" CONTENT-TYPE" content='text/html; charset=" utf-8"'/>
  <title>Iūdex</title>
 </head>
 <body>
  <p>Iūdex test.</p>
 </body>
</html>"####
            .as_bytes());
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("Iūdex test.", body.text().unwrap().as_ref().trim());
}

// Simulates short reads.
struct ShortRead<R: Read>(R);

impl<R: Read> Read for ShortRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let end = std::cmp::min(buf.len(), 17);
        self.0.read(&mut buf[0..end])
    }
}

#[test]
fn test_meta_buffered_short() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::WINDOWS_1252);
    let mut reader = ShortRead(std::io::Cursor::new(
        r####"
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <META http-equiv=" CONTENT-TYPE" content='text/html; charset=" utf-8"'/>
  <title>Iūdex</title>
 </head>
 <body>
  <p>Iūdex test.</p>
 </body>
</html>"####
            .as_bytes()));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("Iūdex test.", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_tag_metadata() {
    let a_meta = TAG_META.get(&t::A).unwrap();
    assert!(a_meta.is_inline());
    assert!(a_meta.has_basic_attr(&a::HREFLANG));

    assert!(TAG_META.get(&t::AREA).unwrap().is_empty());
    assert!(TAG_META.get(&t::ACRONYM).unwrap().is_deprecated());
    assert!(TAG_META.get(&t::BASE).unwrap().is_meta());
    assert!(TAG_META.get(&t::BUTTON).unwrap().is_banned());

    // The "undefined" (by html5ever) cases:
    assert!(TAG_META.get(&t::RBC).unwrap().has_basic_attr(&a::BASE));
    assert!(TAG_META.get(&t::IMG).unwrap().has_basic_attr(&a::DECODING));
}
