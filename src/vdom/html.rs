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
use std::collections::HashSet;
use std::default::Default;

use html5ever::interface::tree_builder::{
    ElementFlags, NodeOrText, QuirksMode, TreeSink
};
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::{parse_document, parse_fragment, ExpandedName, QualName};

use crate::vdom::{
    Attribute, Document, Element, Node, NodeData, NodeId
};

mod meta;

pub use self::meta::{a, ns, t};

/// HTML parsing convenience functions.
impl Document {
    /// Parse HTML from UTF-8 bytes in RAM.  For stream based parsing, or
    /// parsing from alternative encodings use [`crate::decode`].
    pub fn parse_html(utf8_bytes: &[u8]) -> Self {
        let sink = Sink {
            document: Document::new(),
            quirks_mode: QuirksMode::NoQuirks,
        };
        parse_document(sink, Default::default())
            .from_utf8()
            .one(utf8_bytes)
    }

    #[allow(unused)] //FIXME
    pub(crate) fn parse_html_fragment(utf8_bytes: &[u8]) -> Self {
        let sink = Sink {
            document: Document::new(),
            quirks_mode: QuirksMode::NoQuirks,
        };

        let mut doc = parse_fragment(
            sink,
            Default::default(),
            QualName::new(None, ns::HTML, t::DIV),
            vec![])
            .from_utf8()
            .one(utf8_bytes);

        // Note that the above context name, doesn't really get used. A
        // matching element is pushed but never linked, so unless we replace
        // the doc (deep clone, etc.) then it will contain this cruft.

        let root_id = doc.root_element().expect("a root");
        debug_assert!(doc[root_id].is_elem(t::HTML));

        // If the root has a single element child, then make that element child
        // the new root and return.
        // FIXME: Only do this, further, if its a block level element?
        if doc.children(root_id).count() == 1 {
            let child_id = doc[root_id].first_child.unwrap();
            if doc[child_id].as_element().is_some() {
                doc.fold(root_id);
                debug_assert!(doc.root_element().is_some());
                return doc;
            }
        }

        // Otherwise change the "html" root to a div. This is what we asked
        // for, but didn't get, from parse_fragment above.
        let root = doc[root_id].as_element_mut().unwrap();
        *root = Element {
            name: QualName::new(None, ns::HTML, t::DIV),
            attrs: vec![]
        };
        debug_assert!(doc.root_element().is_some());
        doc
    }
}

/// A `TreeSink` implementation for parsing html to a [`crate::vdom::Document`]
/// tree.
pub struct Sink {
    document: Document,
    quirks_mode: QuirksMode,
}

impl Sink {
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
                    // FIXME: Frequently done in test, possibly a minor perf
                    // gain over independent text nodes?
                    let node = &mut self.document[id];
                    if let NodeData::Text(t) = &mut node.data {
                        t.push_tendril(&text);
                        return;
                    }
                }
                self.new_node(NodeData::Text(text))
            }
            NodeOrText::AppendNode(node) => node,
        };

        append(&mut self.document, new_node)
    }
}

impl Default for Sink {
    fn default() -> Self {
        Sink {
            document: Document::new(),
            quirks_mode: QuirksMode::NoQuirks,
        }
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
