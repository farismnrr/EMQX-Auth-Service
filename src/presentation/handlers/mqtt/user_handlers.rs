//! User management MQTT RPC handlers

use serde::Deserialize;
use crate::application::dtos::mqtt_rpc::MqttRpcResponse;
use crate::application::use_cases::{
    CreateUserUseCase, GetUserUseCase, DeleteUserUseCase, ListUsersUseCase,
    CreateUserError, GetUserError, DeleteUserError, ListUsersError
};
use crate::application::dtos::response::{UserDTO, UserListDTO};
use crate::infrastructure::{EncryptionAdapter, MqttUserRepositoryImpl};

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Debug, Deserialize)]
pub struct GetUserPayload {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteUserPayload {
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersPayload {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}

fn default_limit() -> u32 {
    100
}

pub struct UserHandlers {
    pub create_use_case: CreateUserUseCase<MqttUserRepositoryImpl, EncryptionAdapter>,
    pub get_use_case: GetUserUseCase<MqttUserRepositoryImpl, EncryptionAdapter>,
    pub delete_use_case: DeleteUserUseCase<MqttUserRepositoryImpl>,
    pub list_use_case: ListUsersUseCase<MqttUserRepositoryImpl, EncryptionAdapter>,
}

impl UserHandlers {
    pub async fn handle_create(&self, request_id: String, payload: CreateUserPayload) -> MqttRpcResponse<()> {
        match self.create_use_case.execute(&payload.username, &payload.password, payload.is_superuser).await {
            Ok(_) => MqttRpcResponse::success(request_id, "User created successfully", None),
            Err(e) => {
                let (msg, code) = match &e {
                    CreateUserError::UserAlreadyExists(m) => (m.clone(), "USER_ALREADY_EXISTS"),
                    CreateUserError::ValidationError(m) => (m.clone(), "VALIDATION_ERROR"),
                    _ => (e.to_string(), "INTERNAL_ERROR"),
                };
                MqttRpcResponse::error(request_id, msg, code)
            }
        }
    }

    pub async fn handle_get(&self, request_id: String, payload: GetUserPayload) -> MqttRpcResponse<UserDTO> {
        match self.get_use_case.execute_by_username(&payload.username).await {
            Ok(user_dto) => MqttRpcResponse::success(request_id, "User found", Some(user_dto)),
            Err(e) => {
                let (msg, code) = match &e {
                    GetUserError::UserNotFound(m) => (m.clone(), "USER_NOT_FOUND"),
                    GetUserError::Encryption(m) => (m.clone(), "ENCRYPTION_ERROR"),
                    _ => (e.to_string(), "INTERNAL_ERROR"),
                };
                MqttRpcResponse::error(request_id, msg, code)
            }
        }
    }

    pub async fn handle_delete(&self, request_id: String, payload: DeleteUserPayload) -> MqttRpcResponse<()> {
        match self.delete_use_case.execute(&payload.username).await {
            Ok(_) => MqttRpcResponse::success(request_id, "User deleted successfully", None),
            Err(e) => {
                let (msg, code) = match &e {
                    DeleteUserError::UserNotFound(m) => (m.clone(), "USER_NOT_FOUND"),
                    _ => (e.to_string(), "INTERNAL_ERROR"),
                };
                MqttRpcResponse::error(request_id, msg, code)
            }
        }
    }

    pub async fn handle_list(&self, request_id: String, payload: ListUsersPayload) -> MqttRpcResponse<UserListDTO> {
        match self.list_use_case.execute(payload.limit, payload.offset).await {
            Ok(user_list_dto) => MqttRpcResponse::success(request_id, "Users retrieved successfully", Some(user_list_dto)),
            Err(e) => {
                let (msg, code) = match &e {
                    ListUsersError::Encryption(m) => (m.clone(), "ENCRYPTION_ERROR"),
                    _ => (e.to_string(), "INTERNAL_ERROR"),
                };
                MqttRpcResponse::error(request_id, msg, code)
            }
        }
    }
}
