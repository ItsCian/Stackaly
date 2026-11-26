//! Creates the `sessions` table: one row per logged-in device.
//!
//! The cookie only holds a random token; everything else lives here, so the
//! server decides who you are and logout really works (delete the row).
//!
//! # Columns
//! - `id`: UUID primary key (internal; e.g. "log out my other devices").
//! - `token_hash`: SHA-256 of the cookie token, UNIQUE. The raw token is NEVER
//!   stored: a leaked database must not let anyone log in.
//! - `user_id`: who is logged in.
//! - `active_company_id`: the company the user is currently working in.
//!   NULL = no company chosen yet. MUST be checked against `memberships`
//!   in Rust before being set: the database can't enforce that.
//! - `created_at`: compared with `users.password_changed_at`.
//! - `expires_at`: set in Rust at login. Checked on EVERY request, because
//!   expired rows can stay in the table until they're cleaned up.
//! - `last_seen_at`: for the idle timeout (STA-4).
//!
//! # Foreign keys
//! - user_id           -> users.id     ON DELETE CASCADE  (no user, no session)
//! - active_company_id -> companies.id ON DELETE SET NULL (session survives,
//!   the user just has to pick another company)
//!
//! # Down
//! Drops the table: every user is logged out.

use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261124_101419_create_sessions_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("sessions")
                    .if_not_exists()
                    .col(pk_uuid("id"))
                    .col(ColumnDef::new("token_hash").binary().not_null().unique_key())
                    .col(uuid("user_id"))
                    .col(uuid_null("active_company_id"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone("expires_at"))
                    .col(
                        timestamp_with_time_zone("last_seen_at").default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_user")
                            .from("sessions", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_active_company")
                            .from("sessions", "active_company_id")
                            .to("companies", "id")
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_sessions_user_id")
                    .table("sessions")
                    .col("user_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("sessions").to_owned()).await
    }
}
