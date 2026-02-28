use log::{debug, error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

pub enum DbType {
    Mysql,
    Postgres,
}

impl DbType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "postgres" | "postgresql" => DbType::Postgres,
            _ => DbType::Mysql,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DbType::Mysql => "mysql",
            DbType::Postgres => "postgres",
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            DbType::Mysql => "🐬",
            DbType::Postgres => "🐘",
        }
    }
}

/// Initialize database connection and return a `DatabaseConnection`.
pub async fn init_db(
    db_type: DbType,
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    database: &str,
) -> Result<DatabaseConnection, DbErr> {
    let url = match db_type {
        DbType::Mysql => format!(
            "mysql://{}:{}@{}:{}/{}?ssl-mode=disabled",
            user, password, host, port, database
        ),
        DbType::Postgres => format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        ),
    };

    info!(
        "{} Attempting to connect to {} via Sea-ORM at {}:{} (db: {})",
        db_type.emoji(),
        db_type.as_str(),
        host, port, database
    );

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let mut retry_count = 0;
    let max_retries = 10;

    let db = loop {
        match Database::connect(opt.clone()).await {
            Ok(conn) => {
                info!("🟢 {} connected successfully via Sea-ORM", db_type.as_str());
                break conn;
            }
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                info!(
                    "⚠️ {} connection failed (attempt {}/{}): {}. Retrying in 3s...",
                    db_type.as_str(), retry_count, max_retries, e
                );
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            Err(e) => {
                error!(
                    "❌ {} connection exhausted after {} retries: {}",
                    db_type.as_str(), max_retries, e
                );
                return Err(e);
            }
        }
    };

    // Run migrations
    info!("🚀 Running database migrations...");
    Migrator::up(&db, None).await?;
    info!("✅ Migrations completed successfully");

    Ok(db)
}

/// Close the database connection.
pub async fn close_db(conn: DatabaseConnection) {
    if let Err(e) = conn.close().await {
        error!("❌ Error closing database connection: {}", e);
    } else {
        debug!("[Infrastructure | Database] Database connection closed successfully.");
    }
}
