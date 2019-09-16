// Copyright © 2019 David Kellum
//
// This `Serialize` implementation was originally derived from
// `kuchiki::serializer` source, found here:
//
// https://github.com/kuchiki-rs/kuchiki
// (No copyright notice.)
// Licensed (per Cargo.toml) under the MIT license
//
// This in turn may have been derived in part from html5ever's implementation,
// see:
//
// https://github.com/servo/html5ever
// Copyright © 2014-2017 The html5ever Project Developers.
// Licensed under the Apache license v2.0, or the MIT license

use std::io::Write;
use std::io;
use std::iter;
use std::string::ToString;

use html5ever::serialize::TraversalScope::*;
use html5ever::serialize::{serialize, Serialize, SerializeOpts, Serializer, TraversalScope};

use crate::dom::{Document, NodeId, NodeData};

/// Welcome visitors (the pattern as used for `Serialize`) by combining a
/// `NodeId` with its containing `Document` reference.
struct DocNode<'a>{
    doc: &'a Document,
    id: NodeId
}

impl<'a> DocNode<'a> {
    fn document(doc: &'a Document) -> DocNode<'a> {
        DocNode { doc, id: Document::document_node_id() }
    }

    /// Return an iterator over node's direct children.
    ///
    /// Will be empty if the node can not or does not have children.
    fn children(&'a self) -> impl Iterator<Item = DocNode<'a>> + 'a {
        iter::successors(
            self.for_node(self.doc[self.id].first_child),
            move |dn| self.for_node(dn.doc[dn.id].next_sibling)
        )
    }

    fn for_node(&self, id: Option<NodeId>) -> Option<DocNode<'a>> {
        if let Some(id) = id {
            Some(DocNode { doc: self.doc, id })
        } else {
            None
        }
    }
}

impl<'a> Serialize for DocNode<'a> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope)
        -> io::Result<()>
        where S: Serializer
    {
        let node = &self.doc[self.id];

        match (traversal_scope, &node.data) {
            (ref scope, &NodeData::Element(ref edata)) => {
                if *scope == IncludeNode {
                    serializer.start_elem(
                        edata.name.clone(),
                        edata.attrs.iter().map(|a| (&a.name, a.value.as_ref()))
                    )?;
                }
                for child in self.children() {
                    Serialize::serialize(&child, serializer, IncludeNode)?;
                }

                if *scope == IncludeNode {
                    serializer.end_elem(edata.name.clone())?;
                }
                Ok(())
            }

            (_, &NodeData::Document) => {
                for child in self.children() {
                    Serialize::serialize(&child, serializer, IncludeNode)?;
                }
                Ok(())
            }

            (ChildrenOnly(_), _) => Ok(()),

            (IncludeNode, &NodeData::Doctype { ref name, .. }) => {
                serializer.write_doctype(name)
            }
            (IncludeNode, &NodeData::Text(ref t)) => {
                serializer.write_text(&t)
            }
            (IncludeNode, &NodeData::Comment(ref t)) => {
                serializer.write_comment(&t)
            }
            (IncludeNode,
             &NodeData::ProcessingInstruction { ref target, ref data }) => {
                serializer.write_processing_instruction(&target, &data)
            }
        }
    }
}

impl<'a> ToString for Document {
    fn to_string(&self) -> String {
        let mut u8_vec = Vec::new();
        self.serialize(&mut u8_vec).unwrap();
        String::from_utf8(u8_vec).unwrap()
    }
}

impl Document {
    /// Serialize this node and its descendants in HTML syntax to the given
    /// stream.
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
        where W: Write
    {
        serialize(
            writer,
            &DocNode::document(self),
            SerializeOpts {
                traversal_scope: IncludeNode, //ignored for document
                ..Default::default()
            },
        )
    }
}
