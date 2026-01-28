use log::debug;
use std::sync::Arc;

use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_by_username_repository::GetMqttByUsernameRepository;
use crate::services::service_error::{MqttServiceError, ValidationError};
use crate::utils::hash_password::hash_password;

pub struct CreateMqttService {
    repo_create: Arc<CreateMqttRepository>,
    repo_get: Arc<GetMqttByUsernameRepository>,
}

impl CreateMqttService {
    pub fn new(
        repo_create: Arc<CreateMqttRepository>,
        repo_get: Arc<GetMqttByUsernameRepository>,
    ) -> Self {
        Self {
            repo_create,
            repo_get,
        }
    }

    pub async fn create_mqtt(&self, dto: CreateMqttDTO) -> Result<bool, MqttServiceError> {
        self.create_mqtt_validation(&dto)?;

        if self
            .repo_get
            .get_by_username(&dto.username)
            .await?
            .is_some()
        {
            return Err(MqttServiceError::Conflict(
                "MQTT user already exists".into(),
            ));
        }

        let hashed = hash_password(&dto.password);
        self.repo_create
            .create_mqtt(&dto.username, &hashed, dto.is_superuser)
            .await?;
        debug!(
            "[Service | CreateMQTT] User MQTT created successfully: {}",
            &dto.username
        );
        Ok(true)
    }

    fn create_mqtt_validation(&self, dto: &CreateMqttDTO) -> Result<bool, MqttServiceError> {
        let mut errors = Vec::new();
        if dto.username.trim().is_empty() {
            errors.push(ValidationError {
                field: "username".to_string(),
                message: "username cannot be empty".to_string(),
            });
        }

        if dto.password.trim().is_empty() {
            errors.push(ValidationError {
                field: "password".to_string(),
                message: "password cannot be empty".to_string(),
            });
        }

        if !errors.is_empty() {
            return Err(MqttServiceError::BadRequest(errors));
        }

        debug!("[Service | CheckMQTTActive] User MQTT input validation passed.");
        Ok(true)
    }
}
