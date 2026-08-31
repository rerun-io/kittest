//! Core image snapshot comparison.
//!
//! Ported from `egui_kittest::snapshot` with the egui-specific `Harness`
//! methods removed — the functions here operate on plain `image::RgbaImage`s
//! so any harness can feed them.

use std::fmt::Display;
use std::io::ErrorKind;
use std::path::PathBuf;

use image::ImageError;

pub type SnapshotResult = Result<(), SnapshotError>;

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct SnapshotOptions {
    /// Per-pixel comparison threshold passed to `dify`.
    ///
    /// Fallback: `0.6`.
    pub threshold: f32,

    /// The number of pixels that can differ before the snapshot is considered
    /// a failure. Prefer [`Self::threshold`] to control sensitivity; use this
    /// as a last-resort OS-by-OS workaround.
    ///
    /// Fallback: `0`.
    pub failed_pixel_count_threshold: usize,

    /// The path where snapshots are saved, relative to the working directory
    /// (the crate root when running tests).
    ///
    /// Fallback: `tests/snapshots`.
    pub output_path: PathBuf,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            threshold: 0.0,
            output_path: "tests/snapshots".into(),
            failed_pixel_count_threshold: 0,
        }
    }
}

impl SnapshotOptions {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold.into();
        self
    }

    #[inline]
    pub fn output_path(mut self, output_path: impl Into<PathBuf>) -> Self {
        self.output_path = output_path.into();
        self
    }

    #[inline]
    pub fn failed_pixel_count_threshold(
        mut self,
        failed_pixel_count_threshold: usize,
    ) -> Self {
        self.failed_pixel_count_threshold = failed_pixel_count_threshold;
        self
    }
}

#[derive(Debug)]
pub enum SnapshotError {
    /// Image did not match snapshot.
    Diff {
        name: String,
        /// Count of pixels different above the per-pixel threshold.
        diff: i32,
        /// Where the diff image was saved.
        diff_path: PathBuf,
    },

    /// Error opening the existing snapshot (most likely: it doesn't exist).
    OpenSnapshot { path: PathBuf, err: ImageError },

    /// Image dimensions didn't match the snapshot.
    SizeMismatch {
        name: String,
        expected: (u32, u32),
        actual: (u32, u32),
    },

    /// Error writing a snapshot artifact (snapshot, .new, .diff, .old).
    WriteSnapshot { path: PathBuf, err: ImageError },

    /// Error rendering the image.
    RenderError { err: String },
}

const HOW_TO_UPDATE_SCREENSHOTS: &str =
    "Run `UPDATE_SNAPSHOTS=1 cargo test --all-features` to update the snapshots.";

impl Display for SnapshotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Diff {
                name,
                diff,
                diff_path,
            } => {
                let diff_path =
                    std::path::absolute(diff_path).unwrap_or_else(|_| diff_path.clone());
                write!(
                    f,
                    "'{name}' Image did not match snapshot. Diff: {diff}, {}. {HOW_TO_UPDATE_SCREENSHOTS}",
                    diff_path.display()
                )
            }
            Self::OpenSnapshot { path, err } => {
                let path = std::path::absolute(path).unwrap_or_else(|_| path.clone());
                match err {
                    ImageError::IoError(io) => match io.kind() {
                        ErrorKind::NotFound => {
                            write!(
                                f,
                                "Missing snapshot: {}. {HOW_TO_UPDATE_SCREENSHOTS}",
                                path.display()
                            )
                        }
                        err => {
                            write!(
                                f,
                                "Error reading snapshot: {err}\nAt: {}. {HOW_TO_UPDATE_SCREENSHOTS}",
                                path.display()
                            )
                        }
                    },
                    err => {
                        write!(
                            f,
                            "Error decoding snapshot: {err}\nAt: {}.",
                            path.display()
                        )
                    }
                }
            }
            Self::SizeMismatch {
                name,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "'{name}' Image size did not match snapshot. Expected: {expected:?}, Actual: {actual:?}. {HOW_TO_UPDATE_SCREENSHOTS}"
                )
            }
            Self::WriteSnapshot { path, err } => {
                let path = std::path::absolute(path).unwrap_or_else(|_| path.clone());
                write!(f, "Error writing snapshot: {err}\nAt: {}", path.display())
            }
            Self::RenderError { err } => {
                write!(f, "Error rendering image: {err}")
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Test,
    UpdateFailing,
    UpdateAll,
}

impl Mode {
    fn from_env() -> Self {
        let Ok(value) = std::env::var("UPDATE_SNAPSHOTS") else {
            return Self::Test;
        };

        match value.as_str() {
            "false" | "0" | "no" | "off" => Self::Test,
            "true" | "1" | "yes" | "on" => Self::UpdateFailing,
            "force" => Self::UpdateAll,
            unknown => {
                panic!("Unsupported value for UPDATE_SNAPSHOTS: {unknown:?}");
            }
        }
    }

    fn is_update(&self) -> bool {
        !matches!(self, Self::Test)
    }
}

/// Image snapshot test with custom options.
///
/// Snapshot files go under `{options.output_path}/{name}.png`. On diff:
///  - `{output_path}/{name}.new.png` holds the most recent render.
///  - `{output_path}/{name}.diff.png` holds the visualised diff.
///
/// If `UPDATE_SNAPSHOTS` is set, the old image is backed up under
/// `{output_path}/{name}.old.png` before being overwritten.
pub fn try_image_snapshot_options(
    new: &image::RgbaImage,
    name: impl Into<String>,
    options: &SnapshotOptions,
) -> SnapshotResult {
    try_image_snapshot_options_impl(new, name.into(), options)
}

fn try_image_snapshot_options_impl(
    new: &image::RgbaImage,
    name: String,
    options: &SnapshotOptions,
) -> SnapshotResult {
    let mode = Mode::from_env();

    let SnapshotOptions {
        threshold,
        output_path,
        failed_pixel_count_threshold,
    } = options;

    let parent_path = if let Some(parent) = PathBuf::from(&name).parent() {
        output_path.join(parent)
    } else {
        output_path.clone()
    };
    std::fs::create_dir_all(parent_path).ok();

    let snapshot_path = output_path.join(format!("{name}.png"));
    let diff_path = output_path.join(format!("{name}.diff.png"));
    let old_backup_path = output_path.join(format!("{name}.old.png"));
    let new_path = output_path.join(format!("{name}.new.png"));

    std::fs::remove_file(&diff_path).ok();
    std::fs::remove_file(&old_backup_path).ok();
    std::fs::remove_file(&new_path).ok();

    let update_snapshot = || {
        std::fs::rename(&snapshot_path, &old_backup_path).ok();

        new.save(&snapshot_path)
            .map_err(|err| SnapshotError::WriteSnapshot {
                err,
                path: snapshot_path.clone(),
            })?;

        std::fs::remove_file(&new_path).ok();

        eprintln!("Updated snapshot: {}", snapshot_path.display());

        Ok(())
    };

    let write_new_png = || {
        new.save(&new_path)
            .map_err(|err| SnapshotError::WriteSnapshot {
                err,
                path: new_path.clone(),
            })?;
        Ok(())
    };

    let previous = match image::open(&snapshot_path) {
        Ok(image) => image.to_rgba8(),
        Err(err) => {
            // No previous snapshot — probably a new test.
            if mode.is_update() {
                return update_snapshot();
            } else {
                write_new_png()?;

                return Err(SnapshotError::OpenSnapshot {
                    path: snapshot_path.clone(),
                    err,
                });
            }
        }
    };

    if previous.dimensions() != new.dimensions() {
        if mode.is_update() {
            return update_snapshot();
        } else {
            write_new_png()?;

            return Err(SnapshotError::SizeMismatch {
                name,
                expected: previous.dimensions(),
                actual: new.dimensions(),
            });
        }
    }

    // Compare to the existing image.
    let threshold = if mode == Mode::UpdateAll {
        0.0
    } else {
        *threshold
    };

    let result =
        dify::diff::get_results(previous, new.clone(), threshold, true, None, &None, &None);

    let Some((num_wrong_pixels, diff_image)) = result else {
        return Ok(());
    };

    let below_threshold = num_wrong_pixels as i64 <= *failed_pixel_count_threshold as i64;

    if !below_threshold {
        diff_image
            .save(diff_path.clone())
            .map_err(|err| SnapshotError::WriteSnapshot {
                path: diff_path.clone(),
                err,
            })?;
    }

    match mode {
        Mode::Test => {
            if below_threshold {
                Ok(())
            } else {
                write_new_png()?;

                Err(SnapshotError::Diff {
                    name,
                    diff: num_wrong_pixels,
                    diff_path,
                })
            }
        }
        Mode::UpdateFailing => {
            if below_threshold {
                Ok(())
            } else {
                update_snapshot()
            }
        }
        Mode::UpdateAll => update_snapshot(),
    }
}

/// Image snapshot test with default options (config file + fallbacks).
pub fn try_image_snapshot(current: &image::RgbaImage, name: impl Into<String>) -> SnapshotResult {
    try_image_snapshot_options(current, name, &SnapshotOptions::default())
}

/// Panicking version of [`try_image_snapshot_options`].
#[track_caller]
pub fn image_snapshot_options(
    current: &image::RgbaImage,
    name: impl Into<String>,
    options: &SnapshotOptions,
) {
    match try_image_snapshot_options(current, name, options) {
        Ok(()) => {}
        Err(err) => panic!("{err}"),
    }
}

/// Panicking version of [`try_image_snapshot`].
#[track_caller]
pub fn image_snapshot(current: &image::RgbaImage, name: impl Into<String>) {
    match try_image_snapshot(current, name) {
        Ok(()) => {}
        Err(err) => panic!("{err}"),
    }
}

/// Convenience: build an `RgbaImage` from raw RGBA8 pixels + dimensions and
/// snapshot it. Returns the error if the raw buffer size doesn't match.
pub fn try_rgba_snapshot(
    pixels: &[u8],
    width: u32,
    height: u32,
    name: impl Into<String>,
) -> SnapshotResult {
    try_rgba_snapshot_options(pixels, width, height, name, &SnapshotOptions::default())
}

/// Convenience: build an `RgbaImage` from raw RGBA8 pixels + dimensions and
/// snapshot it with custom options.
pub fn try_rgba_snapshot_options(
    pixels: &[u8],
    width: u32,
    height: u32,
    name: impl Into<String>,
    options: &SnapshotOptions,
) -> SnapshotResult {
    let Some(image) = image::RgbaImage::from_raw(width, height, pixels.to_vec()) else {
        return Err(SnapshotError::RenderError {
            err: format!(
                "raw buffer length {} does not match {width}x{height} RGBA8 ({} expected)",
                pixels.len(),
                width as usize * height as usize * 4,
            ),
        });
    };
    try_image_snapshot_options(&image, name, options)
}

/// Panicking version of [`try_rgba_snapshot`].
#[track_caller]
pub fn rgba_snapshot(pixels: &[u8], width: u32, height: u32, name: impl Into<String>) {
    match try_rgba_snapshot(pixels, width, height, name) {
        Ok(()) => {}
        Err(err) => panic!("{err}"),
    }
}

/// Write the given image to a temp file and open it in the OS's default image
/// viewer. Only for debugging — deprecated to prevent accidental CI commits.
#[cfg(all(feature = "debug", not(target_arch = "wasm32")))]
#[deprecated = "Only for debugging, don't commit this."]
pub fn debug_open_snapshot(image: &image::RgbaImage) {
    let temp_file = tempfile::Builder::new()
        .disable_cleanup(true)
        .prefix("kittest-snapshot")
        .suffix(".png")
        .tempfile()
        .expect("Failed to create temp file");

    let path = temp_file.path().to_path_buf();
    image.save(&path).expect("Failed to save debug snapshot");
    let _ = temp_file.into_temp_path();
    eprintln!("Wrote debug snapshot to: {}", path.display());
    if let Err(err) = open::that(&path) {
        eprintln!(
            "Failed to open image {} in default image viewer: {err}",
            path.display()
        );
    }
}
