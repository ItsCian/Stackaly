//! Application error type: every handler returns `Result<_, AppError>`,
//! and every error is sent to the client in the same JSON format:
//! `{ "code": "...", "message": "..." }`.

use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{DbErr, SqlErr};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The requested resource does not exist.
    #[error("Resource not found")]
    NotFound,

    /// No valid session: missing cookie, unknown token or expired session.
    #[error("Authentication required")]
    Unauthorized,

    /// Login failed. Same message for an unknown email and a wrong password,
    /// so the response never reveals which accounts exist.
    #[error("Invalid email or password")]
    InvalidCredentials,

    /// The user is authenticated but not allowed to do this
    /// (e.g. disabled account, missing role).
    #[error("{0}")]
    Forbidden(String),

    /// A business rule was not respected (e.g. invalid VAT number format).
    #[error("{0}")]
    Validation(String),

    /// The request conflicts with existing data (e.g. duplicate VAT number).
    #[error("{0}")]
    Conflict(String),

    /// Axum rejected the request before the handler ran (malformed JSON,
    /// invalid path parameter). Axum's status code (400, 415, 422) is kept.
    #[error("{message}")]
    InvalidRequest { status: StatusCode, message: String },

    /// Unexpected server-side failure. The detail is logged, never sent.
    #[error("Internal server error")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),

            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),

            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),

            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),

            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),

            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),

            // `status` is borrowed here; StatusCode is Copy, so `*` is free.
            AppError::InvalidRequest { status, .. } => (*status, "invalid_request"),

            AppError::Internal(detail) => {
                tracing::error!(detail = %detail, "internal server error");

                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };

        let body = json!({
            "code": code,
            "message": self.to_string(),
        });

        (status, Json(body)).into_response()
    }
}

/// Database errors, so `?` works on SeaORM calls.
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        match err.sql_err() {
            // Safety net: a unique violation not handled by a service gives 409, not 500.
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                AppError::Conflict("This resource already exists".into())
            }
            _ => AppError::Internal(err.to_string()),
        }
    }
}

/// Invalid JSON body (used by the `AppJson` extractor).
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::InvalidRequest { status: rejection.status(), message: rejection.body_text() }
    }
}

/// Invalid path parameter (used by the `AppPath` extractor).
impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::InvalidRequest { status: rejection.status(), message: rejection.body_text() }
    }
}
