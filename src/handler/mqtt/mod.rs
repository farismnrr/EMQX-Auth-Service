pub mod create_user_mqtt_handler;
pub mod delete_user_mqtt_handler;
pub mod get_user_mqtt_handler;
pub mod issue_token_mqtt_handler;
pub mod verify_password_mqtt_handler;

use log::{debug, error, info, warn};
use rumqttc::{AsyncClient, Event, Incoming, Packet};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::dtos::mqtt_admin_dto::topics;
use crate::handler::mqtt::create_user_mqtt_handler::CreateUserHandler;
use crate::handler::mqtt::delete_user_mqtt_handler::DeleteUserHandler;
use crate::handler::mqtt::get_user_mqtt_handler::GetUserHandler;
use crate::handler::mqtt::issue_token_mqtt_handler::IssueTokenHandler;
use crate::handler::mqtt::verify_password_mqtt_handler::VerifyPasswordHandler;
use crate::services::mqtt_admin_service::MqttAdminService;

pub struct MqttHandler {
    _client: AsyncClient,
    mqtt_use_shared_sub: bool,
    shutdown_tx: broadcast::Sender<()>,
}

impl MqttHandler {
    pub async fn new(
        broker_host: &str,
        broker_port: u16,
        client_id: &str,
        username: Option<&str>,
        password: Option<&str>,
        use_tls: bool,
        mqtt_use_shared_sub: bool,
        admin_service: Arc<MqttAdminService>,
    ) -> Result<Self, String> {
        info!(
            "🔌 Connecting to MQTT broker at {}:{} (TLS: {}, Shared Sub: {})",
            broker_host, broker_port, use_tls, mqtt_use_shared_sub
        );

        let mqtt_options = create_mqtt_options(
            broker_host,
            broker_port,
            client_id,
            username,
            password,
            use_tls,
        );

        let (client, eventloop) = AsyncClient::new(mqtt_options, 10);

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel::<()>(1);

        // Create handlers
        let create_user_handler = Arc::new(CreateUserHandler::new(
            client.clone(),
            admin_service.clone(),
        ));
        let delete_user_handler = Arc::new(DeleteUserHandler::new(
            client.clone(),
            admin_service.clone(),
        ));
        let get_user_handler = Arc::new(GetUserHandler::new(client.clone(), admin_service.clone()));
        let issue_token_handler = Arc::new(IssueTokenHandler::new(
            client.clone(),
            admin_service.clone(),
        ));
        let verify_password_handler = Arc::new(VerifyPasswordHandler::new(
            client.clone(),
            admin_service.clone(),
        ));

        // Spawn event loop task
        let shutdown_rx = shutdown_tx.subscribe();
        let client_clone = client.clone();
        tokio::spawn(async move {
            run_event_loop(
                client_clone,
                eventloop,
                shutdown_rx,
                mqtt_use_shared_sub,
                create_user_handler,
                delete_user_handler,
                get_user_handler,
                issue_token_handler,
                verify_password_handler,
            )
            .await;
        });

        info!("✅ MQTT client connected and handler started");

        Ok(Self {
            _client: client,
            mqtt_use_shared_sub,
            shutdown_tx,
        })
    }

    pub async fn shutdown(&self) {
        info!(
            "🛑 Shutting down MQTT client (shared_sub: {})...",
            self.mqtt_use_shared_sub
        );
        let _ = self.shutdown_tx.send(());
    }
}

async fn run_event_loop(
    client: AsyncClient,
    mut eventloop: rumqttc::EventLoop,
    mut shutdown_rx: broadcast::Receiver<()>,
    mqtt_use_shared_sub: bool,
    create_user_handler: Arc<CreateUserHandler>,
    delete_user_handler: Arc<DeleteUserHandler>,
    get_user_handler: Arc<GetUserHandler>,
    issue_token_handler: Arc<IssueTokenHandler>,
    verify_password_handler: Arc<VerifyPasswordHandler>,
) {
    let mut retry_count = 0;
    let max_retries = 5;

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("🛑 MQTT event loop received shutdown signal");
                break;
            }
            event = eventloop.poll() => {
                match event {
                    Ok(event) => {
                        retry_count = 0; // Reset retry count on successful poll
                        match event {
                            Event::Incoming(Packet::ConnAck(connack)) => {
                                info!("🟢 MQTT connection acknowledged: {:?}", connack);
                                // Subscribe to RPC topics on connection
                                subscribe_to_topics(&client, mqtt_use_shared_sub).await;
                            }
                            Event::Incoming(Packet::SubAck(suback)) => {
                                info!("📬 MQTT subscription acknowledged: {:?}", suback);
                            }
                            Event::Incoming(Packet::PubAck(puback)) => {
                                debug!("📤 MQTT publish acknowledged: {:?}", puback);
                            }
                            Event::Incoming(Incoming::Publish(publish)) => {
                                debug!("📨 Received MQTT publish on topic: {}", publish.topic);
                                let payload = String::from_utf8_lossy(&publish.payload);

                                // Normalize topic (remove shared subscription prefix if present)
                                let logical_topic = if mqtt_use_shared_sub && publish.topic.starts_with("$share/") {
                                    // Topic format: $share/{group}/{actual_topic}
                                    let parts: Vec<&str> = publish.topic.splitn(4, '/').collect();
                                    if parts.len() >= 4 {
                                        parts[3]
                                    } else {
                                        publish.topic.as_str()
                                    }
                                } else {
                                    publish.topic.as_str()
                                };

                                // Route to appropriate handler based on logical topic
                                match logical_topic {
                                    topics::RPC_COMMAND_USERS_CREATE => {
                                        create_user_handler.create_user_mqtt_handler(&payload).await;
                                    }
                                    topics::RPC_COMMAND_USERS_DELETE => {
                                        delete_user_handler.delete_user_mqtt_handler(&payload).await;
                                    }
                                    topics::RPC_COMMAND_USERS_GET => {
                                        get_user_handler.get_user_mqtt_handler(&payload).await;
                                    }
                                    topics::RPC_COMMAND_TOKENS_ISSUE => {
                                        issue_token_handler.issue_token_mqtt_handler(&payload).await;
                                    }
                                    topics::RPC_COMMAND_TOKENS_VERIFY => {
                                        verify_password_handler.verify_password_mqtt_handler(&payload).await;
                                    }
                                    _ => {
                                        warn!("⚠️ Received message on unknown topic: {} (logical: {})", publish.topic, logical_topic);
                                    }
                                }
                            }
                            Event::Incoming(Incoming::Disconnect) => {
                                warn!("⚠️ MQTT disconnected by broker");
                            }
                            Event::Outgoing(rumqttc::Outgoing::PingReq) => {
                                debug!("💓 MQTT ping sent");
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        retry_count += 1;
                        warn!("⚠️ MQTT event loop error (retry {}/{}): {}", retry_count, max_retries, e);

                        // Check for fatal errors that shouldn't be retried indefinitely
                        let error_str = e.to_string();
                        if error_str.contains("NotAuthorized") || error_str.contains("Unauthorized") {
                            warn!("❌ MQTT Fatal Error: Authorization failed. Stopping MQTT client to prevent infinite loops.");
                            break;
                        }

                        if retry_count >= max_retries {
                            warn!("❌ MQTT reached maximum retry limit ({}). Stopping MQTT client.", max_retries);
                            break;
                        }

                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }
    }
}

/// Helper function to subscribe to all RPC command topics
async fn subscribe_to_topics(client: &AsyncClient, use_shared_sub: bool) {
    let topics = [
        topics::RPC_COMMAND_USERS_CREATE,
        topics::RPC_COMMAND_USERS_DELETE,
        topics::RPC_COMMAND_USERS_GET,
        topics::RPC_COMMAND_TOKENS_ISSUE,
        topics::RPC_COMMAND_TOKENS_VERIFY,
    ];

    let prefix = if use_shared_sub {
        "$share/emqx_auth_admin/"
    } else {
        ""
    };

    for topic in topics {
        let full_topic = format!("{}{}", prefix, topic);
        info!("📡 Subscribing to RPC topic: {}", full_topic);
        if let Err(e) = client
            .subscribe(&full_topic, rumqttc::QoS::AtLeastOnce)
            .await
        {
            error!("❌ Failed to subscribe to topic {}: {}", full_topic, e);
        }
    }
}

/// Helper function to create MQTT options
fn create_mqtt_options(
    broker_host: &str,
    broker_port: u16,
    client_id: &str,
    username: Option<&str>,
    password: Option<&str>,
    use_tls: bool,
) -> rumqttc::MqttOptions {
    use rumqttc::MqttOptions;

    let mut mqtt_options = MqttOptions::new(client_id, broker_host, broker_port);
    mqtt_options.set_keep_alive(std::time::Duration::from_secs(60));
    mqtt_options.set_clean_session(true);

    if let Some(user) = username {
        let _ = mqtt_options.set_credentials(user, password.unwrap_or(""));
    }

    // Configure TLS if needed
    if use_tls {
        // For TLS, use default configuration
        mqtt_options.set_transport(rumqttc::Transport::tls_with_default_config());
    }

    mqtt_options
}
