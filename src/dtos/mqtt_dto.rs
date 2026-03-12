use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct MqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Serialize)]
pub struct GetMqttListDTO {
    pub users: Vec<MqttDTO>,
}

#[derive(Serialize)]
pub struct PaginationInfo {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Serialize)]
pub struct GetMqttListPaginatedDTO {
    pub users: Vec<MqttDTO>,
    pub pagination: PaginationInfo,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
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

#[derive(Serialize)]
pub struct MqttCredentialsDTO {
    pub username: String,
    pub password: String,
}
