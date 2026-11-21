//! Companies (tenants): sign-up, read, update, suspend.

mod dto; // request/response structs
mod handlers; // HTTP layer
mod service; // business logic + DB

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

/// Routes mounted under /companies by platform::router().
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create).get(handlers::list))
        .route("/{id}", get(handlers::get_by_id).patch(handlers::update).delete(handlers::delete))
}
