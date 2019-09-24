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
use std::ops::Deref;

pub use html5ever::LocalName;
pub use html5ever::{Attribute, QualName};
pub use tendril::StrTendril;
pub use html5ever::{ns, local_name as lname};

pub mod html;
mod xml;
mod serializer;
mod filter;

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

impl Clone for Node {
    fn clone(&self) -> Self {
        Node::new(self.data.clone())
    }
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

    /// Return an iterator over the direct children of this node that
    /// match the specified predicate.
    ///
    /// This is a convenence short hand for
    /// `children().filter(predicate)`.
    pub fn filter<P>(&'a self, predicate: P)
        -> impl Iterator<Item = NodeRef<'a>> + 'a
        where P: FnMut(&NodeRef<'a>) -> bool + 'a
    {
        self.children().filter(predicate)
    }

    /// Return an iterator over all decendents of this node that match
    /// the specified predicate.
    ///
    /// Nodes that fail the predicate will have their child nodes
    /// (r)ecursively scanned, in depth-first order, in search of all
    /// matches.
    pub fn filter_r<P>(&'a self, predicate: P) -> Selector<'a, P>
        where P: FnMut(&NodeRef<'a>) -> bool + 'a
    {
        Selector::new(self.doc, self.first_child, predicate)
    }

    /// Find the first direct child of this node that matches the
    /// specified predicate.
    ///
    /// This is a convenence short hand for
    /// `children().find(predicate)`.
    pub fn find<P>(&'a self, predicate: P) -> Option<NodeRef<'a>>
        where P: FnMut(&NodeRef<'a>) -> bool
    {
        self.children().find(predicate)
    }

    /// Find the first descendant of this node that matches the
    /// specified predicate.
    ///
    /// Nodes that fail the predicate will have their child nodes
    /// (r)ecursively scanned, in depth-first order, in search of the
    /// first match.
    pub fn find_r<P>(&'a self, predicate: P) -> Option<NodeRef<'a>>
        where P: FnMut(&NodeRef<'a>) -> bool + 'a
    {
        Selector::new(self.doc, self.first_child, predicate).next()
    }

    /// Return an iterator over node's direct children.
    ///
    /// Will yield nothing if the node can not or does not have children.
    pub fn children(&'a self) -> impl Iterator<Item = NodeRef<'a>> + 'a {
        iter::successors(
            self.for_some_node(self.first_child),
            move |nref| self.for_some_node(nref.next_sibling)
        )
    }

    /// Return an iterator yielding self and all ancestors, terminating at the
    /// root document node.
    pub fn node_and_ancestors(&'a self)
        -> impl Iterator<Item = NodeRef<'a>> + 'a
    {
        iter::successors(
            Some(*self),
            move |nref| self.for_some_node(nref.parent)
        )
    }

    /// Return any parent node or None.
    pub fn parent(&'a self) -> Option<NodeRef<'a>> {
        self.for_some_node(self.parent)
    }

    /// Return all decendent text content (character data) of this node.
    ///
    /// If this is a Text node, return that text.  If this is an
    /// Element node or the Document root node, return the
    /// concatentation of all text descendants, in tree order. Returns
    /// `None` for all other node types.
    pub fn text(&'a self) -> Option<StrTendril> {
        self.doc.text(self.id)
    }

    #[inline]
    fn for_some_node(&'a self, id: Option<NodeId>) -> Option<NodeRef<'a>> {
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

pub struct Selector<'a, P> {
    doc: &'a Document,
    next: Vec<NodeId>,
    predicate: P,
}

impl<'a, P> Selector<'a, P> {
    fn new(doc: &'a Document, first: Option<NodeId>, predicate: P)
        -> Selector<'a, P>
    {
        let next = if let Some(id) = first {
            vec![id]
        } else {
            vec![]
        };

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
                push_if(&mut self.next, node.next_sibling);
                return Some(node);
            } else {
                push_if(&mut self.next, node.next_sibling);
                push_if(&mut self.next, node.first_child);
            }
        }
        None
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
    #[allow(unused)] //FIXME
    pub(crate) fn root_element_ref(&self) -> Option<NodeRef<'_>> {
        self.root_element().map(|r| NodeRef::new(self, r))
    }

    /// Return the root element NodeId for this Document, or None if there is
    /// no element.
    ///
    /// ## Panics
    ///
    /// Panics on various malformed structures, including multiple "root"
    /// elements or a text node as direct child of the Documnent.
    #[allow(unused)] //FIXME
    pub(crate) fn root_element(&self) -> Option<NodeId> {
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
        root
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

    #[allow(unused)] //FIXME
    pub(crate) fn append_child(&mut self, parent: NodeId, node: Node)
        -> NodeId
    {
        let id = self.push_node(node);
        self.append(parent, id);
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

    /// Return all decendent text content (character data) of this node.
    ///
    /// If this is a Text node, return that text.  If this is an
    /// Element node or the Document root node, return the
    /// concatentation of all text descendants, in tree order. Returns
    /// `None` for all other node types.
    pub(crate) fn text(&self, id: NodeId) -> Option<StrTendril> {
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
    #[allow(unused)] //FIXME
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
    #[allow(unused)] //FIXME
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

    /// Create a new document from the ordered sub-tree rooted in the node
    /// referenced by id.
    #[allow(unused)] //FIXME
    pub(crate) fn deep_clone(&self, id: NodeId) -> Document {
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

fn push_if(stack: &mut Vec<NodeId>, id: Option<NodeId>) {
    if let Some(id) = id {
        stack.push(id);
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

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ElementData {
    pub(crate) name: QualName,
    pub(crate) attrs: Vec<Attribute>,
}

impl ElementData {
    /// Get attribute value by local name.
    pub fn attr<LN>(&self, lname: LN) -> Option<&StrTendril>
        where LN: Into<LocalName>
    {
        let lname = lname.into();
        self.attrs
            .iter()
            .find(|attr| attr.name.local == lname)
            .map(|attr| &attr.value)
    }

    pub fn is_elem<LN>(&self, lname: LN) -> bool
        where LN: Into<LocalName>
    {
        self.name.local == lname.into()
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

    pub fn attr<LN>(&self, lname: LN) -> Option<&StrTendril>
        where LN: Into<LocalName>
    {
        if let Some(edata) = self.as_element() {
            edata.attr(lname)
        } else {
            None
        }
    }

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
    let id = doc.append_child(Document::DOCUMENT_NODE_ID, element);

    assert!(doc.root_element_ref().is_some(), "pushed root Element");
    assert_eq!(id, doc.root_element_ref().unwrap().id);
    assert_eq!(2, doc.nodes().count(), "root + 1 element");
}

#[test]
fn test_fold_filter() {
    let mut doc = Document::parse_html(
        "<div>foo <strike><i>bar</i>s</strike> baz</div>"
            .as_bytes()
    );
    doc.filter(&filter::StrikeFoldFilter {});
    assert_eq!(
        "<html><head></head><body>\
         <div>foo <i>bar</i>s baz</div>\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_remove_filter() {
    let mut doc = Document::parse_html(
        "<div>foo <strike><i>bar</i>s</strike> baz</div>"
            .as_bytes()
    );
    doc.filter(&filter::StrikeRemoveFilter {});
    assert_eq!(
        "<html><head></head><body>\
         <div>foo  baz</div>\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_filter_chain() {
    let mut doc = Document::parse_html_fragment(
        "<div>foo<strike><i>bar</i>s</strike> \n\t baz</div>"
            .as_bytes()
    );
    let fltrs = filter::FilterChain::new(vec![
        Box::new(filter::StrikeRemoveFilter {}),
        Box::new(filter::TextNormalizer)
    ]);

    doc.filter(&fltrs);
    assert_eq!(
        "<div>foo baz</div>",
        doc.to_string()
    );
}

#[test]
fn test_xmp() {
    let doc = Document::parse_html_fragment(
        "<div>foo <xmp><i>bar</i></xmp> baz</div>"
            .as_bytes()
    );
    assert_eq!(
        "<div>foo <xmp><i>bar</i></xmp> baz</div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    eprintln!("the doc nodes:\n{:?}", &doc.nodes[2..]);
    assert_eq!(5, doc.nodes.len() - 2);
}

#[test]
fn test_text_fragment() {
    let doc = Document::parse_html_fragment(
        "plain &lt; text".as_bytes()
    );
    assert_eq!(
        "<div>\
         plain &lt; text\
         </div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    eprintln!("the doc nodes:\n{:?}", &doc.nodes[2..]);
    assert_eq!(2, doc.nodes.len() - 2);
}

#[test]
fn test_shallow_fragment() {
    let doc = Document::parse_html_fragment(
        "<b>b</b> text <i>i</i>".as_bytes()
    );
    assert_eq!(
        "<div>\
         <b>b</b> text <i>i</i>\
         </div>",
        doc.to_string()
    );

    // Currently node count is only ensured by cloning
    let doc = doc.deep_clone(doc.root_element().unwrap());
    eprintln!("the doc nodes:\n{:?}", &doc.nodes[2..]);
    assert_eq!(6, doc.nodes.len() - 2);
}

#[test]
fn test_empty_fragment() {
    let doc = Document::parse_html_fragment("".as_bytes());
    eprintln!("the doc nodes:\n{:?}", &doc.nodes[2..]);
    assert_eq!("<div></div>", doc.to_string());
}

#[test]
fn test_deep_clone() {
    let doc = Document::parse_html(
        "<div>foo <a href=\"link\"><i>bar</i>s</a> baz</div>\
         <div>sibling</div>"
            .as_bytes()
    );

    let doc = doc.deep_clone(doc.root_element().expect("root"));
    assert_eq!(
        "<html><head></head><body>\
           <div>foo <a href=\"link\"><i>bar</i>s</a> baz</div>\
           <div>sibling</div>\
         </body></html>",
        doc.to_string()
    );
}

#[test]
fn test_filter() {
    let doc = Document::parse_html(
        "<p>1</p>\
         <div>\
           fill\
           <p>2</p>\
           <p>3</p>\
           <div>\
             <p>4</p>\
             <i>fill</i>\
           </div>\
         </div>"
            .as_bytes()
    );

    let root = doc.root_element_ref().expect("root");
    let body = root.find(|n| n.is_elem(lname!("body"))).expect("body");
    let f1: Vec<_> = body
        .filter(|n| n.is_elem(lname!("p")))
        .map(|n| n.text().unwrap().to_string())
        .collect();

    assert_eq!(f1, vec!["1"]);
}

#[test]
fn test_filter_r() {
    const P: LocalName = lname!("p");

    let doc = Document::parse_html_fragment(
        "<p>1</p>\
         <div>\
           fill\
           <p>2</p>\
           <p>3</p>\
           <div>\
             <p>4</p>\
             <i>fill</i>\
           </div>\
         </div>"
            .as_bytes()
    );

    let root = doc.root_element_ref().expect("root");

    assert_eq!("1fill234fill", root.text().unwrap().to_string());

    let f1: Vec<_> = root
        .filter_r(|n| n.is_elem(P))
        .map(|n| n.text().unwrap().to_string())
        .collect();

    assert_eq!(f1, vec!["1", "2", "3", "4"]);
}

#[test]
fn test_meta_content_type() {
    // element constants
    const HEAD:         LocalName = lname!("head");
    const META:         LocalName = lname!("meta");

    // attribute constants
    const CHARSET:      LocalName = lname!("charset");
    const HTTP_EQUIV:   LocalName = lname!("http-equiv");
    const CONTENT:      LocalName = lname!("content");

    let doc = Document::parse_html(
        r####"
<html xmlns="http://www.w3.org/1999/xhtml">
 <head>
  <meta charset='UTF-8'/>
  <META http-equiv=" CONTENT-TYPE" content="text/html; charset=utf-8"/>
  <title>Iūdex</title>
 </head>
 <body>
  <p>Iūdex test.</p>
 </body>
</html>"####
            .as_bytes()
    );
    let root = doc.root_element_ref().expect("root");
    let head = root.find(|n| n.is_elem(HEAD)).expect("head");
    let metas: Vec<_> = head.filter(|n| n.is_elem(META)).collect();
    let mut found = false;
    for m in metas {
        if let Some(a) = m.attr(CHARSET) {
            eprintln!("meta charset: {}", a);
        } else if let Some(a) = m.attr(HTTP_EQUIV) {
            // FIXME: Parser doesn't normalize whitespace in
            // attributes. Need to trim.
            if a.as_ref().trim().eq_ignore_ascii_case("Content-Type") {
                if let Some(a) = m.attr(CONTENT) {
                    let ctype = a.as_ref().trim();
                    eprintln!("meta content-type: {}", ctype);
                    assert_eq!("text/html; charset=utf-8", ctype);
                    found = true;
                }
            }
        }
    }
    assert!(found);
}
