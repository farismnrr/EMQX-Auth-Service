use rocksdb::{DB, ReadOptions};
use std::sync::Arc;
use bincode::config::standard;
use bincode::decode_from_slice;
use log::{debug, error};
use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::repository_error::MqttRepositoryError;

pub struct MqttLoginRepository {
    db: Arc<DB>,
}

impl MqttLoginRepository {
    pub fn new(db: Arc<DB>) -> Self {
        MqttLoginRepository { db }
    }

    pub fn login_with_credentials(&self, username: &str) -> Result<Option<MqttEntity>, MqttRepositoryError> {
        // Build RocksDB key
        let key: String = format!("mqtt:{}", username);

        // Configure read options for optimization
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.fill_cache(true);

        // Try to fetch record from DB
        debug!("[Repository | CheckMQTTActive] Attempting to fetch user MQTT '{}' from database.", username);
        let value = match self.db.get_opt(key.as_bytes(), &read_opts) {
            Ok(v) => {
                if v.is_some() {
                    debug!("[Repository | CheckMQTTActive] Database read returned a value for user MQTT '{}'.", username);
                } else {
                    debug!("[Repository | CheckMQTTActive] Database read returned no value for user MQTT '{}'.", username);
                }
                v
            }
            Err(e) => {
                error!("[Repository | CheckMQTTActive] Database read error for user MQTT {username}: {e}");
                debug!("[Repository | CheckMQTTActive] Database read error for user MQTT '{}': {:#?}", username, e);
                return Err(MqttRepositoryError::Database(e));
            }
        };

        let Some(value) = value else {
            debug!("[Repository | CheckMQTTActive] User MQTT '{}' not found in database.", username);
            return Ok(None);
        };

        debug!("[Repository | CheckMQTTActive] Decoding user MQTT data for '{}'.", username);
        let (mqtt, _) = match decode_from_slice::<MqttEntity, _>(&value, standard()) {
            Ok(decoded) => decoded,
            Err(e) => {
                error!("[Repository | CheckMQTTActive] Failed to decode user MQTT data for {username}: {e}");
                debug!("[Repository | CheckMQTTActive] Decode error for user MQTT '{}': {:#?}", username, e);
                return Err(MqttRepositoryError::Decode(e));
            }
        };

        Ok(Some(mqtt))
    }
}
