use crate::event::{Event, SimulatedEvent};
use crate::query::Queryable;
use crate::tree::EventQueue;
use crate::{ElementState, Key, MouseButton};
use accesskit::{ActionRequest, Vec2};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// A node in the accessibility tree. This should correspond to a widget or container in the GUI
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
    /// Create a new node from an [`accesskit_consumer::Node`] and an [`EventQueue`]
    pub(crate) fn new(node: accesskit_consumer::Node<'tree>, queue: &'tree EventQueue) -> Self {
        Self { node, queue }
    }

    pub(crate) fn queue<'node>(&'node self) -> &'tree EventQueue {
        self.queue
    }

    fn event(&self, event: Event) {
        self.queue.lock().push(event);
    }

    /// Request focus for the node via accesskit
    pub fn focus(&self) {
        self.event(Event::ActionRequest(ActionRequest {
            data: None,
            action: accesskit::Action::Focus,
            target: self.node.id(),
        }));
    }

    /// Click the node via accesskit. This will trigger a ['accesskit::Action::Default'] action
    pub fn click(&self) {
        self.event(Event::ActionRequest(ActionRequest {
            data: None,
            action: accesskit::Action::Default,
            target: self.node.id(),
        }));
    }

    /// Hover the cursor at the node center
    pub fn hover(&self) {
        let rect = self.node.raw_bounds().expect("Node has no bounds");
        let center = Vec2::new(rect.x0 + rect.x1 / 2.0, rect.y0 + rect.y1 / 2.0);
        self.event(Event::Simulated(SimulatedEvent::CursorMoved {
            position: center,
        }));
    }

    /// Simulate a click event at the node center
    pub fn simulate_click(&self) {
        ElementState::click().for_each(|state| {
            self.event(Event::Simulated(SimulatedEvent::MouseInput {
                button: MouseButton::Left,
                state,
            }));
        });
    }

    /// Focus the node and type the given text
    pub fn type_text(&self, text: &str) {
        self.focus();
        self.event(Event::Simulated(SimulatedEvent::Ime(text.to_owned())));
    }

    /// Press the given keys in combination
    ///
    /// For e.g. [`Key::Control`] + [`Key::A`] this would generate:
    /// - Press [`Key::Control`]
    /// - Press [`Key::A`]
    /// - Release [`Key::A`]
    /// - Release [`Key::Control`]
    pub fn key_combination(&self, keys: &[Key]) {
        self.focus();
        keys.iter().for_each(|key| {
            self.event(Event::Simulated(SimulatedEvent::KeyInput {
                key: *key,
                state: ElementState::Pressed,
            }));
        });
        keys.iter().rev().for_each(|key| {
            self.event(Event::Simulated(SimulatedEvent::KeyInput {
                key: *key,
                state: ElementState::Released,
            }));
        });
    }

    /// Press the given keys
    /// For e.g. [`Key::Control`] + [`Key::A`] this would generate:
    /// - Press [`Key::Control`]
    /// - Release [`Key::Control`]
    /// - Press [`Key::A`]
    /// - Release [`Key::A`]
    pub fn press_keys(&self, keys: &[Key]) {
        self.focus();
        keys.iter().for_each(|key| {
            ElementState::click().for_each(|state| {
                self.event(Event::Simulated(SimulatedEvent::KeyInput {
                    key: *key,
                    state,
                }));
            });
        });
    }

    /// Press the given key
    pub fn key_down(&self, key: Key) {
        self.focus();
        self.event(Event::Simulated(SimulatedEvent::KeyInput {
            key,
            state: ElementState::Pressed,
        }));
    }

    /// Release the given key
    pub fn key_up(&self, key: Key) {
        self.focus();
        self.event(Event::Simulated(SimulatedEvent::KeyInput {
            key,
            state: ElementState::Released,
        }));
    }

    /// Press and release the given key
    pub fn key_press(&self, key: Key) {
        self.focus();
        ElementState::click().for_each(|state| {
            self.event(Event::Simulated(SimulatedEvent::KeyInput {
                key,
                state,
            }));
        });
    }

    /// Get the parent of the node
    pub fn parent(&self) -> Option<Node<'tree>> {
        self.node.parent().map(|node| Node::new(node, self.queue))
    }
}

impl<'t, 'n> Queryable<'t, 'n> for Node<'t> {
    fn node(&'n self) -> Node<'t> {
        Node::new(self.node, self.queue)
    }
}
