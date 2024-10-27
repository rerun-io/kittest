---
theme: apple-basic
layout: intro-image-right
# some information about your slides (markdown enabled)
title: Welcome to Slidev
transition: slide-left
# enable MDC Syntax: https://sli.dev/features/mdc
mdc: true
# take snapshot for each slide in the overview
overviewSnapshots: true

image: https://media0.giphy.com/media/v1.Y2lkPTc5MGI3NjExcTU2NzNqZG1kZmRsODRyYzRmbHQ2cTg3ZTl2Zmc0cWJ0aXRlZjUycyZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/ule4vhcY1xEKQ/giphy.webp

---

# kittest

Framework-agnostic UI testing library, based on [AccessKit](https://github.com/AccessKit/accesskit/).

By Lucas Meurer


<small>[giphy cat typing](https://giphy.com/gifs/reactionseditor-cat-typing-ule4vhcY1xEKQ)</small>

---

# What is kittest?

- Thin layer over [accesskit_consumer](https://github.com/AccessKit/accesskit/tree/main/consumer)
- Provides convenient api to 
  - query the AccessKit node tree
  - trigger AccessKit events
    - e.g. `accesskit::Action::Default` via `Node::click`
  - trigger "Simulated" events (not sure on the naming here)
    - e.g. click at the node's center via `Node::simulate_click`

---

# Query functions

- Inspired by the popular web-dev [Testing Library](https://testing-library.com/)
- Accessible by Default
- Implemented via `Queryable` trait

--- 

# Query Types

  - get_by_* 
    - panic when node not found
  - get_all_by_*
    - return an iterator of nodes
    - panic when no node found
  - query_by_*
    - returning an option
    - panic when more than one node found
  - query_all_by_*
    - return an iterator of nodes

---

# Helper methods

- by_role
- by_name
- by_role_and_name

- \[...\]

- via custom filter struct `By`
  - e.g. `harness.get(by().role(Role::CheckBox).name_contains("Check"))`

--- 

# Example test (with egui_kittest)

````md magic-move

```rust
let mut checked = false;
let app = |ctx: &Context| {
    CentralPanel::default().show(ctx, |ui| {
        ui.checkbox(&mut checked, "Check me!");
    });
};
```

```rust
let mut checked = false;
let app = |ctx: &Context| {
    CentralPanel::default().show(ctx, |ui| {
        ui.checkbox(&mut checked, "Check me!");
    });
};

let mut harness = Harness::builder().with_size(egui::Vec2::new(200.0, 100.0)).build(app);
```

```rust
let mut checked = false;
let app = |ctx: &Context| {
    CentralPanel::default().show(ctx, |ui| {
        ui.checkbox(&mut checked, "Check me!");
    });
};

let mut harness = Harness::builder().with_size(egui::Vec2::new(200.0, 100.0)).build(app);

let checkbox = harness.get_by_name("Check me!");
assert_eq!(checkbox.toggled(), Some(Toggled::False));
```

```rust
let mut checked = false;
let app = |ctx: &Context| {
    CentralPanel::default().show(ctx, |ui| {
        ui.checkbox(&mut checked, "Check me!");
    });
};

let mut harness = Harness::builder().with_size(egui::Vec2::new(200.0, 100.0)).build(app);

let checkbox = harness.get_by_name("Check me!");
assert_eq!(checkbox.toggled(), Some(Toggled::False));
checkbox.click();

harness.run();
```

```rust
let mut checked = false;
let app = |ctx: &Context| {
    CentralPanel::default().show(ctx, |ui| {
        ui.checkbox(&mut checked, "Check me!");
    });
};

let mut harness = Harness::builder().with_size(egui::Vec2::new(200.0, 100.0)).build(app);

let checkbox = harness.get_by_name("Check me!");
assert_eq!(checkbox.toggled(), Some(Toggled::False));
checkbox.click();

harness.run();

let checkbox = harness.get_by_name("Check me!");
assert_eq!(checkbox.toggled(), Some(Toggled::True));
```

```rust
let mut checked = false;
let app = |ctx: &Context| {
    CentralPanel::default().show(ctx, |ui| {
        ui.checkbox(&mut checked, "Check me!");
    });
};

let mut harness = Harness::builder().with_size(egui::Vec2::new(200.0, 100.0)).build(app);

let checkbox = harness.get_by_name("Check me!");
assert_eq!(checkbox.toggled(), Some(Toggled::False));
checkbox.click();

harness.run();

let checkbox = harness.get_by_name("Check me!");
assert_eq!(checkbox.toggled(), Some(Toggled::True));

// You can even render the ui and do image snapshot tests
#[cfg(all(feature = "wgpu", feature = "snapshot"))]
harness.wgpu_snapshot("readme_example");
```

````

---

# How does a kittest integration look?
- [Egui example integration](https://github.com/rerun-io/kittest/blob/main/examples/basic_integration.rs)

- for existing test frameworks (like [masonry's test harness](https://github.com/linebender/xilem/blob/main/masonry/src/testing/harness.rs))
  kittest could be enabled via a feature, or as a separate crate


--- 

# Pros

- Accessibility gets tested by default
  - You usually query nodes by their label
  - Thus, it’s e.g. ensured that all interactive elements have an accessible label
- There is one well-thought-out api that ui frameworks can rely on
- Hurdle to write a test harness for an ui framework gets reduced

---

# Cons
- The "plumbing" for each ui framework still has to be done manually
  - Straightforward for egui 
  - Might be more complicated for other ui frameworks
- You must use the accessibility labels, some people might prefer using e.g. id strings, those aren’t possible currently with AccessKit
- Nodes hold a reference to the accesskit tree, so they cannot be held across frames

---

# Cons that could be resolved by making `Node` a trait
- AccessKit nodes might not 100% match what the ui framework provides
  - e.g. maybe the ui frameworks checkbox component only has two states while the kittest node will have AccessKits three states
- Currently, kittest only provides access to an accesskit node, not to the original masonry / egui / xilem widget

--- 

# What else could be part of kittest?
- Image snapshot tests
  - Currently implemented in egui_kittest but could e.g. be released as kittest_image_snapshot
- GitHub action to set up an environment where wgpu will run
  - Should install swiftshader / llvmpipe / vulkan sdk
  - Should work with windows / macOS / linux
- Provide Mappings from the kittest event types to the winit types
