// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

//! Support for html5ever parsing to `vdom::Document`.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::default::Default;
use std::io;
use std::rc::Rc;

use encoding_rs as enc;

use html5ever::{
    parse_document, parse_fragment,
    ExpandedName, QualName, Parser, ParseOpts
};
use html5ever::interface::tree_builder::{
    ElementFlags, NodeOrText, QuirksMode, TreeSink
};
use html5ever::tendril::{StrTendril, TendrilSink};
use mime;

use tendril::{fmt as form, Tendril};

use crate::decode::{Decoder, HTML_META_CONF, EncodingHint};

use crate::vdom::{
    Attribute, Document, Element, Node, NodeData, NodeId
};

mod meta;

pub use self::meta::{a, ns, t};

/// Parse HTML document from UTF-8 bytes in RAM.
pub fn parse_utf8(bytes: &[u8]) -> Document {
    let sink = Sink::default();
    parse_document(sink, Default::default())
        .from_utf8()
        .one(bytes)
}

/// Parse an HTML fragement from UTF-8 bytes in RAM.
///
/// A single root element is guaranteed. If the provided fragment does not
/// include one, a root <div> element is included as parent.
pub fn parse_utf8_fragment(bytes: &[u8]) -> Document {
    let sink = Sink::default();

    let mut doc = parse_fragment(
        sink,
        Default::default(),
        QualName::new(None, ns::HTML, t::DIV),
        vec![])
        .from_utf8()
        .one(bytes);

    // Note that the above context name, doesn't really get used. A matching
    // element is pushed but never linked, so unless we replace the doc (deep
    // clone, etc.) then it will contain this cruft.

    let root_id = doc.root_element().expect("a root");
    debug_assert!(doc[root_id].is_elem(t::HTML));

    // If the root has a single element child, then make that element child the
    // new root and return.
    // FIXME: Only do this, further, if its a block level element?
    if doc.children(root_id).count() == 1 {
        let child_id = doc[root_id].first_child.unwrap();
        if doc[child_id].as_element().is_some() {
            doc.fold(root_id);
            debug_assert!(doc.root_element().is_some());
            return doc;
        }
    }

    // Otherwise change the "html" root to a div. This is what we asked for,
    // but didn't get, from parse_fragment above.
    let root = doc[root_id].as_element_mut().unwrap();
    *root = Element {
        name: QualName::new(None, ns::HTML, t::DIV),
        attrs: vec![]
    };
    debug_assert!(doc.root_element().is_some());
    doc
}

const PARSE_BUFFER_SIZE: u32 = 4 * 1024;

/// Parse HTML document, reading from the given stream of bytes until end,
/// processing incrementally.
/// Return the resulting `Document` or any `io::Error`
pub fn parse_buffered<R>(eh: Rc<RefCell<EncodingHint>>, r: &mut R)
    -> Result<Document, io::Error>
    where R: io::Read
{
    let enc = eh.borrow().top().expect("EnodingHint default encoding required");

    let parser_sink: Parser<Sink> = parse_document(
        Sink::new(Some(eh.clone())),
        ParseOpts::default()
    );

    // Decoders are "Sink adaptors". Like the Parser, they also impl trait
    // TendrilSink.
    let mut decoder = Decoder::new(enc, parser_sink);

    let mut tendril = Tendril::<form::Bytes>::new();
    unsafe {
        tendril.push_uninitialized(PARSE_BUFFER_SIZE);
    }

    loop {
        match r.read(&mut tendril) {
            Ok(0) => return Ok(decoder.finish()),
            Ok(n) => {
                // FIXME: Specifically continue filling buffer for _short_
                // reads, to enable full buffer length an encoding hint?
                tendril.pop_back(PARSE_BUFFER_SIZE - n as u32);
                decoder.process(tendril.clone());
                break;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e)
        }
    } // repeat on interrupt

    if let Some(enc) = eh.borrow().changed() {
        eprintln!("EncodingHint change detected, switching to {}", enc.name());
        // Only here once, no need to clear change.

        // Replace decoder and re-process, consuming the original tendril
        // buffer.
        let parser_sink = parse_document(
            Sink::new(None),
            ParseOpts::default()
        );
        decoder = Decoder::new(enc, parser_sink);
        decoder.process(tendril);
    }

    parse_remainder(r, decoder)
}

// Read remaining bytes from reader, process and finish decoder.
fn parse_remainder<R>(
    r: &mut R,
    mut decoder: Decoder<Parser<Sink>>)
    -> Result<Document, io::Error>
    where R: io::Read
{
    loop {
        let mut tendril = Tendril::<form::Bytes>::new();
        unsafe {
            tendril.push_uninitialized(PARSE_BUFFER_SIZE);
        }
        loop {
            match r.read(&mut tendril) {
                Ok(0) => return Ok(decoder.finish()),
                Ok(n) => {
                    tendril.pop_back(PARSE_BUFFER_SIZE - n as u32);
                    decoder.process(tendril);
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e)
            }
        } // repeat on interrupt
    }
}

/// A `TreeSink` implementation for parsing html to a [`crate::vdom::Document`]
/// tree.
pub struct Sink {
    document: Document,
    quirks_mode: QuirksMode,
    enc_hint: Option<Rc<RefCell<EncodingHint>>>,
    enc_check: bool
}

impl Sink {
    /// Construct new sink with optional, shared `EncodingHint`.
    ///
    /// If the `EncodingHint` is provided, charsets from meta elements of the
    /// head element will be hinted as soon as possible in the parse.
    pub fn new(enc_hint: Option<Rc<RefCell<EncodingHint>>>) -> Sink {
        let enc_check = enc_hint.is_some();
        Sink {
            document: Document::new(),
            quirks_mode: QuirksMode::NoQuirks,
            enc_hint,
            enc_check,
        }
    }

    fn new_node(&mut self, data: NodeData) -> NodeId {
        self.document.push_node(Node::new(data))
    }

    fn append_common<P, A>(
        &mut self,
        child: NodeOrText<NodeId>,
        previous: P,
        append: A)
        where P: FnOnce(&mut Document) -> Option<NodeId>,
              A: FnOnce(&mut Document, NodeId)
    {
        let new_node = match child {
            NodeOrText::AppendText(text) => {
                // Append to an existing Text node if we have one.
                if let Some(id) = previous(&mut self.document) {
                    let node = &mut self.document[id];
                    if let NodeData::Text(t) = &mut node.data {
                        t.push_tendril(&text);
                        return;
                    }
                }
                self.new_node(NodeData::Text(text))
            }
            NodeOrText::AppendNode(node) => {
                if self.enc_check && self.document[node].is_elem(t::BODY) {
                    eprintln!("body appended, checking meta charsets now");
                    self.enc_check = false;
                    self.check_meta_charsets();
                }
                node
            }
        };

        append(&mut self.document, new_node);
    }

    fn check_meta_charsets(&mut self) {
        let mut metas = 0;
        let mut charsets = Vec::new();
        let root = self.document.root_element_ref().expect("root");
        if let Some(head) = root.find_child(|n| n.is_elem(t::HEAD)) {
            for m in head.select_children(|n| n.is_elem(t::META)) {
                if let Some(cs) = m.attr(a::CHARSET) {
                    metas += 1;
                    let cs = cs.trim().as_bytes();
                    if let Some(enc) = enc::Encoding::for_label(cs) {
                        charsets.push(enc);
                    }
                } else if let Some(a) = m.attr(a::HTTP_EQUIV) {
                    if a.as_ref().trim().eq_ignore_ascii_case("Content-Type") {
                        if let Some(a) = m.attr(a::CONTENT) {
                            if let Ok(m) = a.as_ref().trim().parse::<mime::Mime>() {
                                if let Some(cs) = m.get_param(mime::CHARSET) {
                                    metas += 1;
                                    let cs = cs.as_str().trim().as_bytes();
                                    if let Some(enc) = enc::Encoding::for_label(cs) {
                                        charsets.push(enc)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if !charsets.is_empty() {
            eprintln!("found charsets: {:?} ({})",
                      charsets.iter().map(|e| e.name()).collect::<Vec<_>>(),
                      metas);
            let conf = HTML_META_CONF / (metas as f32);

            let mut hints = self.enc_hint.as_ref().unwrap().borrow_mut();
            for cs in charsets {
                hints.add_hint(cs, conf);
            }
        }
    }
}

impl Default for Sink {
    fn default() -> Self {
        Sink::new(None)
    }
}

impl TreeSink for Sink {
    type Handle = NodeId;
    type Output = Document;

    fn finish(self) -> Document {
        self.document
    }

    fn parse_error(&mut self, _: Cow<'static, str>) {}

    fn get_document(&mut self) -> NodeId {
        Document::DOCUMENT_NODE_ID
    }

    fn set_quirks_mode(&mut self, mode: QuirksMode) {
        self.quirks_mode = mode;
    }

    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        x == y
    }

    fn elem_name<'a>(&'a self, &target: &'a NodeId) -> ExpandedName<'a> {
        self.document[target]
            .as_element()
            .expect("not an element")
            .name
            .expanded()
    }

    fn get_template_contents(&mut self, &target: &NodeId) -> NodeId {
        target
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags)
        -> NodeId
    {
        self.new_node(NodeData::Elem(Element { name, attrs }))
    }

    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.new_node(NodeData::Comment(text))
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril)
        -> NodeId
    {
        self.new_node(NodeData::ProcessingInstruction { target, data })
    }

    fn append(&mut self, &parent: &NodeId, child: NodeOrText<NodeId>) {
        self.append_common(
            child,
            |document| document[parent].last_child,
            |document, new_node| document.append(parent, new_node),
        )
    }

    fn append_before_sibling(
        &mut self,
        &sibling: &NodeId,
        child: NodeOrText<NodeId>)
    {
        self.append_common(
            child,
            |document| document[sibling].previous_sibling,
            |document, new_node| document.insert_before(sibling, new_node),
        )
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &NodeId,
        prev_element: &NodeId,
        child: NodeOrText<NodeId>)
    {
        if self.document[*element].parent.is_some() {
            self.append_before_sibling(element, child)
        } else {
            self.append(prev_element, child)
        }
    }

    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril)
    {
        let node = self.new_node(NodeData::Doctype {
            name,
            _public_id: public_id,
            _system_id: system_id,
        });
        self.document.append(Document::DOCUMENT_NODE_ID, node)
    }

    fn add_attrs_if_missing(
        &mut self,
        &target: &NodeId,
        attrs: Vec<Attribute>)
    {
        // FIXME: Never called in our normal/test usage thus far?
        let node = &mut self.document[target];
        let element = if let NodeData::Elem(e) = &mut node.data {
            e
        } else {
            panic!("not an element");
        };

        let existing_names = element
            .attrs
            .iter()
            .map(|e| e.name.clone())
            .collect::<HashSet<_>>();
        element.attrs.extend(
            attrs
                .into_iter()
                .filter(|attr| !existing_names.contains(&attr.name)),
        );
    }

    fn remove_from_parent(&mut self, &target: &NodeId) {
        self.document.detach(target)
    }

    fn reparent_children(&mut self, &node: &NodeId, &new_parent: &NodeId) {
        let mut next_child = self.document[node].first_child;
        while let Some(child) = next_child {
            debug_assert_eq!(self.document[child].parent, Some(node));
            // Preserve iteration by updating this here, before `append`
            // detaches the association.
            next_child = self.document[child].next_sibling;
            self.document.append(new_parent, child);
        }
    }
}
