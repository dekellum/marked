// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

use std::borrow::Cow;
use std::fmt;
use std::iter::successors;

use html5ever::LocalName;
pub use html5ever::{Attribute, QualName};
pub use tendril::StrTendril;

pub mod html;
pub mod xml;
mod serializer;

pub use xml::XmlError;

pub struct Document {
    nodes: Vec<Node>,
}

pub struct Node {
    pub(crate) parent: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) data: NodeData,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeId(std::num::NonZeroUsize);

impl Document {
    fn new() -> Self {
        // Dummy node at index 0 so that other indices fit in NonZero
        let dummy = Node::new(NodeData::Document);

        let document_node = Node::new(NodeData::Document);
        Document {
            nodes: vec![dummy, document_node],
        }
    }

    pub(crate) fn document_node_id() -> NodeId {
        NodeId(std::num::NonZeroUsize::new(1).unwrap())
    }

    /// (rel_attribute, href_attribute)
    pub fn html_link_elements(&self) -> impl Iterator<Item = (&str, &str)> {
        self.nodes()
            .filter_map(move |node| self[node].as_element())
            .filter(|e| e.name.expanded() == expanded_name!(html "link"))
            .filter_map(|e| {
                match (e.get_attr(&local_name!("rel")),
                       e.get_attr(&local_name!("href")))
                {
                    (Some(rel), Some(href)) => Some((rel, href)),
                    _ => None,
                }
            })
    }

    pub fn root_element(&self) -> NodeId {
        let document_node = &self[Document::document_node_id()];
        assert!(match document_node.data {
            NodeData::Document => true,
            _ => false
        });
        assert!(document_node.parent.is_none());
        assert!(document_node.next_sibling.is_none());
        assert!(document_node.previous_sibling.is_none());
        let mut root = None;
        for child in self.node_and_following_siblings(
            document_node.first_child.unwrap())
        {
            match &self[child].data {
                NodeData::Doctype { .. }
                | NodeData::Comment(_)
                | NodeData::ProcessingInstruction { .. } => {}
                NodeData::Document | NodeData::Text(_) => {
                    panic!("Unexpected node type under document node");
                }
                NodeData::Element(_) => {
                    assert!(root.is_none(), "Found two root elements");
                    root = Some(child);
                }
            }
        }
        root.unwrap()
    }

    fn push_node(&mut self, node: Node) -> NodeId {
        let next_index = self.nodes.len();
        self.nodes.push(node);
        NodeId(std::num::NonZeroUsize::new(next_index).unwrap())
    }

    fn detach(&mut self, node: NodeId) {
        let (parent, previous_sibling, next_sibling) = {
            let node = &mut self[node];
            (
                node.parent.take(),
                node.previous_sibling.take(),
                node.next_sibling.take(),
            )
        };

        if let Some(next_sibling) = next_sibling {
            self[next_sibling].previous_sibling = previous_sibling
        } else if let Some(parent) = parent {
            self[parent].last_child = previous_sibling;
        }

        if let Some(previous_sibling) = previous_sibling {
            self[previous_sibling].next_sibling = next_sibling;
        } else if let Some(parent) = parent {
            self[parent].first_child = next_sibling;
        }
    }

    fn append(&mut self, parent: NodeId, new_child: NodeId) {
        self.detach(new_child);
        self[new_child].parent = Some(parent);
        if let Some(last_child) = self[parent].last_child.take() {
            self[new_child].previous_sibling = Some(last_child);
            debug_assert!(self[last_child].next_sibling.is_none());
            self[last_child].next_sibling = Some(new_child);
        } else {
            debug_assert!(self[parent].first_child.is_none());
            self[parent].first_child = Some(new_child);
        }
        self[parent].last_child = Some(new_child);
    }

    fn insert_before(&mut self, sibling: NodeId, new_sibling: NodeId) {
        self.detach(new_sibling);
        self[new_sibling].parent = self[sibling].parent;
        self[new_sibling].next_sibling = Some(sibling);
        if let Some(previous_sibling) = self[sibling].previous_sibling.take() {
            self[new_sibling].previous_sibling = Some(previous_sibling);
            debug_assert_eq!(
                self[previous_sibling].next_sibling,
                Some(sibling)
            );
            self[previous_sibling].next_sibling = Some(new_sibling);
        } else if let Some(parent) = self[sibling].parent {
            debug_assert_eq!(self[parent].first_child, Some(sibling));
            self[parent].first_child = Some(new_sibling);
        }
        self[sibling].previous_sibling = Some(new_sibling);
    }

    /// <https://dom.spec.whatwg.org/#concept-child-text-content>
    pub fn child_text_content(&self, node: NodeId) -> Cow<'_, StrTendril> {
        let mut link = self[node].first_child;
        let mut text = None;
        while let Some(child) = link {
            if let NodeData::Text(t) = &self[child].data {
                match &mut text {
                    None => text = Some(Cow::Borrowed(t)),
                    Some(text) => text.to_mut().push_tendril(&t),
                }
            }
            link = self[child].next_sibling;
        }
        text.unwrap_or_else(|| Cow::Owned(StrTendril::new()))
    }

    pub(crate) fn node_and_following_siblings<'a>(
        &'a self,
        node: NodeId,
    ) -> impl Iterator<Item = NodeId> + 'a {
        successors(Some(node), move |&node| self[node].next_sibling)
    }

    pub(crate) fn node_and_ancestors<'a>(
        &'a self,
        node: NodeId,
    ) -> impl Iterator<Item = NodeId> + 'a {
        successors(Some(node), move |&node| self[node].parent)
    }

    fn next_in_tree_order(&self, node: NodeId) -> Option<NodeId> {
        self[node].first_child.or_else(|| {
            self.node_and_ancestors(node)
                .find_map(|ancestor| self[ancestor].next_sibling)
        })
    }

    pub(crate) fn nodes<'a>(&'a self) -> impl Iterator<Item = NodeId> + 'a {
        let root = Self::document_node_id();
        successors(Some(root), move |&node| self.next_in_tree_order(node))
    }
}

impl std::ops::Index<NodeId> for Document {
    type Output = Node;

    #[inline]
    fn index(&self, id: NodeId) -> &Node {
        &self.nodes[id.0.get()]
    }
}

impl std::ops::IndexMut<NodeId> for Document {
    #[inline]
    fn index_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0.get()]
    }
}

pub(crate) enum NodeData {
    Document,
    Doctype {
        name: String,
        _public_id: String,
        _system_id: String,
    },
    Text(StrTendril),
    Comment(StrTendril),
    Element(ElementData),
    ProcessingInstruction {
        target: StrTendril,
        data: StrTendril,
    },
}

pub struct ElementData {
    pub(crate) name: QualName,
    pub(crate) attrs: Vec<Attribute>,
    pub(crate) mathml_annotation_xml_integration_point: bool,
}

impl ElementData {
    pub(crate) fn get_attr(&self, name: &LocalName) -> Option<&str> {
        self.attrs
            .iter()
            .find(|attr| attr.name.ns == ns!() && attr.name.local == *name)
            .map(|attr| &*attr.value)
    }
}

impl Node {
    pub fn as_element(&self) -> Option<&ElementData> {
        match self.data {
            NodeData::Element(ref data) => Some(data),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&StrTendril> {
        match self.data {
            NodeData::Text(ref t) => Some(t),
            _ => None,
        }
    }

    fn new(data: NodeData) -> Self {
        Node {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr: *const Node = self;
        f.debug_tuple("Node").field(&ptr).finish()
    }
}

#[test]
#[cfg(target_pointer_width = "64")]
fn size_of() {
    use std::mem::size_of;
    assert_eq!(size_of::<Node>(), 120);
    assert_eq!(size_of::<NodeData>(), 80);
    assert_eq!(size_of::<ElementData>(), 64);
}
