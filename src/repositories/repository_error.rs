use thiserror::Error;

#[derive(Debug, Error)]
pub enum MqttRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] rocksdb::Error),

    #[error("Serialization error: {0}")]
    Encode(#[from] bincode::error::EncodeError),

    #[error("Deserialization error: {0}")]
    Decode(#[from] bincode::error::DecodeError),

    #[error("UTF8 parse error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
