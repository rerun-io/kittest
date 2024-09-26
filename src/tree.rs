use crate::event::Event;
use crate::query::Queryable;
use crate::Node;
use accesskit::TreeUpdate;
use parking_lot::Mutex;

pub struct State {
    tree: accesskit_consumer::Tree,
    queued_events: Mutex<Vec<Event>>,
}

pub(crate) type EventQueue = Mutex<Vec<Event>>;

impl State {
    pub fn new(update: TreeUpdate) -> Self {
        Self {
            tree: accesskit_consumer::Tree::new(update, true),
            queued_events: Mutex::new(Vec::new()),
        }
    }

    pub fn update(&mut self, update: accesskit::TreeUpdate) {
        self.tree.update(update);
    }

    pub fn root(&self) -> Node<'_> {
        self.node()
    }

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
