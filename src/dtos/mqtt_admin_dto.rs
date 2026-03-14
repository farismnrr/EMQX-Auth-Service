use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// =============================================================================
// RPC-style Request DTOs (for backend MQTT RPC and REST)
// =============================================================================

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MqttRpcRequestEnvelope {
    pub schema_version: u8,
    pub request_id: String,
    pub reply_to: String,
    pub requested_by: String,
    pub timestamp: i64,
    /// Optional: Target user for the operation.
    #[serde(default)]
    pub target_user: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct MqttRpcRequest<T> {
    #[serde(flatten)]
    pub envelope: MqttRpcRequestEnvelope,
    #[serde(flatten)]
    pub data: T,
}

impl<T> MqttRpcRequest<T> {
    pub fn request_id(&self) -> &str {
        &self.envelope.request_id
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RpcCreateUserData {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RpcDeleteUserData {
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RpcGetUserData {
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RpcIssueTokenData {
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RpcVerifyPasswordData {
    pub username: String,
    pub password: String,
}
// =============================================================================
// Response DTOs
// =============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminUserResponse {
    pub id: i32,
    pub username: String,
    pub is_superuser: bool,
}

// =============================================================================
// RPC Response DTOs (for backend MQTT RPC and REST)
// =============================================================================

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MqttRpcResponse<T> {
    pub schema_version: u8,
    pub request_id: String,
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> MqttRpcResponse<T> {
    pub fn success(request_id: String, message: String, data: Option<T>) -> Self {
        Self {
            schema_version: 1,
            request_id,
            success: true,
            message,
            code: None,
            data,
        }
    }

    pub fn error(request_id: String, message: String, code: Option<String>) -> Self {
        Self {
            schema_version: 1,
            request_id,
            success: false,
            message,
            code,
            data: None,
        }
    }
}

// =============================================================================
// Error codes for RPC responses
// =============================================================================
#[allow(dead_code)]
pub mod error_codes {
    pub const USER_ALREADY_EXISTS: &str = "USER_ALREADY_EXISTS";
    pub const USER_NOT_FOUND: &str = "USER_NOT_FOUND";
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const JWT_ISSUE_FAILED: &str = "JWT_ISSUE_FAILED";
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    pub const UNAUTHORIZED_COMMAND: &str = "UNAUTHORIZED_COMMAND";
}

// =============================================================================
// MQTT RPC Topic Constants
// =============================================================================

#[allow(dead_code)]
pub mod topics {
    /// Prefix for all MQTT RPC command topics
    pub const RPC_COMMAND_PREFIX: &str = "iotnet/auth/commands";

    /// Topic to create a new MQTT user
    pub const RPC_COMMAND_USERS_CREATE: &str = "iotnet/auth/commands/users.create";

    /// Topic to delete an MQTT user
    pub const RPC_COMMAND_USERS_DELETE: &str = "iotnet/auth/commands/users.delete";

    /// Topic to retrieve an MQTT user's data
    pub const RPC_COMMAND_USERS_GET: &str = "iotnet/auth/commands/users.get";

    /// Topic to issue a JWT token for an MQTT user
    pub const RPC_COMMAND_TOKENS_ISSUE: &str = "iotnet/auth/commands/tokens.issue";

    /// Topic to verify an MQTT user's password
    pub const RPC_COMMAND_TOKENS_VERIFY: &str = "iotnet/auth/commands/tokens.verify";

    /// Prefix for all MQTT RPC reply topics (must append requester_id)
    pub const RPC_REPLY_PREFIX: &str = "iotnet/auth/replies";
}
