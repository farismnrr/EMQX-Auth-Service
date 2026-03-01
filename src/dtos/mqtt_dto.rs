use serde::{Deserialize, Serialize};

#[derive(Serialize, utoipa::ToSchema)]
pub struct MqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct GetMqttListDTO {
    pub users: Vec<MqttDTO>,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateMqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MqttLoginDTO {
    pub username: String,
    pub password: String,
    pub method: Option<AuthType>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct MqttJwtDTO {
    pub token: String,
}

#[derive(Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Credentials,
    Jwt,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct MqttAclDTO {
    pub username: String,
    pub topic: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DeleteMqttDTO {
    pub username: String,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct MqttCredentialsDTO {
    pub username: String,
    pub password: String,
}
