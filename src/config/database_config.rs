use dotenvy::dotenv;
use std::env;
use std::time::Duration;

/// Database type enumeration
#[derive(Clone, Debug, PartialEq)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
    Mysql,
}

impl DatabaseType {
    /// Parse database type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "sqlite" => Ok(DatabaseType::Sqlite),
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres),
            "mysql" | "mariadb" => Ok(DatabaseType::Mysql),
            _ => Err(format!(
                "Invalid database type: {}. Supported types: sqlite, postgres, mysql",
                s
            )),
        }
    }
}

/// Database configuration loaded from environment variables
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub db_type: DatabaseType,
    pub connection_string: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl DatabaseConfig {
    /// Load database configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();

        // Get database type
        let db_type_str = env::var("DB_TYPE").unwrap_or_else(|_| "sqlite".to_string());
        let db_type = DatabaseType::from_str(&db_type_str)?;

        // Build connection string based on database type
        let connection_string = Self::build_connection_string(&db_type)?;

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
            db_type,
            connection_string,
            max_connections,
            connect_timeout: Duration::from_secs(connect_timeout_secs),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
        })
    }

    /// Build connection string based on database type
    fn build_connection_string(db_type: &DatabaseType) -> Result<String, String> {
        // Check if DATABASE_URL is provided (overrides individual settings)
        if let Ok(database_url) = env::var("DATABASE_URL") {
            if !database_url.is_empty() {
                return Ok(database_url);
            }
        }

        match db_type {
            DatabaseType::Sqlite => {
                let db_path =
                    env::var("DB_PATH").unwrap_or_else(|_| "./mqtt_auth.sqlite".to_string());
                // SQLite URL format: sqlite:path?mode=rwc
                Ok(format!("sqlite:{}?mode=rwc", db_path))
            }
            DatabaseType::Postgres => {
                let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
                let name = env::var("DB_NAME").unwrap_or_else(|_| "emqx_auth".to_string());
                let user = env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
                let password = env::var("DB_PASSWORD").unwrap_or_default();
                let schema = env::var("DB_SCHEMA").unwrap_or_else(|_| "public".to_string());

                // PostgreSQL URL format: postgres://user:password@host:port/database?options=-c search_path%3Dschema
                Ok(format!(
                    "postgres://{}:{}@{}:{}/{}?options=-c%20search_path%3D{}",
                    user, password, host, port, name, schema
                ))
            }
            DatabaseType::Mysql => {
                let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
                let name = env::var("DB_NAME").unwrap_or_else(|_| "emqx_auth".to_string());
                let user = env::var("DB_USER").unwrap_or_else(|_| "root".to_string());
                let password = env::var("DB_PASSWORD").unwrap_or_default();

                // MySQL URL format: mysql://user:password@host:port/database
                Ok(format!(
                    "mysql://{}:{}@{}:{}/{}",
                    user, password, host, port, name
                ))
            }
        }
    }

    /// Check if database is SQLite
    #[allow(dead_code)]
    pub fn is_sqlite(&self) -> bool {
        self.db_type == DatabaseType::Sqlite
    }

    /// Check if database is PostgreSQL
    #[allow(dead_code)]
    pub fn is_postgres(&self) -> bool {
        self.db_type == DatabaseType::Postgres
    }

    /// Check if database is MySQL
    #[allow(dead_code)]
    pub fn is_mysql(&self) -> bool {
        self.db_type == DatabaseType::Mysql
    }
}
