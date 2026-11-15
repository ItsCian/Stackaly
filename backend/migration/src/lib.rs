pub use sea_orm_migration::prelude::*;

mod m20261115_160312_create_company_table;

pub struct Migrator;

// ! Do not change the order once pushed to a real Database

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20261115_160312_create_company_table::Migration)]
    }
}
