//! Business logic for companies. No HTTP here: takes plain values,
//! returns Result, so it can be tested without a server.

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, NotSet, Set, SqlErr};
use uuid::Uuid;

use super::dto::CreateCompany;
use crate::entities::companies;
use crate::errors::AppError;

/// Creates a company. Fails with Conflict if the VAT number already exists.
pub async fn create(
    db: &DatabaseConnection,
    input: CreateCompany,
) -> Result<companies::Model, AppError> {
    let company = companies::ActiveModel {
        id: Set(Uuid::now_v7()),
        legal_name: Set(input.legal_name),
        vat_number: Set(input.vat_number),
        street: Set(input.street),
        postal_code: Set(input.postal_code),
        city: Set(input.city),
        country: Set(input.country),
        suspended_at: NotSet,
        created_at: NotSet,
        updated_at: NotSet,
    };

    let company = company.insert(db).await.map_err(|err| match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            AppError::Conflict("A company with this VAT number already exists".into())
        }
        _ => AppError::from(err),
    })?;

    Ok(company)
}

/// Returns all companies.
pub async fn list(db: &DatabaseConnection) -> Result<Vec<companies::Model>, AppError> {
    let companies = companies::Entity::find().all(db).await?;

    Ok(companies)
}

/// Returns one company by id. Fails with NotFound if it doesn't exist.
pub async fn get_by_id(db: &DatabaseConnection, id: Uuid) -> Result<companies::Model, AppError> {
    let company = companies::Entity::find_by_id(id).one(db).await?;

    company.ok_or(AppError::NotFound)
}
