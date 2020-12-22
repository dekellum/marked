use std::fmt;
use std::iter;
use std::ops::Deref;

use crate::dom::{Document, Node, NodeId, StrTendril, NodeStack1};

/// A `Node` within `Document` lifetime reference.
///
/// This provides convenient but necessarily read-only access.
#[derive(Copy, Clone)]
pub struct NodeRef<'a>{
    doc: &'a Document,
    id: NodeId
}

impl<'a> NodeRef<'a> {
    /// Constructor.
    #[inline]
    pub fn new(doc: &'a Document, id: NodeId) -> Self {
        NodeRef { doc, id }
    }

    /// Return the associated `NodeId`.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Return an iterator over the direct children of this node that
    /// match the specified predicate.
    ///
    /// This is a convenence short hand for `children().filter(predicate)`. The
    /// "filter" name is avoided in deference to the (mutating)
    /// `Document::filter` method.
    pub fn select_children<P>(&self, predicate: P)
        -> impl Iterator<Item = NodeRef<'a>> + 'a
        where P: FnMut(&NodeRef<'a>) -> bool + 'a
    {
        self.children().filter(predicate)
    }

    /// Return an iterator over all decendents of this node that match
    /// the specified predicate.
    ///
    /// When element nodes fail the predicate, their children are scanned,
    /// depth-first, in search of all matches.
    pub fn select<P>(&self, predicate: P) -> Selector<'a, P>
        where P: FnMut(&NodeRef<'a>) -> bool + 'a
    {
        Selector::new(self.doc, self.first_child, predicate)
    }

    /// Find the first direct child of this node that matches the
    /// specified predicate.
    ///
    /// This is a convenence short hand for `children().find(predicate)`.
    pub fn find_child<P>(&self, predicate: P) -> Option<NodeRef<'a>>
        where P: FnMut(&NodeRef<'a>) -> bool
    {
        self.children().find(predicate)
    }

    /// Find the first descendant of this node that matches the specified
    /// predicate.
    ///
    /// When element nodes fail the predicate, their children are scanned,
    /// depth-first, in search of the first match.
    pub fn find<P>(&self, predicate: P) -> Option<NodeRef<'a>>
        where P: FnMut(&NodeRef<'a>) -> bool + 'a
    {
        Selector::new(self.doc, self.first_child, predicate).next()
    }

    /// Return an iterator over node's direct children.
    ///
    /// Will be empty if the node does not (or can not) have children.
    pub fn children(&self) -> impl Iterator<Item = NodeRef<'a>> + 'a {
        let this = *self;
        iter::successors(
            this.for_some_node(this.first_child),
            move |nref| this.for_some_node(nref.next_sibling)
        )
    }

    /// Return an iterator over all descendants in tree order, starting with
    /// the specified node.
    pub fn descendants(&self) -> Descender<'a>
    {
        Descender::new(*self)
    }

    /// Return an iterator yielding self and all ancestors, terminating at the
    /// document node.
    pub fn node_and_ancestors(&'a self)
        -> impl Iterator<Item = NodeRef<'a>> + 'a
    {
        iter::successors(
            Some(*self),
            move |nref| self.for_some_node(nref.parent)
        )
    }

    /// Return any parent node or None.
    pub fn parent(&self) -> Option<NodeRef<'a>> {
        self.for_some_node(self.parent)
    }

    /// Return any previous (left) sibling node or None.
    pub fn prev_sibling(&self) -> Option<NodeRef<'a>> {
        self.for_some_node(self.prev_sibling)
    }

    /// Return any subsequent next (right) sibling node or None.
    pub fn next_sibling(&self) -> Option<NodeRef<'a>> {
        self.for_some_node(self.next_sibling)
    }

    /// Return all decendent text content (character data) of this node.
    ///
    /// If this is a Text node, return that text.  If this is an
    /// Element node or the Document root node, return the
    /// concatentation of all text descendants, in tree order. Returns
    /// `None` for all other node types.
    pub fn text(&self) -> Option<StrTendril> {
        self.doc.text(self.id)
    }

    /// Create a new independent `Document` from the ordered sub-tree
    /// referenced by self.
    pub fn deep_clone(&self) -> Document {
        self.doc.deep_clone(self.id)
    }

    #[inline]
    fn for_some_node(&self, id: Option<NodeId>) -> Option<NodeRef<'a>> {
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

/// Equivalence is defined here for `NodeRef`s if and only if they reference
/// the _same_ `Document` (by identity) and with equal `NodeId`s.
impl PartialEq for NodeRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.doc, other.doc) && self.id == other.id
    }
}

impl fmt::Debug for NodeRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeRef({:p}, {:?})", self.doc, self.id)
    }
}

/// A selecting iterator returned by [`NodeRef::select`].
pub struct Selector<'a, P> {
    doc: &'a Document,
    next: NodeStack1,
    predicate: P,
}

impl<'a, P> Selector<'a, P> {
    fn new(doc: &'a Document, first: Option<NodeId>, predicate: P) -> Self {
        let mut next = NodeStack1::new();
        next.push_if(first);
        Selector { doc, next, predicate }
    }
}

impl<'a, P> Iterator for Selector<'a, P>
    where P: FnMut(&NodeRef<'a>) -> bool + 'a
{
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(id) = self.next.pop() {
            let node = NodeRef::new(self.doc, id);
            if (self.predicate)(&node) {
                self.next.push_if(node.next_sibling);
                return Some(node);
            } else {
                self.next.push_if(node.next_sibling);
                self.next.push_if(node.first_child);
            }
        }
        None
    }
}

/// A depth-first iterator returned by [`NodeRef::descendents`].
pub struct Descender<'a> {
    doc: &'a Document,
    first: Option<NodeId>,
    next: NodeStack1
}

impl<'a> Descender<'a> {
    fn new(first: NodeRef<'a>) -> Self {
        let mut next = NodeStack1::new();
        next.push_if(first.first_child);
        Descender { doc: first.doc, first: Some(first.id), next }
    }
}

impl<'a> Iterator for Descender<'a>
{
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.first.take() {
            return Some(NodeRef::new(self.doc, id));
        }
        if let Some(id) = self.next.pop() {
            let node = NodeRef::new(self.doc, id);
            self.next.push_if(node.next_sibling);
            self.next.push_if(node.first_child);
            Some(node)
        } else {
            None
        }
    }
}

/// `NodeRef` convenence accessor methods.
impl Document {
    /// Return the (single, always present) document node as a `NodeRef`.
    pub fn document_node_ref(&self) -> NodeRef<'_> {
        NodeRef::new(self, Document::DOCUMENT_NODE_ID)
    }

    /// Return the root element `NodeRef` for this `Document`, or `None` if
    /// there is no such qualified element.
    ///
    /// A node with `NodeData::Element` is a root element, if it is a direct
    /// child of the document node, with no other element nor text sibling.
    pub fn root_element_ref(&self) -> Option<NodeRef<'_>> {
        self.root_element().map(|r| NodeRef::new(self, r))
    }
}
