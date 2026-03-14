use crate::dtos::mqtt_dto::{AuthType, MqttLoginDTO};
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;
use crate::services::service_error::{MqttServiceError, ValidationError};
use crate::utils::jwt_sign_util::create_jwt;
use log::debug;
use std::sync::Arc;

pub struct MqttLoginService {
    credentials_service: Arc<GetMqttCredentialsService>,
    secret_key: String,
}

impl MqttLoginService {
    pub fn new(credentials_service: Arc<GetMqttCredentialsService>, secret_key: String) -> Self {
        Self {
            credentials_service,
            secret_key,
        }
    }

    pub async fn login_with_credentials(
        &self,
        dto: MqttLoginDTO,
    ) -> Result<(bool, String, bool), MqttServiceError> {
        self.mqtt_input_credentials_validation(&dto)?;

        let mqtt = match self
            .credentials_service
            .get_mqtt_by_username(&dto.username)
            .await?
        {
            Some(u) => u,
            None => {
                debug!(
                    "[Service | CheckMQTTActive] User MQTT not found: {}",
                    dto.username
                );
                return Err(MqttServiceError::MqttNotFound("User MQTT not found".into()));
            }
        };

        match dto.method.unwrap() {
            AuthType::Credentials => {
                let is_valid = self
                    .credentials_service
                    .verify_password(&dto.username, &dto.password)
                    .await?;

                if !is_valid {
                    debug!(
                        "[Service | CheckMQTTActive] Invalid credentials for user MQTT: {}",
                        dto.username
                    );
                    return Err(MqttServiceError::InvalidCredentials(
                        "Invalid credentials".into(),
                    ));
                }

                Ok((true, String::new(), mqtt.is_superuser))
            }
            AuthType::Jwt => {
                let token = create_jwt(&dto.username, &self.secret_key)
                    .map_err(|e| MqttServiceError::JwtError(e.to_string()))?;
                debug!(
                    "[Service | CheckMQTTActive] JWT token created for user MQTT: {}",
                    dto.username
                );
                Ok((true, token, mqtt.is_superuser))
            }
        }
    }

    fn mqtt_input_credentials_validation(
        &self,
        dto: &MqttLoginDTO,
    ) -> Result<bool, MqttServiceError> {
        let mut errors = Vec::new();
        if dto.username.trim().is_empty() {
            errors.push(ValidationError {
                field: "username".to_string(),
                message: "username cannot be empty".to_string(),
            });
        }

        let method = match dto.method {
            Some(ref m) => m,
            None => {
                errors.push(ValidationError {
                    field: "method".into(),
                    message: "method cannot be empty".into(),
                });
                return Err(MqttServiceError::BadRequest(errors));
            }
        };

        if matches!(method, AuthType::Credentials) && dto.password.trim().is_empty() {
            errors.push(ValidationError {
                field: "password".into(),
                message: "Password is required for credentials login".into(),
            });
        }

        if !errors.is_empty() {
            return Err(MqttServiceError::BadRequest(errors));
        }

        debug!("[Service | CheckMQTTActive] User MQTT input validation passed.");
        Ok(true)
    }
}
