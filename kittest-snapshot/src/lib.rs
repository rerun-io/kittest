//! Framework-agnostic image snapshot testing for kittest-based harnesses.
//!
//! This is a port of the snapshot logic from [`egui_kittest`][1] — the same
//! per-pixel diff via [`dify`], the same `{name}.png` / `{name}.new.png` /
//! `{name}.diff.png` / `{name}.old.png` on-disk contract, the same
//! `UPDATE_SNAPSHOTS` env-var handling, and the same `kittest.toml` config
//! discovery — with the egui-specific `Harness` methods removed.
//!
//! Any harness that can produce an [`image::RgbaImage`] (or raw RGBA8 pixels)
//! can feed it into these functions. See `kittest-winit`'s `FrameCapture` for
//! one such producer.
//!
//! # Minimal usage
//!
//! ```ignore
//! let image: image::RgbaImage = /* render ... */;
//! kittest_snapshot::image_snapshot(&image, "my_test");
//! ```
//!
//! Run `UPDATE_SNAPSHOTS=1 cargo test` to refresh snapshots in place.
//!
//! [1]: https://github.com/emilk/egui/tree/master/crates/egui_kittest

mod results;
mod snapshot;

pub use results::SnapshotResults;
pub use snapshot::{
    SnapshotError, SnapshotOptions, SnapshotResult, image_snapshot,
    image_snapshot_options, rgba_snapshot, try_image_snapshot, try_image_snapshot_options,
    try_rgba_snapshot, try_rgba_snapshot_options,
};

#[cfg(all(feature = "debug", not(target_arch = "wasm32")))]
pub use snapshot::debug_open_snapshot;

/// Re-export of the `image` crate so callers don't need to add a direct
/// dependency just to construct an `RgbaImage`.
pub use image;
