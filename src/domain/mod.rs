//! Domain module - contains business entities and repository traits

pub mod mqtt_user;
pub mod repository;

pub use mqtt_user::MqttUser;
pub use repository::{MqttUserRepository, RepositoryError};
