//! MQTT RPC Adapter - implementation using rumqttc

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport, TlsConfiguration};
use serde_json::Value;
use tracing::{error, info, warn};

use crate::config::MqttConfig;
use crate::application::dtos::mqtt_rpc::MqttRpcResponse;
use crate::presentation::handlers::mqtt::user_handlers::{UserHandlers, CreateUserPayload, GetUserPayload, DeleteUserPayload, ListUsersPayload};
use crate::presentation::handlers::mqtt::token_handlers::{TokenHandlers, IssueTokenPayload, VerifyPasswordPayload};
use crate::infrastructure::AppMetrics;

pub struct MqttRpcAdapter {
    config: MqttConfig,
    api_key: String,
    user_handlers: UserHandlers,
    token_handlers: TokenHandlers,
    metrics: Arc<AppMetrics>,
}

impl MqttRpcAdapter {
    pub fn new(
        config: MqttConfig,
        api_key: String,
        user_handlers: UserHandlers,
        token_handlers: TokenHandlers,
        metrics: Arc<AppMetrics>,
    ) -> Self {
        Self {
            config,
            api_key,
            user_handlers,
            token_handlers,
            metrics,
        }
    }

    pub async fn run(self) {
        if !self.config.enabled {
            info!("MQTT RPC is disabled");
            return;
        }

        let mut mqttoptions = MqttOptions::new(&self.config.client_id, &self.config.broker_host, self.config.broker_port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));

        // Configure TLS if enabled
        if self.config.use_tls {
            info!("🔒 MQTT TLS enabled - connecting to {}:{}", self.config.broker_host, self.config.broker_port);
            // Use TLS with system certificates (no client cert required)
            let tls_config = TlsConfiguration::default();
            mqttoptions.set_transport(Transport::tls_with_config(tls_config));
        } else {
            info!("📡 MQTT TLS disabled - connecting to {}:{}", self.config.broker_host, self.config.broker_port);
        }

        if let (Some(u), Some(p)) = (&self.config.username, &self.config.password) {
            mqttoptions.set_credentials(u, p);
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        let client = Arc::new(client);

        // Subscribe to command topics
        let topic = format!("{}/#", self.config.topic_command_prefix);
        if let Err(e) = client.subscribe(&topic, QoS::AtLeastOnce).await {
            error!("Failed to subscribe to MQTT RPC topic: {}", e);
            return;
        }
        info!("Subscribed to MQTT RPC topic: {}", topic);

        let adapter = Arc::new(self);

        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) = notification {
                        let client_clone = Arc::clone(&client);
                        let adapter_clone = Arc::clone(&adapter);
                        
                        tokio::spawn(async move {
                            adapter_clone.handle_publish(client_clone, publish).await;
                        });
                    }
                }
                Err(e) => {
                    error!("MQTT eventloop error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn handle_publish(&self, client: Arc<AsyncClient>, publish: rumqttc::Publish) {
        let topic = publish.topic.clone();
        let payload_str = match String::from_utf8(publish.payload.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Invalid UTF-8 payload: {}", e);
                return;
            }
        };

        // Record metrics
        self.metrics.auth().record_auth_request();

        let value: Value = match serde_json::from_str(&payload_str) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse JSON payload: {} (topic='{}')", e, topic);
                return;
            }
        };

        // Extract envelope fields
        let request_id = value["request_id"].as_str().unwrap_or("unknown").to_string();
        let reply_to = value["reply_to"].as_str().map(|s| s.to_string());
        let requested_by = value["requested_by"].as_str().unwrap_or("unknown").to_string();
        let api_key = value["api_key"].as_str().unwrap_or("");
        let timestamp = value["timestamp"].as_i64().unwrap_or(0);

        info!("Received MQTT RPC command: topic='{}', requested_by='{}', request_id='{}'", 
            topic, requested_by, request_id);

        // Security check: validate requester and API key
        let is_allowed_requester = self.config.allowed_requesters.split(',').any(|r| r.trim() == requested_by);
        let is_valid_api_key = api_key == self.api_key;

        if !is_allowed_requester || !is_valid_api_key {
            warn!("Unauthorized MQTT RPC request: requested_by='{}', request_id='{}' (valid_requester={}, valid_api_key={})", 
                requested_by, request_id, is_allowed_requester, is_valid_api_key);
            if let Some(reply_topic) = reply_to {
                let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id, "Unauthorized requester or invalid API key", "UNAUTHORIZED_COMMAND");
                let _ = self.publish_response(&client, &reply_topic, &response).await;
            }
            return;
        }

        // Validate timestamp (5-minute window)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        if (now - timestamp).abs() > 300000 {
            warn!("MQTT RPC request EXPIRED: requested_by='{}', request_id='{}', timestamp={}, now={}", 
                requested_by, request_id, timestamp, now);
            if let Some(reply_topic) = reply_to {
                let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id, "Request timestamp expired", "REQUEST_EXPIRED");
                let _ = self.publish_response(&client, &reply_topic, &response).await;
            }
            return;
        }

        let command = topic.strip_prefix(&format!("{}/", self.config.topic_command_prefix)).unwrap_or("");
        
        info!("Executing MQTT RPC: command='{}', requested_by='{}', request_id='{}'", 
            command, requested_by, request_id);

        match command {
            "users.create" => {
                let payload: CreateUserPayload = match serde_json::from_value(value) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid CreateUser payload: {} (request_id='{}')", e, request_id);
                        return;
                    }
                };
                let response = self.user_handlers.handle_create(request_id.clone(), payload).await;
                info!("MQTT RPC COMPLETED: command='{}', request_id='{}', success={}", 
                    command, request_id, response.success);
                if let Some(reply_topic) = reply_to {
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
            "users.get" => {
                let payload: GetUserPayload = match serde_json::from_value(value) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid GetUser payload: {} (request_id='{}')", e, request_id);
                        return;
                    }
                };
                let response = self.user_handlers.handle_get(request_id.clone(), payload).await;
                info!("MQTT RPC COMPLETED: command='{}', request_id='{}', success={}", 
                    command, request_id, response.success);
                if let Some(reply_topic) = reply_to {
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
            "users.delete" => {
                let payload: DeleteUserPayload = match serde_json::from_value(value) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid DeleteUser payload: {} (request_id='{}')", e, request_id);
                        return;
                    }
                };
                let response = self.user_handlers.handle_delete(request_id.clone(), payload).await;
                info!("MQTT RPC COMPLETED: command='{}', request_id='{}', success={}", 
                    command, request_id, response.success);
                if let Some(reply_topic) = reply_to {
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
            "users.list" => {
                let payload: ListUsersPayload = match serde_json::from_value(value) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid ListUsers payload: {} (request_id='{}')", e, request_id);
                        return;
                    }
                };
                let response = self.user_handlers.handle_list(request_id.clone(), payload).await;
                info!("MQTT RPC COMPLETED: command='{}', request_id='{}', success={}", 
                    command, request_id, response.success);
                if let Some(reply_topic) = reply_to {
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
            "tokens.issue" => {
                let payload: IssueTokenPayload = match serde_json::from_value(value) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid IssueToken payload: {} (request_id='{}')", e, request_id);
                        return;
                    }
                };
                let response = self.token_handlers.handle_issue(request_id.clone(), payload).await;
                info!("MQTT RPC COMPLETED: command='{}', request_id='{}', success={}", 
                    command, request_id, response.success);
                if let Some(reply_topic) = reply_to {
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
            "tokens.verify" => {
                let payload: VerifyPasswordPayload = match serde_json::from_value(value) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid VerifyPassword payload: {} (request_id='{}')", e, request_id);
                        return;
                    }
                };
                let response = self.token_handlers.handle_verify(request_id.clone(), payload).await;
                info!("MQTT RPC COMPLETED: command='{}', request_id='{}', success={}", 
                    command, request_id, response.success);
                if let Some(reply_topic) = reply_to {
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
            _ => {
                warn!("Unknown MQTT RPC command: '{}' (request_id='{}')", command, request_id);
                if let Some(reply_topic) = reply_to {
                    let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id, "Unknown command", "UNKNOWN_COMMAND");
                    let _ = self.publish_response(&client, &reply_topic, &response).await;
                }
            }
        }
    }

    async fn publish_response<T: serde::Serialize>(&self, client: &AsyncClient, topic: &str, response: &MqttRpcResponse<T>) -> Result<(), String> {
        let payload = serde_json::to_string(response).map_err(|e| e.to_string())?;
        client.publish(topic, QoS::AtLeastOnce, false, payload).await.map_err(|e| e.to_string())?;
        Ok(())
    }
}
