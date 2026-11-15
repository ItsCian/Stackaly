pub mod errors;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::errors::AppError;

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

// Temporary route to test internal errors
async fn boom() -> Result<Json<Value>, AppError> {
    Err(AppError::Internal("DB Exploded".into()))
}

// Handle requests to unknown routes
async fn not_found() -> AppError {
    AppError::NotFound
}

pub fn build_app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/boom", get(boom))
        .fallback(not_found)
}
