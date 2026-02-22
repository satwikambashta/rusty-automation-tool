//! Execution and node-execution repository functions.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    DbError,
    models::{WorkflowExecutionRow, NodeExecutionRow},
};

// ---------------------------------------------------------------------------
// workflow_executions
// ---------------------------------------------------------------------------

/// Create a new workflow execution record in `pending` status.
pub async fn create_execution(
    pool: &PgPool,
    workflow_id: Uuid,
) -> Result<WorkflowExecutionRow, DbError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = sqlx::query_as!(
        WorkflowExecutionRow,
        r#"
        INSERT INTO workflow_executions (id, workflow_id, status, started_at)
        VALUES ($1, $2, 'pending', $3)
        RETURNING id, workflow_id, status, started_at, finished_at
        "#,
        id,
        workflow_id,
        now,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Update the `status` (and optionally `finished_at`) of a workflow execution.
pub async fn update_execution_status(
    pool: &PgPool,
    execution_id: Uuid,
    status: &str,
    finished: bool,
) -> Result<(), DbError> {
    if finished {
        sqlx::query!(
            r#"
            UPDATE workflow_executions
            SET status = $1, finished_at = $2
            WHERE id = $3
            "#,
            status,
            Utc::now(),
            execution_id,
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            r#"UPDATE workflow_executions SET status = $1 WHERE id = $2"#,
            status,
            execution_id,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// node_executions
// ---------------------------------------------------------------------------

/// Insert a completed node execution record.
pub async fn insert_node_execution(
    pool: &PgPool,
    execution_id: Uuid,
    node_id: &str,
    input: serde_json::Value,
    output: Option<serde_json::Value>,
    status: &str,
    started_at: chrono::DateTime<Utc>,
) -> Result<NodeExecutionRow, DbError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = sqlx::query_as!(
        NodeExecutionRow,
        r#"
        INSERT INTO node_executions
            (id, execution_id, node_id, input, output, status, started_at, finished_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, execution_id, node_id, input, output, status, started_at, finished_at
        "#,
        id,
        execution_id,
        node_id,
        input,
        output,
        status,
        started_at,
        now,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}
