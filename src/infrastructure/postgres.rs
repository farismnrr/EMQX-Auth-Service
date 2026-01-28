use log::info;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::time::Duration;

pub async fn connect() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Connecting to PostgreSQL...");

    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}
