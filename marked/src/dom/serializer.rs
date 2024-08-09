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

use crate::dom::{Document, NodeData, NodeRef};

impl<'a> Serialize for NodeRef<'a> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: TraversalScope)
        -> io::Result<()>
        where S: Serializer
    {
        use NodeData::*;

        match (traversal_scope, &self.data) {
            (ref scope, Elem(ref elm)) => {
                if *scope == IncludeNode {
                    serializer.start_elem(
                        elm.name.clone(),
                        elm.attrs.iter().map(|a| (&a.name, a.value.as_ref()))
                    )?;
                }
                for child in self.children() {
                    Serialize::serialize(&child, serializer, IncludeNode)?;
                }

                if *scope == IncludeNode {
                    serializer.end_elem(elm.name.clone())?;
                }
                Ok(())
            }

            (_, Hole) => {
                panic!("Hole in Document")
            }

            (_, Document) => {
                for child in self.children() {
                    Serialize::serialize(&child, serializer, IncludeNode)?;
                }
                Ok(())
            }

            (ChildrenOnly(_), _) => Ok(()),

            (IncludeNode, DocType(ref dt)) => {
                serializer.write_doctype(&dt.name)
            }
            (IncludeNode, Text(ref t)) => {
                serializer.write_text(t)
            }
            (IncludeNode, Comment(ref t)) => {
                serializer.write_comment(t)
            }
            (IncludeNode, Pi(ref pi)) => {
                serializer.write_processing_instruction("", &pi.data)
            }
        }
    }
}

#[allow(clippy::to_string_trait_impl)]
/// Implemented via [`Document::serialize`].
impl ToString for Document {
    fn to_string(&self) -> String {
        let mut u8_vec = Vec::new();
        self.serialize(&mut u8_vec).unwrap();
        unsafe { String::from_utf8_unchecked(u8_vec) }
    }
}

/// Serialize convenience method.
impl Document {
    /// Serialize the contents of the document node and descendants in HTML
    /// syntax to the given stream.
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

/// Serialize convenience method.
impl<'a> NodeRef<'a> {
    /// Serialize the referenced node and its descendants in HTML syntax to the
    /// given stream.
    pub fn serialize<W>(&'a self, writer: &mut W) -> io::Result<()>
        where W: Write
    {
        serialize(
            writer,
            self,
            SerializeOpts {
                traversal_scope: IncludeNode,
                ..Default::default()
            },
        )
    }
}

#[allow(clippy::to_string_trait_impl)]
/// Implemented via [`NodeRef::serialize`].
impl<'a> ToString for NodeRef<'a> {
    fn to_string(&self) -> String {
        let mut u8_vec = Vec::new();
        self.serialize(&mut u8_vec).unwrap();
        unsafe { String::from_utf8_unchecked(u8_vec) }
    }
}
