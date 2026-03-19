//! Application configuration

pub mod database_config;

pub use database_config::DatabaseConfig;

use dotenvy::dotenv;
use std::env;

/// Application configuration
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub secret_key: String,
    pub api_key: String,
    pub log_level: String,
    pub rate_limit_rpm: u32,
    pub otlp_endpoint: String,
    pub otlp_service_name: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();

        let secret_key = env::var("SECRET_KEY").map_err(|_| "SECRET_KEY is not set".to_string())?;

        let api_key = env::var("API_KEY").map_err(|_| "API_KEY is not set".to_string())?;

        let database = DatabaseConfig::from_env()?;

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        let rate_limit_rpm = env::var("MQTT_AUTH_RATE_LIMIT")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .unwrap_or(100);
        let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());
        let otlp_service_name =
            env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "emqx-auth-service".to_string());

        Ok(Self {
            database,
            secret_key,
            api_key,
            log_level,
            rate_limit_rpm,
            otlp_endpoint,
            otlp_service_name,
        })
    }
}
