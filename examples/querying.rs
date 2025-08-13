//! This example shows how to use the kittest query functions.

/// For this example we'll use the egui integration from the basic_integration example.
/// This allows us to easily create a realistic tree.
#[allow(dead_code)]
mod basic_integration;

use accesskit::Role;
use basic_integration::Harness;
use kittest::{NodeT, Queryable, by};

fn main() {
    let harness = make_tree();

    // You can query nodes by their label (query_by_* functions always return an Option<Node>)
    let _button_1 = harness
        .query_by_label("Button 1")
        .expect("Button 1 not found");

    // You can get nodes by their label (get_by_* functions will panic with a helpful error message
    // if the node is not found)
    let _button_2 = harness.get_by_label("Button 2");

    // You can get all nodes with a certain label
    let buttons = harness.query_all_by_label("Duplicate");
    assert_eq!(
        buttons.count(),
        2,
        "Expected 2 buttons with the label 'Duplicate'"
    );

    // If you have multiple items with the same label, you can query by label and role
    let _submit = harness.get_by_role_and_label(Role::Button, "Submit");

    // If you need more complex queries, you can use the by struct
    let _check_me = harness.get(by().role(Role::CheckBox).label_contains("Check"));

    // You can also query children of a node
    let group = harness.get_by_role_and_label(Role::Label, "My Group");
    // get_by_label won't panic here since we only find the button in the group
    group.get_by_label("Duplicate");

    let btn_in_parent = harness
        .get_all_by_label("Duplicate")
        .next_back()
        .expect("No buttons found");
    assert_eq!(
        btn_in_parent
            .accesskit_node()
            .parent_id()
            .expect("No parent id"),
        group.accesskit_node().id(),
        "Button is not in the group"
    );

    // query_by and get_by functions will panic if more than one node is found
    // harness.get_by_role(Role::Button); // This will panic!
}

#[allow(clippy::let_underscore_must_use)]
fn make_tree() -> Harness<'static> {
    Harness::new(|ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            _ = ui.button("Button 1");
            _ = ui.button("Button 2");

            _ = ui.checkbox(&mut true, "Check me");

            _ = ui.button("Duplicate");

            _ = ui.label("Submit");
            _ = ui.button("Submit");

            let group_label = ui.label("My Group");
            _ = ui
                .group(|ui| {
                    // TODO(lucasmerlin): Egui should probably group widgets by their parent automatically
                    ui.ctx()
                        .clone()
                        .with_accessibility_parent(group_label.id, || {
                            _ = ui.button("Duplicate");
                        });
                })
                .response
                .labelled_by(group_label.id);
        });
    })
}
