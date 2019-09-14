// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

use std::fmt;
use std::error::Error as StdError;

use xml_rs::reader::XmlEvent;
use xml_rs::attribute::OwnedAttribute;

use crate::dom::{
    Attribute, Document, ElementData, Node, NodeData, QualName, StrTendril
};

impl Document {
    pub fn parse_xml(utf8_bytes: &[u8]) -> Result<Self, XmlError> {
        let mut document = Document::new();
        let mut current = Document::document_node_id();
        let mut ancestors = Vec::new();
        for event in xml_rs::EventReader::new(utf8_bytes) {
            match event.map_err(XmlError)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    let id = document.push_node(Node::new(NodeData::Element(ElementData {
                        name: convert_name(name),
                        attrs: attributes
                            .into_iter()
                            .map(|OwnedAttribute { name, value }| Attribute {
                                name: convert_name(name),
                                value: value.into()
                            })
                            .collect(),
                        mathml_annotation_xml_integration_point: false,
                    })));
                    document.append(current, id);
                    ancestors.push(current);
                    current = id;
                }
                XmlEvent::EndElement { .. } => current = ancestors.pop().unwrap(),
                XmlEvent::CData(s) | XmlEvent::Characters(s) | XmlEvent::Whitespace(s) => {
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
                XmlEvent::ProcessingInstruction { name, data } => {
                    let data = if let Some(s) = data {
                        s.into()
                    } else {
                        StrTendril::new()
                    };
                    let id = document.push_node(Node::new(NodeData::ProcessingInstruction {
                        target: name.into(),
                        data
                    }));
                    document.append(current, id);
                }
                XmlEvent::StartDocument { .. } | XmlEvent::EndDocument | XmlEvent::Comment(_) => {}
            }
        }
        Ok(document)
    }
}

fn convert_name(name: xml_rs::name::OwnedName) -> QualName {
    QualName {
        prefix: name.prefix.map(|p| p.into()),
        ns: name.namespace.map_or(ns!(), |ns| ns.into()),
        local: name.local_name.into(),
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct XmlError(xml_rs::reader::Error);

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl StdError for XmlError {
    fn description(&self) -> &str {
        self.0.description()
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}
