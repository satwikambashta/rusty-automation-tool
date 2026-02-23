//! `api` crate — HTTP REST API layer
//!
//! Exposes:
//!   GET    /api/v1/workflows
//!   POST   /api/v1/workflows
//!   GET    /api/v1/workflows/:id
//!   DELETE /api/v1/workflows/:id
//!   POST   /api/v1/workflows/:id/execute
//!   POST   /webhook/:path

pub mod handlers;

use axum::{
    routing::{get, post, delete},
    Router,
};
use db::DbPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
}

pub async fn serve(bind: &str, pool: DbPool) -> Result<(), std::io::Error> {
    let state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_router = Router::new()
        .route("/workflows", get(handlers::workflows::list).post(handlers::workflows::create))
        .route("/workflows/:id", get(handlers::workflows::get).delete(handlers::workflows::delete))
        .route("/workflows/:id/execute", post(handlers::executions::execute));

    let app = Router::new()
        .nest("/api/v1", api_router)
        .route("/webhook/:path", post(handlers::webhooks::handle_webhook))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("Server listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await
}
