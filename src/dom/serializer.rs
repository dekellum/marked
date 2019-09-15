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

use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::string::ToString;

use html5ever::serialize::TraversalScope::*;
use html5ever::serialize::{serialize, Serialize, SerializeOpts, Serializer, TraversalScope};

use crate::dom::{Document, NodeId, NodeData};

/// Welcome visitors (the pattern as used for `Serialize`) by combining a
/// `NodeId` with its containing `Document` reference.
struct DocNode<'a>(&'a Document, NodeId);

impl<'a> Serialize for DocNode<'a> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope)
        -> io::Result<()>
        where S: Serializer
    {
        let node = &self.0[self.1];

        match (traversal_scope, &node.data) {
            (ref scope, &NodeData::Element(ref edata)) => {
                if *scope == IncludeNode {
                    serializer.start_elem(
                        edata.name.clone(),
                        edata.attrs.iter().map(|a| (&a.name, &a.value[..]))
                    )?
                }

                if let Some(c) = node.first_child {
                    for child in self.0.node_and_following_siblings(c) {
                        Serialize::serialize(
                            &DocNode(self.0, child),
                            serializer,
                            IncludeNode
                        )?
                    }
                }

                if *scope == IncludeNode {
                    serializer.end_elem(edata.name.clone())?
                }
                Ok(())
            }

            (_, &NodeData::Document) => {
                if let Some(c) = node.first_child {
                    for child in self.0.node_and_following_siblings(c) {
                        Serialize::serialize(
                            &DocNode(self.0, child),
                            serializer,
                            IncludeNode
                        )?
                    }
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
             &NodeData::ProcessingInstruction { ref target, ref data }
            ) => {
                serializer.write_processing_instruction(&target, &data)
            }
        }
    }
}

impl<'a> ToString for Document {
    #[inline]
    fn to_string(&self) -> String {
        let mut u8_vec = Vec::new();
        self.serialize(&mut u8_vec).unwrap();
        String::from_utf8(u8_vec).unwrap()
    }
}

impl Document {
    /// Serialize this node and its descendants in HTML syntax to the given
    /// stream.
    #[inline]
    pub fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        serialize(
            writer,
            &DocNode(self, Document::document_node_id()),
            SerializeOpts {
                traversal_scope: IncludeNode,
                ..Default::default()
            },
        )
    }

    /// Serialize this node and its descendants in HTML syntax to a new file at
    /// the given path.
    #[inline]
    pub fn serialize_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()>
    {
        let mut file = File::create(&path)?;
        self.serialize(&mut file)
    }
}
