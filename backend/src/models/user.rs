use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// AuthMethod enum - manually mapped from database string
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Google,
    Email,
}

impl AuthMethod {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "google" => AuthMethod::Google,
            _ => AuthMethod::Email,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMethod::Google => "google",
            AuthMethod::Email => "email",
        }
    }
}

// NOTE: FromRow removed because sqlx uuid feature is disabled
// Users are manually deserialized in auth.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub full_name: Option<String>,
    pub email_verified: bool,
    pub auth_method: AuthMethod,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub wallet_address: Option<String>,
    pub solana_address: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            email_verified: user.email_verified,
            created_at: user.created_at,
            wallet_address: None, // Wallet address needs to be fetched separately
            solana_address: None, // Solana address needs to be fetched separately
        }
    }
}
