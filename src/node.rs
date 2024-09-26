use crate::event::{Event, SimulatedEvent};
use crate::query::Queryable;
use crate::tree::EventQueue;
use accesskit::{ActionRequest, Vec2};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

pub struct Node<'tree> {
    node: accesskit_consumer::Node<'tree>,
    pub(crate) queue: &'tree EventQueue,
}

impl<'a> Debug for Node<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("Node");
        s.field("id", &self.node.id());
        s.field("role", &self.node.role());
        if let Some(name) = self.node.name() {
            s.field("name", &name);
        }
        if let Some(value) = self.node.value() {
            s.field("value", &value);
        }
        if let Some(numeric) = self.node.numeric_value() {
            s.field("numeric_value", &numeric);
        }
        s.field("focused", &self.node.is_focused());
        s.field("hidden", &self.node.is_hidden());
        s.field("disabled", &self.node.is_disabled());
        if let Some(toggled) = self.node.toggled() {
            s.field("toggled", &toggled);
        }
        s.finish()
    }
}

/// We should probably add our own methods to query the node state but for now this should work
impl<'tree> Deref for Node<'tree> {
    type Target = accesskit_consumer::Node<'tree>;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<'tree> Node<'tree> {
    pub fn new(node: accesskit_consumer::Node<'tree>, queue: &'tree EventQueue) -> Self {
        Self { node, queue }
    }

    pub fn queue<'node>(&'node self) -> &'tree EventQueue {
        self.queue
    }

    pub fn focus(&self) {
        self.queue.lock().push(Event::ActionRequest(ActionRequest {
            data: None,
            action: accesskit::Action::Focus,
            target: self.node.id(),
        }));
    }

    /// Click the node via accesskit
    pub fn click(&self) {
        self.queue.lock().push(Event::ActionRequest(ActionRequest {
            data: None,
            action: accesskit::Action::Default,
            target: self.node.id(),
        }));
    }

    /// Simulate a click event on the node center
    pub fn simulate_click(&self) {
        let rect = self.node.raw_bounds().expect("Node has no bounds");
        let center = Vec2::new(rect.x0 + rect.x1 / 2.0, rect.y0 + rect.y1 / 2.0);
        self.queue
            .lock()
            .push(Event::Simulated(SimulatedEvent::Click { position: center }));
    }

    pub fn type_text(&self, text: &str) {
        self.focus();
        self.queue
            .lock()
            .push(Event::Simulated(SimulatedEvent::Type {
                text: text.to_owned(),
            }));
    }

    pub fn parent(&self) -> Option<Node<'tree>> {
        self.node.parent().map(|node| Node::new(node, self.queue))
    }
}

impl<'t, 'n> Queryable<'t, 'n> for Node<'t> {
    fn node(&'n self) -> Node<'t> {
        Node::new(self.node, self.queue)
    }
}
