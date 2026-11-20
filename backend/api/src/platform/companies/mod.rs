//! Companies (tenants): sign-up, read, update, suspend.

mod dto; // request/response structs
mod handlers; // HTTP layer
mod service; // business logic + DB

use axum::{Router, routing::post};

use crate::state::AppState;

/// Routes mounted under /companies by platform::router().
pub fn router() -> Router<AppState> {
    Router::new().route("/", post(handlers::create))
}
