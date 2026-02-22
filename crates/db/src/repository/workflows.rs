//! Workflow CRUD operations.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{DbError, models::WorkflowRow};

/// Insert a new workflow into the database.
///
/// `definition` must be a valid JSON object produced by serialising the
/// domain `Workflow` type from the `engine` crate.
pub async fn create_workflow(
    pool: &PgPool,
    name: &str,
    definition: serde_json::Value,
) -> Result<WorkflowRow, DbError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = sqlx::query_as!(
        WorkflowRow,
        r#"
        INSERT INTO workflows (id, name, definition, created_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, definition, created_at
        "#,
        id,
        name,
        definition,
        now,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Fetch a single workflow by its primary key.
pub async fn get_workflow(pool: &PgPool, id: Uuid) -> Result<WorkflowRow, DbError> {
    let row = sqlx::query_as!(
        WorkflowRow,
        r#"SELECT id, name, definition, created_at FROM workflows WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(DbError::NotFound)?;

    Ok(row)
}

/// Return all workflows ordered by creation time (newest first).
pub async fn list_workflows(pool: &PgPool) -> Result<Vec<WorkflowRow>, DbError> {
    let rows = sqlx::query_as!(
        WorkflowRow,
        r#"SELECT id, name, definition, created_at FROM workflows ORDER BY created_at DESC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Permanently delete a workflow by its primary key.
///
/// Returns `DbError::NotFound` if no row was deleted.
pub async fn delete_workflow(pool: &PgPool, id: Uuid) -> Result<(), DbError> {
    let result = sqlx::query!("DELETE FROM workflows WHERE id = $1", id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    Ok(())
}
