use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use uuid::Uuid;
use super::AppState;
use db::repository::workflows as wf_repo;
use engine::Workflow;

#[derive(serde::Deserialize)]
pub struct CreateWorkflowDto {
    pub name: String,
    pub definition: Value,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<db::models::WorkflowRow>>, StatusCode> {
    match wf_repo::list_workflows(&state.pool).await {
        Ok(workflows) => Ok(Json(workflows)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<db::models::WorkflowRow>, StatusCode> {
    match wf_repo::get_workflow(&state.pool, id).await {
        Ok(wf) => Ok(Json(wf)),
        Err(db::DbError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkflowDto>,
) -> Result<(StatusCode, Json<db::models::WorkflowRow>), StatusCode> {
    // Basic validation to ensure definition is a valid Workflow struct
    if let Err(_) = serde_json::from_value::<Workflow>(payload.definition.clone()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    match wf_repo::create_workflow(&state.pool, &payload.name, payload.definition).await {
        Ok(wf) => Ok((StatusCode::CREATED, Json(wf))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    match wf_repo::delete_workflow(&state.pool, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(db::DbError::NotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
