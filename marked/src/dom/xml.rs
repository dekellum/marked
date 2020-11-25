// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

//! Support for XML parsing to `Document` (_xml_ feature).
//!
//! This module is enabled at build time via the _xml_ non-default feature.

use std::error::Error as StdError;
use std::fmt;

use xml_rs::attribute::OwnedAttribute;
use xml_rs::reader::XmlEvent;

use crate::chars::is_all_ctrl_ws;
use crate::dom::{
    Attribute, Document, Element, Node, NodeData, ProcessingInstruction, QualName, StrTendril,
};

/// Parse XML document from UTF-8 bytes in RAM.
pub fn parse_utf8(utf8_bytes: &[u8]) -> Result<Document, XmlError> {
    let mut document = Document::new();
    let mut current = Document::DOCUMENT_NODE_ID;
    let mut ancestors = Vec::new();
    for event in xml_rs::EventReader::new(utf8_bytes) {
        match event.map_err(XmlError)? {
            XmlEvent::StartElement {
                name, attributes, ..
            } => {
                let id = document.push_node(Node::new(NodeData::Elem(Element {
                    name: convert_name(name),
                    attrs: attributes
                        .into_iter()
                        .map(|OwnedAttribute { name, value }| Attribute {
                            name: convert_name(name),
                            value: value.into(),
                        })
                        .collect(),
                    _priv: (),
                })));
                document.append(current, id);
                ancestors.push(current);
                current = id;
            }
            XmlEvent::EndElement { .. } => current = ancestors.pop().unwrap(),
            XmlEvent::CData(s) | XmlEvent::Characters(s) => {
                if let Some(last_child) = document[current].last_child {
                    let node = &mut document[last_child];
                    if let NodeData::Text(t) = &mut node.data {
                        t.push_slice(&s);
                        continue;
                    }
                }
                let id = document.push_node(Node::new(NodeData::Text(s.into())));
                document.append(current, id);
            }
            XmlEvent::Whitespace(s) => {
                debug_assert!(is_all_ctrl_ws(&s.into()));
                //FIXME: Push as above, if in "preserve mode"?
            }
            XmlEvent::ProcessingInstruction { name: _, data } => {
                let data = if let Some(s) = data {
                    s.into()
                } else {
                    StrTendril::new()
                };
                let id = document.push_node(Node::new(NodeData::Pi(ProcessingInstruction {
                    data,
                    _priv: (),
                })));
                document.append(current, id);
            }
            XmlEvent::StartDocument { .. } | XmlEvent::EndDocument | XmlEvent::Comment(_) => {}
        }
    }
    Ok(document)
}

fn convert_name(name: xml_rs::name::OwnedName) -> QualName {
    QualName {
        prefix: name.prefix.map(|p| p.into()),
        ns: name.namespace.map_or(ns!(), |ns| ns.into()),
        local: name.local_name.into(),
    }
}

/// An XML parsing error.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct XmlError(xml_rs::reader::Error);

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for XmlError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}
