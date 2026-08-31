//! Minimal demo: a hand-rolled winit `ApplicationHandler` that exposes a single
//! AccessKit checkbox. kittest drives it through `WinitHarness` with no real OS
//! window, no graphics surface, and no accesskit_winit / accesskit_xplat adapter.

use accesskit::{Action, Node, NodeId, Rect, Role, Toggled, Tree, TreeId, TreeUpdate};
use kittest::{NodeT as _, Queryable};
use kittest_winit::{KittestApp, WinitHarness};
use winit_core::application::ApplicationHandler;
use winit_core::event::{ButtonSource, ElementState, MouseButton, WindowEvent};
use winit_core::event_loop::ActiveEventLoop;
use winit_core::window::{Window, WindowAttributes, WindowId};

const ROOT_ID: NodeId = NodeId(1);
const CHECKBOX_ID: NodeId = NodeId(2);
const CHECKBOX_RECT: Rect = Rect {
    x0: 20.0,
    y0: 20.0,
    x1: 160.0,
    y1: 60.0,
};

struct MinimalApp {
    window: Option<Box<dyn Window>>,
    checked: bool,
    pending_tree: Option<TreeUpdate>,
}

impl MinimalApp {
    fn new() -> Self {
        Self {
            window: None,
            checked: false,
            pending_tree: None,
        }
    }

    fn rebuild_tree(&mut self) {
        let mut root = Node::new(Role::Window);
        root.set_children(vec![CHECKBOX_ID]);
        root.set_label("Root");

        let mut checkbox = Node::new(Role::CheckBox);
        checkbox.set_label("Check me!");
        checkbox.set_bounds(CHECKBOX_RECT);
        checkbox.set_toggled(if self.checked {
            Toggled::True
        } else {
            Toggled::False
        });
        checkbox.add_action(Action::Click);
        checkbox.add_action(Action::Focus);

        self.pending_tree = Some(TreeUpdate {
            nodes: vec![(ROOT_ID, root), (CHECKBOX_ID, checkbox)],
            tree: Some(Tree::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: CHECKBOX_ID,
        });
    }

    fn hit_test(&self, position: dpi::PhysicalPosition<f64>) -> Option<NodeId> {
        if position.x >= CHECKBOX_RECT.x0
            && position.x <= CHECKBOX_RECT.x1
            && position.y >= CHECKBOX_RECT.y0
            && position.y <= CHECKBOX_RECT.y1
        {
            Some(CHECKBOX_ID)
        } else {
            None
        }
    }
}

impl ApplicationHandler for MinimalApp {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .expect("failed to create fake window");
        self.window = Some(window);
        self.rebuild_tree();
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::PointerButton {
            state: ElementState::Pressed,
            position,
            button: ButtonSource::Mouse(MouseButton::Left),
            ..
        } = event
        {
            if let Some(CHECKBOX_ID) = self.hit_test(position) {
                self.checked = !self.checked;
                self.rebuild_tree();
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {}
}

impl KittestApp for MinimalApp {
    type Inner = Self;
    fn inner(&mut self) -> &mut Self {
        self
    }
    fn take_accesskit_update(&mut self) -> Option<TreeUpdate> {
        self.pending_tree.take()
    }
}

fn main() {
    let mut harness = WinitHarness::new(|_event_loop| MinimalApp::new());

    let initial_state = harness
        .get_by_label("Check me!")
        .accesskit_node()
        .toggled();
    assert_eq!(initial_state, Some(Toggled::False), "initial state");

    harness.get_by_label("Check me!").click();
    harness.run_frame();
    let toggled = harness
        .get_by_label("Check me!")
        .accesskit_node()
        .toggled();
    assert_eq!(toggled, Some(Toggled::True), "after click");

    harness.get_by_label("Check me!").click();
    harness.run_frame();
    let toggled = harness
        .get_by_label("Check me!")
        .accesskit_node()
        .toggled();
    assert_eq!(toggled, Some(Toggled::False), "after second click");

    println!("minimal_app: all assertions passed");
}
