//! Our own extractors: they parse exactly like Axum's, but a failure
//! becomes an AppError, so every error response has the same JSON format.

use axum::extract::{FromRequest, FromRequestParts};

use crate::errors::AppError;

/// Like `axum::Json`, but a bad body becomes `AppError::InvalidRequest`.
/// `via(axum::Json)`: let Axum do the parsing.
/// `rejection(AppError)`: on failure, convert with From<JsonRejection>.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

/// Like `axum::extract::Path`, but a bad path parameter becomes
/// `AppError::InvalidRequest` (via From<PathRejection>).
/// FromRequestParts (not FromRequest) because the path is not the body.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(AppError))]
pub struct AppPath<T>(pub T);
