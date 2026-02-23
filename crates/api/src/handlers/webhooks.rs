use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use super::AppState;
use db::repository::{executions as exec_repo, jobs as job_repo, workflows as wf_repo};
use engine::Workflow;

pub async fn handle_webhook(
    Path(path): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    // 1. Find workflow by webhook path
    let workflows = match wf_repo::list_workflows(&state.pool).await {
        Ok(wfs) => wfs,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let matched_wf = workflows.into_iter().find(|w| {
        let wf: Result<Workflow, _> = serde_json::from_value(w.definition.clone());
        if let Ok(workflow) = wf {
            if let engine::Trigger::Webhook { path: trigger_path } = &workflow.trigger {
                if trigger_path == &path {
                    return true;
                }
            }
        }
        false
    });

    let wf_row = match matched_wf {
        Some(w) => w,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // 2. Trigger execution
    let exec = match exec_repo::create_execution(&state.pool, wf_row.id).await {
        Ok(e) => e,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let _job = match job_repo::enqueue_job(&state.pool, exec.id, wf_row.id, payload.clone()).await {
        Ok(j) => j,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({"message": "webhook accepted"}))))
}
