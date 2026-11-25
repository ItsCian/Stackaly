pub mod auth;
pub mod config;
pub mod entities;
pub mod errors;
pub mod extract;
pub mod platform;
pub mod state;

use axum::{Json, Router, extract::State, routing::get};
use serde_json::{Value, json};

use crate::errors::AppError;
use crate::state::AppState;

async fn health(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    state.db.ping().await?;

    Ok(Json(json!({
        "status": "ok",
        "database": "ok"
    })))
}

async fn not_found() -> AppError {
    AppError::NotFound
}

pub fn build_app(state: AppState) -> Router {
    // All API routes, written without the /api prefix
    let api = Router::new()
        .route("/health", get(health))
        .merge(platform::router())
        // Unknown URL under /api → JSON NotFound error
        .fallback(not_found);

    Router::new()
        // Prefixes every route in `api`: /health → /api/health,
        // /companies → /api/companies
        .nest("/api", api)
        // Gives the state to every route, nested ones included
        .with_state(state)
}
