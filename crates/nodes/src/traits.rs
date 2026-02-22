//! The `ExecutableNode` trait — the contract every node must fulfil.

use async_trait::async_trait;
use serde_json::Value;

use crate::NodeError;

/// Shared context passed to every node during execution.
///
/// Defined here (in the nodes crate) so both the engine and individual node
/// implementations can import it without a circular dependency.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// ID of the parent workflow.
    pub workflow_id: uuid::Uuid,
    /// ID of the current execution run.
    pub execution_id: uuid::Uuid,
    /// Initial input supplied when the execution was triggered.
    pub input: Value,
    /// Decrypted secrets scoped to this workflow.
    pub secrets: std::collections::HashMap<String, String>,
}

/// The core node trait.
///
/// All built-in nodes and WASM plugins must implement this.
#[async_trait]
pub trait ExecutableNode: Send + Sync {
    /// Execute the node, receive the *previous* node's JSON output as `input`,
    /// and return this node's JSON output.
    async fn execute(
        &self,
        input: Value,
        ctx: &ExecutionContext,
    ) -> Result<Value, NodeError>;
}
