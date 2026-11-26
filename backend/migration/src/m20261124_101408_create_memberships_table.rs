//! Creates the `memberships` table: the link between a user and a company,
//! with the user's role IN that company.
//!
//! Many-to-many: one user can belong to several companies, one company has
//! several users. The role lives here (not on `users`) because the same person
//! can be an employee in one company and a manager in another.
//!
//! # Columns
//! - `user_id`, `company_id`: composite primary key, so the same user can't
//!   be added twice to the same company.
//! - `role`: 'owner' | 'manager' | 'employee', enforced by a CHECK constraint.
//! - `deactivated_at`: NULL = active member; a timestamp = left this company.
//!   The user can still log in to their other companies.
//! - `added_by`: who added this person to this company. NULL for the owner
//!   who registered the company (US-01) and if the adder is deleted.
//! - `created_at`: default now().
//!
//! # Foreign keys
//! - user_id    -> users.id     ON DELETE CASCADE  (no user, no membership)
//! - company_id -> companies.id ON DELETE CASCADE  (no company, no membership)
//! - added_by   -> users.id     ON DELETE SET NULL (keep the membership)
//!
//! # Down
//! Drops the table and ALL its rows. Development use only.

use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20261125_000001_create_memberships_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table("memberships")
                    .if_not_exists()
                    .col(uuid("user_id"))
                    .col(uuid("company_id"))
                    .col(
                        string_len("role", 20)
                            .check(Expr::col("role").is_in(["owner", "manager", "employee"])),
                    )
                    .col(timestamp_with_time_zone_null("deactivated_at"))
                    .col(uuid_null("added_by"))
                    .col(timestamp_with_time_zone("created_at").default(Expr::current_timestamp()))
                    .primary_key(Index::create().col("user_id").col("company_id"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_memberships_user")
                            .from("memberships", "user_id")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_memberships_company")
                            .from("memberships", "company_id")
                            .to("companies", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_memberships_added_by")
                            .from("memberships", "added_by")
                            .to("users", "id")
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_memberships_company_id")
                    .table("memberships")
                    .col("company_id")
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table("memberships").to_owned()).await
    }
}
