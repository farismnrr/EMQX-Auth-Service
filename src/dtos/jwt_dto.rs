use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
}
