//! The platform core: features every company uses,
//! whatever modules it has enabled.

pub mod companies;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().nest("/companies", companies::router())
}
