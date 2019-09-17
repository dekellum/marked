// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

use std::borrow::Cow;
use std::convert::TryInto;
use std::fmt;
use std::iter;
use std::num::NonZeroU32;
use std::ops::Deref;

use html5ever::LocalName;
pub use html5ever::{Attribute, QualName};
pub use tendril::StrTendril;

pub mod html;
pub mod xml;
mod serializer;

pub use xml::XmlError;

/// A DOM-like container for a tree of markup elements and text.
///
/// Unlike `RcDom`, this uses a simple vector of `Node`s and indexes for
/// parent/child and sibling ordering. Attributes are stored as separately
/// allocated vectors for each element. For memory efficiency, a single
/// document is limited to 4 billion (2^32) total nodes.
pub struct Document {
    nodes: Vec<Node>,
}

/// A typed node (e.g. text, element, etc.) within a `Document`.
#[derive(Debug)]
pub struct Node {
    pub(crate) parent: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    pub(crate) data: NodeData,
}

/// A `Node` identifier, as u32 index into a `Document`s `Node` vector.
///
/// Should only be used with the `Document` it was obtained from.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NodeId(NonZeroU32);

/// A `Node` within `Document` lifetime reference.
#[derive(Copy, Clone)]
pub struct NodeRef<'a>{
    pub(crate) doc: &'a Document,
    pub(crate) id: NodeId
}

impl<'a> NodeRef<'a> {

    #[inline]
    fn new(doc: &'a Document, id: NodeId) -> Self {
        NodeRef { doc, id }
    }

    /// Return the associated `NodeId`
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Return an iterator over node's direct children.
    ///
    /// Will yield nothing if the node can not or does not have children.
    pub fn children(&'a self) -> impl Iterator<Item = NodeRef<'a>> + 'a {
        iter::successors(
            self.for_node(self.first_child),
            move |nref| self.for_node(nref.next_sibling)
        )
    }

    /// Return an iterator yielding self and all ancestors, terminating at the
    /// root document node.
    pub fn node_and_ancestors(&'a self)
        -> impl Iterator<Item = NodeRef<'a>> + 'a
    {
        iter::successors(
            Some(*self),
            move |nref| self.for_node(nref.parent)
        )
    }

    /// Return any parent node or None.
    pub fn parent(&'a self) -> Option<NodeRef<'a>> {
        self.for_node(self.parent)
    }

    #[inline]
    fn for_node(&'a self, id: Option<NodeId>) -> Option<NodeRef<'a>> {
        if let Some(id) = id {
            Some(NodeRef::new(self.doc, id))
        } else {
            None
        }
    }
}

impl<'a> Deref for NodeRef<'a> {
    type Target = Node;

    #[inline]
    fn deref(&self) -> &Node {
        &self.doc[self.id]
    }
}

impl PartialEq for NodeRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        (std::ptr::eq(self.doc, other.doc) && self.id == other.id)
    }
}

impl fmt::Debug for NodeRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeRef({:p}, {:?})", self.doc, self.id)
    }
}

impl Document {

    /// The constant `NodeId` for the document root node of all `Document`s.
    pub const DOCUMENT_NODE_ID: NodeId = NodeId(
        unsafe { NonZeroU32::new_unchecked(1) }
    );

    fn new() -> Self {
        Document { nodes: vec![
            Node::new(NodeData::Document), // dummy padding, index 0
            Node::new(NodeData::Document)  // the real root, index 1
        ]}
    }

    /// Return the document root node reference.
    pub fn document_node_ref(&self) -> NodeRef<'_> {
        NodeRef::new(self, Document::DOCUMENT_NODE_ID)
    }

    /// Return the root element node for this Document, or None if there is no
    /// element.
    ///
    /// ## Panics
    ///
    /// Panics on various malformed structures, including multiple "root"
    /// elements or a text node as direct child of the Documnent.
    #[allow(unused)]
    pub(crate) fn root_element_ref(&self) -> Option<NodeRef<'_>> {
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
                NodeData::Document | NodeData::Text(_) => {
                    panic!("Unexpected node type under document node");
                }
                NodeData::Element(_) => {
                    assert!(root.is_none(), "Found two root elements");
                    root = Some(child);
                }
            }
        }
        root.map(|r| NodeRef::new(self, r))
    }

    fn push_node(&mut self, node: Node) -> NodeId {
        let next_index = self.nodes.len()
            .try_into()
            .expect("dom::Document (u32) Node index overflow");
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

    /// Return concatenation of all text under the given node, in tree
    /// order. May return the empty string.
    ///
    /// <https://dom.spec.whatwg.org/#concept-child-text-content>
    #[allow(unused)]
    pub(crate) fn child_text_content(&self, node: NodeId) -> Cow<'_, StrTendril> {
        // FIXME: What if the initial node is a text node?
        // FIXME: Use children iterator?
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
        // FIXME: Use an option for empty case instead?
        text.unwrap_or_else(|| Cow::Owned(StrTendril::new()))
    }

    /// Return an iterator over this node's direct children.
    ///
    /// Will be empty if the node can not or does not have children.
    pub(crate) fn children<'a>(&'a self, node: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(
            self[node].first_child,
            move |&node| self[node].next_sibling
        )
    }

    /// Return an iterator over the specified node and all its following,
    /// direct siblings, within the same parent.
    #[allow(unused)]
    pub(crate) fn node_and_following_siblings<'a>(&'a self, node: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(Some(node), move |&node| self[node].next_sibling)
    }

    /// Return an iterator over the specified node and all its ancestors,
    /// terminating at the root document node.
    pub(crate) fn node_and_ancestors<'a>(&'a self, node: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(Some(node), move |&node| self[node].parent)
    }

    /// Return an iterator over all nodes, starting with the Document node, and
    /// including all descendants in tree order.
    #[allow(unused)]
    pub(crate) fn nodes<'a>(&'a self) -> impl Iterator<Item = NodeId> + 'a {
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

#[derive(Debug)]
pub(crate) enum NodeData {
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
#[derive(Debug)]
pub struct ElementData {
    pub(crate) name: QualName,
    pub(crate) attrs: Vec<Attribute>,
}

impl ElementData {
    /// Get attribute value by local name.
    pub fn attr_local(&self, name: &LocalName) -> Option<&str> {
        self.attrs
            .iter()
            .find(|attr| &attr.name.local == name)
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

#[test]
#[cfg(target_pointer_width = "64")]
fn size_of() {
    use std::mem::size_of;
    assert_eq!(size_of::<Node>(), 88);
    assert_eq!(size_of::<NodeData>(), 64);
    assert_eq!(size_of::<ElementData>(), 56);
    assert_eq!(size_of::<Attribute>(), 48);
    assert_eq!(size_of::<Vec<Attribute>>(), 24);
    assert_eq!(size_of::<QualName>(), 32);
    assert_eq!(size_of::<StrTendril>(), 16);
}

#[test]
fn empty_document() {
    let doc = Document::new();
    assert_eq!(None, doc.root_element_ref(), "no root Element");
    assert_eq!(1, doc.nodes().count(), "one Document node");
}

#[test]
fn one_element() {
    let mut doc = Document::new();
    let element = Node::new(NodeData::Element(
        ElementData {
            name: QualName::new(None, ns!(), "one".into()),
            attrs: vec![]
        }
    ));
    let id = doc.push_node(element);
    doc.append(Document::DOCUMENT_NODE_ID, id);

    assert!(doc.root_element_ref().is_some(), "pushed root Element");
    assert_eq!(id, doc.root_element_ref().unwrap().id);
    assert_eq!(2, doc.nodes().count(), "root + 1 element");
}
