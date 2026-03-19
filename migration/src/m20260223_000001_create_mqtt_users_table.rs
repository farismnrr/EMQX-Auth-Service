use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Set search path to emqx schema
        manager.get_connection().execute_unprepared("SET search_path TO emqx").await?;

        // Create seaql_migrations table if not exists (SeaORM needs this)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("seaql_migrations"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("version"))
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("applied_at"))
                            .big_integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
