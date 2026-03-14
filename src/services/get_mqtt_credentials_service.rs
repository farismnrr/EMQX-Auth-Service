use log::debug;
use std::sync::Arc;

use crate::dtos::mqtt_dto::MqttCredentialsDTO;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::services::service_error::MqttServiceError;
use crate::utils::encryption_util::decrypt_password;

pub struct GetMqttCredentialsService {
    repo: Arc<GetMqttByUsernameRepository>,
}

impl GetMqttCredentialsService {
    pub fn new(repo: Arc<GetMqttByUsernameRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_mqtt_by_username(
        &self,
        username: &str,
    ) -> Result<Option<crate::entities::mqtt_entity::Model>, MqttServiceError> {
        match self.repo.get_mqtt_by_username(username).await {
            Ok(m) => Ok(Some(m)),
            Err(crate::repositories::repository_error::MqttRepositoryError::NotFound) => Ok(None),
            Err(e) => Err(MqttServiceError::InternalError(e.to_string())),
        }
    }

    pub async fn verify_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<bool, MqttServiceError> {
        let mqtt = match self.get_mqtt_by_username(username).await? {
            Some(u) => u,
            None => {
                debug!(
                    "[Service | VerifyPassword] User MQTT not found: {}",
                    username
                );
                return Err(MqttServiceError::MqttNotFound("User MQTT not found".into()));
            }
        };

        let decrypted_stored =
            decrypt_password(&mqtt.password).map_err(MqttServiceError::InternalError)?;

        let is_valid = password == decrypted_stored;
        if !is_valid {
            debug!(
                "[Service | VerifyPassword] Invalid credentials for user MQTT: {}",
                username
            );
        }

        Ok(is_valid)
    }
}
