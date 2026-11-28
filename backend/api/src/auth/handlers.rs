//! HTTP layer for authentication: extract, call the service, set the cookie.

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use time::Duration;

use super::dto::{LoginRequest, MeResponse, RegisterRequest};
use super::service::{self, AuthUser, NewSession, SESSION_DURATION_HOURS};
use crate::errors::AppError;
use crate::extract::AppJson;
use crate::state::AppState;

/// Name of the session cookie, shared by login, logout and the auth extractor.
pub const SESSION_COOKIE: &str = "session";

/// POST /auth/login
///
/// On success, responds 204 No Content with the session cookie.
/// The token is only ever sent in the cookie, never in the response body.
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    AppJson(input): AppJson<LoginRequest>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let session = service::login(&state.db, input).await?;

    Ok((jar.add(session_cookie(session)), StatusCode::NO_CONTENT))
}

/// POST /auth/register (US-01)
///
/// Creates the company, its owner and logs the owner in.
/// On success, responds 201 Created with the session cookie, no body:
/// the front end then calls GET /auth/me to get the company id.
pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    AppJson(input): AppJson<RegisterRequest>,
) -> Result<(CookieJar, StatusCode), AppError> {
    let session = service::register(&state.db, input).await?;

    Ok((jar.add(session_cookie(session)), StatusCode::CREATED))
}

/// Builds the session cookie. Shared by login and register so both
/// always set exactly the same security flags.
fn session_cookie(session: NewSession) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, session.token))
        .http_only(true) // JavaScript can't read it (XSS can't steal it)
        .secure(true) // HTTPS only (browsers allow http://localhost)
        .same_site(SameSite::Strict) // never sent on cross-site requests
        .path("/")
        .max_age(Duration::hours(SESSION_DURATION_HOURS)) // same lifetime as the session
        .build()
}

/// POST /auth/logout
///
/// Always responds 204 No Content, even without a valid session
/// (logging out twice is not an error).
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), AppError> {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        service::logout(&state.db, cookie.value()).await?;
    }

    let jar = jar.remove(Cookie::build(SESSION_COOKIE).path("/"));

    Ok((jar, StatusCode::NO_CONTENT))
}

/// GET /auth/me
///
/// Returns the authenticated caller. No `Result`: if there is no valid
/// session, the `AuthUser` extractor rejects the request before this runs.
pub async fn me(auth: AuthUser) -> Json<MeResponse> {
    let (company_id, role) = match auth.company {
        Some(company) => (Some(company.id), Some(company.role)),
        None => (None, None),
    };

    Json(MeResponse { user_id: auth.user_id, company_id, role })
}
