use std::fs::File;
use std::{io, io::Read};

use crate::{
    Attribute, Document, Element, Node, NodeData, NodeId, NodeRef,
    QualName, StrTendril,
    filter, filter::Action,
    html, html::{a, t, TAG_META},
    HTTP_CTYPE_CONF,
};

#[cfg(feature = "xml")]
use crate::xml;

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
    assert_eq!(size_of::<NodeId>(), 4);
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
    assert_eq!(1, doc.len());
    assert!(doc.is_empty());
}

#[test]
fn one_element() {
    ensure_logger();
    let mut doc = Document::new();
    let element = Node::new_elem(Element::new("one"));
    let id = doc.append_child(Document::DOCUMENT_NODE_ID, element);

    assert!(doc.root_element_ref().is_some(), "pushed root Element");
    assert_eq!(id, doc.root_element_ref().unwrap().id());
    assert_eq!(2, doc.nodes().count(), "root + 1 element");
}

#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn suitable_parent_asserted() {
    ensure_logger();
    let mut doc = Document::new();
    let eid = doc.append_child(
        Document::DOCUMENT_NODE_ID,
        Node::new_elem(Element::new("one"))
    );
    let tid = doc.append_child(eid, Node::new_text("text"));
    doc.append_child(tid, Node::new_elem(Element::new("bogus")));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic]
fn redundant_document_node_asserted() {
    ensure_logger();
    let mut doc = Document::new();
    doc.append_child(
        Document::DOCUMENT_NODE_ID,
        Node::new(NodeData::Document));
}

#[test]
fn element_attrs() {
    ensure_logger();
    let mut el = Element::new(t::A);
    assert!(el.set_attr("href", "/where").is_none());
    assert_eq!("/where", el.set_attr("href", "/other").unwrap().as_ref());
    assert_eq!("/other", el.remove_attr(a::HREF).unwrap().as_ref());
}

#[test]
fn element_attrs_dups() {
    ensure_logger();
    let mut el = Element::new(t::A);
    // Manually, for duplicates:
    el.attrs = vec![
        Attribute {
            name: QualName::new(None, ns!(), a::REL),
            value: "nofollow".into()
        },
        Attribute {
            name: QualName::new(None, ns!(), a::HREF),
            value: "/some".into()
        },
        Attribute {
            name: QualName::new(None, ns!(), a::REL),
            value: "noreferrer".into()
        },
    ];
    assert_eq!(3, el.attrs.len());
    assert_eq!("/some", el.set_attr("href", "/other").unwrap().as_ref());
    assert_eq!(3, el.attrs.len());
    assert_eq!("noreferrer", el.set_attr(a::REL, "external").unwrap().as_ref());
    assert_eq!(2, el.attrs.len());
    assert_eq!("external", el.attr("rel").unwrap().as_ref());
}

#[test]
fn mixed_text_no_root() {
    ensure_logger();
    let mut doc = Document::new();
    let element = Node::new_elem(Element::new("one"));
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
    let det = doc.detach(doc.root_element_ref().unwrap().id());
    assert!(doc.root_element_ref().is_none());
    assert_eq!("", doc.to_string());
    assert_eq!("<html><head></head><body>text</body></html>", det.to_string());
}

#[test]
fn test_detach_fragment() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>text</div>"
            .as_bytes()
    );
    let det = doc.detach(doc.root_element_ref().unwrap().id());
    assert!(doc.root_element_ref().is_none());
    assert_eq!("", doc.to_string());
    assert_eq!("<div>text</div>", det.to_string());

    // Re-attach
    doc.attach_child(Document::DOCUMENT_NODE_ID, det);
    assert_eq!("<div>text</div>", doc.to_string());
}

#[test]
fn test_detach_root_doctype() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<!DOCTYPE html><html>text</html>"
            .as_bytes()
    );
    let det = doc.detach(doc.root_element_ref().unwrap().id());
    assert!(doc.root_element_ref().is_none());
    assert_eq!("<!DOCTYPE html>", doc.to_string());
    assert_eq!("<html><head></head><body>text</body></html>", det.to_string());
}

#[test]
fn test_detach_text() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>text</div>"
            .as_bytes()
    );
    assert_eq!(2, doc.nodes().count() - 1);

    let rid = doc.root_element().unwrap();
    let tid = doc.children(rid).next().unwrap();
    assert_eq!(1, doc.descendants(tid).count());
    let det = doc.detach(tid);
    assert!(det.root_element().is_none());
    assert_eq!(1, det.nodes().count() - 1);
    assert_eq!("<div></div>", doc.to_string());
    assert_eq!(1, doc.nodes().count() - 1);
    doc.compact();
    assert_eq!(1, doc.len() - 1);
    assert_eq!("text", det.to_string());
    assert_eq!(1, det.len() - 1);
}

#[test]
fn test_detach_text_sib() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>\
          <p>text</p>\
          <p>more</p>\
         </div>"
            .as_bytes()
    );
    assert_eq!(5, doc.nodes().count() - 1);

    let rid = doc.root_element().unwrap();
    let pid = doc.children(rid).next().unwrap(); // <p>(text)
    assert_eq!(2, doc.descendants(pid).count());
    let tid = doc.children(pid).next().unwrap(); // (text)
    assert_eq!(1, doc.descendants(tid).count());
    let det = doc.detach(tid);
    assert!(det.root_element().is_none());
    assert_eq!(1, det.nodes().count() - 1);
    assert_eq!("<div><p></p><p>more</p></div>", doc.to_string());
    assert_eq!(4, doc.nodes().count() - 1);
    doc.compact();
    assert_eq!(4, doc.len() - 1);
    assert_eq!("text", det.to_string());
    assert_eq!(1, det.len() - 1);
}

#[test]
fn test_detach_attach_sib() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<div>\
          <p>text</p>\
          <p>more</p>\
         </div>"
            .as_bytes()
    );
    let div = doc.root_element().unwrap();
    let pid = doc.children(div).next().unwrap(); // <p>(text)
    let p1 = doc.detach(pid);
    assert_eq!("<p>text</p>", p1.to_string());
    assert_eq!("<div><p>more</p></div>", doc.to_string());
    let pid = doc.children(div).next().unwrap(); // <p>(more)
    let p2 = doc.detach(pid);
    assert_eq!("<p>more</p>", p2.to_string());
    assert_eq!("<div></div>", doc.to_string());
    doc.attach_child(div, p2);
    assert_eq!("<div><p>more</p></div>", doc.to_string());
    let pid = doc.children(div).next().unwrap(); // <p>(more)
    doc.attach_before_sibling(pid, p1);
    assert_eq!("<div><p>text</p><p>more</p></div>", doc.to_string());
}


#[test]
fn test_fold_filter() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<div>foo <strike><strike>\
         <strike/><i>bar</i><strike>s</strike>\
         </strike></strike> baz</div>"
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
fn test_fold_filter_breadth() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<div>foo <strike><strike>\
         <strike/><i>bar</i><strike>s</strike>\
         </strike></strike> baz</div>"
            .as_bytes()
    );
    doc.filter_breadth(strike_fold_filter);
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

    assert_eq!(10, doc.len() - 1);
    doc.compact();
    assert_eq!(6, doc.len() - 1);

    assert_eq!(
        "<html><head></head><body>\
         <div>foo  baz</div>\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_remove_filter_breadth() {
    ensure_logger();
    let mut doc = html::parse_utf8(
        "<div>foo <strike><i>bar</i>s</strike> baz</div>"
            .as_bytes()
    );
    // Non-useful chain but confirms it works on one!
    doc.filter_breadth(chain_filters!(strike_remove_filter));
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
    assert_eq!(5500, doc.len());
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
    doc.compact();
    assert_eq!(1497, doc.len());
    assert_eq!(2, doc.children(Document::DOCUMENT_NODE_ID).count());

    assert_eq!(25893, doc.to_string().len(), "{}", doc.to_string());
}

#[test]
fn test_filter_chain_large_sample_breadth() {
    ensure_logger();
    let eh = EncodingHint::shared_default(enc::UTF_8);
    let mut reader = sample_file("github-dekellum.html");
    let mut doc = html::parse_buffered(eh, &mut reader).unwrap();
    assert_eq!(5500, doc.len());
    let pass_0 = chain_filters!(
        filter::detach_banned_elements,
        filter::detach_comments,
        filter::detach_pis,
        filter::retain_basic_attributes,
        filter::xmp_to_pre,
    );
    doc.filter_breadth(pass_0);
    doc.filter(filter::fold_empty_inline);
    doc.filter(filter::text_normalize);
    assert_eq!(25893, doc.to_string().len(), /*"{}", doc.to_string()*/);

    // Make sure filtering is stable/idempotent
    doc.filter_breadth(pass_0);
    doc.filter(filter::fold_empty_inline);
    doc.filter(filter::text_normalize);

    doc.compact();
    assert_eq!(1497, doc.len());
    assert_eq!(2, doc.children(Document::DOCUMENT_NODE_ID).count());

    assert_eq!(25893, doc.to_string().len(), /*"{}", doc.to_string()*/);
}

#[test]
#[cfg(feature = "xml")]
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
#[cfg(feature = "xml")]
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

    doc.compact();
    assert_eq!(4, doc.len() - 1);

    assert_eq!(
        "<div>text 2 3<br>end</div>",
        doc.to_string()
    );

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

    doc.compact();
    assert_eq!(5, doc.len() - 1);

    assert_eq!(
        "<div>foo<pre>&lt;i&gt;bar\n&lt;/i&gt;\n</pre>baz</div>",
        doc.to_string()
    );
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

    doc.compact();
    assert_eq!(3, doc.len() - 1);

    assert_eq!(
        "<div><pre>bar\n\tbaz&lt;/div&gt;</pre></div>",
        doc.to_string()
    );
}

#[test]
fn test_img_decoding_unknown() {
    ensure_logger();
    // The decoding attribute is unknown to html5ever
    let mut doc = html::parse_utf8_fragment(
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

    doc.compact();
    assert_eq!(2, doc.len() - 1);

    assert_eq!(
        r##"<div><img href="foo" decoding="sync"></div>"##,
        doc.to_string()
    );
}

#[test]
fn test_text_fragment() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "plain &lt; text".as_bytes()
    );

    doc.compact();
    assert_eq!(2, doc.len() - 1);

    assert_eq!(
        "<div>\
         plain &lt; text\
         </div>",
        doc.to_string()
    );

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
    let mut doc = html::parse_utf8_fragment(
        "plain<wbr>text".as_bytes()
    );

    doc.compact();
    assert_eq!(4, doc.len() - 1);

    assert_eq!(
        "<div>\
         plain<wbr>text\
         </div>",
        doc.to_string()
    );
}

#[test]
fn test_parsed_attrs() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        r##"<a rel="nofollow" href=".." rel="noindex">link</a>"##.as_bytes()
    );
    // *5ever won't duplicate attributes:
    assert_eq!(
        r##"<div><a rel="nofollow" href="..">link</a></div>"##,
        doc.to_string()
    );
    let root = doc.root_element_ref().expect("root");
    let aid = root.find(|n| n.is_elem(t::A)).expect("find <a>").id();

    doc[aid]
        .as_element_mut()
        .unwrap()
        .set_attr(a::REL, "reset");

    assert_eq!(
        r##"<div><a rel="reset" href="..">link</a></div>"##,
        doc.to_string()
    );

    doc[aid]
        .as_element_mut()
        .unwrap()
        .remove_attr(a::REL);

    assert_eq!(
        r##"<div><a href="..">link</a></div>"##,
        doc.to_string()
    );

    doc[aid]
        .as_element_mut()
        .unwrap()
        .set_attr(a::REL, "replace");

    assert_eq!(
        r##"<div><a href=".." rel="replace">link</a></div>"##,
        doc.to_string()
    );
}

#[test]
fn test_html_attr() {
    // Found parser call of `Sink::add_attrs_if_missing`
    ensure_logger();
    let doc = html::parse_utf8(
        "<body><html lang=\"en\">text</html></body>"
            .as_bytes()
    );
    assert_eq!(
        "<html lang=\"en\"><head></head><body>\
         text\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_shallow_fragment() {
    ensure_logger();
    let mut doc = html::parse_utf8_fragment(
        "<b>b</b> text <i>i</i>".as_bytes()
    );

    doc.compact();
    assert_eq!(6, doc.len() - 1);

    assert_eq!(
        "<div>\
         <b>b</b> text <i>i</i>\
         </div>",
        doc.to_string()
    );
}

#[test]
fn test_inline_fragment() {
    ensure_logger();

    // An single inline element such as <i> will not be used as the root
    let mut doc = html::parse_utf8_fragment(
        "<i>text</i>".as_bytes()
    );

    doc.compact();
    assert_eq!(3, doc.len() - 1);

    assert_eq!(
        "<div>\
         <i>text</i>\
         </div>",
        doc.to_string()
    );
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

    let doc = doc.deep_clone(Document::DOCUMENT_NODE_ID);
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
fn test_append_deep_clone() {
    ensure_logger();
    let frag1 = html::parse_utf8(
        "<div>foo <a href=\"link\"><i>bar</i>s</a> baz</div>\
         <div>sibling</div>"
            .as_bytes()
    );
    let root = frag1.root_element_ref().expect("root");
    let aref = root.find(|n| n.is_elem(t::A)).expect("<a>");

    let mut frag2 = Document::new();
    let ul = frag2.append_child(
        Document::DOCUMENT_NODE_ID,
        Node::new_elem(Element::new(t::UL))
    );
    let li1 = frag2.append_child(ul, Node::new_elem(Element::new(t::LI)));
    frag2.append_deep_clone(li1, &frag1, aref.id());
    let li2 = frag2.append_child(ul, Node::new_elem(Element::new(t::LI)));
    frag2.append_deep_clone(li2, &frag1, aref.id());
    assert_eq!(
        "<ul>\
           <li><a href=\"link\"><i>bar</i>s</a></li>\
           <li><a href=\"link\"><i>bar</i>s</a></li>\
         </ul>",
        frag2.to_string()
    );
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
