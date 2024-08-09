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

pub use node_ref::{NodeRef, Descender, Selector};

/// A DOM-like container for a tree of markup elements and text.
///
/// Unlike `RcDom`, this uses a simple vector of [`Node`]s and indexes for
/// parent/child and sibling ordering. Attributes are stored as separately
/// allocated vectors for each element. For memory efficiency, a single
/// document is limited to 4 billion (2^32 - 1) total nodes.
///
/// All `Document` instances, even logically "empty" ones as freshly
/// constructed, contain a synthetic document node at the fixed
/// [`Document::DOCUMENT_NODE_ID`] that serves as a container for N top level
/// nodes, including the [`Document::root_element()`], if present.
pub struct Document {
    nodes: Vec<Node>,
}

/// A `Node` identifier as a u32 index into a `Document`s `Node` vector.
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

#[allow(clippy::manual_non_exhaustive)] // Maybe?
/// Document type definition details.
#[derive(Clone, Debug)]
pub struct DocumentType {
    pub name: StrTendril,
    _priv: ()
}

#[allow(clippy::manual_non_exhaustive)] // Maybe?
/// Processing instruction details.
#[derive(Clone, Debug)]
pub struct ProcessingInstruction {
    pub data: StrTendril,
    _priv: ()
}

#[allow(clippy::manual_non_exhaustive)] // Maybe?
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

    // An accepted amount of excess Vec<Node> capacity
    const WASTE_ALLOWANCE: usize = 1024;

    /// Construct a new `Document` with the single empty document node.
    pub fn new() -> Self {
        Document::with_capacity(8)
    }

    /// Construct a new `Document` with the single empty document node and
    /// specified capacity.
    pub fn with_capacity(count: u32) -> Self {
        let mut nodes = Vec::with_capacity(count as usize);
        nodes.push(Node::new(NodeData::Hole));     // Index 0: Padding
        nodes.push(Node::new(NodeData::Document)); // Index 1: DOCUMENT_NODE_ID
        Document { nodes }
    }

    /// Return total number of `Node`s.
    ///
    /// This includes the document node and all occupied nodes, some of which
    /// may not be accessable from the document node. The value returned
    /// may be more than the accessable nodes counted via `nodes().count()`,
    /// unless [`Document::compact`] or [`Document::deep_clone`] is first used.
    #[inline]
    pub fn len(&self) -> u32 {
        let nodes: u32 = self.nodes.len()
            .try_into()
            .expect("Document (u32) node index overflow");
        debug_assert!(nodes > 0);
        nodes - 1 // but don't include padding (index 0) in len
    }

    /// Return true if this document only contains the single, empty document
    /// node.
    ///
    /// Note that when "empty" the [`Document::len`] is still one (1).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() < 2
    }

    /// Return the root element `NodeId` for this Document, or None if there is
    /// no such qualified element.
    ///
    /// A node with `NodeData::Element` is a root element, if it is a direct
    /// child of the document node, with no other element or text sibling.
    pub fn root_element(&self) -> Option<NodeId> {
        let document_node = &self[Document::DOCUMENT_NODE_ID];
        debug_assert!(
        	matches!(document_node.data, NodeData::Document),
            "not document node: {document_node:?}",
        );
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
        	! matches!(node.data, NodeData::Document | NodeData::Hole),
            "Invalid push {:?}", node.data,
        );
        let next_index = self.nodes.len()
            .try_into()
            .expect("Document (u32) node index overflow");
        debug_assert!(next_index > 1);
        self.nodes.push(node);
        NodeId(unsafe { NonZeroU32::new_unchecked(next_index) })
    }

    /// Detach the specified node ID and return it and its children moved into
    /// a new independent `Document` fragment.
    ///
    /// If the complete `Document` sub-tree fragment isn't needed, use
    /// [`Document::unlink`] instead.
    ///
    /// Panics if called with the synthetic DOCUMENT_NODE_ID.
    /// Detaching the root element results in a document with no root
    /// element.
    ///
    /// Detach just removes references and replaces all node data in self with
    /// `NodeData::Hole`. To free up the `Vec<Node>` slots for these nodes as
    /// well, use [`Document::compact`].
    #[inline]
    #[must_use="If the fragment isn't needed, use `unlink()` instead."]
    pub fn detach(&mut self, id: NodeId) -> Document {
        self.unlink_only(id);

        // Not a great guess of the subtree, but faster than counting
        let guess_cap = std::cmp::max(8, self.len() - id.0.get() + 2);
        let mut ndoc = Document::with_capacity(guess_cap);

        ndoc.append_move(Document::DOCUMENT_NODE_ID, self, id);

        // If guess cap was higher then allowance, shrink it
        if (ndoc.nodes.capacity() - ndoc.nodes.len()) >
            Document::WASTE_ALLOWANCE
        {
            ndoc.nodes.shrink_to_fit();
        }
        ndoc
    }

    /// Attach the contents of an other `Document` to self, by appending
    /// its nodes under the given parent node.
    ///
    /// The `Document` is consumed (its contents moved to self). This is an
    /// inverse of [`Document::detach`].
    pub fn attach_child(&mut self, parent: NodeId, mut other: Document) {
        self.nodes.reserve(other.len() as usize - 1); // ignore DOCUMENT node
        let children = other.children(Document::DOCUMENT_NODE_ID)
            .collect::<Vec<_>>();
        for child in children {
            self.append_move(parent, &mut other, child);
        }
    }

    /// Attach the contents of an other `Document` to self, by inserting its
    /// nodes before the given sibling node.
    ///
    /// The `Document` is consumed (its contents moved to self). This is an
    /// inverse of [`Document::detach`].
    pub fn attach_before_sibling(
        &mut self,
        sibling: NodeId,
        mut other: Document)
    {
        self.nodes.reserve(other.len() as usize - 1);
        let children = other.children(Document::DOCUMENT_NODE_ID)
            .collect::<Vec<_>>();
        for oid in children {
            let onode = &mut other[oid];
            let nid = self.insert_before_sibling(
                sibling,
                Node::new(onode.take_data()));
            for coid in other.children(oid).collect::<Vec<_>>() {
                self.append_move(nid, &mut other, coid);
            }
        }
    }

    /// Move node oid in odoc and all its descendants, appending to id in
    /// self.
    fn append_move(&mut self, id: NodeId, odoc: &mut Document, oid: NodeId) {
        let id = self.append_child(id, Node::new(odoc[oid].take_data()));
        let mut ns = NodeStack2::new();
        ns.push_if(odoc[oid].first_child, id);

        while let Some((oid, id)) = ns.pop() {
            let onode = &mut odoc[oid];
            let nid = self.append_child(id, Node::new(onode.take_data()));
            ns.push_if(onode.next_sibling, id);
            ns.push_if(onode.first_child, nid);
        }
    }

    /// Unlink the specified node ID from the Document, and return the replaced
    /// `NodeData`.
    ///
    /// Panics if called with the synthetic DOCUMENT_NODE_ID.
    /// Unlinking the root element results in an document with no root
    /// element.
    ///
    /// Unlink removes references and replaces the single node data with
    /// `NodeData::Hole`, leaving all children in place but un-referenced. Use
    /// [`Document::detach`] to instead obtain the entire sub-tree. To free up
    /// the `Vec<Node>` slots for the node and any children, use
    /// [`Document::compact`].
    #[inline]
    pub fn unlink(&mut self, id: NodeId) -> NodeData {
        self.unlink_only(id);
        self[id].take_data()
    }

    /// Unlink the specified node from the Document.
    fn unlink_only(&mut self, id: NodeId) {
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

    /// Append node as new last child of given parent, and return its new ID.
    pub fn append_child(&mut self, parent: NodeId, node: Node)
        -> NodeId
    {
        let id = self.push_node(node);
        self.append(parent, id);
        id
    }

    fn append(&mut self, parent: NodeId, new_child: NodeId) {
        self.unlink_only(new_child);
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
        self.unlink_only(new_sibling);
        let parent = self[sibling].parent
            .expect("insert_before sibling has no parent");
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

    /// Return all descendant text content (character data) of the given node.
    ///
    /// If node is a text node, return that text.  If this is an element node
    /// or the document node, return the concatentation of all text
    /// descendants, in tree order. Return `None` for all other node types.
    pub fn text(&self, id: NodeId) -> Option<StrTendril> {
        let mut ns = NodeStack1::new();
        ns.push_if(self[id].first_child);
        let mut text = None;
        while let Some(id) = ns.pop() {
            let node = &self[id];
            if let NodeData::Text(t) = &node.data {
                match &mut text {
                    None => text = Some(t.clone()),
                    Some(text) => text.push_tendril(t),
                }
                ns.push_if(node.next_sibling);
            } else {
                ns.push_if(node.next_sibling);
                ns.push_if(node.first_child);
            }
        }
        text
    }

    /// Return an iterator over the given node's direct children.
    ///
    /// Will be empty if the node does not (or can not) have children.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        iter::successors(
            self[id].first_child,
            move |&id| self[id].next_sibling
        )
    }

    /// Return an iterator over the specified node and all its following,
    /// direct siblings, within the same parent.
    pub fn node_and_following_siblings(&self, id: NodeId)
        -> impl Iterator<Item = NodeId> + '_
    {
        iter::successors(Some(id), move |&id| self[id].next_sibling)
    }

    /// Return an iterator over the specified node and all its ancestors,
    /// terminating at the document node.
    pub fn node_and_ancestors(&self, id: NodeId)
        -> impl Iterator<Item = NodeId> + '_
    {
        iter::successors(Some(id), move |&id| self[id].parent)
    }

    /// Return an iterator over all nodes, starting with the document node, and
    /// including all descendants in tree order.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.descendants(Document::DOCUMENT_NODE_ID)
    }

    /// Return an iterator over all descendants in tree order, starting with
    /// the specified node.
    #[inline]
    pub fn descendants(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_
    {
        NodeRef::new(self, id).descendants().map(|nr| nr.id())
    }

    /// Compact in place, by removing `Node`s that are no longer referenced
    /// from the document node.
    pub fn compact(&mut self) {
        let mut ndoc = Document::with_capacity(self.len() + 1);
        let mut ns = NodeStack2::new();
        ns.push_if(
            self[Document::DOCUMENT_NODE_ID].first_child,
            Document::DOCUMENT_NODE_ID);

        while let Some((id, nid)) = ns.pop() {
            let nnode = Node::new(self[id].take_data());
            let ncid = ndoc.append_child(nid, nnode);
            ns.push_if(self[id].next_sibling, nid);
            ns.push_if(self[id].first_child, ncid);
        }

        // If guess cap was higher then allowance, shrink it
        if (ndoc.nodes.capacity() - ndoc.nodes.len()) >
            Document::WASTE_ALLOWANCE
        {
            ndoc.nodes.shrink_to_fit();
        }

        self.nodes = ndoc.nodes;
    }

    /// Create a new `Document` from the ordered sub-tree rooted in the node
    /// referenced by ID.
    pub fn deep_clone(&self, id: NodeId) -> Document {

        // Conservative guess of sub tree length. Shrinking after is likely
        // expensive in this case, so avoid it.
        let guess_cap = std::cmp::max(8, (self.len() - id.0.get() + 2) / 8);
        let mut ndoc = Document::with_capacity(guess_cap);

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
    pub fn append_deep_clone(
        &mut self,
        id: NodeId,
        odoc: &Document,
        oid: NodeId)
    {
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

    /// Replace the specified node ID with its children, and return the
    /// replaced `NodeData`.
    ///
    /// Panics if called with the synthetic DOCUMENT_NODE_ID. Folding the root
    /// element may result in a `Document` with no single root element, or
    /// which is otherwise invalid based on its _doctype_, e.g. the HTML or XML
    /// specifications.
    ///
    /// After repositioning children the specified node is
    /// [unlinked][Document::unlink], removing references, then its data is
    /// replaced with `NodeData::Hole` and the original is returned. To free up
    /// the remaining `Vec<Node>` slot for the node, use
    /// [`Document::compact`]. For a node with no children, fold is equivalent
    /// to [`Document::unlink`].
    #[inline]
    pub fn fold(&mut self, id: NodeId) -> NodeData {
        self.fold_only(id);
        self[id].take_data()
    }

    /// Replace the specified node ID with its children.
    fn fold_only(&mut self, id: NodeId) {
        assert!(
            id != Document::DOCUMENT_NODE_ID,
            "Can't fold the synthetic document node");

        let mut next_child = self[id].first_child;
        while let Some(child) = next_child {
            debug_assert_eq!(self[child].parent, Some(id));
            next_child = self[child].next_sibling;
            self.insert_before(id, child);
        }
        self.unlink_only(id);
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
    /// returned.  Parsers may allow same named attributes or multiples might
    /// be introduced via manual mutations.
    pub fn set_attr<LN, V>(&mut self, lname: LN, value: V)
        -> Option<StrTendril>
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

    /// Replace this node's data with a `NodeData::Hole`, and return the
    /// original `NodeData`.
    fn take_data(&mut self) -> NodeData {
        // This remains private because if the Hole is reachable from
        // DOCUMENT_NODE_ID node may assert panic.
        mem::replace(&mut self.data, NodeData::Hole)
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
        	matches!(self, NodeData::Document | NodeData::Elem(_)),
            "Not a suitable parent: {:?}", self,
        )
    }
}

struct NodeStack1(Vec<NodeId>);

impl NodeStack1 {
    #[inline]
    fn new() -> Self {
        NodeStack1(Vec::with_capacity(16))
    }

    #[inline]
    fn push_if(&mut self, id: Option<NodeId>) {
        if let Some(id) = id {
            self.0.push(id);
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<NodeId> {
        self.0.pop()
    }
}

struct NodeStack2(Vec<(NodeId, NodeId)>);

impl NodeStack2 {
    #[inline]
    fn new() -> Self {
        NodeStack2(Vec::with_capacity(16))
    }

    #[inline]
    fn push_if(&mut self, id: Option<NodeId>, oid: NodeId) {
        if let Some(id) = id {
            self.0.push((id, oid));
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<(NodeId, NodeId)> {
        self.0.pop()
    }
}
