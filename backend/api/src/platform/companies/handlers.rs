//! HTTP layer for companies: extract, call the service, respond.

use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::dto::{CreateCompany, UpdateCompany};
use super::service;
use crate::auth::service::AuthUser;
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

/// GET /companies/{id} (members of this company only)
pub async fn get_by_id(
    auth: AuthUser,
    State(state): State<AppState>,
    AppPath(id): AppPath<Uuid>,
) -> Result<Json<companies::Model>, AppError> {
    auth.company(id)?;

    let company = service::get_by_id(&state.db, id).await?;

    Ok(Json(company))
}

/// PATCH /companies/{id} (owner only)
pub async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    AppPath(id): AppPath<Uuid>,
    AppJson(input): AppJson<UpdateCompany>,
) -> Result<Json<companies::Model>, AppError> {
    auth.owned_company(id)?;

    let company = service::update(&state.db, id, input).await?;

    Ok(Json(company))
}

/// DELETE /companies/{id} (owner only; closes the company, nothing is erased)
pub async fn delete(
    auth: AuthUser,
    State(state): State<AppState>,
    AppPath(id): AppPath<Uuid>,
) -> Result<StatusCode, AppError> {
    auth.owned_company(id)?;

    service::close(&state.db, id).await?;

    Ok(StatusCode::NO_CONTENT)
}
