use std::sync::Arc;
use log::debug;
use crate::repositories::mqtt_login_repository::MqttLoginRepository;
use crate::services::service_error::{MqttServiceError, ValidationError};
use crate::dtos::mqtt_dto::{AuthType, CheckMqttActiveDTO};
use crate::utils::hash_password::verify_password;
use crate::utils::jwt_sign::create_jwt;

pub struct MqttLoginService {
    repo: Arc<MqttLoginRepository>,
    secret_key: String,
}

impl MqttLoginService {
    pub fn new(repo: Arc<MqttLoginRepository>, secret_key: String) -> Self {
        Self { repo, secret_key }
    }

    pub fn login_with_credentials(&self, dto: CheckMqttActiveDTO) -> Result<(bool, String), MqttServiceError> {
        self.mqtt_input_credentials_validation(&dto)?;

        let mqtt = match self.repo.login_with_credentials(&dto.username)? {
            Some(u) => u,
            None => {
                debug!("[Service | CheckMQTTActive] User MQTT not found: {}", dto.username);
                return Err(MqttServiceError::MqttNotFound(" User MQTT not found".into()));
            }
        };

        if mqtt.is_deleted {
            debug!("[Service | CheckMQTTActive] User MQTT is deleted or inactive: {}", dto.username);
            return Err(MqttServiceError::MqttNotActive("User MQTT is not active or deleted".into()));
        }

        match dto.method.unwrap() {
            AuthType::Credentials => {
                let is_valid = verify_password(&dto.password, &mqtt.password);
                if !is_valid {
                    debug!("[Service | CheckMQTTActive] Invalid credentials for user MQTT: {}", dto.username);
                    return Err(MqttServiceError::InvalidCredentials("Invalid credentials".into()));
                }

                Ok((true, String::new()))
            }
            AuthType::Jwt => {
                let token = create_jwt(&dto.username, &self.secret_key)
                    .map_err(|e| MqttServiceError::JwtError(e.to_string()))?;
                debug!("[Service | CheckMQTTActive] JWT token created for user MQTT: {}", dto.username);
                Ok((true, token))
            }
        }
    }
    
    fn mqtt_input_credentials_validation(&self, dto: &CheckMqttActiveDTO) -> Result<bool, MqttServiceError> {
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