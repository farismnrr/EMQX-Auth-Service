use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize)]
pub struct ResponseDTO<'a, T = ()>
where
    T: Serialize,
{
    pub success: bool,
    pub message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct ErrorResponseDTO<'a, D = ()>
where
    D: Serialize,
{
    pub success: bool,
    pub message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<D>,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponseValidation {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<crate::services::service_error::ValidationError>>,
}
