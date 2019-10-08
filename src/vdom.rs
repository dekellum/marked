// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

use std::convert::TryInto;
use std::fmt;
use std::iter;
use std::num::NonZeroU32;

pub use html5ever::{Attribute, LocalName, Namespace, QualName};
pub use tendril::StrTendril;

pub mod filter;
pub mod html;

mod node_ref;
mod serializer;
mod xml;

pub use xml::XmlError;
pub use node_ref::{NodeRef, Selector};

/// A DOM-like container for a tree of markup elements and text.
///
/// Unlike `RcDom`, this uses a simple vector of `Node`s and indexes for
/// parent/child and sibling ordering. Attributes are stored as separately
/// allocated vectors for each element. For memory efficiency, a single
/// document is limited to 4 billion (2^32) total nodes.
///
/// All `Document` instances, even logically "empty" ones as freshly
/// constructed, contain a synthetic document node at the fixed
/// `DOCUMENT_NODE_ID` that serves as a container for N top level nodes,
/// including the `root_element` if present.
pub struct Document {
    nodes: Vec<Node>,
}

/// A `Node` identifier, as u32 index into a `Document`s `Node` vector.
///
/// Should only be used with the `Document` it was obtained from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeId(NonZeroU32);

/// A typed node (e.g. text, element, etc.) within a `Document`.
#[derive(Debug)]
pub struct Node {
    parent: Option<NodeId>,
    next_sibling: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    data: NodeData,
}

#[derive(Clone, Debug)]
enum NodeData {
    Document,
    Doctype {
        name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    },
    Text(StrTendril),
    Comment(StrTendril),
    Element(ElementData),
    ProcessingInstruction {
        target: StrTendril,
        data: StrTendril,
    },
}

/// A markup element with name and attributes.
#[derive(Clone, Debug)]
pub struct ElementData {
    pub name: QualName,
    pub attrs: Vec<Attribute>,
}

impl Document {
    /// The constant `NodeId` for the document node of all `Document`s.
    pub const DOCUMENT_NODE_ID: NodeId = NodeId(
        unsafe { NonZeroU32::new_unchecked(1) }
    );

    /// Construct a new `Document` with the single empty document node.
    pub fn new() -> Self {
        Document { nodes: vec![
            Node::new(NodeData::Document), // dummy padding, index 0
            Node::new(NodeData::Document)  // the real root, index 1
        ]}
    }

    /// Return the root element `NodeId` for this Document, or None if there is
    /// no such qualified element.
    ///
    /// A node with `NodeData::Element` is a root element, if it is a direct
    /// child of the document node, with no other element or text sibling.
    pub fn root_element(&self) -> Option<NodeId> {
        let document_node = &self[Document::DOCUMENT_NODE_ID];
        debug_assert!(match document_node.data {
            NodeData::Document => true,
            _ => false
        });
        debug_assert!(document_node.parent.is_none());
        debug_assert!(document_node.next_sibling.is_none());
        debug_assert!(document_node.previous_sibling.is_none());
        let mut root = None;
        for child in self.children(Document::DOCUMENT_NODE_ID) {
            match &self[child].data {
                NodeData::Doctype { .. }
                | NodeData::Comment(_)
                | NodeData::ProcessingInstruction { .. } => {}
                NodeData::Document => {
                    panic!("Document child of Document");
                }
                NodeData::Text(_) => {
                    root = None;
                    break;
                }
                NodeData::Element(_) => {
                    if root.is_none() {
                        root = Some(child);
                    } else {
                        root = None; // Only one accepted
                        break;
                    }
                }
            }
        }
        root
    }

    fn push_node(&mut self, node: Node) -> NodeId {
        let next_index = self.nodes.len()
            .try_into()
            .expect("vdom::Document (u32) node index overflow");
        debug_assert!(next_index > 1);
        self.nodes.push(node);
        NodeId(unsafe { NonZeroU32::new_unchecked(next_index) })
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

    /// Append node as new last child of parent, and return its new ID.
    pub fn append_child(&mut self, parent: NodeId, node: Node)
        -> NodeId
    {
        let id = self.push_node(node);
        self.append(parent, id);
        id
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

    /// Insert node before the given sibling and return its new ID.
    pub fn insert_before_sibling(&mut self, sibling: NodeId, node: Node)
        -> NodeId
    {
        let id = self.push_node(node);
        self.insert_before(sibling, id);
        id
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

    /// Return all decendent text content (character data) of the given node
    /// ID.
    ///
    /// If node is a text node, return that text.  If this is an element node
    /// or the document node, return the concatentation of all text
    /// descendants, in tree order. Return `None` for all other node types.
    pub fn text(&self, id: NodeId) -> Option<StrTendril> {
        let mut next = Vec::new();
        push_if(&mut next, self[id].first_child);
        let mut text = None;
        while let Some(id) = next.pop() {
            let node = &self[id];
            if let NodeData::Text(t) = &node.data {
                match &mut text {
                    None => text = Some(t.clone()),
                    Some(text) => text.push_tendril(&t),
                }
                push_if(&mut next, node.next_sibling);
            } else {
                push_if(&mut next, node.next_sibling);
                push_if(&mut next, node.first_child);
            }
        }
        text
    }

    /// Return an iterator over this node's direct children.
    ///
    /// Will be empty if the node can not or does not have children.
    pub fn children<'a>(&'a self, node: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(
            self[node].first_child,
            move |&node| self[node].next_sibling
        )
    }

    /// Return an iterator over the specified node and all its following,
    /// direct siblings, within the same parent.
    pub fn node_and_following_siblings<'a>(&'a self, node: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(Some(node), move |&node| self[node].next_sibling)
    }

    /// Return an iterator over the specified node and all its ancestors,
    /// terminating at the document node.
    pub fn node_and_ancestors<'a>(&'a self, node: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(Some(node), move |&node| self[node].parent)
    }

    /// Return an iterator over all nodes, starting with the document node, and
    /// including all descendants in tree order.
    pub fn nodes<'a>(&'a self) -> impl Iterator<Item = NodeId> + 'a {
        iter::successors(
            Some(Document::DOCUMENT_NODE_ID),
            move |&node| self.next_in_tree_order(node)
        )
    }

    fn next_in_tree_order(&self, node: NodeId) -> Option<NodeId> {
        self[node].first_child.or_else(|| {
            self.node_and_ancestors(node)
                .find_map(|ancestor| self[ancestor].next_sibling)
        })
    }

    /// Create a new `Document` from the ordered sub-tree rooted in the node
    /// referenced by ID.
    pub fn deep_clone(&self, id: NodeId) -> Document {
        let mut ndoc = Document::new();
        ndoc.deep_clone_to(Document::DOCUMENT_NODE_ID, self, id);
        ndoc
    }

    fn deep_clone_to(&mut self, id: NodeId, odoc: &Document, oid: NodeId) {
        let id = self.append_child(id, odoc[oid].clone());
        for child in odoc.children(oid) {
            self.deep_clone_to(id, odoc, child);
        }
    }
}

impl Default for Document {
    fn default() -> Document {
        Document::new()
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&self.nodes[1..]).finish()
    }
}

impl std::ops::Index<NodeId> for Document {
    type Output = Node;

    #[inline]
    fn index(&self, id: NodeId) -> &Node {
        &self.nodes[id.0.get() as usize]
    }
}

impl std::ops::IndexMut<NodeId> for Document {
    #[inline]
    fn index_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0.get() as usize]
    }
}

impl ElementData {
    /// Return attribute value by local attribute name, if present.
    pub fn attr<LN>(&self, lname: LN) -> Option<&StrTendril>
        where LN: Into<LocalName>
    {
        let lname = lname.into();
        self.attrs
            .iter()
            .find(|attr| attr.name.local == lname)
            .map(|attr| &attr.value)
    }

    /// Return true if this element has the given local name.
    pub fn is_elem<LN>(&self, lname: LN) -> bool
        where LN: Into<LocalName>
    {
        self.name.local == lname.into()
    }
}

impl Node {

    /// Construct a new element node by name and attributes.
    pub fn new_element(name: QualName, attrs: Vec<Attribute>) -> Node {
        Node::new(NodeData::Element(
            ElementData { name, attrs }
        ))
    }

    /// Construct a new text node.
    pub fn new_text<T>(text: T) -> Node
        where T: Into<StrTendril>
    {
        Node::new(NodeData::Text(text.into()))
    }

    /// Return `ElementData` is this is an element.
    pub fn as_element(&self) -> Option<&ElementData> {
        match self.data {
            NodeData::Element(ref data) => Some(data),
            _ => None,
        }
    }

    /// Return mutable `ElementData` reference if this is an element.
    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match self.data {
            NodeData::Element(ref mut data) => Some(data),
            _ => None,
        }
    }

    /// Return text (char data) if this is a text node.
    pub fn as_text(&self) -> Option<&StrTendril> {
        match self.data {
            NodeData::Text(ref t) => Some(t),
            _ => None,
        }
    }

    /// Return mutable text (char data) reference if this is a text node.
    pub fn as_text_mut(&mut self) -> Option<&mut StrTendril> {
        match self.data {
            NodeData::Text(ref mut t) => Some(t),
            _ => None,
        }
    }

    /// Return attribute value by given local attribute name, if this is an
    /// element with that attribute present.
    pub fn attr<LN>(&self, lname: LN) -> Option<&StrTendril>
        where LN: Into<LocalName>
    {
        if let Some(edata) = self.as_element() {
            edata.attr(lname)
        } else {
            None
        }
    }

    /// Return true if this Node is an element with the given local name.
    pub fn is_elem<LN>(&self, lname: LN) -> bool
        where LN: Into<LocalName>
    {
        if let Some(edata) = self.as_element() {
            edata.is_elem(lname)
        } else {
            false
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

impl Clone for Node {
    fn clone(&self) -> Self {
        Node::new(self.data.clone())
    }
}

fn push_if(stack: &mut Vec<NodeId>, id: Option<NodeId>) {
    if let Some(id) = id {
        stack.push(id);
    }
}

#[cfg(test)]
mod tests;
