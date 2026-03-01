pub use sea_orm_migration::prelude::*;

mod m20260223_000001_create_mqtt_users_table;
mod m20260302_000001_drop_is_deleted_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260223_000001_create_mqtt_users_table::Migration),
            Box::new(m20260302_000001_drop_is_deleted_column::Migration),
        ]
    }
}
