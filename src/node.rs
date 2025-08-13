use crate::AccessKitNode;
use crate::query::Queryable;
use std::fmt::{Debug, Formatter};
use std::iter::once;

/// A kittest node.
///
/// Implement this for your testing node type to make it work with kittest.
pub trait NodeT<'tree>: Clone + Debug {
    /// Provide access to the [`AccessKitNode`]
    fn accesskit_node(&self) -> AccessKitNode<'tree>;

    /// Construct a related (child / parent / sibling) node from the [`AccessKitNode`]
    fn new_related(&self, child_node: AccessKitNode<'tree>) -> Self;

    /// Iterate over the children of the node recursively
    fn children_recursive(&self) -> Box<dyn Iterator<Item = Self> + 'tree>
    where
        Self: 'tree,
    {
        let node = self.clone();
        Box::new(self.accesskit_node().children().flat_map(move |child| {
            let child_node = node.new_related(child);
            once(child_node.clone()).chain(child_node.children_recursive())
        }))
    }

    /// Iterate over the direct children of the node
    fn children(&self) -> impl Iterator<Item = Self> + 'tree
    where
        Self: 'tree,
    {
        let node = self.clone();
        self.accesskit_node()
            .children()
            .map(move |child| node.new_related(child))
    }

    /// Iterate over the children of the node, either recursively or not
    fn children_maybe_recursive(&self, recursive: bool) -> Box<dyn Iterator<Item = Self> + 'tree>
    where
        Self: 'tree,
    {
        if recursive {
            self.children_recursive()
        } else {
            Box::new(self.children())
        }
    }

    /// Get the parent of the node
    fn parent(&self) -> Option<Self> {
        self.accesskit_node()
            .parent()
            .map(|node| Self::new_related(self, node))
    }
}

impl<'tree, 'node, Node: NodeT<'tree> + 'tree> Queryable<'tree, 'node, Node> for Node {
    fn queryable_node(&'node self) -> Node {
        self.clone()
    }
}

/// A helper function to nicely format AccessKit nodes.
///
/// # Errors
/// Returns an error if the formatting fails.
pub fn debug_fmt_node<'tree, Node: NodeT<'tree> + 'tree>(
    node: &Node,
    f: &mut Formatter<'_>,
) -> std::fmt::Result {
    let accesskit_node = node.accesskit_node();

    let mut s = f.debug_struct("Node");
    s.field("id", &accesskit_node.id());
    s.field("role", &accesskit_node.role());
    if let Some(label) = accesskit_node.label() {
        s.field("label", &label);
    }
    if let Some(value) = accesskit_node.value() {
        s.field("value", &value);
    }
    if let Some(numeric) = accesskit_node.numeric_value() {
        s.field("numeric_value", &numeric);
    }
    s.field("focused", &accesskit_node.is_focused());
    s.field("hidden", &accesskit_node.is_hidden());
    s.field("disabled", &accesskit_node.is_disabled());
    if let Some(toggled) = accesskit_node.toggled() {
        s.field("toggled", &toggled);
    }

    let children = node.children().collect::<Vec<_>>();

    s.field("children", &children);

    s.finish()
}
