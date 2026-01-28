use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sqlx::{PgPool, Row};
use std::sync::Arc;

pub struct GetMqttByUsernameRepository {
    pool: Arc<PgPool>,
}

impl GetMqttByUsernameRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        GetMqttByUsernameRepository { pool }
    }

    pub async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        debug!("[Repository | GetByUsername] Fetching user: {}", username);

        let row = sqlx::query(
            "SELECT username, password_hash, is_superuser FROM mqtt_users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| {
            error!(
                "[Repository | GetByUsername] Failed to fetch user {}: {}",
                username, e
            );
            MqttRepositoryError::Postgres(e)
        })?;

        if let Some(row) = row {
            Ok(Some(MqttEntity {
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                is_superuser: row.get("is_superuser"),
                client_id: None,
            }))
        } else {
            Ok(None)
        }
    }
}
