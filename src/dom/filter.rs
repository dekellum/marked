//! Mutating visitor support for `Document`.

use crate::chars::replace_chars;
use crate::dom::{Document, Node, NodeData, NodeId};

/// An instruction returned by the `Fn` closure used by `Document::filter`.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Continue filtering, without further changes to this `Node`.
    Continue,

    /// Replace this `Node` with its children. Equivalent to `Remove` if
    /// returned for a `Node` with no children.
    Fold,

    /// Detach this `Node`, and any children, from the tree.
    Detach,

    // Replace this element with the given NodeData, for the same position in
    // the tree.
    // FIXME: Any case we need this for?
    // Replace(NodeData)
}

// FIXME: This is a limited and very simple application PoC which at least gets
// rid of known problem chars. It will go too far with replacing newlines in
// `<pre>` (or `<xmp>`!) blocks. We don't presently have inline vs. block
// element classification to use when considering to trim start or end of a
// text node.
pub fn text_normalize(node: &mut Node) -> Action {
    if let NodeData::Text(ref mut t) = node.data {
        replace_chars(t, true, true, false, false);
    }
    Action::Continue
}

/// Mutating filter methods.
impl Document {
    /// Perform a depth-first (e.g. children before parent nodes) walk of the
    /// entire document, from the document root node, allowing the provided
    /// function to make changes to each `Node`.
    pub fn filter<F>(&mut self, mut f: F)
        where F: Fn(&mut Node) -> Action
    {
        self.filter_at(Document::DOCUMENT_NODE_ID, &mut f);
    }

    /// Perform a depth-first (e.g. children before parent nodes) walk from the
    /// specified node ID, allowing the provided function to make changes to
    /// each `Node`.
    pub fn filter_at<F>(&mut self, id: NodeId, f: &mut F) -> Action
        where F: Fn(&mut Node) -> Action
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

        f(&mut self[id])
    }
}

/// Compose a new filter closure, by chaining a list of closures or function
/// paths. Each is executed in order, while the return action remains
/// `Continue`.
#[macro_export]
macro_rules! chain_filters {
    ($first:expr $(, $subs:expr)* $(,)?) => (
        |node: &mut $crate::Node| {
            let mut action: $crate::filter::Action = $first(node);
        $(
            if action == $crate::filter::Action::Continue {
                action = $subs(node);
            }
        )*
            action
        }
    );
}
