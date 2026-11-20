//! HTTP layer for companies: extract, call the service, respond.

use axum::{Json, extract::State, http::StatusCode};

use super::dto::CreateCompany;
use super::service;
use crate::entities::companies;
use crate::errors::AppError;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateCompany>,
) -> Result<(StatusCode, Json<companies::Model>), AppError> {
    let company = service::create(&state.db, input).await?;

    Ok((StatusCode::CREATED, Json(company)))
}
