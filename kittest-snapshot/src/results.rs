//! Collector that panics on drop if any snapshot errored.
//!
//! Ported verbatim from `egui_kittest::snapshot::SnapshotResults`.

use std::fmt::Display;

use crate::{SnapshotError, SnapshotResult};

/// Collect [`SnapshotResult`]s and panic on drop if any failed. Lets a test
/// run all of its snapshots and see every failure at once instead of bailing
/// at the first.
///
/// # Panics
/// Panics on drop if there are any errors, unless `.into_result()` / `.into_inner()` /
/// `.unwrap()` has been called. This ensures you can't silently forget to
/// handle a result.
#[derive(Debug)]
pub struct SnapshotResults {
    errors: Vec<SnapshotError>,
    handled: bool,
    location: std::panic::Location<'static>,
}

impl Default for SnapshotResults {
    #[track_caller]
    fn default() -> Self {
        Self {
            errors: Vec::new(),
            handled: true, // Empty → nothing to handle.
            location: *std::panic::Location::caller(),
        }
    }
}

impl Display for SnapshotResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            write!(f, "All snapshots passed")
        } else {
            writeln!(f, "Snapshot errors:")?;
            for error in &self.errors {
                writeln!(f, "  {error}")?;
            }
            Ok(())
        }
    }
}

impl SnapshotResults {
    #[track_caller]
    pub fn new() -> Self {
        Default::default()
    }

    /// Record a result. Errors are retained; `Ok(())` is dropped.
    pub fn add(&mut self, result: SnapshotResult) {
        self.handled = false;
        if let Err(err) = result {
            self.errors.push(err);
        }
    }

    /// Merge another `SnapshotResults` into `self`.
    pub fn extend(&mut self, other: Self) {
        self.handled = false;
        self.errors.extend(other.into_inner());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Consume self; return `Ok(())` if no errors, otherwise `Err(self)` so
    /// the caller can `?` it.
    pub fn into_result(self) -> Result<(), Self> {
        if self.has_errors() {
            Err(self)
        } else {
            Ok(())
        }
    }

    /// Consume self and return the list of errors.
    pub fn into_inner(mut self) -> Vec<SnapshotError> {
        self.handled = true;
        std::mem::take(&mut self.errors)
    }

    /// Panics if any results errored. Exists purely to name the act of
    /// consuming the value; the actual panic is in [`Drop`].
    pub fn unwrap(self) {
        // Handled in drop.
    }
}

impl From<SnapshotResults> for Vec<SnapshotError> {
    fn from(results: SnapshotResults) -> Self {
        results.into_inner()
    }
}

impl Drop for SnapshotResults {
    #[track_caller]
    fn drop(&mut self) {
        // Don't panic if we're already panicking (the test failed for another
        // reason, and double-panic aborts without output).
        if std::thread::panicking() {
            return;
        }
        if self.has_errors() {
            panic!("{}", self);
        }

        thread_local! {
            static UNHANDLED_SNAPSHOT_RESULTS_COUNTER: std::cell::RefCell<usize> =
                const { std::cell::RefCell::new(0) };
        }

        if !self.handled {
            let count = UNHANDLED_SNAPSHOT_RESULTS_COUNTER.with(|counter| {
                let mut count = counter.borrow_mut();
                *count += 1;
                *count
            });

            if count >= 2 {
                panic!(
                    "\n\
Multiple SnapshotResults were dropped without being handled.\n\
\n\
In order to allow consistent snapshot updates, all snapshot results within a test \
should be merged in a single SnapshotResults instance. Usually this is handled \
internally in a harness; if you have multiple harnesses, merge their results \
with `SnapshotResults::extend`.\n\
\n\
The SnapshotResults was constructed at {}\n\
                    ",
                    self.location
                );
            }
        }
    }
}
