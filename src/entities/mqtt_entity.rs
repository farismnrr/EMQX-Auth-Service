use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct MqttEntity {
    pub username: String,
    pub password_hash: String,
    pub is_superuser: bool,
    pub client_id: Option<String>,
    // Removed is_deleted as we are using Hard Delete for now in Postgres
}

impl MqttEntity {
    // Methods can be added here as needed
}
