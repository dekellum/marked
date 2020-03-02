use std::fs::File;
use std::{io, io::Read};

use crate::{
    Attribute, Document, Element, Node, NodeData, NodeRef,
    QualName, StrTendril,
    filter, filter::Action,
    html, html::{a, t, TAG_META},
    xml,
    HTTP_CTYPE_CONF,
};

use crate::chain_filters;
use crate::logger::ensure_logger;
use crate::decode::EncodingHint;

use encoding_rs as enc;
use log::debug;
use rand::Rng;

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

fn strike_fold_filter(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if data.is_elem(t::STRIKE) { Action::Fold } else { Action::Continue }
}

fn strike_remove_filter(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if data.is_elem(t::STRIKE) { Action::Detach } else { Action::Continue }
}

#[test]
fn test_detach_root() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<html>text</html>"
            .as_bytes()
    );
    doc.detach(doc.root_element_ref().unwrap().id());
    assert!(doc.root_element_ref().is_none());
    assert_eq!("", doc.to_string());
}

#[test]
fn test_detach_root_doctype() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<!DOCTYPE html><html>text</html>"
            .as_bytes()
    );
    doc.detach(doc.root_element_ref().unwrap().id());
    assert!(doc.root_element_ref().is_none());
    assert_eq!("<!DOCTYPE html>", doc.to_string());
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
    // Non-useful chain but confirms it works on one!
    doc.filter(chain_filters!(strike_remove_filter));
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
    let other_filter = |_p: NodeRef<'_>, data: &mut NodeData| {
        if data.is_elem(t::META) { Action::Detach } else { Action::Continue }
    };

    doc.filter(chain_filters!(
        other_filter,
        strike_remove_filter,
        // in place noop
        |_pos: NodeRef<'_>, _nd: &mut NodeData| { Action::Continue },
        filter::text_normalize, // best not used liked this
    ));

    assert_eq!(
        "<div>foo baz</div>",
        doc.to_string()
    );
}

#[test]
fn test_filter_chain_large_sample() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = sample_file("github-dekellum.html");
    let mut doc = html::parse_buffered(eh, &mut reader).unwrap();
    let pass_0 = chain_filters!(
        filter::detach_banned_elements,
        filter::detach_comments,
        filter::detach_pis,
    );
    doc.filter_breadth(pass_0);

    let pass_1 = chain_filters!(
        filter::fold_empty_inline,
        filter::retain_basic_attributes,
        filter::xmp_to_pre,
    );
    doc.filter(pass_1);
    doc.filter(filter::text_normalize);
    assert_eq!(25893, doc.to_string().len(), "{}",
               doc.to_string());

    // Make sure filtering is stable/idempotent
    doc.filter(pass_1);
    doc.filter(filter::text_normalize);
    assert_eq!(25893, doc.to_string().len(), "{}",
               doc.to_string());
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
fn test_empty_inline() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>text<i> </i> 2 <i></i> 3 <i> <br> </i> end</div>"
            .as_bytes()
    );

    assert_eq!(
        "<div>text<i> </i> 2 <i></i> 3 <i> <br> </i> end</div>",
        doc.to_string()
    );

    doc.filter(chain_filters!(
        filter::fold_empty_inline,
        filter::text_normalize
    ));

    // Its not recomended to combine text_normalize as above, but make sure it
    // just leaves some whitespace.

    assert_eq!(
        "<div>text  2  3 <br>  end</div>",
        doc.to_string()
    );

    // Now normalize properly:
    doc.filter(filter::text_normalize);

    assert_eq!(
        "<div>text 2 3<br>end</div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    debug!("the doc nodes:\n{:?}", doc);
    assert_eq!(4, doc.nodes.len() - 2);
}

#[test]
fn test_xmp() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>foo <xmp><i>bar\n</i>\n</xmp> baz</div>"
            .as_bytes()
    );

    assert_eq!(
        "<div>foo <xmp><i>bar\n</i>\n</xmp> baz</div>",
        doc.to_string()
    );

    doc.filter(chain_filters!(
        filter::xmp_to_pre,
        filter::text_normalize
    ));

    assert_eq!(
        "<div>foo<pre>&lt;i&gt;bar\n&lt;/i&gt;\n</pre>baz</div>",
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
    let mut doc = html::parse_utf8_fragment(
        "<div><plaintext>bar\n\tbaz</div>"
            .as_bytes()
    );
    // Serializer isn't aware that <plaintext> doesn't need end tags, etc.
    assert_eq!(
        "<div><plaintext>bar\n\tbaz</div></plaintext></div>",
        doc.to_string()
    );

    doc.filter(chain_filters!(
        filter::xmp_to_pre,
        filter::text_normalize
    ));

    assert_eq!(
        "<div><pre>bar\n\tbaz&lt;/div&gt;</pre></div>",
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

// Simulates randomized short reads and interrupts
struct ShortRead<R: Read>(R);

impl<R: Read> Read for ShortRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let mut rng = rand::thread_rng();
        let end = if rng.gen_bool(0.20) {
            buf.len()
        } else {
            rng.gen_range(0, buf.len()+1)
        };
        if end > 0 {
            self.0.read(&mut buf[0..end])
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "short interruption!"
            ))
        }
    }
}

fn sample_file(fname: &str) -> File {
    let root = env!("CARGO_MANIFEST_DIR");
    let fpath = format!("{}/samples/{}", root, fname);
    File::open(fpath).unwrap()
}

#[test]
fn test_documento_utf8() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("documento_utf8.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf8_bom() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("documento_utf8_bom.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf16be_bom() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("documento_utf16be_bom.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf16be_meta_utf16le() {
    ensure_logger();
    let eh = EncodingHint::shared_with_hint(enc::UTF_16BE, HTTP_CTYPE_CONF);
    // The contained meta should be ignored, since, if it was correct, we
    // couldn't read it from the initial UTF-16BE encoding!
    let mut reader = ShortRead(sample_file("documento_utf16be_meta_utf16le.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf16le_bom() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("documento_utf16le_bom.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf16le() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_16LE);
    let mut reader = ShortRead(sample_file("documento_utf16le.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf16le_meta_utf8() {
    ensure_logger();
    let eh = EncodingHint::shared_with_hint(enc::UTF_16LE, HTTP_CTYPE_CONF);
    // The contained meta should be ignored, since, if it was correct, we
    // couldn't read it from the initial UTF-16 encoding!
    let mut reader = ShortRead(sample_file("documento_utf16le_meta_utf8.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf8_meta_utf16() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    // The contained meta should be ignored, since, if it was correct, we
    // couldn't read it from the initial UTF-16 encoding!
    let mut reader = ShortRead(sample_file("documento_utf8_meta_utf16.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_windows1252_meta() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("documento_windows1252_meta.html"));
    let doc = html::parse_buffered(eh, &mut reader).unwrap();
    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_documento_utf8_meta() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::WINDOWS_1252);
    let mut reader = ShortRead(sample_file("documento_utf8_meta.html"));
    let mut doc = html::parse_buffered(eh, &mut reader).unwrap();
    doc.filter(chain_filters!(
        filter::detach_banned_elements,
        filter::detach_comments,
        filter::detach_pis,
        filter::retain_basic_attributes,
        filter::xmp_to_pre,
    ));
    doc.filter(filter::text_normalize);

    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!("¿De donde eres tú?", body.text().unwrap().as_ref().trim());
}

#[test]
fn test_iro0094_shiftjis_meta() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("iro0094_shiftjis_meta.html"));
    let mut doc = html::parse_buffered(eh.clone(), &mut reader).unwrap();
    doc.filter(chain_filters!(
        filter::detach_banned_elements,
        filter::detach_comments,
        filter::detach_pis,
        filter::retain_basic_attributes,
        filter::xmp_to_pre,
    ));
    doc.filter(filter::text_normalize);

    let root = doc.root_element_ref().expect("root");
    let _body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!(eh.borrow().top().as_ref().unwrap(), &enc::SHIFT_JIS);
    assert_eq!(eh.borrow().errors(), 0);
}

#[test]
fn test_matsunami_eucjp_meta() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("matsunami_eucjp_meta.html"));
    let mut doc = html::parse_buffered(eh.clone(), &mut reader).unwrap();
    doc.filter(chain_filters!(
        filter::detach_banned_elements,
        filter::detach_comments,
        filter::detach_pis,
        filter::retain_basic_attributes,
        filter::xmp_to_pre,
    ));
    doc.filter(filter::text_normalize);

    let root = doc.root_element_ref().expect("root");
    let _body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!(eh.borrow().top().as_ref().unwrap(), &enc::EUC_JP);
    assert_eq!(eh.borrow().errors(), 0);
}

#[test]
fn test_russez_windows1251_meta() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = ShortRead(sample_file("russez_windows1251_meta.html"));
    let mut doc = html::parse_buffered(eh.clone(), &mut reader).unwrap();
    doc.filter(chain_filters!(
        filter::detach_banned_elements,
        filter::detach_comments,
        filter::detach_pis,
        filter::retain_basic_attributes,
        filter::xmp_to_pre,
    ));
    doc.filter(filter::text_normalize);

    let root = doc.root_element_ref().expect("root");
    let body = root.find_child(|n| n.is_elem(t::BODY)).expect("body");
    assert_eq!(eh.borrow().top().as_ref().unwrap(), &enc::WINDOWS_1251);
    assert_eq!(eh.borrow().errors(), 0);
    assert!(
        body.find(|n| {
            if let Some(st) = n.as_text() {
                st.as_ref().contains("Промышленно")
            } else {
                false
            }
        }).is_some(),
        "txt: {}", body.text().unwrap().as_ref()
    );
}
