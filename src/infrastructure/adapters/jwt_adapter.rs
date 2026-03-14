//! JWT adapter - jsonwebtoken implementation

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::application::ports::jwt_port::{JwtClaims, JwtError, JwtPort};

/// JWT adapter implementation
#[derive(Clone)]
pub struct JwtAdapter {
    secret_key: String,
    issuer: String,
}

impl JwtAdapter {
    pub fn new(secret_key: impl Into<String>, issuer: impl Into<String>) -> Self {
        Self {
            secret_key: secret_key.into(),
            issuer: issuer.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenClaims {
    sub: String,
    iss: String,
    iat: i64,
    exp: i64,
    is_superuser: bool,
}

impl JwtPort for JwtAdapter {
    fn generate_token(
        &self,
        username: &str,
        is_superuser: bool,
        expiration_hours: i64,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::hours(expiration_hours);

        let claims = TokenClaims {
            sub: username.to_string(),
            iss: self.issuer.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            is_superuser,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )
        .map_err(|e| JwtError::CreationFailed(e.to_string()))
    }

    fn validate_token(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let mut validation = Validation::default();
        validation.validate_exp = true;

        let token_data = decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &validation,
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
            _ => JwtError::ValidationFailed(e.to_string()),
        })?;

        Ok(JwtClaims {
            username: token_data.claims.sub,
            is_superuser: token_data.claims.is_superuser,
            exp: token_data.claims.exp,
        })
    }
}
