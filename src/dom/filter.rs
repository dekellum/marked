//! Mutating visitor support for `Document`.

use std::cell::RefCell;

use log::debug;

use crate::chars::{is_all_ctrl_ws, replace_chars};
use crate::dom::{
    html::{t, TAG_META},
    Document, Element, NodeData, NodeId, NodeRef, StrTendril
};

/// An instruction returned by the `Fn` closure used by [`Document::filter`].
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Continue filtering, without further changes to this `Node`.
    Continue,

    /// Detach this `Node`, and any children, from the tree.
    Detach,

    /// Replace this `Node` with its children. Equivalent to `Detach` if
    /// returned for a `Node` with no children.
    Fold,
}

/// Mutating filter methods.
impl Document {
    /// Perform a depth-first (e.g. children before parent nodes) walk of the
    /// entire `Document`, including synthetic document node, applying the
    /// provided function.
    ///
    /// See [`Document::filter_at`] for additional details.
    pub fn filter<F>(&mut self, mut f: F)
        where F: Fn(NodeRef<'_>, &mut NodeData) -> Action
    {
        self.filter_at_ref(Document::DOCUMENT_NODE_ID, true, &mut f);
    }


    pub fn filter_breadth<F>(&mut self, mut f: F)
        where F: Fn(NodeRef<'_>, &mut NodeData) -> Action
    {
        self.filter_at_ref(Document::DOCUMENT_NODE_ID, false, &mut f);
    }

    /// Perform a depth-first (e.g. children before parent nodes) walk from the
    /// specified node ID, applying the provided function.
    ///
    /// The `Fn` can be a closure or free-function in the form:
    ///
    /// ```norun
    /// fn a_filter_fn(pos: NodeRef<'_>, data: &mut NodeData) -> Action;
    /// ```
    ///
    /// Where `data` provides read-write access to the the `NodeData` of the
    /// current node being visited, and `pos` gives a read-only view to the
    /// remainder of the `Document`, e.g. parent, children, and siblings of the
    /// current node. Note that to avoid aliasing issues, the `NodeData` is
    /// actually moved out of the `Document` and replaced with a
    /// `NodeData::Hole` value which could be observed via `pos`. The
    /// potentially modified `NodeData` is moved back to the `Document` if the
    /// function returns `Action::Continue`. The function may also modify the
    /// `Document` by returning other [`Action`] values.
    ///
    /// For convenience and efficiency, multiple filter functions can be
    /// combined via the [`chain_filters`] macro and run in one pass.
    ///
    /// Note that to free up all memory associated with filtered `Node`s that
    /// have been detached, use [`Document::deep_clone`] and drop the original
    /// `Document.`.
    pub fn filter_at<F>(&mut self, id: NodeId, mut f: F)
        where F: Fn(NodeRef<'_>, &mut NodeData) -> Action
    {
        self.filter_at_ref(id, true, &mut f);
    }

    fn filter_at_ref<F>(&mut self, id: NodeId, depth_first: bool, f: &mut F)
        -> Action
        where F: Fn(NodeRef<'_>, &mut NodeData) -> Action
    {
        let res = self.walk(id, depth_first, f);
        match res {
            Action::Continue => {},
            Action::Fold => {
                self.fold(id);
            }
            Action::Detach => {
                self.detach(id);
            }
        }
        res
    }

    fn walk<F>(&mut self, id: NodeId, depth_first: bool, f: &mut F) -> Action
        where F: Fn(NodeRef<'_>, &mut NodeData) -> Action
    {
        if !depth_first {
            let res = self.filter_node(id, f);
            if res != Action::Continue {
                return res;
            }
        }

        // Children first, recursively:
        let mut next_child = self[id].first_child;
        while let Some(child) = next_child {
            // set before possible loss by filter action
            let fchild = self[child].first_child;
            next_child = self[child].next_sibling;
            let res = self.filter_at_ref(child, depth_first, f);
            if !depth_first && res == Action::Fold {
                next_child = fchild;
            }
        }

        if depth_first {
            self.filter_node(id, f)
        } else {
            Action::Continue
        }
    }

    fn filter_node<F>(&mut self, id: NodeId, f: &mut F) -> Action
        where F: Fn(NodeRef<'_>, &mut NodeData) -> Action
    {
        // We need to replace node.data with a placeholder (Hole) to appease
        // the borrow checker. Otherwise there would be an aliasing problem
        // where the Document (&self) reference could see the same NodeData
        // passed as &mut.
        let mut ndata = std::mem::replace(&mut self[id].data, NodeData::Hole);
        let res = f(NodeRef::new(self, id), &mut ndata);
        // We only need to reset the potentially mutated node.data if the
        // action is to continue, as all other cases result in the node
        // being detached.
        if res == Action::Continue {
            let node = &mut self[id];
            match ndata {
                NodeData::Document | NodeData::Elem(_) => {}
                NodeData::Hole => {
                    debug_assert!(false, "Filter changed to {:?}", ndata);
                }
                _ => {
                    debug_assert!(
                        node.first_child.is_none() && node.last_child.is_none(),
                        "Filter changed node {:?} with children to {:?}",
                        id, ndata);
                }
            }
            node.data = ndata;
        }
        res
    }
}

/// Compose a new filter closure, by chaining a list of 1 to many closures or
/// function paths. Each is executed in order, while the returned action remains
/// `Action::Continue`, or otherwise terminated early.
#[macro_export]
macro_rules! chain_filters {
    ($solo:expr $(,)?) => (
        |pos: $crate::NodeRef<'_>, data: &mut $crate::NodeData| {
            $solo(pos, data)
        }
    );
    ($first:expr $(, $subs:expr)+ $(,)?) => (
        |pos: $crate::NodeRef<'_>, data: &mut $crate::NodeData| {
            let mut action: $crate::filter::Action = $first(pos, data);
        $(
            if action == $crate::filter::Action::Continue {
                action = $subs(pos, data);
            }
        )*
            action
        }
    );
}

/// Detach known banned elements
/// ([`TagMeta::is_banned`](crate::html::TagMeta::is_banned)) and any elements
/// which are unknown.
pub fn detach_banned_elements(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(ref mut elm) = data.as_element_mut() {
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

/// Fold meaningless inline elements, which are empty or contain only logical
/// whitespace.
///
/// Logical whitespace is defined as all Unicode whitespace or control chars in
/// child text, or the `<br>` element. Non-text oriented inline elements like
/// `<img>` and `<video>` and other multi-media are excluded from
/// consideration.
pub fn fold_empty_inline(pos: NodeRef<'_>, data: &mut NodeData) -> Action {
    if  is_inline(data) &&
        !is_multi_media(data) &&
        pos.children().all(is_logical_ws)
    {
        Action::Fold
    } else {
        Action::Continue
    }
}

/// Detach any comment nodes.
pub fn detach_comments(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let NodeData::Comment(_) = data {
        Action::Detach
    } else {
        Action::Continue
    }
}

/// Detach any processing instruction nodes.
pub fn detach_pis(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let NodeData::ProcessingInstruction {..} = data {
        Action::Detach
    } else {
        Action::Continue
    }
}

/// Filter out attributes that are not included in the "basic" set
/// [`TagMeta`](crate::html::TagMeta) for each element.
pub fn retain_basic_attributes(_p: NodeRef<'_>, data: &mut NodeData)
    -> Action
{
    if let Some(ref mut elm) = data.as_element_mut() {
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
/// assumes, without knowledge of any potential unconventional external styling,
/// that leading and trailing whitespace may be removed at block element
/// boundaries.
///
/// Because this filter works on text nodes, depth first, results are better if
/// applied in its own `Document::filter` pass, _after_ any pass containing
/// filters that detach or fold elements, such as [`detach_banned_elements`] or
/// [`fold_empty_inline`]. Otherwise the filter may not be able to merge text
/// node's which become siblings too late in the process, resulting in
/// additional unnecessary whitespace.
pub fn text_normalize(pos: NodeRef<'_>, data: &mut NodeData) -> Action {
    thread_local! {
        static MERGE_Q: RefCell<StrTendril> = RefCell::new(StrTendril::new())
    };

    if let Some(t) = data.as_text_mut() {
        // If the immediately following sibling is also text, then push this
        // tendril to the merge queue and detach.
        let node_r = pos.next_sibling();
        if node_r.map_or(false, |n| n.as_text().is_some()) {
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

        let parent = pos.parent().unwrap();
        let parent_is_block = is_block(parent);
        let in_pre = parent.node_and_ancestors().any(is_preform_node);

        let node_l = pos.prev_sibling();
        let trim_l = node_l.map_or(parent_is_block, is_block);
        let trim_r = node_r.map_or(parent_is_block, is_block);

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
/// applied, these are roughly equivalent to `<pre>`, and its safer if
/// converted.
pub fn xmp_to_pre(_p: NodeRef<'_>, data: &mut NodeData) -> Action {
    if let Some(elm) = data.as_element_mut() {
        if is_preformatted(elm) {
            elm.name.local = t::PRE;
        }
    }
    Action::Continue
}

fn is_block(node: NodeRef<'_>) -> bool {
    if let Some(elm) = node.as_element() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            return !tmeta.is_inline();
        }
    }
    false
}

// Note this isn't an exact negation of `is_block`: it still returns false for
// unknown elements.
fn is_inline(data: &NodeData) -> bool {
    if let Some(elm) = data.as_element() {
        if let Some(tmeta) = TAG_META.get(&elm.name.local) {
            return tmeta.is_inline();
        }
    }
    false
}

fn is_preformatted(e: &Element) -> bool {
    e.is_elem(t::PRE) || e.is_elem(t::XMP) || e.is_elem(t::PLAINTEXT)
}

fn is_preform_node(n: NodeRef<'_>) -> bool {
    n.is_elem(t::PRE) || n.is_elem(t::XMP) || n.is_elem(t::PLAINTEXT)
}

fn is_logical_ws(n: NodeRef<'_>) -> bool {
    if let Some(t) = n.as_text() {
        is_all_ctrl_ws(t)
    } else {
        n.is_elem(t::BR)
    }
}

fn is_multi_media(n: &NodeData) -> bool {
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
