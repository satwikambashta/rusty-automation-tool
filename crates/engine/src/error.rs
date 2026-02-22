//! Engine-level error types.

use thiserror::Error;

/// Errors produced by the workflow engine (validation + execution).
#[derive(Debug, Error)]
pub enum EngineError {
    // ------ Validation errors ------

    /// Two or more nodes share the same ID.
    #[error("duplicate node ID: '{0}'")]
    DuplicateNodeId(String),

    /// An edge references a node ID that doesn't exist in the workflow.
    #[error("edge references unknown node '{node_id}' ({side} side)")]
    UnknownNodeReference {
        node_id: String,
        side: &'static str,
    },

    /// Topological sort detected a cycle.
    #[error("workflow graph contains a cycle")]
    CycleDetected,

    // ------ Execution errors ------

    /// A node failed with a fatal error; the whole execution is aborted.
    #[error("node '{node_id}' failed fatally: {message}")]
    NodeFatal {
        node_id: String,
        message: String,
    },

    /// A node's retryable error was exhausted.
    #[error("node '{node_id}' exceeded retry limit: {message}")]
    NodeRetryExhausted {
        node_id: String,
        message: String,
    },

    /// Persistence error from the db crate.
    #[error("database error: {0}")]
    Database(#[from] db::DbError),
}
