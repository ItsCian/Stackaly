pub mod errors;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::errors::AppError;

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok"
    }))
}

async fn not_found() -> AppError {
    AppError::NotFound
}

pub fn build_app() -> Router {
    Router::new()
        .route("/health", get(health))
        .fallback(not_found)
}
