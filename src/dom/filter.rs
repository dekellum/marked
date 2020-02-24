//! Mutating visitor support for `Document`.

use log::debug;

use crate::chars::replace_chars;
use crate::dom::{
    html::{t, TAG_META},
    Document, Element, Node, NodeData, NodeId
};

/// An instruction returned by the `Fn` closure used by [`Document::filter`].
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Continue filtering, without further changes to this `Node`.
    Continue,

    /// Replace this `Node` with its children. Equivalent to `Remove` if
    /// returned for a `Node` with no children.
    Fold,

    /// Detach this `Node`, and any children, from the tree.
    Detach,
}

/// Remove known banned-elements
/// [`TagMeta::is_banned`](crate::html::TagMeta::is_banned) and any elements
/// which are unknown.
pub fn detach_banned_elements(_d: &Document, node: &mut Node) -> Action {
    if let Some(ref mut elm) = node.as_element_mut() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            if tmeta.is_banned() {
                return Action::Detach;
            }
        } else {
            debug!("Detaching unknown element tag {}", &elm.name.local);
            return Action::Detach;
        }
    }
    Action::Continue
}

/// Filter out attributes that are not included in the "basic" set
/// [`TagMeta`](crate::html::TagMeta) for each element.
pub fn retain_basic_attributes(_d: &Document, node: &mut Node) -> Action {
    if let Some(ref mut elm) = node.as_element_mut() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            elm.attrs.retain(|a| tmeta.has_basic_attr(&a.name.local));
        } else {
            debug!("unknown tag {:?}, attributes unmodified", &elm.name.local);
        }
    }
    Action::Continue
}

/// Normalize text nodes by replacing control characters and minimizing
/// whitespace.
///
/// The filter is aware of whitespace significance rules in HTML `<pre>` (or
/// similar tag) blocks as well as block vs inline elements in general. It
/// assumes, without knowledge of any potential unconventinal external styling,
/// that leading and trailing whitespace may be removed at block element
/// boundaries.
pub fn text_normalize(doc: &Document, node: &mut Node) -> Action {
    if let NodeData::Text(ref mut t) = node.data {
        let parent = node.parent.unwrap();
        let parent_is_block = is_block(&doc[parent]);
        let in_pre = doc
            .node_and_ancestors(parent)
            .find(|id| is_preform_node(&doc[*id]))
            .is_some();
        let trim_l = parent_is_block &&
            (node.prev_sibling.is_none() ||
             is_block(&doc[node.prev_sibling.unwrap()]));
        let trim_r = parent_is_block &&
            (node.next_sibling.is_none() ||
             is_block(&doc[node.next_sibling.unwrap()]));

        replace_chars(t, !in_pre, true, trim_l, trim_r);
    }
    Action::Continue
}

// FIXME: Consider also offering a simpler version of the above for XML or
// where speed trumps precision.

/// Convert any `<xmp>`, `<listing>`, or `<plaintext>` elements to `<pre>`.
///
/// The `<xmp>`, `<listing>` and `<plaintext>` tags are deprecated in later
/// HTML versions and are XML/XHTML incompatible, but can still can be found in
/// the wild.  After the HTML parse where special internal markup rules are
/// applied, these are roughly equivelent to `<pre>`, and its safer if
/// converted.
pub fn xmp_to_pre(_doc: &Document, node: &mut Node) -> Action {
    if let Some(ref mut elm) = node.as_element_mut() {
        if is_preformatted(elm) {
            elm.name.local = t::PRE;
        }
    }
    Action::Continue
}

fn is_block(node: &Node) -> bool {
    if let Some(elm) = node.as_element() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            return !tmeta.is_inline();
        }
    }
    false
}

fn is_preformatted(e: &Element) -> bool {
    e.is_elem(t::PRE) || e.is_elem(t::XMP) || e.is_elem(t::PLAINTEXT)
}

fn is_preform_node(n: &Node) -> bool {
    n.is_elem(t::PRE) || n.is_elem(t::XMP) || n.is_elem(t::PLAINTEXT)
}

/// Mutating filter methods.
impl Document {
    /// Perform a depth-first (e.g. children before parent nodes) walk of the
    /// entire document, from the document root node, allowing the provided
    /// function to make changes to each `Node`.
    pub fn filter<F>(&mut self, mut f: F)
        where F: Fn(&Document, &mut Node) -> Action
    {
        self.filter_at(Document::DOCUMENT_NODE_ID, &mut f);
    }

    /// Perform a depth-first (e.g. children before parent nodes) walk from the
    /// specified node ID, allowing the provided function to make changes to
    /// each `Node`.
    pub fn filter_at<F>(&mut self, id: NodeId, f: &mut F) -> Action
        where F: Fn(&Document, &mut Node) -> Action
    {
        let mut next_child = self[id].first_child;
        while let Some(child) = next_child {
            next_child = self[child].next_sibling;
            match self.filter_at(child, f) {
                Action::Continue => {},
                Action::Fold => {
                    self.fold(child);
                    // next child set above, these children already walked
                }
                Action::Detach => {
                    self.detach(child);
                }
            }
        }

        // Safety: The filter `Fn` needs only a non-mutable reference to
        // Document, but borrow-check doesn't allow this.  We hold a mutable
        // (self) reference across the call and no safe mutations of `Node` can
        // invalidate the `Document`. `Node` itself contains no references to
        // `Document`, only indexes and its own data.
        let d = &*self as *const Document;
        f(unsafe { &*d }, &mut self[id])
    }
}

/// Compose a new filter closure, by chaining a list of 1 to many closures or
/// function paths. Each is executed in order, while the return action remains
/// `Continue`.
#[macro_export]
macro_rules! chain_filters {
    ($solo:expr $(,)?) => (
        |doc: & $crate::Document, node: &mut $crate::Node| {
            $solo(doc, node)
        }
    );
    ($first:expr $(, $subs:expr)+ $(,)?) => (
        |doc: & $crate::Document, node: &mut $crate::Node| {
            let mut action: $crate::filter::Action = $first(doc, node);
        $(
            if action == $crate::filter::Action::Continue {
                action = $subs(doc, node);
            }
        )*
            action
        }
    );
}
