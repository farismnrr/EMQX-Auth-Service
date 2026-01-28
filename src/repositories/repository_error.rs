use thiserror::Error;

#[derive(Debug, Error)]
pub enum MqttRepositoryError {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Postgres error: {0}")]
    Postgres(#[from] sqlx::Error),

    // Unified for simple internal usage if needed, or keep separate
    // In code we used MqttRepositoryError::Database(e) which expected RocksDB error.
    // We should refactor code to use explicit variant or map generic "Database" to one of them.
    // Given the amount of code changes, let's keep variants specialized.
    #[error("Serialization error: {0}")]
    Encode(#[from] bincode::error::EncodeError),

    #[error("Deserialization error: {0}")]
    Decode(#[from] bincode::error::DecodeError),

    #[error("UTF8 parse error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

// Helper to allow legacy code `MqttRepositoryError::Database(e)` to work if we alias it?
// No, better to search and replace `Database` with `RocksDB` or `Postgres`.
