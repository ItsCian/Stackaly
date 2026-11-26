//! Companies (tenants): sign-up, read, update, suspend.

pub(crate) mod dto; // request/response structs (auth::register needs CreateCompany)
mod handlers; // HTTP layer
pub(crate) mod service;
pub(crate) mod vat; // Belgian VAT number validation // business logic + DB (auth::register calls create)

use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

/// Routes mounted under /companies by platform::router().
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create))
        .route("/{id}", get(handlers::get_by_id).patch(handlers::update).delete(handlers::delete))
}
