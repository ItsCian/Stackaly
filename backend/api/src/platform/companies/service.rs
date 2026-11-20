//! Business logic for companies. No HTTP here: takes plain values,
//! returns Result, so it can be tested without a server.

use sea_orm::{ActiveModelTrait, DatabaseConnection, NotSet, Set, SqlErr};
use uuid::Uuid;

use super::dto::CreateCompany;
use crate::entities::companies;
use crate::errors::AppError;

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
        suspended_at: NotSet, // active on creation
        created_at: NotSet,   // DB default
        updated_at: NotSet,   // DB default
    };

    let company = company.insert(db).await.map_err(|err| match err.sql_err() {
        // companies has one unique column, so a violation here means the VAT number.
        // Precise message for the user (US-01), handled before From<DbErr> runs.
        Some(SqlErr::UniqueConstraintViolation(_)) => {
            AppError::Conflict("A company with this VAT number already exists".into())
        }
        // Anything else: defer to From<DbErr> (safety net, then Internal)
        _ => AppError::from(err),
    })?;

    Ok(company)
}
