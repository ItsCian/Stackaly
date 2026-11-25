//! Creates the `users` table: one row per person who can log in.
//!
//! A user is NOT tied to a company here: the link (and the role) lives in
//! `memberships`, because one person can work for several companies.
//!
//! # Columns
//! - `id`: UUID primary key, generated in Rust (same as `companies`).
//! - `email`: varchar(254) UNIQUE. Always lowercased in Rust before insert
//!   AND lookup, because UNIQUE is case-sensitive in Postgres.
//!   On anonymization, replaced by a unique placeholder (`deleted-<id>@invalid`)
//!   so the column can stay NOT NULL.
//! - `password_hash`: full Argon2id PHC string. `text` on purpose: its length
//!   grows if the Argon2 parameters are raised later.
//! - `first_name`, `last_name`: display name (employee list, activity log).
//! - `disabled_at`: NULL = active; a timestamp = banned from the whole
//!   platform by the super-admin. (Leaving ONE company is
//!   `memberships.deactivated_at`, not this.)
//! - `email_verified_at`: NULL = email not verified yet.
//! - `must_change_password`: true when the account was created with a
//!   temporary password (e.g. by an owner, US-03).
//! - `password_changed_at`: sessions created before this moment are rejected,
//!   so changing the password logs out every other device.
//! - `failed_login_count`, `locked_until`: account lockout (STA-4).
//!   `locked_until` NULL = not locked.
//! - `last_login_at`: NULL = never logged in.
//! - `created_at`, `updated_at`: default to now() on INSERT.
//!   `updated_at` is NOT updated by the database: set it in Rust on every update.
//!
//! # Down
//! Drops the table and ALL its rows. Development use only.

use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261124_101355_create_users_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("users")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(string_len_uniq("email", 254))
                    .col(text("password_hash"))
                    .col(string_len("first_name", 100))
                    .col(string_len("last_name", 100))
                    .col(timestamp_with_time_zone_null("disabled_at"))
                    .col(timestamp_with_time_zone_null("email_verified_at"))
                    .col(boolean("must_change_password").default(false))
                    .col(
                        timestamp_with_time_zone("password_changed_at")
                            .default(Expr::current_timestamp()),
                    )
                    .col(integer("failed_login_count").default(0))
                    .col(timestamp_with_time_zone_null("locked_until"))
                    .col(timestamp_with_time_zone_null("last_login_at"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("updated_at").default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("users").to_owned()).await
    }
}
