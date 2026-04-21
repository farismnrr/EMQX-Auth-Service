//! JWT Claims for EMQX authentication

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// JWT claims structure compatible with EMQX JWT authentication
/// 
/// EMQX Configuration Reference:
/// - mechanism: jwt
/// - algorithm: hmac-based (HS256)
/// - verify_claims: { aud: "mqtt", iss: "broker.i-ot.net" }
/// - from: password (token passed in password field)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject - the username/client ID
    pub sub: String,
    /// Issuer - must match EMQX config: "broker.i-ot.net"
    pub iss: String,
    /// Audience - must match EMQX config: "mqtt"
    pub aud: String,
    /// Issued at timestamp (Unix epoch seconds)
    pub iat: i64,
    /// Expiration timestamp (Unix epoch seconds)
    pub exp: i64,
}

impl JwtClaims {
    /// Create new JWT claims for a username with 24-hour expiration
    pub fn new(username: impl Into<String>) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24);

        Self {
            sub: username.into(),
            iss: "broker.i-ot.net".to_string(),
            aud: "mqtt".to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        }
    }

    /// Create new JWT claims with custom expiration
    pub fn with_expiry(username: impl Into<String>, expires_at: DateTime<Utc>) -> Self {
        let now = Utc::now();

        Self {
            sub: username.into(),
            iss: "broker.i-ot.net".to_string(),
            aud: "mqtt".to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        }
    }

    /// Get expiration time as DateTime
    pub fn expires_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.exp, 0).unwrap_or_else(|| Utc::now())
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// Token pair containing JWT and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// The JWT token string
    pub token: String,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
}

impl TokenPair {
    pub fn new(token: String, expires_at: DateTime<Utc>) -> Self {
        Self { token, expires_at }
    }
}
