//! MQTT RPC DTOs - Data structures for the MQTT RPC protocol

use serde::{Deserialize, Serialize};

/// Generic MQTT RPC Request envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttRpcRequest<T> {
    pub schema_version: u32,
    pub request_id: String,
    pub reply_to: String,
    pub requested_by: String,
    pub timestamp: i64,
    #[serde(flatten)]
    pub data: T,
}

/// Generic MQTT RPC Response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttRpcResponse<T> {
    pub schema_version: u32,
    pub request_id: String,
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> MqttRpcResponse<T> {
    /// Create a success response
    pub fn success(request_id: String, message: impl Into<String>, data: Option<T>) -> Self {
        Self {
            schema_version: 1,
            request_id,
            success: true,
            message: message.into(),
            code: None,
            data,
        }
    }

    /// Create an error response
    pub fn error(request_id: String, message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            schema_version: 1,
            request_id,
            success: false,
            message: message.into(),
            code: Some(code.into()),
            data: None,
        }
    }
}
