use log::{debug, error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;

/// Initialize SQLite database connection and return a `DatabaseConnection`.
pub async fn init_db(db_path: &str) -> Result<DatabaseConnection, DbErr> {
    let url = format!("sqlite:{}?mode=rwc", db_path);

    info!("📁 Attempting to connect to SQLite via Sea-ORM at {}", db_path);

    let mut opt = ConnectOptions::new(url);
    opt.max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    let mut retry_count = 0;
    let max_retries = 5;

    let db = loop {
        match Database::connect(opt.clone()).await {
            Ok(conn) => {
                info!("🟢 SQLite connected successfully via Sea-ORM");
                break conn;
            }
            Err(e) if retry_count < max_retries => {
                retry_count += 1;
                info!(
                    "⚠️ SQLite connection failed (attempt {}/{}): {}. Retrying in 3s...",
                    retry_count, max_retries, e
                );
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            Err(e) => {
                error!(
                    "❌ SQLite connection exhausted after {} retries: {}",
                    max_retries, e
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
