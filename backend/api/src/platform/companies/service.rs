//! Business logic for companies. No HTTP here: takes plain values,
//! returns Result, so it can be tested without a server.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, NotSet, Set, SqlErr,
};
use uuid::Uuid;

use super::dto::{CreateCompany, UpdateCompany};
use super::vat::normalize_vat;
use crate::entities::companies;
use crate::errors::AppError;

/// Creates a company.
/// Fails with Validation if the VAT number is invalid,
/// and Conflict if it already exists.
///
/// Generic over `ConnectionTrait`: accepts a plain connection
/// (`&DatabaseConnection`) or a transaction (`&DatabaseTransaction`),
/// so `register` can create the company inside its own transaction.
pub async fn create<C: ConnectionTrait>(
    db: &C,
    input: CreateCompany,
) -> Result<companies::Model, AppError> {
    // Required field: always validate. Done first so we fail fast.
    let vat_number = normalize_vat(&input.vat_number)?;

    let company = companies::ActiveModel {
        id: Set(Uuid::now_v7()),
        legal_name: Set(input.legal_name),
        vat_number: Set(vat_number), // the normalized form, never the raw input
        street: Set(input.street),
        postal_code: Set(input.postal_code),
        city: Set(input.city),
        country: Set(input.country),
        suspended_at: NotSet,
        closed_at: NotSet,
        created_at: NotSet,
        updated_at: NotSet,
    };

    let company = company.insert(db).await.map_err(vat_conflict)?;

    Ok(company)
}

/// Updates the fields that were sent. Fails with NotFound or Conflict.
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateCompany,
) -> Result<companies::Model, AppError> {
    let company = get_by_id(db, id).await?;

    let mut company: companies::ActiveModel = company.into();

    if let Some(legal_name) = input.legal_name {
        company.legal_name = Set(legal_name);
    }
    if let Some(vat_number) = input.vat_number {
        company.vat_number = Set(normalize_vat(&vat_number)?);
    }
    if let Some(street) = input.street {
        company.street = Set(street);
    }
    if let Some(postal_code) = input.postal_code {
        company.postal_code = Set(postal_code);
    }
    if let Some(city) = input.city {
        company.city = Set(city);
    }
    if let Some(country) = input.country {
        company.country = Set(country);
    }

    company.updated_at = Set(Utc::now().into());
    let company = company.update(db).await.map_err(vat_conflict)?;

    Ok(company)
}

/// Returns one company by id. Fails with NotFound if it doesn't exist.
pub async fn get_by_id(db: &DatabaseConnection, id: Uuid) -> Result<companies::Model, AppError> {
    let company = companies::Entity::find_by_id(id).one(db).await?;

    company.ok_or(AppError::NotFound)
}

/// Turns a duplicate VAT number into a Conflict error.
fn vat_conflict(err: DbErr) -> AppError {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            AppError::Conflict("A company with this VAT number already exists".into())
        }
        _ => AppError::from(err),
    }
}

/// Closes a company: sets closed_at instead of deleting the row,
/// so the history is kept and the closure can be undone.
pub async fn close(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
    let company = get_by_id(db, id).await?;
    let mut company: companies::ActiveModel = company.into();
    company.closed_at = Set(Some(Utc::now().into()));
    company.updated_at = Set(Utc::now().into());
    company.update(db).await?;
    Ok(())
}
