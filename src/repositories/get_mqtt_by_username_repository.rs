use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::cache_repository::CacheRepository;
use crate::repositories::repository_error::MqttRepositoryError;
use log::{debug, error, warn};
use sqlx::{PgPool, Row};
use std::sync::Arc;

pub struct GetMqttByUsernameRepository {
    pool: Arc<PgPool>,
    cache: Arc<CacheRepository>,
}

impl GetMqttByUsernameRepository {
    pub fn new(pool: Arc<PgPool>, cache: Arc<CacheRepository>) -> Self {
        GetMqttByUsernameRepository { pool, cache }
    }

    pub async fn get_by_username(
        &self,
        username: &str,
    ) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        // 1. Try Cache first
        match self.cache.get_cached_user(username) {
            Ok(Some(user)) => {
                debug!("[Repository | GetByUsername] Cache HIT for: {}", username);
                return Ok(Some(user));
            }
            Ok(None) => debug!("[Repository | GetByUsername] Cache MISS for: {}", username),
            Err(e) => warn!("[Repository | GetByUsername] Cache error: {}", e),
        }

        // 2. Fallback to Postgres
        debug!(
            "[Repository | GetByUsername] Fetching from Postgres: {}",
            username
        );
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
            let entity = MqttEntity {
                username: row.get("username"),
                password_hash: row.get("password_hash"),
                is_superuser: row.get("is_superuser"),
                client_id: None,
            };

            // 3. Populate Cache
            if let Err(e) = self.cache.set_cached_user(entity.clone()) {
                warn!(
                    "[Repository | GetByUsername] Failed to set cache for {}: {}",
                    username, e
                );
            }

            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }
}
