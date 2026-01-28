use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error, info};
use sqlx::PgPool;
use std::sync::Arc;

pub struct CreateMqttRepository {
    pool: Arc<PgPool>,
}

impl CreateMqttRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        CreateMqttRepository { pool }
    }

    pub async fn create_mqtt(
        &self,
        username: &str,
        password_hash: &str,
        is_superuser: bool,
    ) -> Result<(), MqttRepositoryError> {
        debug!(
            "[Repository | CreateMQTT] Starting user creation for: {}",
            username
        );

        sqlx::query(
            "INSERT INTO mqtt_users (username, password_hash, is_superuser, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW())"
        )
        .bind(username)
        .bind(password_hash)
        .bind(is_superuser)
        .execute(&*self.pool)
        .await
        .map_err(|e| {
             error!("[Repository | CreateMQTT] Failed to create user {}: {}", username, e);
             MqttRepositoryError::Postgres(e)
        })?;

        info!(
            "[Repository | CreateMQTT] Successfully created user {}",
            username
        );
        Ok(())
    }
}
