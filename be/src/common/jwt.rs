use crate::error::{Error, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Organization ID
    pub org: String,
    /// Issued at (timestamp)
    pub iat: i64,
    /// Expiration time (timestamp)
    pub exp: i64,
    /// JWT ID (for revocation tracking)
    pub jti: String,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: Uuid, org_id: Uuid, expires_in_hours: i64) -> Self {
        let now = Utc::now();
        let expiration = now + Duration::hours(expires_in_hours);

        Self {
            sub: user_id.to_string(),
            org: org_id.to_string(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Get user ID from claims
    pub fn user_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.sub).map_err(|_| Error::Unauthorized("Invalid user ID in token".to_string()))
    }

    /// Get organization ID from claims
    pub fn org_id(&self) -> Result<Uuid> {
        Uuid::parse_str(&self.org).map_err(|_| Error::Unauthorized("Invalid org ID in token".to_string()))
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}

/// JWT token manager
pub struct JwtManager {
    secret: String,
    token_lifetime_hours: i64,
}

impl JwtManager {
    /// Create a new JWT manager
    pub fn new(secret: String, token_lifetime_hours: i64) -> Self {
        Self {
            secret,
            token_lifetime_hours,
        }
    }

    /// Create a JWT token for a user
    pub fn create_token(&self, user_id: Uuid, org_id: Uuid) -> Result<String> {
        let claims = Claims::new(user_id, org_id, self.token_lifetime_hours);

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!("Failed to create JWT token: {}", e);
            Error::Internal("Failed to create authentication token".to_string())
        })
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::default();

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map_err(|e| {
            tracing::warn!("JWT validation failed: {}", e);
            Error::Unauthorized("Invalid or expired token".to_string())
        })?;

        // Additional check for expiration (belt and suspenders)
        if token_data.claims.is_expired() {
            return Err(Error::Unauthorized("Token has expired".to_string()));
        }

        Ok(token_data.claims)
    }

    /// Create a refresh token with longer lifetime
    pub fn create_refresh_token(&self, user_id: Uuid, org_id: Uuid) -> Result<String> {
        let claims = Claims::new(user_id, org_id, self.token_lifetime_hours * 24); // 24x longer

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!("Failed to create refresh token: {}", e);
            Error::Internal("Failed to create refresh token".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let manager = JwtManager::new("test_secret_key_minimum_32_chars".to_string(), 24);
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let token = manager.create_token(user_id, org_id).unwrap();
        let claims = manager.validate_token(&token).unwrap();

        assert_eq!(claims.user_id().unwrap(), user_id);
        assert_eq!(claims.org_id().unwrap(), org_id);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_invalid_token() {
        let manager = JwtManager::new("test_secret_key_minimum_32_chars".to_string(), 24);
        let result = manager.validate_token("invalid.token.here");

        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret() {
        let manager1 = JwtManager::new("secret1_minimum_32_characters_long".to_string(), 24);
        let manager2 = JwtManager::new("secret2_minimum_32_characters_long".to_string(), 24);

        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();

        let token = manager1.create_token(user_id, org_id).unwrap();
        let result = manager2.validate_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_claims_parsing() {
        let user_id = Uuid::new_v4();
        let org_id = Uuid::new_v4();
        let claims = Claims::new(user_id, org_id, 24);

        assert_eq!(claims.user_id().unwrap(), user_id);
        assert_eq!(claims.org_id().unwrap(), org_id);
        assert!(!claims.is_expired());
    }
}
