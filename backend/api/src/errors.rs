use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{DbErr, SqlErr}; // NEW
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Conflict(String),

    #[error("Internal server error")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),

            AppError::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),

            AppError::Conflict(_) => (StatusCode::CONFLICT, "conflict"),

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

// Converts any SeaORM error into an AppError, so `?` works on DB calls.
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        match err.sql_err() {
            // NEW: safety net. Any UNIQUE violation a service didn't handle itself
            // becomes a 409 with a generic message instead of a 500.
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                AppError::Conflict("This resource already exists".into())
            }
            // Everything else: logged in detail, generic 500 for the client.
            _ => AppError::Internal(err.to_string()),
        }
    }
}
