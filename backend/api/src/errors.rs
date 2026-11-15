use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("{0}")]
    Validation(String),

    #[error("Internal server error")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "not_found"),

            AppError::Validation(_) => {
                (StatusCode::BAD_REQUEST, "validation_error")
            }

            AppError::Internal(detail) => {
                tracing::error!("Internal server error: {}", detail);

                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };

        let body = json!({
            "error": code,
            "message": self.to_string(),
        });

        (status, Json(body)).into_response()
    }
}
