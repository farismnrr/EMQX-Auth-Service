use log::{debug, error};
use rumqttc::QoS;
use serde_json;

use crate::dtos::mqtt_admin_dto::{MqttRpcRequestEnvelope, MqttRpcResponse};

/// Security: Validate RPC request envelope
///
/// The security model for MQTT RPC is based on two layers:
/// 1. Broker-level ACLs: The publisher identity must have permission to publish to
///    iotnet/auth/commands/* topics.
/// 2. Application-level Allowlist: The requested_by field must match an identity in the
///    MQTT_ADMIN_ALLOWED_REQUESTERS environment variable.
/// 3. Routing Integrity: The reply_to field must exactly match iotnet/auth/replies/{requested_by}
///    to prevent attackers from routing sensitive responses to arbitrary topics.
///
/// Note: requested_by represents the SERVICE identity (e.g., 'iotnet-backend'),
/// while target_user represents the SUBJECT of the operation (e.g., a device ID).
pub fn validate_rpc_envelope(
    envelope: &MqttRpcRequestEnvelope,
) -> Result<(), (&'static str, String)> {
    // Validate schema version

    if envelope.schema_version != 1 {
        return Err((
            "VALIDATION_ERROR",
            format!(
                "Invalid schema_version: expected 1, got {}",
                envelope.schema_version
            ),
        ));
    }

    // Validate requested_by against allowed requesters
    // This is the primary authorization control - only known backend services can issue commands
    let allowed_requesters = std::env::var("MQTT_ADMIN_ALLOWED_REQUESTERS")
        .unwrap_or_else(|_| "iotnet-backend".to_string());
    if !allowed_requesters
        .split(',')
        .any(|r| r.trim() == envelope.requested_by)
    {
        return Err((
            "UNAUTHORIZED_COMMAND",
            format!("Unauthorized requester: '{}'", envelope.requested_by),
        ));
    }

    // Validate reply_to - must exactly match iotnet/auth/replies/{requested_by}
    // This binds response routing to the requester identity and prevents
    // attackers from routing responses (including JWTs) to arbitrary topics.
    let expected_reply_to = format!("iotnet/auth/replies/{}", envelope.requested_by);
    if envelope.reply_to != expected_reply_to {
        return Err((
            "VALIDATION_ERROR",
            format!(
                "Invalid reply_to: must be exactly '{}', got '{}'",
                expected_reply_to, envelope.reply_to
            ),
        ));
    }

    // Validate timestamp (within 5 minutes)
    // This prevents replay attacks with old captured messages
    let now = chrono::Utc::now().timestamp_millis();
    let max_age_ms = 5 * 60 * 1000; // 5 minutes
    if (now - envelope.timestamp).abs() > max_age_ms {
        return Err((
            "VALIDATION_ERROR",
            format!(
                "Timestamp too old or in future: request timestamp={}, current={}",
                envelope.timestamp, now
            ),
        ));
    }

    Ok(())
}

/// Attempt to extract request_id and reply_to from malformed JSON for error response
pub fn extract_routing_from_payload(payload: &str) -> Option<(String, String)> {
    if let Ok(partial) = serde_json::from_str::<serde_json::Value>(payload) {
        let request_id = partial
            .get("request_id")
            .and_then(|v| v.as_str())?
            .to_string();
        let reply_to = partial
            .get("reply_to")
            .and_then(|v| v.as_str())?
            .to_string();
        return Some((request_id, reply_to));
    }
    None
}

/// Publish an RPC response to the specified topic
pub async fn publish_rpc_response<T: serde::Serialize>(
    mqtt_client: &rumqttc::AsyncClient,
    reply_to: &str,
    response: &MqttRpcResponse<T>,
) {
    let json = match serde_json::to_string(response) {
        Ok(j) => j,
        Err(e) => {
            error!("❌ Failed to serialize RPC response: {}", e);
            return;
        }
    };

    // Publish WITHOUT retain flag for RPC responses (per spec)
    match mqtt_client
        .publish(reply_to, QoS::AtLeastOnce, false, json)
        .await
    {
        Ok(_) => debug!(
            "📤 Published RPC response to topic: {} (not retained)",
            reply_to
        ),
        Err(e) => error!("❌ Failed to publish RPC response to {}: {}", reply_to, e),
    }
}

/// Create an error RPC response
pub fn error_response<T>(
    request_id: String,
    message: String,
    code: Option<String>,
) -> MqttRpcResponse<T> {
    MqttRpcResponse::error(request_id, message, code)
}
