use crate::AccessKitNode;
use accesskit::TreeUpdate;
use std::fmt::{Debug, Formatter};

/// The kittest state. This is a wrapper around [`accesskit_consumer::Tree`]. You could
/// also use [`accesskit_consumer::Tree`] directly.
pub struct State {
    tree: accesskit_consumer::Tree,
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish_non_exhaustive()
    }
}

struct NoOpChangeHandler;

impl accesskit_consumer::TreeChangeHandler for NoOpChangeHandler {
    fn node_added(&mut self, _node: &AccessKitNode<'_>) {}

    fn node_updated(&mut self, _old_node: &AccessKitNode<'_>, _new_node: &AccessKitNode<'_>) {}

    fn focus_moved(
        &mut self,
        _old_node: Option<&AccessKitNode<'_>>,
        _new_node: Option<&AccessKitNode<'_>>,
    ) {
    }

    fn node_removed(&mut self, _node: &AccessKitNode<'_>) {}
}

impl State {
    /// Create a new State from a `TreeUpdate`
    pub fn new(update: TreeUpdate) -> Self {
        Self {
            tree: accesskit_consumer::Tree::new(update, true),
        }
    }

    /// Update the state with a new `TreeUpdate` (this should be called after each frame)
    pub fn update(&mut self, update: accesskit::TreeUpdate) {
        self.tree
            .update_and_process_changes(update, &mut NoOpChangeHandler);
    }

    /// Get the root accesskit node
    pub fn root(&self) -> AccessKitNode<'_> {
        self.tree.state().root()
    }
}
