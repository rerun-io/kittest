#![cfg_attr(doc, doc = include_str!("../README.md"))]
mod filter;
mod node;
mod query;
mod state;

/// Re-export of the [`accesskit_consumer::NodeRef`] with a more convenient name.
pub use accesskit_consumer::NodeRef as AccessKitNode;
pub use filter::*;
pub use node::*;
pub use query::*;
pub use state::*;
