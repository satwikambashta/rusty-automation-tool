//! Workflow execution engine.
//!
//! `WorkflowExecutor` is the central orchestrator:
//! 1. Validates the DAG and produces a topological ordering.
//! 2. Iterates through nodes in order, dispatching each via `ExecutableNode`.
//! 3. Passes the previous node's JSON output as input to the next node.
//! 4. Persists per-node results via the `db` crate.
//! 5. Handles `NodeError::Retryable` (up to `max_retries`) and
//!    `NodeError::Fatal` (abort immediately).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use serde_json::Value;
use tracing::{info, warn, error, instrument};

use db::DbPool;
use nodes::{ExecutableNode, NodeError};
use nodes::traits::ExecutionContext;

use crate::{EngineError, Workflow};
use crate::dag::validate_dag;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Tuning knobs for the executor.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum number of times a retryable node failure will be retried.
    pub max_retries: u32,
    /// Base delay for exponential back-off between retries.
    pub retry_base_delay: Duration,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_base_delay: Duration::from_millis(100),
        }
    }
}

// ---------------------------------------------------------------------------
// Node registry
// ---------------------------------------------------------------------------

/// Maps `node_type` strings to boxed `ExecutableNode` implementations.
pub type NodeRegistry = HashMap<String, Arc<dyn ExecutableNode>>;

// ---------------------------------------------------------------------------
// Output of a completed execution
// ---------------------------------------------------------------------------

/// The result of running a full workflow.
#[derive(Debug)]
pub struct ExecutionResult {
    /// ID of the `workflow_executions` row created for this run.
    pub execution_id: uuid::Uuid,
    /// The JSON output produced by the *last* node in the sorted order.
    pub output: Value,
}

// ---------------------------------------------------------------------------
// WorkflowExecutor
// ---------------------------------------------------------------------------

/// Stateless orchestrator that runs a single workflow execution.
///
/// Construct one executor per process (or even per execution) and call
/// [`WorkflowExecutor::run`] with the workflow and initial input.
pub struct WorkflowExecutor {
    pool: DbPool,
    registry: NodeRegistry,
    config: ExecutorConfig,
}

impl WorkflowExecutor {
    /// Create a new executor.
    pub fn new(pool: DbPool, registry: NodeRegistry, config: ExecutorConfig) -> Self {
        Self { pool, registry, config }
    }

    /// Run the workflow and return the final output.
    ///
    /// # Errors
    /// Returns `EngineError` for validation failures, fatal node errors,
    /// retry exhaustion, or database problems.
    #[instrument(skip(self, initial_input), fields(workflow_id = %workflow.id))]
    pub async fn run(
        &self,
        workflow: &Workflow,
        initial_input: Value,
    ) -> Result<ExecutionResult, EngineError> {
        // ------------------------------------------------------------------
        // Validate and topologically sort the DAG.
        // ------------------------------------------------------------------
        let sorted_ids = validate_dag(workflow)?;
        info!(
            "DAG validated — executing {} nodes in order: {:?}",
            sorted_ids.len(), sorted_ids
        );

        // ------------------------------------------------------------------
        // Create the workflow_execution row.
        // ------------------------------------------------------------------
        let exec_row = db::repository::executions::create_execution(&self.pool, workflow.id)
            .await?;
        let execution_id = exec_row.id;

        db::repository::executions::update_execution_status(
            &self.pool, execution_id, "running", false,
        )
        .await?;

        // ------------------------------------------------------------------
        // Build a lookup map: node_id → NodeDefinition.
        // ------------------------------------------------------------------
        let node_map: HashMap<&str, _> = workflow
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        // ------------------------------------------------------------------
        // Build the shared context (secrets not implemented yet — empty map).
        // ------------------------------------------------------------------
        let ctx = ExecutionContext {
            workflow_id: workflow.id,
            execution_id,
            input: initial_input.clone(),
            secrets: HashMap::new(),
        };

        // ------------------------------------------------------------------
        // Execute nodes sequentially.
        // ------------------------------------------------------------------
        let mut current_input = initial_input;

        for node_id in &sorted_ids {
            let node_def = node_map[node_id.as_str()];

            let node_impl = self.registry.get(&node_def.node_type).ok_or_else(|| {
                EngineError::NodeFatal {
                    node_id: node_id.clone(),
                    message: format!(
                        "no implementation registered for node_type '{}'",
                        node_def.node_type
                    ),
                }
            })?;

            let node_output = self
                .execute_with_retry(node_id, node_impl.as_ref(), current_input.clone(), &ctx)
                .await;

            match node_output {
                Ok(output) => {
                    // Persist success.
                    let started_at = Utc::now(); // approximate — good enough for scaffold
                    db::repository::executions::insert_node_execution(
                        &self.pool,
                        execution_id,
                        node_id,
                        current_input.clone(),
                        Some(output.clone()),
                        "succeeded",
                        started_at,
                    )
                    .await?;

                    info!("node '{}' succeeded", node_id);
                    current_input = output;
                }

                Err(engine_err) => {
                    // Persist failure.
                    let started_at = Utc::now();
                    let _ = db::repository::executions::insert_node_execution(
                        &self.pool,
                        execution_id,
                        node_id,
                        current_input.clone(),
                        None,
                        "failed",
                        started_at,
                    )
                    .await;

                    error!("node '{}' failed: {}", node_id, engine_err);

                    // Mark the whole execution as failed.
                    let _ = db::repository::executions::update_execution_status(
                        &self.pool,
                        execution_id,
                        "failed",
                        true,
                    )
                    .await;

                    return Err(engine_err);
                }
            }
        }

        // ------------------------------------------------------------------
        // Mark execution as succeeded.
        // ------------------------------------------------------------------
        db::repository::executions::update_execution_status(
            &self.pool, execution_id, "succeeded", true,
        )
        .await?;

        info!("workflow '{}' execution {} succeeded", workflow.id, execution_id);

        Ok(ExecutionResult {
            execution_id,
            output: current_input,
        })
    }

    // -----------------------------------------------------------------------
    // Internal: execute a single node with retry logic.
    // -----------------------------------------------------------------------

    async fn execute_with_retry(
        &self,
        node_id: &str,
        node: &dyn ExecutableNode,
        input: Value,
        ctx: &ExecutionContext,
    ) -> Result<Value, EngineError> {
        let mut attempts = 0u32;

        loop {
            match node.execute(input.clone(), ctx).await {
                Ok(output) => return Ok(output),

                Err(NodeError::Fatal(msg)) => {
                    return Err(EngineError::NodeFatal {
                        node_id: node_id.to_owned(),
                        message: msg,
                    });
                }

                Err(NodeError::Retryable(msg)) => {
                    attempts += 1;
                    if attempts > self.config.max_retries {
                        return Err(EngineError::NodeRetryExhausted {
                            node_id: node_id.to_owned(),
                            message: msg,
                        });
                    }

                    let delay = self.config.retry_base_delay
                        * 2u32.pow(attempts.saturating_sub(1));

                    warn!(
                        "node '{}' retryable error (attempt {}/{}), retrying in {:?}: {}",
                        node_id, attempts, self.config.max_retries, delay, msg
                    );

                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}
