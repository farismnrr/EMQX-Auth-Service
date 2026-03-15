//! Response DTOs

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use serde_json::Value;

/// Generic success response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// Success response with JSON data (for OpenAPI docs - non-generic)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponseJson {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl<T> SuccessResponse<T> {
    pub fn new(message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data,
        }
    }

    pub fn ok(message: impl Into<String>) -> Self
    where
        T: Default,
    {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// User DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserDTO {
    pub id: i64,
    pub username: String,
    pub password: Option<String>,
    pub is_superuser: bool,
}

impl UserDTO {
    pub fn from_domain(user: crate::domain::MqttUser, decrypted_password: Option<String>) -> Self {
        Self {
            id: user.id,
            username: user.username,
            password: decrypted_password,
            is_superuser: user.is_superuser,
        }
    }
}

/// User list DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserListDTO {
    pub users: Vec<UserDTO>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}
