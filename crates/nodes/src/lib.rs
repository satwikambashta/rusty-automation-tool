//! `nodes` crate — the `ExecutableNode` trait and built-in node implementations.
//!
//! Every node — built-in and plugin alike — must implement [`ExecutableNode`].
//! The engine crate dispatches execution through this trait object.

pub mod error;
pub mod traits;
pub mod mock;

pub use error::NodeError;
pub use traits::ExecutableNode;
