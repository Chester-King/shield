use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub exp: i64,    // Expiry time
    pub iat: i64,    // Issued at
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

pub struct JwtManager {
    secret: String,
    access_token_expiry: i64,
    refresh_token_expiry: i64,
}

impl JwtManager {
    pub fn new(secret: String, access_token_expiry: i64, refresh_token_expiry: i64) -> Self {
        Self {
            secret,
            access_token_expiry,
            refresh_token_expiry,
        }
    }

    pub fn generate_access_token(&self, user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let expiry = now + Duration::seconds(self.access_token_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Access,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let expiry = now + Duration::seconds(self.refresh_token_expiry);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            token_type: TokenType::Refresh,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}
