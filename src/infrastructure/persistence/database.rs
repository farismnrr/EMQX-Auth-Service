//! Database connection management

use log::{error, info};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

use crate::config::DatabaseConfig;

/// Initialize database connection
pub async fn init_db(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    info!("📁 Connecting to PostgreSQL database");

    let mut opt = ConnectOptions::new(config.connection_string.clone());
    opt.max_connections(config.max_connections)
        .connect_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .sqlx_logging(true);

    let db = Database::connect(opt).await?;

    info!("🟢 Database connected successfully");

    // Run migrations
    info!("🚀 Running database migrations...");
    Migrator::up(&db, None).await?;
    info!("✅ Migrations completed");

    Ok(db)
}

/// Close database connection
pub async fn close_db(conn: DatabaseConnection) {
    if let Err(e) = conn.close().await {
        error!("❌ Error closing database: {}", e);
    }
}
