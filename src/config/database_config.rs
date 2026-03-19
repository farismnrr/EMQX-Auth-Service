use dotenvy::dotenv;
use std::env;
use std::time::Duration;

/// Database configuration loaded from environment variables
/// Only PostgreSQL is supported
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub connection_string: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl DatabaseConfig {
    /// Load database configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();

        // Check if DATABASE_URL is provided (overrides individual settings)
        let connection_string = if let Ok(database_url) = env::var("DATABASE_URL") {
            if !database_url.is_empty() {
                database_url
            } else {
                Self::build_postgres_url()?
            }
        } else {
            Self::build_postgres_url()?
        };

        // Load pool configuration with defaults
        let max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .unwrap_or(10);

        let connect_timeout_secs = env::var("DB_CONNECT_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .unwrap_or(5);

        let idle_timeout_secs = env::var("DB_IDLE_TIMEOUT")
            .unwrap_or_else(|_| "600".to_string())
            .parse::<u64>()
            .unwrap_or(600);

        Ok(Self {
            connection_string,
            max_connections,
            connect_timeout: Duration::from_secs(connect_timeout_secs),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
        })
    }

    /// Build PostgreSQL connection string from individual settings
    fn build_postgres_url() -> Result<String, String> {
        let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let name = env::var("DB_NAME").unwrap_or_else(|_| "emqx_auth".to_string());
        let user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = env::var("DB_PASSWORD").unwrap_or_default();
        let schema = env::var("DB_SCHEMA").unwrap_or_else(|_| "public".to_string());

        // PostgreSQL URL format: postgres://user:password@host:port/database?options=-c%20search_path%3Dschema
        Ok(format!(
            "postgres://{}:{}@{}:{}/{}?options=-c%20search_path%3D{}",
            user, password, host, port, name, schema
        ))
    }

    /// Check if database is PostgreSQL
    #[allow(dead_code)]
    pub fn is_postgres(&self) -> bool {
        true // Only PostgreSQL is supported
    }
}
