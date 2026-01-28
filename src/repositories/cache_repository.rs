use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::repository_error::MqttRepositoryError;
use bincode::{Decode, Encode, config::standard, decode_from_slice, encode_to_vec};
use log::{debug, error, info};
use rocksdb::{DB, WriteOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Encode, Decode)]
struct CachedMqttEntity {
    entity: MqttEntity,
    expires_at: u64,
}

pub struct CacheRepository {
    db: Arc<DB>,
}

impl CacheRepository {
    pub fn new(db: Arc<DB>) -> Self {
        CacheRepository { db }
    }

    pub fn get_cached_user(
        &self,
        username: &str,
    ) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        let key = format!("mqtt:cache:{}", username);
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => {
                let (cached, _): (CachedMqttEntity, usize) =
                    match decode_from_slice(&value, standard()) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(
                                "[Cache] Failed to decode cached entity for {}: {}",
                                username, e
                            );
                            return Ok(None);
                        }
                    };

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                if now > cached.expires_at {
                    debug!("[Cache] Cache expired for user {}", username);
                    // Lazy delete
                    let _ = self.db.delete(key.as_bytes());
                    return Ok(None);
                }

                debug!("[Cache] HIT for user {}", username);
                Ok(Some(cached.entity))
            }
            Ok(None) => {
                debug!("[Cache] MISS for user {}", username);
                Ok(None)
            }
            Err(e) => {
                error!("[Cache] Error reading cache for {}: {}", username, e);
                Err(MqttRepositoryError::RocksDB(e))
            }
        }
    }

    pub fn set_cached_user(&self, entity: MqttEntity) -> Result<(), MqttRepositoryError> {
        let key = format!("mqtt:cache:{}", entity.username);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = now + 3600; // 1 hour TTL

        let cached = CachedMqttEntity {
            entity: entity.clone(),
            expires_at,
        };

        let value = match encode_to_vec(&cached, standard()) {
            Ok(v) => v,
            Err(e) => return Err(MqttRepositoryError::Encode(e)),
        };

        let mut opts = WriteOptions::default();
        opts.set_sync(false);
        opts.disable_wal(true); // Ephemeral cache, speed over safety

        self.db
            .put_opt(key.as_bytes(), value, &opts)
            .map_err(MqttRepositoryError::RocksDB)?;
        debug!("[Cache] Set cache for user {} (TTL 1h)", entity.username);
        Ok(())
    }

    pub fn invalidate_user(&self, username: &str) -> Result<(), MqttRepositoryError> {
        let key = format!("mqtt:cache:{}", username);
        self.db
            .delete(key.as_bytes())
            .map_err(MqttRepositoryError::RocksDB)?;
        info!("[Cache] Invalidated cache for user {}", username);
        Ok(())
    }
}
