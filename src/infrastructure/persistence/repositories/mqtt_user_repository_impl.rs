//! MQTT User Repository Implementation

use async_trait::async_trait;
use log::debug;
use sea_orm::PaginatorTrait;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};

use crate::domain::repository::RepositoryError;
use crate::domain::{MqttUser, MqttUserRepository};
use crate::infrastructure::persistence::models::{MqttUserEntity, MqttUserModel};

/// SeaORM-based repository implementation
#[derive(Clone)]
pub struct MqttUserRepositoryImpl {
    db: DatabaseConnection,
}

impl MqttUserRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MqttUserRepository for MqttUserRepositoryImpl {
    async fn find_by_id(&self, id: i32) -> Result<Option<MqttUser>, RepositoryError> {
        debug!("[Repository] Finding user by ID: {}", id);

        let model = MqttUserEntity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(model.map(|m| m.into()))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<MqttUser>, RepositoryError> {
        debug!("[Repository] Finding user by username: {}", username);

        let model = MqttUserEntity::find()
            .filter(super::super::models::mqtt_user_model::Column::Username.eq(username))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(model.map(|m| m.into()))
    }

    async fn find_all(&self, limit: u32, offset: u32) -> Result<Vec<MqttUser>, RepositoryError> {
        debug!(
            "[Repository] Finding users (limit={}, offset={})",
            limit, offset
        );

        let models = MqttUserEntity::find()
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    async fn count(&self) -> Result<u64, RepositoryError> {
        debug!("[Repository] Counting users");

        let count = MqttUserEntity::find()
            .count(&self.db)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(count)
    }

    async fn insert(&self, user: MqttUser) -> Result<MqttUser, RepositoryError> {
        debug!("[Repository] Inserting user: {}", user.username);

        let active_model =
            crate::infrastructure::persistence::models::mqtt_user_model::ActiveModel {
                id: NotSet,
                username: sea_orm::Set(user.username),
                password_hash: sea_orm::Set(user.password),
                is_superuser: sea_orm::Set(Some(user.is_superuser)),
                created_at: sea_orm::Set(user.created_at),
                updated_at: sea_orm::Set(user.updated_at),
            };

        let inserted: MqttUserModel = active_model.insert(&self.db).await.map_err(|e| {
            if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate") {
                RepositoryError::UniqueViolation("Username already exists".to_string())
            } else {
                RepositoryError::Database(e.to_string())
            }
        })?;

        Ok(inserted.into())
    }

    async fn update(&self, user: MqttUser) -> Result<MqttUser, RepositoryError> {
        debug!("[Repository] Updating user: {}", user.username);

        let active_model =
            crate::infrastructure::persistence::models::mqtt_user_model::ActiveModel {
                id: sea_orm::Set(user.id),
                username: sea_orm::Set(user.username),
                password_hash: sea_orm::Set(user.password),
                is_superuser: sea_orm::Set(Some(user.is_superuser)),
                created_at: sea_orm::Set(user.created_at),
                updated_at: sea_orm::Set(user.updated_at),
            };

        let updated = active_model
            .update(&self.db)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(updated.into())
    }

    async fn delete_by_username(&self, username: &str) -> Result<(), RepositoryError> {
        debug!("[Repository] Deleting user: {}", username);

        let user = MqttUserEntity::find()
            .filter(
                crate::infrastructure::persistence::models::mqtt_user_model::Column::Username
                    .eq(username),
            )
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match user {
            Some(model) => {
                let active_model =
                    crate::infrastructure::persistence::models::mqtt_user_model::ActiveModel::from(
                        model,
                    );
                MqttUserEntity::delete(active_model)
                    .exec(&self.db)
                    .await
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;
                Ok(())
            }
            None => Err(RepositoryError::NotFound(username.to_string())),
        }
    }

    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError> {
        debug!("[Repository] Checking if user exists: {}", username);

        let count: u64 = MqttUserEntity::find()
            .filter(
                crate::infrastructure::persistence::models::mqtt_user_model::Column::Username
                    .eq(username),
            )
            .count(&self.db)
            .await
            .map_err(|e: sea_orm::DbErr| RepositoryError::Database(e.to_string()))?;

        Ok(count > 0)
    }
}
