//! Job queue repository functions.
//!
//! The MVP queue is backed by the `job_queue` Postgres table.
//! Workers poll the table and use `SELECT … FOR UPDATE SKIP LOCKED`
//! for safe concurrent processing.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{DbError, models::JobRow};

/// Enqueue a new job for the given execution.
///
/// `payload` is arbitrary JSON that the worker will pass back to the engine.
pub async fn enqueue_job(
    pool: &PgPool,
    execution_id: Uuid,
    workflow_id: Uuid,
    payload: serde_json::Value,
) -> Result<JobRow, DbError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let row = sqlx::query_as!(
        JobRow,
        r#"
        INSERT INTO job_queue
            (id, execution_id, workflow_id, status, attempts, max_attempts, payload, created_at, updated_at)
        VALUES ($1, $2, $3, 'pending', 0, 3, $4, $5, $5)
        RETURNING id, execution_id, workflow_id, status, attempts, max_attempts, payload, created_at, updated_at
        "#,
        id,
        execution_id,
        workflow_id,
        payload,
        now,
    )
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Atomically fetch the oldest pending job and mark it as `processing`.
///
/// Uses `SELECT … FOR UPDATE SKIP LOCKED` so multiple workers can poll
/// safely without stepping on each other.
///
/// Returns `None` if no pending jobs exist.
pub async fn fetch_next_job(pool: &PgPool) -> Result<Option<JobRow>, DbError> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as!(
        JobRow,
        r#"
        SELECT id, execution_id, workflow_id, status, attempts, max_attempts, payload, created_at, updated_at
        FROM job_queue
        WHERE status = 'pending'
        ORDER BY created_at ASC
        LIMIT 1
        FOR UPDATE SKIP LOCKED
        "#,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(ref job) = row {
        let now = Utc::now();
        sqlx::query!(
            r#"
            UPDATE job_queue
            SET status = 'processing', attempts = attempts + 1, updated_at = $1
            WHERE id = $2
            "#,
            now,
            job.id,
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
    } else {
        tx.rollback().await?;
    }

    Ok(row)
}

/// Mark a job as completed.
pub async fn complete_job(pool: &PgPool, job_id: Uuid) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE job_queue SET status = 'completed', updated_at = $1 WHERE id = $2",
        Utc::now(),
        job_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark a job as failed (or dead-lettered when `max_attempts` is reached).
pub async fn fail_job(pool: &PgPool, job_id: Uuid, max_attempts: i32) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE job_queue
        SET status = CASE WHEN attempts >= $1 THEN 'dead_lettered' ELSE 'pending' END,
            updated_at = $2
        WHERE id = $3
        "#,
        max_attempts,
        Utc::now(),
        job_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
