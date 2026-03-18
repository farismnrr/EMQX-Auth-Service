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
        if let Err(e) = client.subscribe(&topic, QoS::ExactlyOnce).await {
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
        
        // Record metrics
        self.metrics.auth().record_auth_request();

        // Try to parse UTF-8, use lossy conversion for malformed bytes
        let payload_str = match String::from_utf8(publish.payload.to_vec()) {
            Ok(s) => s,
            Err(e) => {
                error!("Invalid UTF-8 payload: {} (topic='{}')", e, topic);
                // Attempt to extract reply_to, requested_by, and request_id from malformed payload
                let lossy_payload = String::from_utf8_lossy(&publish.payload).to_string();
                let request_id = self.extract_field(&lossy_payload, "request_id").unwrap_or_else(|| "unknown".to_string());
                let requested_by = self.extract_field(&lossy_payload, "requested_by");
                let reply_to = self.extract_field(&lossy_payload, "reply_to");

                // Security: Only send error response if requested_by is allowed and reply_to matches expected format
                if let (Some(requester), Some(reply_topic)) = (requested_by, reply_to) {
                    let is_allowed_requester = self.config.allowed_requesters.split(',').any(|r| r.trim() == requester);
                    let expected_reply_to = format!("{}/{}", self.config.topic_reply_prefix, requester);
                    
                    if is_allowed_requester && reply_topic == expected_reply_to {
                        let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id, "Invalid UTF-8 payload", "BAD_REQUEST");
                        let _ = self.publish_response(&client, &reply_topic, &response).await;
                    } else {
                        warn!("Rejected error response for malformed payload: requested_by='{}', reply_to='{}' (security check failed)", requester, reply_topic);
                    }
                }
                return;
            }
        };

        let value: Value = match serde_json::from_str(&payload_str) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse JSON payload: {} (topic='{}')", e, topic);
                // Attempt to extract reply_to, requested_by, and request_id manually to send an error response
                let request_id = self.extract_field(&payload_str, "request_id").unwrap_or_else(|| "unknown".to_string());
                let requested_by = self.extract_field(&payload_str, "requested_by");
                let reply_to = self.extract_field(&payload_str, "reply_to");

                // Security: Only send error response if requested_by is allowed and reply_to matches expected format
                if let (Some(requester), Some(reply_topic)) = (requested_by, reply_to) {
                    let is_allowed_requester = self.config.allowed_requesters.split(',').any(|r| r.trim() == requester);
                    let expected_reply_to = format!("{}/{}", self.config.topic_reply_prefix, requester);
                    
                    if is_allowed_requester && reply_topic == expected_reply_to {
                        let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id, "Malformed JSON payload", "BAD_REQUEST");
                        let _ = self.publish_response(&client, &reply_topic, &response).await;
                    } else {
                        warn!("Rejected error response for malformed JSON: requested_by='{}', reply_to='{}' (security check failed)", requester, reply_topic);
                    }
                }
                return;
            }
        };

        // Extract envelope fields
        let schema_version = value["schema_version"].as_u64().unwrap_or(0);
        let request_id = value["request_id"].as_str().unwrap_or("").to_string();
        let reply_to = value["reply_to"].as_str().map(|s| s.to_string());
        let requested_by = value["requested_by"].as_str().unwrap_or("").to_string();
        let api_key = value["api_key"].as_str().unwrap_or("");
        let timestamp = value["timestamp"].as_i64().unwrap_or(0);

        if schema_version != 1 {
            warn!("Unsupported schema_version: {} (request_id='{}')", schema_version, request_id);
            if let Some(reply_topic) = &reply_to {
                let response: MqttRpcResponse<()> = MqttRpcResponse::error(
                    if request_id.is_empty() { "unknown".to_string() } else { request_id.clone() }, 
                    "Unsupported schema version", 
                    "UNSUPPORTED_SCHEMA_VERSION"
                );
                let _ = self.publish_response(&client, reply_topic, &response).await;
            }
            return;
        }

        // Validation for missing required envelope fields
        if request_id.is_empty() {
            warn!("Missing or empty request_id in envelope");
            if let Some(reply_topic) = &reply_to {
                let response: MqttRpcResponse<()> = MqttRpcResponse::error("unknown".to_string(), "Missing request_id", "BAD_REQUEST");
                let _ = self.publish_response(&client, reply_topic, &response).await;
            }
            return;
        }

        if requested_by.is_empty() {
            warn!("Missing or empty requested_by in envelope: request_id='{}'", request_id);
            if let Some(reply_topic) = &reply_to {
                let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Missing requested_by", "BAD_REQUEST");
                let _ = self.publish_response(&client, reply_topic, &response).await;
            }
            return;
        }

        let reply_to_topic = match reply_to {
            Some(ref t) if !t.is_empty() => t.clone(),
            _ => {
                warn!("Missing or empty reply_to in envelope: request_id='{}'", request_id);
                // Cannot reply without a topic
                return;
            }
        };

        // Validate reply_to format
        let expected_reply_to = format!("{}/{}", self.config.topic_reply_prefix, requested_by);
        if reply_to_topic != expected_reply_to {
            warn!("Invalid reply_to topic: expected='{}', got='{}' (request_id='{}')", expected_reply_to, reply_to_topic, request_id);
            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid reply_to topic", "INVALID_REPLY_TOPIC");
            let _ = self.publish_response(&client, &reply_to_topic, &response).await;
            return;
        }

        info!("Received MQTT RPC command: topic='{}', requested_by='{}', request_id='{}'", 
            topic, requested_by, request_id);

        // Security check: validate requester and API key
        let is_allowed_requester = self.config.allowed_requesters.split(',').any(|r| r.trim() == requested_by);
        let is_valid_api_key = !api_key.is_empty() && api_key == self.api_key;

        if !is_allowed_requester || !is_valid_api_key {
            warn!("Unauthorized MQTT RPC request: requested_by='{}', request_id='{}' (valid_requester={}, valid_api_key={})", 
                requested_by, request_id, is_allowed_requester, is_valid_api_key);
            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Unauthorized requester or invalid API key", "UNAUTHORIZED_COMMAND");
            let _ = self.publish_response(&client, &reply_to_topic, &response).await;
            return;
        }

        // Validate timestamp (5-minute window)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        if (now - timestamp).abs() > 300000 {
            warn!("MQTT RPC request EXPIRED: requested_by='{}', request_id='{}', timestamp={}, now={}", 
                requested_by, request_id, timestamp, now);
            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Request timestamp expired", "REQUEST_EXPIRED");
            let _ = self.publish_response(&client, &reply_to_topic, &response).await;
            return;
        }

        let command = topic.strip_prefix(&format!("{}/", self.config.topic_command_prefix)).unwrap_or("");
        
        info!("Executing MQTT RPC: command='{}', requested_by='{}', request_id='{}'", 
            command, requested_by, request_id);

        match command {
            "users.create" => {
                let payload: CreateUserPayload = match serde_json::from_value(value.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid CreateUser payload: {} (request_id='{}')", e, request_id);
                        if let Some(reply_topic) = &reply_to {
                            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid payload format", "VALIDATION_ERROR");
                            let _ = self.publish_response(&client, reply_topic, &response).await;
                        }
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
                let payload: GetUserPayload = match serde_json::from_value(value.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid GetUser payload: {} (request_id='{}')", e, request_id);
                        if let Some(reply_topic) = &reply_to {
                            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid payload format", "VALIDATION_ERROR");
                            let _ = self.publish_response(&client, reply_topic, &response).await;
                        }
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
                let payload: DeleteUserPayload = match serde_json::from_value(value.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid DeleteUser payload: {} (request_id='{}')", e, request_id);
                        if let Some(reply_topic) = &reply_to {
                            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid payload format", "VALIDATION_ERROR");
                            let _ = self.publish_response(&client, reply_topic, &response).await;
                        }
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
                let payload: ListUsersPayload = match serde_json::from_value(value.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid ListUsers payload: {} (request_id='{}')", e, request_id);
                        if let Some(reply_topic) = &reply_to {
                            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid payload format", "VALIDATION_ERROR");
                            let _ = self.publish_response(&client, reply_topic, &response).await;
                        }
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
                let payload: IssueTokenPayload = match serde_json::from_value(value.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid IssueToken payload: {} (request_id='{}')", e, request_id);
                        if let Some(reply_topic) = &reply_to {
                            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid payload format", "VALIDATION_ERROR");
                            let _ = self.publish_response(&client, reply_topic, &response).await;
                        }
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
                let payload: VerifyPasswordPayload = match serde_json::from_value(value.clone()) {
                    Ok(p) => p,
                    Err(e) => {
                        error!("Invalid VerifyPassword payload: {} (request_id='{}')", e, request_id);
                        if let Some(reply_topic) = &reply_to {
                            let response: MqttRpcResponse<()> = MqttRpcResponse::error(request_id.clone(), "Invalid payload format", "VALIDATION_ERROR");
                            let _ = self.publish_response(&client, reply_topic, &response).await;
                        }
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
        client.publish(topic, QoS::ExactlyOnce, false, payload).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    fn extract_field(&self, payload: &str, field: &str) -> Option<String> {
        let pattern = format!("\"{}\":\\s*\"([^\"]+)\"", field);
        let re = regex::Regex::new(&pattern).ok()?;
        re.captures(payload).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string())
    }
}
