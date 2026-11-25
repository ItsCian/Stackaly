pub use sea_orm_migration::prelude::*;

mod m20261115_160312_create_company_table;
mod m20261124_101355_create_users_table;
mod m20261124_101408_create_memberships_table;
mod m20261124_101419_create_sessions_table;

pub struct Migrator;

// ! Do not change the order once pushed to a real Database

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261115_160312_create_company_table::Migration),
            Box::new(m20261124_101355_create_users_table::Migration),
            Box::new(m20261124_101408_create_memberships_table::Migration),
            Box::new(m20261124_101419_create_sessions_table::Migration),
        ]
    }
}
