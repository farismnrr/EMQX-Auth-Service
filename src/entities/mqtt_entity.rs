use bincode::{Encode, Decode};

#[derive(Encode, Decode)]
pub struct MqttEntity {
    pub username: String,
    pub password: String,
    pub is_deleted: bool,
    pub is_superuser: bool,
}

impl MqttEntity {
    pub fn create(username: impl Into<String>, password: impl Into<String>, is_superuser: impl Into<bool>) -> Self {
        MqttEntity {
            username: username.into(),
            password: password.into(),
            is_deleted: false,
            is_superuser: is_superuser.into(),
        }
    }
}
