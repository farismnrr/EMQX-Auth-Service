use log::{debug, error, info};
use rumqttc::{AsyncClient, Event, Incoming, Packet};
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::handler::mqtt_admin_handler::{MqttAdminHandler, create_mqtt_options};
use crate::services::mqtt_admin_service::MqttAdminService;

pub struct MqttClientManager {
    _client: AsyncClient,
    shutdown_tx: broadcast::Sender<()>,
}

impl MqttClientManager {
    pub async fn new(
        broker_host: &str,
        broker_port: u16,
        client_id: &str,
        username: Option<&str>,
        password: Option<&str>,
        use_tls: bool,
        admin_service: Arc<MqttAdminService>,
    ) -> Result<Self, String> {
        info!("🔌 Connecting to MQTT broker at {}:{} (TLS: {})", broker_host, broker_port, use_tls);

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

        // Create admin handler
        let handler = Arc::new(MqttAdminHandler::new(
            client.clone(),
            admin_service,
        ));

        // Spawn event loop task
        let shutdown_rx = shutdown_tx.subscribe();
        let handler_clone = Arc::clone(&handler);
        tokio::spawn(async move {
            run_event_loop(eventloop, shutdown_rx, handler_clone).await;
        });

        // Start the admin handler (subscribe to topics)
        let handler_clone = Arc::clone(&handler);
        tokio::spawn(async move {
            handler_clone.start().await;
        });

        info!("✅ MQTT client connected and admin handler started");

        Ok(Self {
            _client: client,
            shutdown_tx,
        })
    }

    pub async fn shutdown(&self) {
        info!("🛑 Shutting down MQTT client...");
        let _ = self.shutdown_tx.send(());
    }
}

async fn run_event_loop(
    mut eventloop: rumqttc::EventLoop,
    mut shutdown_rx: broadcast::Receiver<()>,
    handler: Arc<MqttAdminHandler>,
) {
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("🛑 MQTT event loop received shutdown signal");
                break;
            }
            event = eventloop.poll() => {
                match event {
                    Ok(event) => {
                        match event {
                            Event::Incoming(Packet::ConnAck(connack)) => {
                                info!("🟢 MQTT connection acknowledged: {:?}", connack);
                            }
                            Event::Incoming(Packet::SubAck(suback)) => {
                                info!("📬 MQTT subscription acknowledged: {:?}", suback);
                            }
                            Event::Incoming(Packet::PubAck(puback)) => {
                                debug!("📤 MQTT publish acknowledged: {:?}", puback);
                            }
                            Event::Incoming(Incoming::Publish(publish)) => {
                                debug!("📨 Received MQTT publish on topic: {}", publish.topic);
                                handler.handle_message(&publish.topic, &publish.payload).await;
                            }
                            Event::Incoming(Incoming::Disconnect) => {
                                error!("❌ MQTT disconnected by broker");
                            }
                            Event::Outgoing(rumqttc::Outgoing::PingReq) => {
                                debug!("💓 MQTT ping sent");
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("❌ MQTT event loop error: {}", e);
                        // Reconnection is handled automatically by rumqttc
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }
    }
}
