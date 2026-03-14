use dotenvy::dotenv;
use std::env;

/// MQTT configuration for admin RPC functionality
#[derive(Clone, Debug)]
pub struct MqttConfig {
    pub enabled: bool,
    pub broker_host: String,
    pub broker_port: u16,
    pub use_tls: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub allowed_requesters: String,
    pub use_shared_sub: bool,
    pub client_id: String,
    pub topic_command_prefix: String,
    pub topic_reply_prefix: String,
}

impl MqttConfig {
    /// Load MQTT configuration from environment variables
    pub fn from_env() -> Self {
        dotenv().ok();

        let enabled = env::var("MQTT_ADMIN_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let broker_host = env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());

        let broker_port = env::var("MQTT_BROKER_PORT")
            .unwrap_or_else(|_| "1883".to_string())
            .parse::<u16>()
            .unwrap_or(1883);

        let use_tls = env::var("MQTT_USE_TLS")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        let username = env::var("MQTT_ADMIN_USERNAME").ok();
        let password = env::var("MQTT_ADMIN_PASSWORD").ok();

        let allowed_requesters = env::var("MQTT_ADMIN_ALLOWED_REQUESTERS")
            .unwrap_or_else(|_| "iotnet-backend".to_string());

        let use_shared_sub = env::var("MQTT_USE_SHARED_SUB")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        // Generate unique client ID if not provided
        let client_id = env::var("MQTT_ADMIN_CLIENT_ID").unwrap_or_else(|_| {
            let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
            let random_suffix = uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("rand")
                .to_string();
            format!("emqx_auth_admin-{}-{}", hostname, random_suffix)
        });

        let topic_command_prefix = env::var("MQTT_TOPIC_AUTH_COMMAND_PREFIX")
            .unwrap_or_else(|_| "iotnet/auth/commands".to_string());

        let topic_reply_prefix = env::var("MQTT_TOPIC_AUTH_REPLY_PREFIX")
            .unwrap_or_else(|_| "iotnet/auth/replies".to_string());

        Self {
            enabled,
            broker_host,
            broker_port,
            use_tls,
            username,
            password,
            allowed_requesters,
            use_shared_sub,
            client_id,
            topic_command_prefix,
            topic_reply_prefix,
        }
    }
}
