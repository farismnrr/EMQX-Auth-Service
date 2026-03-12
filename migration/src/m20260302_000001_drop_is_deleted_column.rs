//! Dummy migration - column already dropped in production
//! This migration exists only to satisfy SeaORM migration tracking

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // This migration is a no-op - column was already dropped
        // This file exists only to satisfy migration tracking
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op - column was already dropped
        Ok(())
    }
}
