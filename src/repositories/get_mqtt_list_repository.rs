use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error};
use sqlx::{PgPool, Row};
use std::sync::Arc;

pub struct GetMqttListRepository {
    pool: Arc<PgPool>,
}

impl GetMqttListRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        GetMqttListRepository { pool }
    }

    pub async fn get_mqtt_list(&self) -> Result<Vec<MqttEntity>, MqttRepositoryError> {
        debug!("[Repository | GetList] Fetching all users");

        let rows = sqlx::query("SELECT username, password_hash, is_superuser FROM mqtt_users")
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| {
                error!("[Repository | GetList] Failed to fetch users: {}", e);
                MqttRepositoryError::Postgres(e)
            })?;

        let users = rows
            .into_iter()
            .map(|row| MqttEntity {
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                is_superuser: row.get("is_superuser"),
                client_id: None,
            })
            .collect();

        Ok(users)
    }
}
