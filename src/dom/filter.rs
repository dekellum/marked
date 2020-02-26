//! Mutating visitor support for `Document`.

use std::iter;
use std::cell::RefCell;

use log::debug;

use crate::chars::{is_all_ctrl_ws, replace_chars};
use crate::dom::{
    html::{t, TAG_META},
    Document, Element, Node, NodeData, NodeId, StrTendril
};

/// An instruction returned by the `Fn` closure used by [`Document::filter`].
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Continue filtering, without further changes to this `Node`.
    Continue,

    /// Replace this `Node` with its children. Equivalent to `Detach` if
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

/// Detach any effectively pointless inline elements, which contain only logical
/// whitespace.
///
/// Logical whitespace is defined as all Unicode whitespace or control chars in
/// child text, or the `<br>` element. Non-text oriented inline elements like
/// `<img>` and `<video>` and other multi-media are excluded from
/// consideration.
pub fn detach_empty_inline(doc: &Document, node: &mut Node) -> Action {
    if is_inline(node) && !is_multi_media(node) {
        let mut children = iter::successors(
            node.first_child,
            move |&id| doc[id].next_sibling);
        if children.all(|id| is_logical_ws(&doc[id])) {
            return Action::Fold;
        }
    }
    Action::Continue
}

/// Detach any comment nodes.
pub fn detach_comments(_d: &Document, node: &mut Node) -> Action {
    if let NodeData::Comment(_) = node.data {
        Action::Detach
    } else {
        Action::Continue
    }
}

/// Detach any processing instruction nodes.
pub fn detach_pis(_d: &Document, node: &mut Node) -> Action {
    if let NodeData::ProcessingInstruction {..} = node.data {
        Action::Detach
    } else {
        Action::Continue
    }
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

/// Normalize text nodes by merging, replacing control characters and
/// minimizing whitespace.
///
/// The filter is aware of whitespace significance rules in HTML `<pre>` (or
/// similar tag) blocks as well as block vs inline elements in general. It
/// assumes, without knowledge of any potential unconventinal external styling,
/// that leading and trailing whitespace may be removed at block element
/// boundaries.
///
/// Because this filter works on text nodes, depth first, results are better if
/// applied in its own `Document::filter` pass, _after_ any pass containing
/// filters that detach or fold elements, such as
/// [`detach_banned_elements`]. Otherwise the filter may not be able to merge
/// text node's which become siblings in the process, resulting in additional
/// whitespace.
pub fn text_normalize(doc: &Document, node: &mut Node) -> Action {
    thread_local! {
        static MERGE_Q: RefCell<StrTendril> = RefCell::new(StrTendril::new())
    };

    if let NodeData::Text(ref mut t) = node.data {

        // If the immediately folowing sibbling is also Text, then push this
        // tendril to the merge queue and detach.
        let node_r = node.next_sibling.map(|id| &doc[id]);
        if node_r.and_then(Node::as_text).is_some() {
            MERGE_Q.with(|q| {
                q.borrow_mut().push_tendril(t)
            });
            return Action::Detach;
        }

        // Otherwise add this tendril to anything in the queue, consuming it.
        MERGE_Q.with(|q| {
            let mut qt = q.borrow_mut();
            if qt.len() > 0 {
                qt.push_tendril(t);
                drop(qt);
                *t = q.replace(StrTendril::new());
            }
        });

        let node_l = node.prev_sibling.map(|id| &doc[id]);

        let parent = node.parent.unwrap();
        let parent_is_block = is_block(&doc[parent]);
        let in_pre = doc
            .node_and_ancestors(parent)
            .any(|id| is_preform_node(&doc[id]));

        let trim_l = (parent_is_block && node_l.is_none()) ||
            node_l.map_or(false, is_block);

        let trim_r = (parent_is_block && node_r.is_none()) ||
            node_r.map_or(false, is_block);

        replace_chars(t, !in_pre, true, trim_l, trim_r);

        if t.is_empty() {
            return Action::Detach;
        }
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

// Note this isn't an exact negation of `is_block`: it still returns false for
// unknown elements.
fn is_inline(node: &Node) -> bool {
    if let Some(elm) = node.as_element() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            return tmeta.is_inline();
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

fn is_logical_ws(n: &Node) -> bool {
    if let Some(t) = n.as_text() {
        is_all_ctrl_ws(t)
    } else {
        n.is_elem(t::BR)
    }
}

fn is_multi_media(n: &Node) -> bool {
    /**/n.is_elem(t::AUDIO) ||
        n.is_elem(t::EMBED) ||
        n.is_elem(t::IFRAME) ||
        n.is_elem(t::IMG) ||
        n.is_elem(t::METER) ||
        n.is_elem(t::OBJECT) ||
        n.is_elem(t::PICTURE) ||
        n.is_elem(t::PROGRESS) ||
        n.is_elem(t::SVG) ||
        n.is_elem(t::VIDEO)
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
