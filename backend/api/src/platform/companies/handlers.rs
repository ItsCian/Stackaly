//! HTTP layer for companies: extract, call the service, respond.

use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::dto::{CreateCompany, UpdateCompany};
use super::service;
use crate::entities::companies;
use crate::errors::AppError;
use crate::extract::{AppJson, AppPath};
use crate::state::AppState;

/// POST /companies
pub async fn create(
    State(state): State<AppState>,
    AppJson(input): AppJson<CreateCompany>, // our extractor: errors in our JSON format
) -> Result<(StatusCode, Json<companies::Model>), AppError> {
    let company = service::create(&state.db, input).await?;

    Ok((StatusCode::CREATED, Json(company)))
}

/// GET /companies
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<companies::Model>>, AppError> {
    let company = service::list(&state.db).await?;

    Ok(Json(company))
}

/// GET /companies/{id}
pub async fn get_by_id(
    State(state): State<AppState>,
    AppPath(id): AppPath<Uuid>, // our extractor: "abc" → 400 in our JSON format
) -> Result<Json<companies::Model>, AppError> {
    let company = service::get_by_id(&state.db, id).await?;

    Ok(Json(company))
}

/// PATCH /companies/{id}
pub async fn update(
    State(state): State<AppState>,
    AppPath(id): AppPath<Uuid>,
    AppJson(input): AppJson<UpdateCompany>,
) -> Result<Json<companies::Model>, AppError> {
    let company = service::update(&state.db, id, input).await?;

    Ok(Json(company))
}

/// DELETE /companies/{id}
pub async fn delete(
    State(state): State<AppState>,
    AppPath(id): AppPath<Uuid>,
) -> Result<StatusCode, AppError> {
    service::delete(&state.db, id).await?;

    Ok(StatusCode::NO_CONTENT)
}
