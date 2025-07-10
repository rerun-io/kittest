#![allow(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]
mod filter;
mod node;
mod query;
mod state;

/// Re-export of the [`accesskit_consumer::Node`] with a more convenient name.
pub use accesskit_consumer::Node as AccessKitNode;
pub use filter::*;
pub use node::*;
pub use query::*;
pub use state::*;
