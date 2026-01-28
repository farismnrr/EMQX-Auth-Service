use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error, info};
use sqlx::PgPool;
use std::sync::Arc;

pub struct SoftDeleteMqttRepository {
    pool: Arc<PgPool>,
}

impl SoftDeleteMqttRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        SoftDeleteMqttRepository { pool }
    }

    pub async fn soft_delete_mqtt(&self, username: &str) -> Result<(), MqttRepositoryError> {
        debug!("[Repository | Delete] Deleting user: {}", username);

        sqlx::query("DELETE FROM mqtt_users WHERE username = $1")
            .bind(username)
            .execute(&*self.pool)
            .await
            .map_err(|e| {
                error!(
                    "[Repository | Delete] Failed to delete user {}: {}",
                    username, e
                );
                MqttRepositoryError::Postgres(e)
            })?;

        info!(
            "[Repository | Delete] Successfully deleted user {}",
            username
        );
        Ok(())
    }
}
