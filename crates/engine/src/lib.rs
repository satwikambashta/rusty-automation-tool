//! `engine` crate — core domain models, DAG validation, and the execution engine.

pub mod models;
pub mod error;
pub mod dag;
pub mod executor;

pub use models::{Workflow, Trigger, NodeDefinition, Edge};
pub use error::EngineError;
pub use dag::validate_dag;
pub use executor::WorkflowExecutor;

#[cfg(test)]
mod executor_tests;
