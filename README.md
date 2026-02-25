# 💻🐈 kittest: UI Testing Library for Rust Powered by AccessKit

**kittest** is a GUI testing library for Rust, inspired by [Testing Library](https://testing-library.com/). 
It leverages [AccessKit](https://github.com/AccessKit/accesskit/) to provide a framework-agnostic solution 
for testing user interfaces, with a focus on accessibility.

This library is designed to be flexible and works with any GUI framework that supports AccessKit.
Creating new **kittest** integrations is simple and straightforward. To get started, check out our 
[basic integration example](./integration_example/src/bin/basic_integration.rs).

## Available Integrations
- [egui_kittest](https://github.com/emilk/egui/tree/master/crates/egui_kittest): Official integration for 
  [egui](https://github.com/emilk/egui).

If you create a new integration, please open a PR to add it to this list!

## Example usage with [egui_kittest](https://github.com/emilk/egui/tree/master/crates/egui_kittest)

```rust ignore
use egui::accesskit::Toggled;
use egui_kittest::{Harness, kittest::Queryable};

fn main() {
    let mut checked = false;
    let app = |ui: &mut egui::Ui| {
        ui.checkbox(&mut checked, "Check me!");
    };

    let mut harness = Harness::new_ui(app);
    
    let checkbox = harness.get_by_label("Check me!");
    assert_eq!(checkbox.toggled(), Some(Toggled::False));
    checkbox.click();
    
    harness.run();

    let checkbox = harness.get_by_label("Check me!");
    assert_eq!(checkbox.toggled(), Some(Toggled::True));
}
```

Also see the [querying example](./integration_example/src/bin/querying.rs).
