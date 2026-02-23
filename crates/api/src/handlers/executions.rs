use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use uuid::Uuid;
use super::AppState;
use db::repository::{executions as exec_repo, jobs as job_repo};

#[derive(serde::Deserialize)]
pub struct ExecuteWorkflowDto {
    pub input: Value,
}

pub async fn execute(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<ExecuteWorkflowDto>,
) -> Result<(StatusCode, Json<db::models::JobRow>), StatusCode> {
    // 1. Create a `pending` execution record
    let exec = match exec_repo::create_execution(&state.pool, id).await {
        Ok(e) => e,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // 2. Queue the job for background worker
    // The payload represents initial input.
    let job = match job_repo::enqueue_job(&state.pool, exec.id, id, payload.input).await {
        Ok(j) => j,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok((StatusCode::ACCEPTED, Json(job)))
}
