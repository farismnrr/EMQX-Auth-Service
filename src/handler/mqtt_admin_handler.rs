use log::{debug, error, info, warn};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde_json;
use std::sync::Arc;

use crate::dtos::mqtt_admin_dto::{
    AdminCreateUserRequest, AdminDeleteUserRequest, AdminGetUserByUsernameRequest,
    AdminListUsersRequest, AdminResponse, topics,
};
use crate::services::mqtt_admin_service::MqttAdminService;

pub struct MqttAdminHandler {
    mqtt_client: AsyncClient,
    admin_service: Arc<MqttAdminService>,
}

impl MqttAdminHandler {
    pub fn new(
        mqtt_client: AsyncClient,
        admin_service: Arc<MqttAdminService>,
    ) -> Self {
        Self {
            mqtt_client,
            admin_service,
        }
    }

    pub async fn start(&self) {
        info!("📡 Starting MQTT Admin Handler");

        // Use shared subscriptions for multi-instance support
        // Only one instance will receive each message (load balancing)
        let topics_to_subscribe = [
            (topics::ADMIN_USERS_CREATE_SHARED, QoS::AtLeastOnce),
            (topics::ADMIN_USERS_DELETE_SHARED, QoS::AtLeastOnce),
            (topics::ADMIN_USERS_LIST_SHARED, QoS::AtLeastOnce),
            (topics::ADMIN_USERS_GET_BY_USERNAME_SHARED, QoS::AtLeastOnce),
        ];

        for (topic, qos) in topics_to_subscribe.iter() {
            match self.mqtt_client.subscribe(*topic, *qos).await {
                Ok(_) => info!("📬 Subscribed to topic: {}", topic),
                Err(e) => error!("❌ Failed to subscribe to topic {}: {}", topic, e),
            }
        }

        info!("✅ MQTT Admin Handler started successfully");
        info!("⚠️  Using shared subscriptions for multi-instance support");
        info!("⚠️  Note: Message handling is done via event loop in mqtt_client.rs");
    }

    pub async fn handle_message(&self, topic: &str, payload: &[u8]) {
        debug!("📨 Received MQTT message on topic: {}", topic);

        let payload_str = match std::str::from_utf8(payload) {
            Ok(s) => s,
            Err(e) => {
                error!("❌ Invalid UTF-8 payload: {}", e);
                return;
            }
        };

        match topic {
            topics::ADMIN_USERS_CREATE => {
                self.handle_create_user(payload_str).await;
            }
            topics::ADMIN_USERS_DELETE => {
                self.handle_delete_user(payload_str).await;
            }
            topics::ADMIN_USERS_LIST => {
                self.handle_list_users(payload_str).await;
            }
            topic if topic.starts_with(topics::ADMIN_USERS_GET_BY_USERNAME) => {
                self.handle_get_user_by_username(topic, payload_str).await;
            }
            _ => {
                warn!("⚠️ Received message on unknown topic: {}", topic);
            }
        }
    }

    async fn handle_create_user(&self, payload: &str) {
        debug!("📝 Handling create user request");

        let request: AdminCreateUserRequest = match serde_json::from_str(payload) {
            Ok(req) => req,
            Err(e) => {
                error!("❌ Failed to parse create user request: {}", e);
                let error_response = AdminResponse::error(
                    "unknown".to_string(),
                    "Invalid JSON payload".to_string(),
                    Some(vec![format!("Parse error: {}", e)]),
                );
                self.publish_response(topics::RESPONSE_CREATE, &error_response).await;
                return;
            }
        };

        let response = self.admin_service.create_user(request).await;
        self.publish_response(topics::RESPONSE_CREATE, &response).await;
    }

    async fn handle_delete_user(&self, payload: &str) {
        debug!("🗑️ Handling delete user request");

        let request: AdminDeleteUserRequest = match serde_json::from_str(payload) {
            Ok(req) => req,
            Err(e) => {
                error!("❌ Failed to parse delete user request: {}", e);
                let error_response = AdminResponse::error(
                    "unknown".to_string(),
                    "Invalid JSON payload".to_string(),
                    Some(vec![format!("Parse error: {}", e)]),
                );
                self.publish_response(topics::RESPONSE_DELETE, &error_response).await;
                return;
            }
        };

        let response = self.admin_service.delete_user(request).await;
        self.publish_response(topics::RESPONSE_DELETE, &response).await;
    }

    async fn handle_list_users(&self, payload: &str) {
        debug!("📋 Handling list users request");

        let request: AdminListUsersRequest = match serde_json::from_str(payload) {
            Ok(req) => req,
            Err(e) => {
                error!("❌ Failed to parse list users request: {}", e);
                let error_response = AdminResponse::error(
                    "unknown".to_string(),
                    "Invalid JSON payload".to_string(),
                    Some(vec![format!("Parse error: {}", e)]),
                );
                self.publish_response(topics::RESPONSE_LIST, &error_response).await;
                return;
            }
        };

        let response = self.admin_service.list_users(request).await;
        self.publish_response(topics::RESPONSE_LIST, &response).await;
    }

    async fn handle_get_user_by_username(&self, topic: &str, payload: &str) {
        debug!("🔍 Handling get user by username request");

        // Extract username from topic: admins/users/{username}
        let username = match topic.strip_prefix(topics::ADMIN_USERS_GET_BY_USERNAME) {
            Some(name) => name.trim().to_string(),
            None => {
                error!("❌ Could not extract username from topic: {}", topic);
                return;
            }
        };

        let request: AdminGetUserByUsernameRequest = match serde_json::from_str(payload) {
            Ok(req) => req,
            Err(e) => {
                error!("❌ Failed to parse get user by username request: {}", e);
                let error_response = AdminResponse::error(
                    "unknown".to_string(),
                    "Invalid JSON payload".to_string(),
                    Some(vec![format!("Parse error: {}", e)]),
                );
                self.publish_response(topics::RESPONSE_DETAIL, &error_response).await;
                return;
            }
        };

        // Validate that username in payload matches topic
        if request.username != username {
            error!("❌ Username mismatch: topic={}, payload={}", username, request.username);
            let error_response = AdminResponse::error(
                request.request_id,
                "Username mismatch".to_string(),
                Some(vec!["Username in topic must match username in payload".to_string()]),
            );
            self.publish_response(topics::RESPONSE_DETAIL, &error_response).await;
            return;
        }

        let response = self.admin_service.get_user_by_username(request).await;
        self.publish_response(topics::RESPONSE_DETAIL, &response).await;
    }

    pub async fn publish_response(&self, topic: &str, response: &AdminResponse) {
        let json = match serde_json::to_string(response) {
            Ok(j) => j,
            Err(e) => {
                error!("❌ Failed to serialize response: {}", e);
                return;
            }
        };

        // Publish with retain flag so clients can see the response even if they subscribe later
        match self.mqtt_client.publish(topic, QoS::AtLeastOnce, true, json).await {
            Ok(_) => debug!("📤 Published response to topic: {} (retained)", topic),
            Err(e) => error!("❌ Failed to publish response to {}: {}", topic, e),
        }
    }
}

/// Helper function to create MQTT options
pub fn create_mqtt_options(
    broker_host: &str,
    broker_port: u16,
    client_id: &str,
    username: Option<&str>,
    password: Option<&str>,
    use_tls: bool,
) -> MqttOptions {
    let mut mqtt_options = MqttOptions::new(client_id, broker_host, broker_port);
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(60));
    mqtt_options.set_clean_session(true);

    if let Some(user) = username {
        let _ = mqtt_options.set_credentials(user, password.unwrap_or(""));
    }

    // Configure TLS if needed
    if use_tls {
        use rumqttc::TlsConfiguration;
        mqtt_options.set_transport(rumqttc::Transport::tls_with_config(TlsConfiguration::default()));
    }

    mqtt_options
}
