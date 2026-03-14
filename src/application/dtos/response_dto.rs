use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User data transfer object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDTO {
    pub id: i64,
    pub username: String,
    pub is_superuser: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User list with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListDTO {
    pub users: Vec<UserDTO>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// Generic success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ErrorResponse {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

/// Validation error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
}

/// Validation error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    pub success: bool,
    pub message: String,
    pub errors: Vec<ValidationErrorDetail>,
}

impl ValidationErrorResponse {
    pub fn new(message: impl Into<String>, errors: Vec<ValidationErrorDetail>) -> Self {
        Self {
            success: false,
            message: message.into(),
            errors,
        }
    }
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub result: String, // "allow" or "deny"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// ACL check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclResult {
    pub result: String, // "allow" or "deny"
}
