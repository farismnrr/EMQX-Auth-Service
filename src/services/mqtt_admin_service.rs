use crate::dtos::jwt_dto::Claims;
use crate::dtos::mqtt_admin_dto::error_codes;
use crate::dtos::mqtt_admin_dto::{
    AdminUserResponse, MqttRpcResponse, RpcCreateUserData, RpcDeleteUserData, RpcGetUserData,
    RpcIssueTokenData, RpcVerifyPasswordData,
};
use crate::dtos::mqtt_dto::CreateMqttDTO;
use crate::repositories::create_mqtt_repository::CreateMqttRepository;
use crate::repositories::get_mqtt_list_repository::GetMqttListRepository;
use crate::services::get_mqtt_credentials_service::GetMqttCredentialsService;
use crate::services::service_error::MqttServiceError;
use chrono::{Duration, Utc};
use jsonwebtoken::encode;
use log::{debug, error, info};
use std::sync::Arc;

pub struct MqttAdminService {
    create_repo: Arc<CreateMqttRepository>,
    list_repo: Arc<GetMqttListRepository>,
    credentials_service: Arc<GetMqttCredentialsService>,
    secret_key: String,
}

impl MqttAdminService {
    pub fn new(
        create_repo: Arc<CreateMqttRepository>,
        list_repo: Arc<GetMqttListRepository>,
        credentials_service: Arc<GetMqttCredentialsService>,
        secret_key: String,
    ) -> Self {
        Self {
            create_repo,
            list_repo,
            credentials_service,
            secret_key,
        }
    }

    // =============================================================================
    // RPC-style methods for backend MQTT integration and REST
    // =============================================================================

    pub async fn rpc_create_user(
        &self,
        request_id: String,
        data: RpcCreateUserData,
    ) -> MqttRpcResponse<()> {
        debug!("[MQTT Admin RPC] Creating user: {}", data.username);

        // Check if username already exists
        match self.create_repo.get_by_username(&data.username).await {
            Ok(Some(_)) => {
                return MqttRpcResponse::error(
                    request_id,
                    "Username already exists".to_string(),
                    Some(error_codes::USER_ALREADY_EXISTS.to_string()),
                );
            }
            Ok(None) => {}
            Err(e) => {
                error!("[MQTT Admin RPC] Error checking username: {}", e);
                return MqttRpcResponse::error(
                    request_id,
                    "Internal error".to_string(),
                    Some(error_codes::INTERNAL_ERROR.to_string()),
                );
            }
        }

        // Create the user
        let dto = CreateMqttDTO {
            username: data.username.clone(),
            password: data.password,
            is_superuser: data.is_superuser,
        };

        match self.create_repo.create(dto).await {
            Ok(_) => {
                info!(
                    "[MQTT Admin RPC] User created successfully: {}",
                    data.username
                );
                MqttRpcResponse::success(request_id, "User created successfully".to_string(), None)
            }
            Err(e) => {
                error!("[MQTT Admin RPC] Error creating user: {}", e);
                MqttRpcResponse::error(
                    request_id,
                    "Failed to create user".to_string(),
                    Some(error_codes::INTERNAL_ERROR.to_string()),
                )
            }
        }
    }

    pub async fn rpc_delete_user(
        &self,
        request_id: String,
        data: RpcDeleteUserData,
    ) -> MqttRpcResponse<()> {
        debug!("[MQTT Admin RPC] Deleting user: {}", data.username);

        match self.list_repo.delete_mqtt_by_username(&data.username).await {
            Ok(_) => {
                info!(
                    "[MQTT Admin RPC] User deleted successfully: {}",
                    data.username
                );
                MqttRpcResponse::success(request_id, "User deleted successfully".to_string(), None)
            }
            Err(e) => {
                // Check if error is NotFound - treat as idempotent success
                if matches!(
                    e,
                    crate::repositories::repository_error::MqttRepositoryError::NotFound
                ) {
                    info!("[MQTT Admin RPC] User not found, treating delete as idempotent success: {}", data.username);
                    MqttRpcResponse::success(
                        request_id,
                        "User not found (delete is idempotent)".to_string(),
                        None,
                    )
                } else {
                    error!("[MQTT Admin RPC] Error deleting user: {}", e);
                    MqttRpcResponse::error(
                        request_id,
                        "Failed to delete user".to_string(),
                        Some(error_codes::INTERNAL_ERROR.to_string()),
                    )
                }
            }
        }
    }

    pub async fn rpc_get_user(
        &self,
        request_id: String,
        data: RpcGetUserData,
    ) -> MqttRpcResponse<AdminUserResponse> {
        debug!("[MQTT Admin RPC] Getting user: {}", data.username);

        match self
            .credentials_service
            .get_mqtt_by_username(&data.username)
            .await
        {
            Ok(Some(user)) => {
                let username = user.username.clone();
                let user_response = AdminUserResponse {
                    id: user.id,
                    username: user.username,
                    is_superuser: user.is_superuser,
                };
                info!("[MQTT Admin RPC] User retrieved successfully: {}", username);
                MqttRpcResponse::success(
                    request_id,
                    "User retrieved successfully".to_string(),
                    Some(user_response),
                )
            }
            Ok(None) => {
                debug!("[MQTT Admin RPC] User not found: {}", data.username);
                MqttRpcResponse::error(
                    request_id,
                    "User not found".to_string(),
                    Some(error_codes::USER_NOT_FOUND.to_string()),
                )
            }
            Err(e) => {
                error!("[MQTT Admin RPC] Error getting user: {}", e);
                MqttRpcResponse::error(
                    request_id,
                    "Failed to get user".to_string(),
                    Some(error_codes::INTERNAL_ERROR.to_string()),
                )
            }
        }
    }

    pub async fn rpc_issue_token(
        &self,
        request_id: String,
        data: RpcIssueTokenData,
    ) -> MqttRpcResponse<serde_json::Value> {
        debug!("[MQTT Admin RPC] Issuing token for user: {}", data.username);

        // Check if user exists
        match self
            .credentials_service
            .get_mqtt_by_username(&data.username)
            .await
        {
            Ok(Some(_)) => {
                // Generate JWT token
                let now = Utc::now();
                let claims = Claims {
                    username: data.username.clone(),
                    exp: (now + Duration::hours(1)).timestamp() as usize,
                    iat: now.timestamp() as usize,
                    sub: "IoTNet".parse().unwrap(),
                };

                match encode(
                    &jsonwebtoken::Header::default(),
                    &claims,
                    &jsonwebtoken::EncodingKey::from_secret(self.secret_key.as_ref()),
                ) {
                    Ok(token) => {
                        info!(
                            "[MQTT Admin RPC] Token issued successfully for user: {}",
                            data.username
                        );
                        MqttRpcResponse::success(
                            request_id,
                            "Token issued successfully".to_string(),
                            Some(serde_json::json!({ "token": token })),
                        )
                    }
                    Err(e) => {
                        error!("[MQTT Admin RPC] Failed to issue token: {}", e);
                        MqttRpcResponse::error(
                            request_id,
                            "Failed to issue token".to_string(),
                            Some(error_codes::JWT_ISSUE_FAILED.to_string()),
                        )
                    }
                }
            }
            Ok(None) => {
                debug!("[MQTT Admin RPC] User not found: {}", data.username);
                MqttRpcResponse::error(
                    request_id,
                    "User not found".to_string(),
                    Some(error_codes::USER_NOT_FOUND.to_string()),
                )
            }
            Err(e) => {
                error!("[MQTT Admin RPC] Error checking user: {}", e);
                MqttRpcResponse::error(
                    request_id,
                    "Internal error".to_string(),
                    Some(error_codes::INTERNAL_ERROR.to_string()),
                )
            }
        }
    }

    pub async fn rpc_verify_password(
        &self,
        request_id: String,
        data: RpcVerifyPasswordData,
    ) -> MqttRpcResponse<serde_json::Value> {
        debug!(
            "[MQTT Admin RPC] Verifying password for user: {}",
            data.username
        );

        match self
            .credentials_service
            .verify_password(&data.username, &data.password)
            .await
        {
            Ok(is_valid) => {
                if is_valid {
                    info!(
                        "[MQTT Admin RPC] Password verified successfully for user: {}",
                        data.username
                    );
                    MqttRpcResponse::success(
                        request_id,
                        "Password verified successfully".to_string(),
                        Some(serde_json::json!({ "valid": true })),
                    )
                } else {
                    debug!(
                        "[MQTT Admin RPC] Invalid password for user: {}",
                        data.username
                    );
                    MqttRpcResponse::success(
                        request_id,
                        "Invalid password".to_string(),
                        Some(serde_json::json!({ "valid": false })),
                    )
                }
            }
            Err(MqttServiceError::MqttNotFound(_)) => {
                debug!("[MQTT Admin RPC] User not found: {}", data.username);
                MqttRpcResponse::error(
                    request_id,
                    "User not found".to_string(),
                    Some(error_codes::USER_NOT_FOUND.to_string()),
                )
            }
            Err(e) => {
                error!("[MQTT Admin RPC] Error verifying password: {}", e);
                MqttRpcResponse::error(
                    request_id,
                    "Failed to verify password".to_string(),
                    Some(error_codes::INTERNAL_ERROR.to_string()),
                )
            }
        }
    }
}
