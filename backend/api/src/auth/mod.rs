//! Authentication: login, logout and session checks.

pub mod dto;
pub mod extractor;
pub mod handlers;
pub mod password;
pub mod service;
pub mod token;

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

/// Routes mounted under /auth by `build_app`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
}

/// Trims surrounding whitespace and lowercases an email, so that
/// " Cian@X.be" and "cian@x.be" are treated as the same account.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}
