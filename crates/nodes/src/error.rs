//! Node-level error type.

use thiserror::Error;

/// Errors returned by a node's `execute` method.
///
/// The engine uses the variant to decide retry behaviour:
/// - `Retryable` — the job is re-queued with exponential back-off.
/// - `Fatal`     — the execution is immediately marked as failed.
#[derive(Debug, Error, Clone)]
pub enum NodeError {
    /// Transient failure; the engine should re-try the job.
    #[error("retryable node error: {0}")]
    Retryable(String),

    /// Permanent failure; no retry should be attempted.
    #[error("fatal node error: {0}")]
    Fatal(String),
}
