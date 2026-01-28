use crate::repositories::cache_repository::CacheRepository;
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error, info, warn};
use sqlx::PgPool;
use std::sync::Arc;

pub struct SoftDeleteMqttRepository {
    pool: Arc<PgPool>,
    cache: Arc<CacheRepository>,
}

impl SoftDeleteMqttRepository {
    pub fn new(pool: Arc<PgPool>, cache: Arc<CacheRepository>) -> Self {
        SoftDeleteMqttRepository { pool, cache }
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

        // Invalidate cache
        if let Err(e) = self.cache.invalidate_user(username) {
            warn!(
                "[Repository | Delete] Failed to invalidate cache for {}: {}",
                username, e
            );
        }

        info!(
            "[Repository | Delete] Successfully deleted user {}",
            username
        );
        Ok(())
    }
}
