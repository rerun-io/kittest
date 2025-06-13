//! This example shows how to build a basic kittest integration for some ui framework (in this case egui).
//! If you actually want to use kittest with egui, I suggest you check out the official
//! [egui_kittest][1] integration.
//!
//! [1]: <https://github.com/emilk/egui/tree/master/crates/egui_kittest>

use accesskit::{Action, ActionRequest};
use kittest::{debug_fmt_node, AccessKitNode, NodeT, Queryable};
use parking_lot::Mutex;
use std::fmt::{Debug, Formatter};
use std::mem;

/// The test Harness. This contains everything needed to run the test.
pub struct Harness<'a> {
    /// A handle to the ui framework
    ctx: egui::Context,
    /// the ui component that should be tested (for egui that's just a closure).
    app: Box<dyn FnMut(&egui::Context) + 'a>,
    /// The kittest State
    pub state: kittest::State,
    /// A queue of events that will be processed in the next frame.
    /// It's a mutex, so we can pass it to [`EguiNode`] and trigger events from there.
    queued_events: Mutex<Vec<egui::Event>>,
}

impl<'a> Harness<'a> {
    pub fn new(mut app: impl FnMut(&egui::Context) + 'a) -> Self {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();

        let output = ctx.run(Default::default(), &mut app);

        Self {
            ctx,
            app: Box::new(app),
            state: kittest::State::new(
                output
                    .platform_output
                    .accesskit_update
                    .expect("AccessKit not enabled"),
            ),
            queued_events: Default::default(),
        }
    }

    pub fn run_frame(&mut self) {
        let events = mem::take(&mut *self.queued_events.lock());

        let output = self.ctx.run(
            egui::RawInput {
                events,
                ..Default::default()
            },
            self.app.as_mut(),
        );

        self.state.update(
            output
                .platform_output
                .accesskit_update
                .expect("AccessKit not enabled"),
        );
    }
}

// This allows us to directly query the harness as if it's the root node.
impl<'tree, 'node> Queryable<'tree, 'node, EguiNode<'tree>> for Harness<'_>
where
    'node: 'tree,
{
    fn queryable_node(&'node self) -> EguiNode<'tree> {
        // Construct the root node from `kittest::State` and the event queue.
        EguiNode {
            queue: &self.queued_events,
            node: self.state.root(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct EguiNode<'tree> {
    node: AccessKitNode<'tree>,
    queue: &'tree Mutex<Vec<egui::Event>>,
}

impl Debug for EguiNode<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // You can use this helper to get nice recursive debug output for the node.
        debug_fmt_node(self, f)
    }
}

impl<'tree> NodeT<'tree> for EguiNode<'tree> {
    fn accesskit_node(&self) -> AccessKitNode<'tree> {
        self.node
    }

    fn new_related(&self, child_node: AccessKitNode<'tree>) -> Self {
        Self {
            queue: self.queue,
            node: child_node,
        }
    }
}

impl EguiNode<'_> {
    pub fn click(&self) {
        self.queue.lock().push(
            // You probably want to do mouse move, pointer down, pointer up here, but for
            // brevity, let's use the accesskit event.
            egui::Event::AccessKitActionRequest(ActionRequest {
                action: Action::Click,
                target: self.accesskit_node().id(),
                data: None,
            }),
        );
    }
}

fn main() {
    let mut checked = false;
    let mut harness = Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.checkbox(&mut checked, "Check me!");
        });
    });

    harness.get_by_label("Check me!").click();
    harness.run_frame();

    drop(harness);

    assert!(checked, "Should be checked");
}
