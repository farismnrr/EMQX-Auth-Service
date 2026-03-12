use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct MqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Serialize, ToSchema)]
pub struct GetMqttListDTO {
    pub users: Vec<MqttDTO>,
}

#[derive(Serialize, ToSchema)]
pub struct PaginationInfo {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Serialize, ToSchema)]
pub struct GetMqttListPaginatedDTO {
    pub users: Vec<MqttDTO>,
    pub pagination: PaginationInfo,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateMqttDTO {
    pub username: String,
    pub password: String,
    pub is_superuser: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct MqttLoginDTO {
    pub username: String,
    pub password: String,
    pub method: Option<AuthType>,
}

#[derive(Serialize, ToSchema)]
pub struct MqttJwtDTO {
    pub token: String,
}

#[derive(Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Credentials,
    Jwt,
}

#[derive(Deserialize, ToSchema)]
pub struct MqttAclDTO {
    pub username: String,
    pub topic: String,
}

#[derive(Serialize, ToSchema)]
pub struct MqttCredentialsDTO {
    pub username: String,
    pub password: String,
}
