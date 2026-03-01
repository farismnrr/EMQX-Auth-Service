use crate::dtos::mqtt_dto::MqttAclDTO;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::services::service_error::{MqttServiceError, ValidationError};
use log::debug;
use std::sync::Arc;

pub struct MqttAclService {
    repo: Arc<GetMqttByUsernameRepository>,
}

impl MqttAclService {
    pub fn new(repo: Arc<GetMqttByUsernameRepository>) -> MqttAclService {
        Self { repo }
    }

    pub async fn check_acl_permission(&self, dto: MqttAclDTO) -> Result<bool, MqttServiceError> {
        self.mqtt_input_acl_validation(&dto)?;

        let mqtt = match self.repo.get_mqtt_by_username(&dto.username).await {
            Ok(u) => u,
            Err(_) => {
                debug!(
                    "[Service | CheckMQTTACL] User MQTT not found: {}",
                    dto.username
                );
                return Err(MqttServiceError::MqttNotFound("User MQTT not found".into()));
            }
        };


        if mqtt.is_superuser {
            debug!(
                "[Service | CheckMQTTACL] Superuser `{}` → access granted",
                dto.username
            );
            return Ok(true);
        }

        let expected_prefix = format!("users/{}/", dto.username);
        if !dto.topic.starts_with(&expected_prefix) {
            debug!(
                "[Service | CheckMQTTACL] Topic `{}` does not start with expected prefix `{}` → access denied",
                dto.topic, expected_prefix
            );
            return Ok(false);
        }

        debug!(
            "[Service | CheckMQTTACL] ACL check passed for user `{}` on topic `{}`",
            dto.username, dto.topic
        );
        Ok(true)
    }

    fn mqtt_input_acl_validation(&self, dto: &MqttAclDTO) -> Result<bool, MqttServiceError> {
        let mut errors = Vec::new();
        if dto.username.trim().is_empty() {
            errors.push(ValidationError {
                field: "username".to_string(),
                message: "username cannot be empty".to_string(),
            });
        }

        if dto.topic.trim().is_empty() {
            errors.push(ValidationError {
                field: "topic".to_string(),
                message: "topic cannot be empty".to_string(),
            });
        }

        if !errors.is_empty() {
            return Err(MqttServiceError::BadRequest(errors));
        }

        debug!("[Service | CheckMQTTActive] User MQTT input validation passed.");
        Ok(true)
    }
}
