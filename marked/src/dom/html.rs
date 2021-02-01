// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

//! Support for html5 parsing to `Document`.

use std::borrow::Cow;
use std::collections::HashSet;
use std::default::Default;
use std::io;

use encoding_rs as enc;

use html5ever::{
    parse_document, parse_fragment,
    ExpandedName, QualName, Parser, ParseOpts
};
use html5ever::interface::tree_builder::{
    ElementFlags, NodeOrText, QuirksMode, TreeSink
};
use html5ever::tendril::{StrTendril, TendrilSink};
use log::{debug, info, trace};
use tendril::{fmt as form, Tendril};

use crate::{
    Attribute, Decoder, Document, DocumentType, Element, EncodingHint,
    Node, NodeData, NodeId, ProcessingInstruction, SharedEncodingHint,
    BOM_CONF, HTML_META_CONF, INITIAL_BUFFER_SIZE,
};

mod meta;

pub use self::meta::{
    a, ns, t,
    TagMeta, TAG_META
};

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
/// contain a single, block level (e.g. not [`TagMeta::is_inline`]) element, a
/// root `<div>` element is included as parent.
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

    // If the root has a single element child, which is not an inline
    // element, then make that element child the new root and return.
    if doc.children(root_id).count() == 1 {
        let child_id = doc[root_id].first_child.unwrap();
        if let Some(elm) = doc[child_id].as_element() {
            if let Some(meta) = elm.html_tag_meta() {
                if !meta.is_inline() {
                    doc.fold(root_id);
                    debug_assert!(doc.root_element().is_some());
                    return doc;
                }
            }
        }
    }

    // Otherwise change the "html" root to a div. This is what we asked for,
    // but didn't get, from parse_fragment above.
    let root = doc[root_id].as_element_mut().unwrap();
    *root = Element::new(t::DIV);
    debug_assert!(doc.root_element().is_some());
    doc
}

/// Parse and return an HTML `Document`, reading from the given stream of bytes
/// until end, processing incrementally.
///
/// The [`SharedEncodingHint`] must have a top (e.g. default) encoding, which
/// will be used initially for decoding bytes. The [`INITIAL_BUFFER_SIZE`]
/// bytes of the stream are buffered and if a compelling alternative encoding
/// hint is found via a leading Byte-Order-Mark (BOM) or in the documents
/// `<head>`, the parse will be restarted from the beginning with that encoding
/// and continuing until the end.
pub fn parse_buffered<R>(hint: SharedEncodingHint, r: &mut R)
    -> Result<Document, io::Error>
    where R: io::Read
{
    let enc = hint.borrow().top().expect("EnodingHint default encoding required");

    let parser_sink: Parser<Sink> = parse_document(
        Sink::new(hint.clone(), true),
        ParseOpts::default()
    );

    // Decoders are "Sink adaptors" that also impl TendrilSink.
    // The decoder is consumed to finish the parse.
    let mut decoder = Some(Decoder::new(enc, parser_sink));

    // Read up to _SIZE bytes, processing as we go and finishing if end is
    // reached in that size.
    let mut buff = Tendril::<form::Bytes>::new();
    unsafe {
        buff.push_uninitialized(INITIAL_BUFFER_SIZE);
    }
    let mut i = 0;
    let mut finished = None;
    loop {
        match r.read(&mut buff[i as usize..]) {
            Ok(0) => {
                trace!("read 0 bytes (end len {})", i);
                finished = Some(decoder.take().unwrap().finish());
                break;
            }
            Ok(n) => {
                let n = n as u32;
                trace!("read {} bytes (len {})", n, i + n);

                // One time, leading Byte-order-mark (BOM) detection for UTF-16
                // little/big endian, or UTF-8, after reading initial 3 bytes.
                // This is part of the `decode` algorithm of the Encoding
                // Standard which is not implemented by either encoding_rs or
                // html5ever. html5ever will ignore a BOM character so we need
                // not remove it before processing.  If the new hint is
                // compelling, then break early to reprocess with a new
                // decoder.
                if i < 3 && (i + n) >= 3 {
                    if let Some(enc) = bom_enc(&buff) {
                        if hint.borrow_mut().add_hint(enc, BOM_CONF) {
                            i += n;
                            break;
                        }
                    }
                }

                decoder.as_mut().unwrap().process(buff.subtendril(i, n));
                i += n;
                if i == INITIAL_BUFFER_SIZE || hint.borrow().changed().is_some() {
                    break;
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e)
        }
    } // repeat on interrupt or short read.

    // Avoid any uninitialized trailing bytes
    buff.pop_back(INITIAL_BUFFER_SIZE - i);

    let (changed, errors) = {
        let hint = hint.borrow();
        trace!("revised hint: {:?}", hint);
        (hint.changed(), hint.errors())
    };

    if let Some(enc) = changed {
        info!(
            "Reparsing with enc {}, buffered: {}, prior enc errors: {}",
            enc.name(), buff.len(), errors
        );
        hint.borrow_mut().clear_errors();
        finished = None;

        // Replace decoder and re-process, consuming the original tendril
        // buffer, which was previously cloned.
        let parser_sink = parse_document(
            Sink::new(hint.clone(), false),
            ParseOpts::default()
        );
        decoder = Some(Decoder::new(enc, parser_sink));
        decoder.as_mut().unwrap().process(buff);
    }

    // If (still) finished, return that Document, else read and process to end.
    let res = if let Some(d) = finished {
        Ok(d)
    } else {
        decoder.take().unwrap().read_to_end(r)
    };
    if res.is_ok() {
        debug!("Final encoding errors {}", hint.borrow().errors());
    }
    res
}

// Return encoding for any Byte-Order-Mark found at start of buff.
fn bom_enc(buff: &Tendril::<form::Bytes>) -> Option<&'static enc::Encoding>
{
    match (buff[0], buff[1], buff[2]) {
        (0xFE, 0xFF,    _) => Some(enc::UTF_16BE),
        (0xFF, 0xFE,    _) => Some(enc::UTF_16LE),
        (0xEF, 0xBB, 0xBF) => Some(enc::UTF_8),
        _ => None
    }
}

/// A `TreeSink` implementation for parsing html to a
/// [`Document`](crate::Document) tree.
pub struct Sink {
    document: Document,
    #[allow(unused)]
    quirks_mode: QuirksMode,
    enc_hint: SharedEncodingHint,
    enc_check: bool,
}

impl Sink {
    /// Construct new sink with shared `EncodingHint`.
    ///
    /// If enc_check is true, encodings mentioned in html meta elements will be
    /// added to the encoding hint as soon as possible in the parse.
    pub fn new(enc_hint: SharedEncodingHint, enc_check: bool) -> Sink {

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
                    debug!("body appended, checking meta charsets now");
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
            debug!(
                "found charsets: {:?} ({})",
                charsets.iter().map(|e| e.name()).collect::<Vec<_>>(),
                metas);
            let conf = HTML_META_CONF / (metas as f32);

            let mut hints = self.enc_hint.borrow_mut();
            for cs in charsets {
                if hints.could_read_from(cs) {
                    hints.add_hint(cs, conf);
                } else {
                    debug!("Ignoring impossible hint: {}", cs.name());
                }
            }
        }
    }
}

impl Default for Sink {
    fn default() -> Self {
        Sink::new(EncodingHint::shared_default(enc::UTF_8), false)
    }
}

impl TreeSink for Sink {
    type Handle = NodeId;
    type Output = Document;

    fn finish(self) -> Document {
        self.document
    }

    fn parse_error(&mut self, err: Cow<'static, str>) {
        // Not the nicest error type to work with.
        if err == "invalid byte sequence" {
            // From tendril crate (src/stream.rs) or our Decoder
            self.enc_hint.borrow_mut().increment_error();
        } else {
            debug!("other parser error: {}", err);
        }
    }

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
        self.new_node(NodeData::Elem(Element { name, attrs, _priv: () }))
    }

    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.new_node(NodeData::Comment(text))
    }

    fn create_pi(&mut self, _target: StrTendril, data: StrTendril)
        -> NodeId
    {
        self.new_node(NodeData::Pi(ProcessingInstruction { data, _priv: () }))
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
            |document| document[sibling].prev_sibling,
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
        _p_id: StrTendril,
        _s_id: StrTendril)
    {
        let node = self.new_node(NodeData::DocType(
            DocumentType { name, _priv: () }
        ));
        self.document.append(Document::DOCUMENT_NODE_ID, node)
    }

    fn add_attrs_if_missing(
        &mut self,
        &target: &NodeId,
        attrs: Vec<Attribute>)
    {
        // Note this is only used in few, strange cases involving re-working of
        // html and body node attributes, but it definitely needs to be
        // implemented.

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
        self.document.unlink_only(target)
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
