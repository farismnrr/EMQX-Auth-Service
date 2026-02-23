use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct MqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
    pub is_deleted: bool,
}

#[derive(Serialize)]
pub struct GetMqttListDTO {
    pub users: Vec<MqttDTO>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateMqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Deserialize)]
pub struct MqttLoginDTO {
    pub username: String,
    pub password: String,
    pub method: Option<AuthType>,
}

#[derive(Serialize)]
pub struct MqttJwtDTO {
    pub token: String,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Credentials,
    Jwt,
}

#[derive(Deserialize)]
pub struct MqttAclDTO {
    pub username: String,
    pub topic: String,
}

#[derive(Deserialize)]
pub struct DeleteMqttDTO {
    pub username: String,
}
