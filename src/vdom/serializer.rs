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

use std::io;
use std::io::Write;
use std::string::ToString;

use html5ever::serialize::{
    serialize, Serialize, SerializeOpts, Serializer,
    TraversalScope, TraversalScope::*
};

use crate::vdom::{Document, NodeData, NodeRef};

impl<'a> Serialize for NodeRef<'a> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope)
        -> io::Result<()>
        where S: Serializer
    {
        match (traversal_scope, &self.data) {
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
            &self.document_node_ref(),
            SerializeOpts {
                traversal_scope: ChildrenOnly(None),
                ..Default::default()
            },
        )
    }
}