use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MqttUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MqttUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(MqttUsers::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(MqttUsers::Password).string().not_null())
                    .col(
                        ColumnDef::new(MqttUsers::IsDeleted)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(MqttUsers::IsSuperuser)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MqttUsers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MqttUsers {
    Table,
    Id,
    Username,
    Password,
    IsDeleted,
    IsSuperuser,
}
