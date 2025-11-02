use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct MqttDTO {
    pub username: String,
    pub password: String,
    pub is_deleted: bool,
}

#[derive(Serialize)]
pub struct GetMqttListDTO {
    pub mqtts: Vec<MqttDTO>,
}

#[derive(Serialize)]
pub struct CreateMqttDTO {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CheckMqttActiveDTO {
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
