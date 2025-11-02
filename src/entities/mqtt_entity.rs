use bincode::{Encode, Decode};

#[derive(Encode, Decode)]
pub struct MqttEntity {
    pub username: String,
    pub password: String,
    pub is_deleted: bool,
}

impl MqttEntity {
    pub fn create(username: impl Into<String>, password: impl Into<String>) -> Self {
        MqttEntity {
            username: username.into(),
            password: password.into(),
            is_deleted: false,
        }
    }
}
