//! This example shows how to build a basic kittest integration for some ui framework (in this case egui).
//! If you actually want to use kittest with egui, I suggest you check out the official
//! [egui_kittest][1] integration.
//!
//! [1]: <https://github.com/emilk/egui/tree/master/crates/egui_kittest>

use kittest::{Event, Node, Queryable};

/// The test Harness. This contains everything needed to run the test.
pub struct Harness<'a> {
    /// A handle to the ui framework
    ctx: egui::Context,
    /// the ui component that should be tested (for egui that's just a closure).
    app: Box<dyn FnMut(&egui::Context) + 'a>,
    /// The kittest State
    pub state: kittest::State,
}

impl<'a> Harness<'a> {
    pub fn new(mut app: impl FnMut(&egui::Context) + 'a) -> Harness<'a> {
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
        }
    }

    pub fn run_frame(&mut self) {
        let events = self
            .state
            .take_events()
            .into_iter()
            .map(|e| match e {
                Event::ActionRequest(action) => egui::Event::AccessKitActionRequest(action),
                Event::Simulated(_) => {
                    panic!("Check egui_kittest for a full implementation");
                }
            })
            .collect();

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
impl<'tree, 'node, 'app> Queryable<'tree, 'node> for Harness<'app>
where
    'node: 'tree,
{
    fn node(&'node self) -> Node<'tree> {
        self.state.root()
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
