use crate::dtos::jwt_dto::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode, errors::Error as JwtError};

pub fn create_jwt(username: &str, secret: &str) -> Result<String, JwtError> {
    let now = Utc::now();
    let claims = Claims {
        username: username.to_string(),
        exp: (now + Duration::hours(1)).timestamp() as usize,
        iat: now.timestamp() as usize,
        sub: "IoTNet".parse().unwrap(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}
