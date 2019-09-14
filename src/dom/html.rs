// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

use std::borrow::Cow;
use std::collections::HashSet;
use std::default::Default;

use html5ever::interface::tree_builder::{
    ElementFlags, NodeOrText, QuirksMode, TreeSink
};
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::{self, parse_document, ExpandedName, QualName};

use crate::dom::{Attribute, Document, ElementData, Node, NodeData, NodeId};

impl Document {
    pub fn parse_html(utf8_bytes: &[u8]) -> Self {
        let sink = Sink {
            document: Document::new(),
            quirks_mode: QuirksMode::NoQuirks,
        };
        parse_document(sink, Default::default())
            .from_utf8()
            .one(utf8_bytes)
    }
}

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
                    if let Node {
                        data: NodeData::Text { contents },
                        ..
                    } = &mut self.document[id]
                    {
                        contents.push_tendril(&text);
                        return;
                    }
                }
                self.new_node(NodeData::Text {
                    contents: text.into(),
                })
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
        Document::document_node_id()
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

    fn is_mathml_annotation_xml_integration_point(&self, &target: &NodeId)
        -> bool
    {
        self.document[target]
            .as_element()
            .expect("not an element")
            .mathml_annotation_xml_integration_point
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        ElementFlags {
            mathml_annotation_xml_integration_point,
            ..
        }: ElementFlags)
        -> NodeId
    {
        self.new_node(NodeData::Element(ElementData {
            name,
            attrs,
            mathml_annotation_xml_integration_point,
        }))
    }

    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.new_node(NodeData::Comment {
            contents: text.into(),
        })
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril)
        -> NodeId
    {
        self.new_node(NodeData::ProcessingInstruction {
            target: target.into(),
            contents: data.into(),
        })
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
            name: name.into(),
            _public_id: public_id.into(),
            _system_id: system_id.into(),
        });
        self.document.append(Document::document_node_id(), node)
    }

    fn add_attrs_if_missing(
        &mut self,
        &target: &NodeId,
        attrs: Vec<Attribute>)
    {
        // FIXME: Never called in our normal/test usage thus far?
        let node = &mut self.document[target];
        let element = if let NodeData::Element(e) = &mut node.data {
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
            self.document.append(new_parent, child);
            next_child = self.document[child].next_sibling
        }
    }
}
