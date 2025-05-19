use crate::event::Event;
use crate::query::Queryable;
use crate::Node;
use accesskit::TreeUpdate;
use parking_lot::Mutex;
use std::fmt::{Debug, Formatter};

/// The kittest state
pub struct State {
    tree: accesskit_consumer::Tree,
    queued_events: Mutex<Vec<Event>>,
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("tree", &self.node())
            .finish_non_exhaustive()
    }
}

pub(crate) type EventQueue = Mutex<Vec<Event>>;

struct NoOpChangeHandler;

impl accesskit_consumer::TreeChangeHandler for NoOpChangeHandler {
    fn node_added(&mut self, _node: &accesskit_consumer::Node<'_>) {}

    fn node_updated(
        &mut self,
        _old_node: &accesskit_consumer::Node<'_>,
        _new_node: &accesskit_consumer::Node<'_>,
    ) {
    }

    fn focus_moved(
        &mut self,
        _old_node: Option<&accesskit_consumer::Node<'_>>,
        _new_node: Option<&accesskit_consumer::Node<'_>>,
    ) {
    }

    fn node_removed(&mut self, _node: &accesskit_consumer::Node<'_>) {}
}

impl State {
    /// Create a new State from a `TreeUpdate`
    pub fn new(update: TreeUpdate) -> Self {
        Self {
            tree: accesskit_consumer::Tree::new(update, true),
            queued_events: Mutex::new(Vec::new()),
        }
    }

    /// Update the state with a new `TreeUpdate` (this should be called after each frame)
    pub fn update(&mut self, update: accesskit::TreeUpdate) {
        self.tree
            .update_and_process_changes(update, &mut NoOpChangeHandler);
    }

    /// Get the root node
    pub fn root(&self) -> Node<'_> {
        self.node()
    }

    /// Take all queued events. (These should then be passed to the UI framework)
    pub fn take_events(&self) -> Vec<Event> {
        self.queued_events.lock().drain(..).collect()
    }
}

impl<'tree, 'node> Queryable<'tree, 'node> for State
where
    'node: 'tree,
{
    /// Return the root node
    fn node(&'node self) -> Node<'tree> where {
        Node::new(self.tree.state().root(), &self.queued_events)
    }
}
