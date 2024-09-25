use egui::{CentralPanel, Context, RawInput};
use kittest::{Event, Node, Queryable};

pub struct Harness<'a> {
    ctx: Context,
    tree: kittest::Tree,
    app: Box<dyn FnMut(&Context) + 'a>,
}

impl<'a> Harness<'a> {
    pub fn new(mut app: impl FnMut(&Context) + 'a) -> Harness<'a> {
        let ctx = Context::default();
        ctx.enable_accesskit();

        let output = ctx.run(Default::default(), &mut app);

        Self {
            ctx,
            app: Box::new(app),
            tree: kittest::Tree::new(
                output
                    .platform_output
                    .accesskit_update
                    .expect("AccessKit not enabled"),
            ),
        }
    }

    pub fn run_frame(&mut self) {
        let events = self
            .tree
            .take_events()
            .into_iter()
            .map(|e| match e {
                Event::ActionRequest(action) => egui::Event::AccessKitActionRequest(action),
                Event::Simulated(_) => {
                    todo!("Check egui_kittest for a full implementation");
                }
            })
            .collect();

        let output = self.ctx.run(
            RawInput {
                events,
                ..Default::default()
            },
            self.app.as_mut(),
        );

        self.tree.update(
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
        self.tree.root()
    }
}

fn main() {
    let mut checked = false;
    let mut harness = Harness::new(|ctx| {
        CentralPanel::default().show(ctx, |ui| {
            ui.checkbox(&mut checked, "Check me!");
        });
    });

    harness.get_by_name("Check me!").click();
    harness.run_frame();

    drop(harness);

    assert!(checked, "Should be checked");
}
