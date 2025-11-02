use std::sync::Arc;
use uuid::Uuid;
use log::debug;
use argon2::password_hash::rand_core::{OsRng, RngCore};

use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::services::service_error::MqttServiceError;
use crate::utils::hash_password::hash_password;

pub struct CreateMqttService {
    repo: Arc<CreateMqttRepository>,
}

impl CreateMqttService {
    pub fn new(repo: Arc<CreateMqttRepository>) -> Self {
        Self { repo }
    }

    pub fn create_mqtt(&self) -> Result<(String, String), MqttServiceError> {
        let username = Self::create_username();
        let password = Self::create_password();
        let hashed = hash_password(&password);
        
        self.repo.create_mqtt(&username, &hashed)?;
        debug!("[Service | CreateMQTT] User MQTT created successfully: {}", username);
        Ok((username, password))
    }

    fn create_username() -> String {
        debug!("[Service | CreateMQTT] Generating new UUID for username.");
        format!("{}", Uuid::new_v4())
    }
    
    fn create_password() -> String {
        let mut buf = [0u8; 32];
        let mut rng = OsRng;
        rng.try_fill_bytes(&mut buf).ok();
        let password = hex::encode(buf);
        debug!("[Service | CreateMQTT] Generated random password of length {}.", password.len());
        password
    }
}
