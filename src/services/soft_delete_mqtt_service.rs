use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::repositories::soft_delete_mqtt_repository::SoftDeleteMqttRepository;
use crate::services::service_error::{MqttServiceError, ValidationError};
use log::debug;
use std::sync::Arc;

pub struct SoftDeleteMqttService {
    repo_get: Arc<GetMqttByUsernameRepository>,
    repo_delete: Arc<SoftDeleteMqttRepository>,
}

impl SoftDeleteMqttService {
    pub fn new(
        repo_get: Arc<GetMqttByUsernameRepository>,
        repo_delete: Arc<SoftDeleteMqttRepository>,
    ) -> SoftDeleteMqttService {
        Self {
            repo_get,
            repo_delete,
        }
    }

    pub async fn soft_delete_mqtt(&self, username: &str) -> Result<bool, MqttServiceError> {
        self.validate_username(username)?;

        let _mqtt = match self.repo_get.get_by_username(username).await? {
            Some(u) => u,
            None => {
                debug!(
                    "[Service | SoftDeleteMQTT] User MQTT not found: {}",
                    username
                );
                return Err(MqttServiceError::MqttNotFound("User MQTT not found".into()));
            }
        };

        // Removed is_deleted check (Hard deletion for now)

        self.repo_delete.soft_delete_mqtt(username).await?;
        debug!(
            "[Service | SoftDeleteMQTT] Successfully soft deleted user MQTT: {}",
            username
        );
        Ok(true)
    }

    fn validate_username(&self, username: &str) -> Result<bool, MqttServiceError> {
        let mut errors = Vec::new();

        if username.trim().is_empty() {
            errors.push(ValidationError {
                field: "username".to_string(),
                message: "username cannot be empty".to_string(),
            });
        }

        if !errors.is_empty() {
            return Err(MqttServiceError::BadRequest(errors));
        }

        debug!("[Service | SoftDeleteMQTT] Username validation passed.");
        Ok(true)
    }
}
