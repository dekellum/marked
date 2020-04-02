// Copyright © 2019 David Kellum
//
// This DOM-like markup tree module was originally based on `victor::dom`, as
// of commit fdb11f3e8 of the source as found here:
//
// https://github.com/SimonSapin/victor
// (No copyright notice.)
// Licensed under the Apache license v2.0, or the MIT license

//! An efficient and simple DOM-like container and associated tools.

use std::convert::TryInto;
use std::fmt;
use std::iter;
use std::mem;
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};

#[doc(no_inline)]
pub use html5ever::{Attribute, LocalName, Namespace, QualName};

#[doc(no_inline)]
pub use tendril::StrTendril;

// custom ordering of these effects rustdoc for Document, etc.

mod node_ref;
mod serializer;
#[macro_use] pub mod filter;
pub mod html;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(test)]
mod tests;

pub use node_ref::{NodeRef, Selector};

/// A DOM-like container for a tree of markup elements and text.
///
/// Unlike `RcDom`, this uses a simple vector of `Node`s and indexes for
/// parent/child and sibling ordering. Attributes are stored as separately
/// allocated vectors for each element. For memory efficiency, a single
/// document is limited to 4 billion (2^32 - 1) total nodes.
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId(NonZeroU32);

/// A typed node (e.g. text, element, etc.) within a `Document` including
/// identifiers to parent, siblings and children.
#[derive(Clone, Debug)]
pub struct Node {
    data: NodeData,
    parent: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

/// The node kind and payload data associated with that kind.
#[derive(Clone, Debug)]
pub enum NodeData {
    /// A place holder value. Used temporarily while filtering and for nodes
    /// that have been removed.
    Hole,

    /// The document node which contains all other nodes.
    Document,

    /// The document type definition.
    DocType(DocumentType),

    /// Character data content.
    Text(StrTendril),

    /// A comment.
    Comment(StrTendril),

    /// An element.
    Elem(Element),

    /// A processing instruction node.
    Pi(ProcessingInstruction),
}

/// Document type definition details.
#[derive(Clone, Debug)]
pub struct DocumentType {
    pub name: StrTendril,
    _priv: ()
}

/// Processing instruction details.
#[derive(Clone, Debug)]
pub struct ProcessingInstruction {
    pub data: StrTendril,
    _priv: ()
}

/// A markup element with name and attributes.
#[derive(Clone, Debug)]
pub struct Element {
    pub name: QualName,
    pub attrs: Vec<Attribute>,
    _priv: ()
}

/// Core implementation.
impl Document {
    /// The constant `NodeId` for the document node of all `Document`s.
    pub const DOCUMENT_NODE_ID: NodeId = NodeId(
        unsafe { NonZeroU32::new_unchecked(1) }
    );

    /// Construct a new `Document` with the single empty document node.
    pub fn new() -> Self {
        Document::with_capacity(8)
    }

    /// Construct a new `Document` with the single empty document node and
    /// specified capacity.
    pub fn with_capacity(count: u32) -> Self {
        let mut nodes = Vec::with_capacity(count as usize);
        nodes.push(Node::new(NodeData::Hole));        // Index 0: Padding
        nodes.push(Node::new(NodeData::Document));    // Index 1: DOCUMENT_NODE_ID
        Document { nodes }
    }

    /// Return total number of `Node`s.
    ///
    /// This includes the document node and all occupied nodes, some of which
    /// may not be accessable from the document node. The value returned
    /// may be more than the accessable nodes counted via `nodes().count()`,
    /// unless [`Document::compact`] or [`Document::deep_clone`] is first used.
    pub fn len(&self) -> usize {
        self.nodes.len() - 1
    }

    /// Return true if this document only contains the single empty document
    /// node.
    ///
    /// Note that when "empty" the [`Document::len`] is still one (1).
    pub fn is_empty(&self) -> bool {
        self.nodes.len() < 3
    }

    /// Return the root element `NodeId` for this Document, or None if there is
    /// no such qualified element.
    ///
    /// A node with `NodeData::Element` is a root element, if it is a direct
    /// child of the document node, with no other element or text sibling.
    pub fn root_element(&self) -> Option<NodeId> {
        let document_node = &self[Document::DOCUMENT_NODE_ID];
        debug_assert!(
            (if let NodeData::Document = document_node.data { true }
             else { false }),
            "not document node: {:?}", document_node);
        debug_assert!(document_node.parent.is_none());
        debug_assert!(document_node.next_sibling.is_none());
        debug_assert!(document_node.prev_sibling.is_none());
        let mut root = None;
        for child in self.children(Document::DOCUMENT_NODE_ID) {
            match &self[child].data {
                NodeData::DocType(_) |
                NodeData::Comment(_) |
                NodeData::Pi(_) => {}
                NodeData::Document => {
                    debug_assert!(false, "Document child of Document");
                    root = None;
                    break;
                }
                NodeData::Hole => {
                    debug_assert!(false, "Hole in Document");
                    root = None;
                    break;
                }
                NodeData::Text(_) => {
                    root = None;
                    break;
                }
                NodeData::Elem(_) => {
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
        debug_assert!(
            (if let NodeData::Document | NodeData::Hole = node.data { false }
             else { true }),
            "Invalid push {:?}", node.data);
        let next_index = self.nodes.len()
            .try_into()
            .expect("Document (u32) node index overflow");
        debug_assert!(next_index > 1);
        self.nodes.push(node);
        NodeId(unsafe { NonZeroU32::new_unchecked(next_index) })
    }

    /// Detach the specified node ID.
    ///
    /// Panics if called with the synthetic DOCUMENT_NODE_ID.
    /// Detaching the root element results in an empty document with no root
    /// element.
    ///
    /// Detach just removes references from other nodes. To free up the memory
    /// associated with the node and its children, use [`Document::compact`].
    pub fn detach(&mut self, id: NodeId) {
        assert!(
            id != Document::DOCUMENT_NODE_ID,
            "Can't detach the synthetic document node");

        let (parent, prev_sibling, next_sibling) = {
            let node = &mut self[id];
            (node.parent.take(),
             node.prev_sibling.take(),
             node.next_sibling.take())
        };

        if let Some(next_sibling) = next_sibling {
            self[next_sibling].prev_sibling = prev_sibling
        } else if let Some(parent) = parent {
            self[parent].last_child = prev_sibling;
        }

        if let Some(prev_sibling) = prev_sibling {
            self[prev_sibling].next_sibling = next_sibling;
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
        self[parent].assert_suitable_parent();
        if let Some(last_child) = self[parent].last_child.take() {
            self[new_child].prev_sibling = Some(last_child);
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
        let parent = self[sibling].parent
            .expect("insert_before sibling has parent");
        self[parent].assert_suitable_parent();
        self[new_sibling].parent = Some(parent);
        self[new_sibling].next_sibling = Some(sibling);
        if let Some(prev_sibling) = self[sibling].prev_sibling.take() {
            self[new_sibling].prev_sibling = Some(prev_sibling);
            debug_assert_eq!(
                self[prev_sibling].next_sibling,
                Some(sibling)
            );
            self[prev_sibling].next_sibling = Some(new_sibling);
        } else {
            debug_assert_eq!(self[parent].first_child, Some(sibling));
            self[parent].first_child = Some(new_sibling);
        }
        self[sibling].prev_sibling = Some(new_sibling);
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
    pub fn children<'a>(&'a self, id: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(
            self[id].first_child,
            move |&id| self[id].next_sibling
        )
    }

    /// Return an iterator over the specified node and all its following,
    /// direct siblings, within the same parent.
    pub fn node_and_following_siblings<'a>(&'a self, id: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(Some(id), move |&id| self[id].next_sibling)
    }

    /// Return an iterator over the specified node and all its ancestors,
    /// terminating at the document node.
    pub fn node_and_ancestors<'a>(&'a self, id: NodeId)
        -> impl Iterator<Item = NodeId> + 'a
    {
        iter::successors(Some(id), move |&id| self[id].parent)
    }

    /// Return an iterator over all nodes, starting with the document node, and
    /// including all descendants in tree order.
    pub fn nodes<'a>(&'a self) -> impl Iterator<Item = NodeId> + 'a {
        iter::successors(
            Some(Document::DOCUMENT_NODE_ID),
            move |&id| self.next_in_tree_order(id)
        )
    }

    fn next_in_tree_order(&self, id: NodeId) -> Option<NodeId> {
        self[id].first_child.or_else(|| {
            self.node_and_ancestors(id)
                .find_map(|ancestor| self[ancestor].next_sibling)
        })
    }

    /// Compact in place, by removing `Node`s that are no longer referenced
    /// from the document node.
    pub fn compact(&mut self) {
        let mut ndoc = Document::with_capacity(self.len() as u32);
        let mut next = Vec::new();
        push_if_pair(
            &mut next,
            self[Document::DOCUMENT_NODE_ID].first_child,
            Document::DOCUMENT_NODE_ID);

        while let Some((id, nid)) = next.pop() {
            let data = std::mem::replace(&mut self[id].data, NodeData::Hole);
            let ncid = ndoc.append_child(nid, Node::new(data));
            push_if_pair(&mut next, self[id].next_sibling, nid);
            push_if_pair(&mut next, self[id].first_child, ncid);
        }

        ndoc.nodes.shrink_to_fit();

        std::mem::swap(&mut self.nodes, &mut ndoc.nodes);
    }

    /// Create a new `Document` from the ordered sub-tree rooted in the node
    /// referenced by ID.
    pub fn deep_clone(&self, id: NodeId) -> Document {
        let mut ndoc = Document::with_capacity(self.len() as u32 / 2);
        if id == Document::DOCUMENT_NODE_ID {
            for child in self.children(id) {
                ndoc.append_deep_clone(Document::DOCUMENT_NODE_ID, self, child);
            }
        } else {
            ndoc.append_deep_clone(Document::DOCUMENT_NODE_ID, self, id);
        }
        ndoc
    }

    /// Clone node oid in odoc and all its descendants, appending to id in
    /// self.
    pub fn append_deep_clone(&mut self, id: NodeId, odoc: &Document, oid: NodeId) {
        let id = self.append_child(id, Node::new(odoc[oid].data.clone()));
        for child in odoc.children(oid) {
            self.append_deep_clone(id, odoc, child);
        }
    }

    /// Return a clone of self by bulk clone of all `Node`s.
    ///
    /// This clone is performed without regard for what nodes are reachable
    /// from the document node. The [`Document::len`] of the clone will be the
    /// same as the original. As compared with `deep_clone(DOCUMENT_NODE_ID)`
    /// this is faster but potentially much less memory efficient.
    pub fn bulk_clone(&self) -> Document {
        Document { nodes: self.nodes.clone() }
    }

    /// Replace the specified node ID with its children.
    ///
    /// Panics if called with the synthetic DOCUMENT_NODE_ID. Folding the root
    /// element may result in a `Document` with no single root element, or
    /// which is otherwise invalid based on its _doctype_, e.g. the HTML or XML
    /// specifications.
    ///
    /// After repositioning children the specified node is detached, which only
    /// removes references. To free up the memory associated with the node, use
    /// [`Document::compact`]. For a node with no children, fold is equivalent
    /// to [`Document::detach`].
    pub fn fold(&mut self, id: NodeId) {
        assert!(
            id != Document::DOCUMENT_NODE_ID,
            "Can't fold the synthetic document node");

        let mut next_child = self[id].first_child;
        while let Some(child) = next_child {
            debug_assert_eq!(self[child].parent, Some(id));
            next_child = self[child].next_sibling;
            self.insert_before(id, child);
        }
        self.detach(id);
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

impl Element {
    /// Construct new element by local name, with no attributes.
    pub fn new<LN>(lname: LN) -> Element
        where LN: Into<LocalName>
    {
        Element {
            name: QualName::new(None, ns!(), lname.into()),
            attrs: Vec::new(),
            _priv: ()
        }
    }

    /// Return true if this element has the given local name.
    pub fn is_elem<LN>(&self, lname: LN) -> bool
        where LN: Into<LocalName>
    {
        self.name.local == lname.into()
    }

    /// Return [`html::TagMeta`] for this element, if the tag is a known part
    /// of the HTML `Namespace`.
    pub fn html_tag_meta(&self) -> Option<&'static html::TagMeta> {
        if self.name.ns == html::ns::HTML {
            html::TAG_META.get(&self.name.local)
        } else {
            None
        }
    }

    /// Return attribute value by local name, if present.
    pub fn attr<LN>(&self, lname: LN) -> Option<&StrTendril>
        where LN: Into<LocalName>
    {
        let lname = lname.into();
        self.attrs
            .iter()
            .find(|attr| attr.name.local == lname)
            .map(|attr| &attr.value)
    }

    /// Remove attribute by local name, returning any value found.
    ///
    /// This removes _all_ instances of attributes with the given local name
    /// and returns the value of the _last_ such attribute. Parsers may allow
    /// same named attributes or multiples might be introduced via manual
    /// mutations.
    pub fn remove_attr<LN>(&mut self, lname: LN) -> Option<StrTendril>
        where LN: Into<LocalName>
    {
        let mut found = None;
        let mut i = 0;
        let lname = lname.into();
        while i < self.attrs.len() {
            if self.attrs[i].name.local == lname {
                found = Some(self.attrs.remove(i).value);
            } else {
                i += 1;
            }
        }
        found
    }

    /// Set attribute by local name, returning any prior value found.
    ///
    /// This replaces the value of the first attribute with the given local
    /// name and removes any other instances.  If no existing attribute is
    /// found, the attribute is added to the end. To guarantee placement at the
    /// end, use [`Element::remove_attr`] first.  In the case where multiple
    /// existing instances of the attribute are found, the _last_ value is
    /// returned.  Parsers may allow same named attributes or multiples might be
    /// introduced via manual mutations.
    pub fn set_attr<LN, V>(&mut self, lname: LN, value: V) -> Option<StrTendril>
        where LN: Into<LocalName>, V: Into<StrTendril>
    {
        let mut found = None;
        let mut i = 0;
        let lname = lname.into();

        // Need to Option::take value below to appease borrow checking and
        // avoid a clone.
        let mut value = Some(value.into());

        while i < self.attrs.len() {
            if self.attrs[i].name.local == lname {
                if found.is_none() {
                    found = Some(mem::replace(
                        &mut self.attrs[i].value,
                        value.take().unwrap(),
                    ));
                    i += 1;
                } else {
                    found = Some(self.attrs.remove(i).value);
                };
            } else {
                i += 1;
            }
        }
        if found.is_none() {
            self.attrs.push(Attribute {
                name: QualName::new(None, ns!(), lname),
                value: value.take().unwrap()
            });
        }
        found
    }
}

impl Node {
    /// Construct a new element node.
    pub fn new_elem(element: Element) -> Node {
        Node::new(NodeData::Elem(element))
    }

    /// Construct a new text node.
    pub fn new_text<T>(text: T) -> Node
        where T: Into<StrTendril>
    {
        Node::new(NodeData::Text(text.into()))
    }

    fn new(data: NodeData) -> Self {
        Node {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }
}

impl Deref for Node {
    type Target = NodeData;

    #[inline]
    fn deref(&self) -> &NodeData {
        &self.data
    }
}

impl DerefMut for Node {
    #[inline]
    fn deref_mut(&mut self) -> &mut NodeData {
        &mut self.data
    }
}

impl NodeData {
    /// Return `Element` is this is an element.
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            NodeData::Elem(ref data) => Some(data),
            _ => None,
        }
    }

    /// Return mutable `Element` reference if this is an element.
    pub fn as_element_mut(&mut self) -> Option<&mut Element> {
        match self {
            NodeData::Elem(ref mut data) => Some(data),
            _ => None,
        }
    }

    /// Return text (char data) if this is a text node.
    pub fn as_text(&self) -> Option<&StrTendril> {
        match self {
            NodeData::Text(ref t) => Some(t),
            _ => None,
        }
    }

    /// Return mutable text (char data) reference if this is a text node.
    pub fn as_text_mut(&mut self) -> Option<&mut StrTendril> {
        match self {
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

    #[inline]
    fn assert_suitable_parent(&self) {
        debug_assert!(
            (if let NodeData::Document | NodeData::Elem(_) = self { true }
             else { false }),
            "Not a suitable parent: {:?}", self)
    }
}

fn push_if(stack: &mut Vec<NodeId>, id: Option<NodeId>) {
    if let Some(id) = id {
        stack.push(id);
    }
}

fn push_if_pair(
    stack: &mut Vec<(NodeId, NodeId)>,
    id: Option<NodeId>,
    oid: NodeId)
{
    if let Some(id) = id {
        stack.push((id, oid));
    }
}
