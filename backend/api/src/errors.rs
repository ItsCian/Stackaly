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
    #[error("Resource not found")]
    NotFound,

    // Our own business checks (e.g. a VAT number in the wrong format).
    #[error("{0}")]
    Validation(String),

    // The request conflicts with existing data (e.g. duplicate VAT number).
    #[error("{0}")]
    Conflict(String),

    // Axum rejected the request before our handler ran (bad JSON, bad path…).
    // We keep Axum's status code (400, 415, 422…) and only change the format.
    #[error("{message}")]
    InvalidRequest { status: StatusCode, message: String },

    // Something broke on our side. The detail is logged, never sent.
    #[error("Internal server error")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),

            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),

            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),

            // `status` is a &StatusCode here (we match on &self),
            // so * copies the value out. StatusCode is Copy, so this is free.
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

// Database errors → AppError, so `?` works on SeaORM calls.
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        match err.sql_err() {
            // Safety net: a UNIQUE violation a service didn't handle → 409, not 500.
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                AppError::Conflict("This resource already exists".into())
            }
            _ => AppError::Internal(err.to_string()),
        }
    }
}

// Invalid JSON body → AppError (used by the AppJson extractor).
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::InvalidRequest {
            status: rejection.status(),     // the status Axum chose: 400, 415 or 422
            message: rejection.body_text(), // Axum's explanation, e.g. "missing field `city`"
        }
    }
}

// Invalid path parameter → AppError (used by the AppPath extractor).
impl From<PathRejection> for AppError {
    fn from(rejection: PathRejection) -> Self {
        AppError::InvalidRequest { status: rejection.status(), message: rejection.body_text() }
    }
}
