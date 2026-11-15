//! Creates the `companies` table: one row per company (tenant) on the platform.
//!
//! Every business table added later references this one, so it is the root of
//! data isolation between companies (cahier des charges, §4.7).
//!
//! # Columns
//! - `id`: UUID primary key, generated in Rust (not by the database) so the
//!   value is known before insert and IDs can't be enumerated.
//! - `legal_name`, `vat_number`: from the sign-up form (US-01).
//!   `vat_number` is UNIQUE: two companies can't share one.
//! - `street`, `postal_code`, `city`, `country`: structured address.
//!   `postal_code` is a string (leading zeros, letters in some countries);
//!   `country` is an ISO 3166-1 alpha-2 code (e.g. "BE").
//! - `suspended_at`: NULL = active; a timestamp = suspended since then (US-09).
//! - `created_at`, `updated_at`: default to now() on INSERT.
//!   `updated_at` is NOT updated by the database: set it in Rust on every update.
//!
//! # Down
//! Drops the table and ALL its rows. Development use only.

use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261115_160312_create_company_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("companies")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string("legal_name"))
                    .col(string_uniq("vat_number"))
                    .col(string("street"))
                    .col(string("postal_code"))
                    .col(string("city"))
                    .col(string("country"))
                    .col(timestamp_with_time_zone_null("suspended_at"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table("companies").to_owned())
            .await
    }
}
