use log::debug;
use std::sync::Arc;

use crate::dtos::mqtt_dto::MqttCredentialsDTO;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::services::service_error::MqttServiceError;
use crate::utils::encryption::decrypt_password;

pub struct GetMqttCredentialsService {
    repo: Arc<GetMqttByUsernameRepository>,
}

impl GetMqttCredentialsService {
    pub fn new(repo: Arc<GetMqttByUsernameRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_credentials(&self, username: &str) -> Result<MqttCredentialsDTO, MqttServiceError> {
        let mqtt = match self.repo.get_mqtt_by_username(username).await {
            Ok(u) => u,
            Err(_) => {
                debug!("[Service | GetMqttCredentials] User MQTT not found: {}", username);
                return Err(MqttServiceError::MqttNotFound("User MQTT not found".into()));
            }
        };


        let decrypted_password = decrypt_password(&mqtt.password)
            .map_err(|e| MqttServiceError::InternalError(e))?;

        debug!("[Service | GetMqttCredentials] Credentials retrieved and decrypted for: {}", username);

        Ok(MqttCredentialsDTO {
            username: mqtt.username,
            password: decrypted_password,
        })
    }
}
