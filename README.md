# 💻🐈 kittest: UI Testing Library for Rust Powered by AccessKit

**kittest** is a GUI testing library for Rust, inspired by [Testing Library](https://testing-library.com/). 
It leverages [AccessKit](https://github.com/AccessKit/accesskit/) to provide a framework-agnostic solution 
for testing user interfaces.

This library is designed to be flexible and works with any GUI framework that supports AccessKit.
Creating new **kittest** integrations is simple and straightforward. To get started, check out our 
[basic integration example](https://github.com/rerun-io/kittest/blob/main/examples/basic_integration.rs).

## Available Integrations
- [egui_kittest](https://github.com/emilk/egui/tree/master/crates/egui_kittest): Official integration for 
  [egui](https://github.com/emilk/egui).
