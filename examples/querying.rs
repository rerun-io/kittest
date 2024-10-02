//! This example shows how to use the kittest query functions.

/// For this example we'll use the egui integration from the basic_integration example.
/// This allows us to easily create a realistic tree.
#[allow(dead_code)]
mod basic_integration;

use accesskit::Role;
use basic_integration::Harness;
use kittest::{by, Queryable};

fn main() {
    let harness = make_tree();

    // You can query nodes by their name (query_by_* functions always return an Option<Node>)
    let _button_1 = harness
        .query_by_name("Button 1")
        .expect("Button 1 not found");

    // You can get nodes by their name (get_by_* functions will panic with a helpful error message
    // if the node is not found)
    let _button_2 = harness.get_by_name("Button 2");

    // You can get all nodes with a certain name
    let buttons = harness.query_all_by_name("Duplicate");
    assert_eq!(
        buttons.count(),
        2,
        "Expected 2 buttons with the name 'Duplicate'"
    );

    // If you have multiple items with the same name, you can query by name and role
    let _submit = harness.get_by_role_and_name(Role::Button, "Submit");

    // If you need more complex queries, you can use the by struct
    let _check_me = harness.get(by().role(Role::CheckBox).name_contains("Check"));

    // You can also query children of a node
    let _group = harness.get_by_name("My Group");
    // get_by_name won't panic here since we only find the button in the group
    // TODO(lucas): Egui doesn't add node as children of their container right now
    // group.get_by_name("Duplicate");

    // TODO(lucas): This should match once egui adds children to their containers
    // let btn_in_parent = harness.get_all_by_name("Duplicate").next_back().unwrap();
    // assert_eq!(btn_in_parent.parent_id().unwrap(), group.id());

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
                    _ = ui.button("Duplicate");
                })
                .response
                .labelled_by(group_label.id);
        });
    })
}
