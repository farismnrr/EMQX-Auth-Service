use rocksdb::{DB, ReadOptions};
use std::sync::Arc;
use bincode::{decode_from_slice, config::standard};
use log::{debug, error};
use crate::entities::mqtt_entity::MqttEntity;
use crate::repositories::repository_error::MqttRepositoryError;

pub struct GetMqttListRepository {
    db: Arc<DB>,
}

impl GetMqttListRepository {
    pub fn new(db: Arc<DB>) -> Self {
        GetMqttListRepository { db }
    }

    pub fn get_mqtt_list(&self) -> Result<Vec<MqttEntity>, MqttRepositoryError> {
        let mut mqtts = Vec::new();

        // Configure read options to possibly improve iteration performance
        let mut read_opts = ReadOptions::default();
        read_opts.set_verify_checksums(false);
        read_opts.fill_cache(true);

        debug!("[Repository | GetMQTTList] Starting iteration to collect users mqtt from DB.");

    for (key, value) in self.db.iterator_opt(rocksdb::IteratorMode::Start, read_opts).flatten() {
            // Try to convert key to UTF-8 string
            let key_str = match String::from_utf8(key.to_vec()) {
                Ok(s) => {
                    debug!("[Repository | GetMQTTList] Found key: {}", s);
                    s
                }
                Err(e) => {
                    error!("[Repository | GetMQTTList] Failed to convert key to string: {e}");
                    debug!("[Repository | GetMQTTList] Key bytes: {:#?}", key);
                    return Err(MqttRepositoryError::Utf8(e))
                }
            };

            // Skip non-mqtt keys
            if !key_str.starts_with("mqtt:") {
                debug!("[Repository | GetMQTTList] Non-mqtt key skipped: {}", key_str);
                continue;
            }

            // Decode mqtt data from bincode
            debug!("[Repository | GetMQTTList] Decoding user mqtt data for key: {}", key_str);
            let mqtt = match decode_from_slice::<MqttEntity, _>(&value, standard()) {
                Ok((mqtt, _)) => {
                    debug!("[Repository | GetMQTTList] Successfully decoded user mqtt: {}", mqtt.username);
                    mqtt
                }
                Err(e) => {
                    error!("[Repository | GetMQTTList] Failed to decode user mqtt for key {}: {}", key_str, e);
                    debug!("[Repository | GetMQTTList] Value bytes for key {}: {:#?}", key_str, value);
                    return Err(MqttRepositoryError::Decode(e))
                }
            };

            // Skip deleted mqtts
            if mqtt.is_deleted {
                debug!("[Repository | GetMQTTList] Deleted flag set for user mqtt: {}", mqtt.username);
                continue;
            }

            debug!("[Repository | GetMQTTList] Adding user mqtt to results: {}", mqtt.username);
            mqtts.push(mqtt);
        }

        Ok(mqtts)
    }
}