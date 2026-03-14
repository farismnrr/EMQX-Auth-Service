use log::{debug, error};
use rumqttc::AsyncClient;
use serde_json;
use std::sync::Arc;

use crate::dtos::mqtt_admin_dto::{MqttRpcRequest, MqttRpcRequestEnvelope, RpcVerifyPasswordData};
use crate::utils::mqtt_rpc_util::{
    error_response, extract_routing_from_payload, publish_rpc_response, validate_rpc_envelope,
};
use crate::services::mqtt_admin_service::MqttAdminService;

pub struct VerifyPasswordHandler {
    mqtt_client: AsyncClient,
    admin_service: Arc<MqttAdminService>,
}

impl VerifyPasswordHandler {
    pub fn new(mqtt_client: AsyncClient, admin_service: Arc<MqttAdminService>) -> Self {
        Self {
            mqtt_client,
            admin_service,
        }
    }

    pub async fn verify_password_mqtt_handler(&self, payload: &str) {
        debug!("🔐 Handling RPC verify password request");

        // First parse just the envelope to get reply_to
        let envelope: MqttRpcRequestEnvelope = match serde_json::from_str(payload) {
            Ok(env) => env,
            Err(e) => {
                error!("❌ Failed to parse RPC envelope: {}", e);
                // Attempt to extract request_id for error response
                if let Some((request_id, reply_to)) = extract_routing_from_payload(payload) {
                    let error_response = error_response::<()>(
                        request_id,
                        format!("Invalid RPC envelope: {}", e),
                        Some("VALIDATION_ERROR".to_string()),
                    );
                    publish_rpc_response(&self.mqtt_client, &reply_to, &error_response).await;
                } else {
                    error!("❌ Cannot send error response: missing request_id or reply_to in malformed envelope");
                }
                return;
            }
        };

        let request_id = envelope.request_id.clone();
        let reply_to = envelope.reply_to.clone();

        if reply_to.is_empty() {
            error!("❌ RPC request missing reply_to");
            let error_response = error_response::<()>(
                request_id,
                "Missing reply_to field".to_string(),
                Some("VALIDATION_ERROR".to_string()),
            );
            publish_rpc_response(&self.mqtt_client, &reply_to, &error_response).await;
            return;
        }

        // Security validation (no target_user match required for verify, but target_user should be set)
        if let Err((error_code, error_msg)) = validate_rpc_envelope(&envelope) {
            error!(
                "❌ RPC envelope validation failed: {} - {}",
                error_code, error_msg
            );
            let error_response =
                error_response::<()>(request_id, error_msg, Some(error_code.to_string()));
            publish_rpc_response(&self.mqtt_client, &reply_to, &error_response).await;
            return;
        }

        let request: MqttRpcRequest<RpcVerifyPasswordData> = match serde_json::from_str(payload) {
            Ok(req) => req,
            Err(e) => {
                error!("❌ Failed to parse RPC verify password request: {}", e);
                let error_response = error_response::<()>(
                    request_id,
                    format!("Invalid JSON payload: {}", e),
                    Some("VALIDATION_ERROR".to_string()),
                );
                publish_rpc_response(&self.mqtt_client, &reply_to, &error_response).await;
                return;
            }
        };

        let response = self
            .admin_service
            .rpc_verify_password(request.request_id().to_string(), request.data)
            .await;
        publish_rpc_response(&self.mqtt_client, &reply_to, &response).await;
    }
}
