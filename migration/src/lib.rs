pub use sea_orm_migration::prelude::*;

mod m20260101_000001_create_anchor_model;
mod m20260101_000002_seed_products;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260101_000001_create_anchor_model::Migration),
            Box::new(m20260101_000002_seed_products::Migration),
        ]
    }
}
