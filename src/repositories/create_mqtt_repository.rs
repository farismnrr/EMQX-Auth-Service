use rocksdb::{DB, WriteOptions};
use std::sync::Arc;
use bincode::{encode_to_vec, config::standard};
use log::{debug, error};
use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::repository_error::MqttRepositoryError;

pub struct CreateMqttRepository {
    db: Arc<DB>,
}

impl CreateMqttRepository {
    pub fn new(db: Arc<DB>) -> Self {
        CreateMqttRepository { db }
    }

    pub fn create_mqtt(&self, username: &str, password_hash: &str) -> Result<(), MqttRepositoryError> {
    debug!("[Repository | CreateMQTT] Starting user MQTT creation for username: {}", username);
        
        // Build mqtt entity
        let mqtt = MqttEntity::create(username, password_hash);
        let key = format!("mqtt:{}", mqtt.username);
    debug!("[Repository | CreateMQTT] Created user MQTT entity with key: {}", key);

        // Encode mqtt to binary
        let value = match encode_to_vec(&mqtt, standard()) {
            Ok(v) => {
                debug!("[Repository | CreateMQTT] Successfully encoded user MQTT to binary, size: {} bytes", v.len());
                v
            }
            Err(e) => {
                error!("[Repository | CreateMQTT] Failed to serialize user MQTT {}: {e}", username);
                return Err(MqttRepositoryError::Encode(e));
            }
        };

        // Configure write options for performance tuning
        let mut opts: WriteOptions = WriteOptions::default();
        opts.set_sync(false);
        opts.disable_wal(true);
    debug!("[Repository | CreateMQTT] Write options configured: sync=false, wal=disabled");

        // Write to RocksDB
        match self.db.put_opt(key.as_bytes(), value, &opts) {
            Ok(_) => {
                debug!("[Repository | CreateMQTT] User MQTT {} successfully written to database", username);
                Ok(())
            }
            Err(e) => {
                error!("[Repository | CreateMQTT] Failed to write user MQTT {} to database: {e}", username);
                Err(MqttRepositoryError::Database(e))
            }
        }
    }
}