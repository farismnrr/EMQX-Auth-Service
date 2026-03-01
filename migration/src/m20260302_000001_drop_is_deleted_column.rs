use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MqttUsers::Table)
                    .drop_column(MqttUsers::IsDeleted)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(MqttUsers::Table)
                    .add_column(
                        ColumnDef::new(MqttUsers::IsDeleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum MqttUsers {
    Table,
    IsDeleted,
}
