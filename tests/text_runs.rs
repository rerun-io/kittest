use accesskit::{Node, NodeId, Role, TreeId, TreeInfo, TreeUpdate};
use kittest::{AccessKitNode, By, NodeT, Queryable as _, State};

#[derive(Clone, Debug)]
struct TestNode<'tree>(AccessKitNode<'tree>);

impl<'tree> NodeT<'tree> for TestNode<'tree> {
    fn accesskit_node(&self) -> AccessKitNode<'tree> {
        self.0
    }

    fn new_related(&self, node: AccessKitNode<'tree>) -> Self {
        Self(node)
    }
}

/// A window with a single-line text input holding "abc",
/// with the text run below it that holds the same text for screen readers.
fn text_input_with_text_run() -> State {
    let mut window = Node::new(Role::Window);
    window.set_children(vec![NodeId(1)]);

    let mut text_input = Node::new(Role::TextInput);
    text_input.set_value("abc");
    text_input.set_children(vec![NodeId(2)]);

    let mut text_run = Node::new(Role::TextRun);
    text_run.set_value("abc");

    State::new(TreeUpdate {
        nodes: vec![
            (NodeId(0), window),
            (NodeId(1), text_input),
            (NodeId(2), text_run),
        ],
        tree: Some(TreeInfo::new(NodeId(0))),
        tree_id: TreeId::ROOT,
        focus: NodeId(0),
    })
}

#[test]
fn value_query_finds_the_text_input_not_its_text_run() {
    let state = text_input_with_text_run();
    let root = TestNode(state.root());

    assert_eq!(root.get_by_value("abc").0.role(), Role::TextInput);
}

#[test]
fn text_runs_can_still_be_found_by_value() {
    let state = text_input_with_text_run();
    let root = TestNode(state.root());

    assert_eq!(
        root.query_all(By::new().value("abc").include_text_runs())
            .count(),
        2
    );
    assert_eq!(
        root.get(By::new().role(Role::TextRun).value("abc"))
            .0
            .role(),
        Role::TextRun
    );
}
