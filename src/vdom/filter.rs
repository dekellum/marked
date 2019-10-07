use crate::chars::replace_ctrl_ws;
use crate::vdom::{Document, Node, NodeData, NodeId};

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

pub trait TreeFilter {
    fn filter(&self, node: &mut Node) -> Action;
}

// FIXME: This is a limited and very simple application PoC which at least gets
// rid of known problem chars. It will go too far with replacing newlines in
// `<pre>` (or `<xmp>`!) blocks. We don't presently have inline vs. block
// element classification to use when considering to trim start or end of a
// text node.
pub(crate) struct TextNormalizer;

impl TreeFilter for TextNormalizer {
    fn filter(&self, node: &mut Node) -> Action {
        if let NodeData::Text(ref mut t) = node.data {
            replace_ctrl_ws(t, false, false);
        }
        Action::Continue
    }
}

// FIXME: Dynamic dispatch can be costly for this (called for every
// node). Consider some static helper or require manual setup?
pub(crate) struct FilterChain {
    filters: Vec<Box<dyn TreeFilter>>
}

impl FilterChain {
    #[allow(unused)] //FIXME
    pub(crate) fn new(filters: Vec<Box<dyn TreeFilter>>) -> Self {
        FilterChain { filters }
    }
}

impl TreeFilter for FilterChain {
    fn filter(&self, node: &mut Node) -> Action {
        let mut action = Action::Continue;
        for f in self.filters.iter() {
            action = f.filter(node);
            if action != Action::Continue {
                break;
            }
        }
        action
    }
}

impl Document {
    /// Perform a depth-first (e.g. children before parent nodes) walk of the
    /// entire document, allowing the given `TreeFilter` to make changes
    /// to each `Node`.
    pub fn filter<TF>(&mut self, f: &TF)
        where TF: TreeFilter
    {
        self.filter_node(f, Document::DOCUMENT_NODE_ID);
    }

    fn filter_node<TF>(&mut self, f: &TF, id: NodeId) -> Action
        where TF: TreeFilter
    {
        let mut next_child = self[id].first_child;
        while let Some(child) = next_child {
            next_child = self[child].next_sibling;
            match self.filter_node(f, child) {
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

        f.filter(&mut self[id])
    }

    /// Replace the given node with its children.
    // FIXME: Move to top level vdom mod?
    pub(crate) fn fold(&mut self, id: NodeId) {
        let mut next_child = self[id].first_child;
        while let Some(child) = next_child {
            debug_assert_eq!(self[child].parent, Some(id));
            next_child = self[child].next_sibling;
            self.insert_before(id, child);
        }
        self.detach(id);
    }

}
