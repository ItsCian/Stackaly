//! `AuthUser` as an Axum extractor: any handler that takes an `AuthUser`
//! parameter only runs for an authenticated caller.

use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use super::handlers::SESSION_COOKIE;
use super::service::{self, AuthUser};
use crate::errors::AppError;
use crate::state::AppState;

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar.get(SESSION_COOKIE).ok_or(AppError::Unauthorized)?;
        service::authenticate(&state.db, cookie.value()).await
    }
}
