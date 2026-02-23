use thiserror::Error;

#[derive(Debug, Error)]
pub enum MqttRepositoryError {
    #[error("SeaORM Database error: {0}")]
    SeaOrm(#[from] sea_orm::DbErr),

    #[error("User not found")]
    NotFound,
}
