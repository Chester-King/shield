use crate::{
    middleware::{AppError, Result},
    models::{
        session::{AuthResponse, Session},
        user::{User, UserResponse, CreateUserRequest, LoginRequest, AuthMethod},
    },
    utils::JwtManager,
};
use axum::{extract::{Query, State}, Json, response::Redirect};
use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;
use serde::{Deserialize, Serialize};
use reqwest;
use bip39::Mnemonic;
use rand::RngCore;
use zcash_protocol::consensus::Network;

/// Helper to parse DateTime string from database
fn parse_datetime(s: &str) -> std::result::Result<DateTime<Utc>, sqlx::Error> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            // Try parsing with space instead of T
            chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f%#z")
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

/// Helper to parse User from a database row (since sqlx uuid feature is disabled)
fn user_from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<User, sqlx::Error> {
    let id_str: String = row.try_get("id")?;
    let auth_method_str: String = row.try_get("auth_method")?;
    let created_at_str: String = row.try_get("created_at")?;
    let updated_at_str: String = row.try_get("updated_at")?;

    Ok(User {
        id: Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        email: row.try_get("email")?,
        password_hash: row.try_get("password_hash")?,
        full_name: row.try_get("full_name")?,
        email_verified: row.try_get("email_verified")?,
        auth_method: AuthMethod::from_str(&auth_method_str),
        created_at: parse_datetime(&created_at_str)?,
        updated_at: parse_datetime(&updated_at_str)?,
    })
}

/// Helper to parse Session from a database row
fn session_from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Session, sqlx::Error> {
    let id_str: String = row.try_get("id")?;
    let user_id_str: String = row.try_get("user_id")?;
    let expires_at_str: String = row.try_get("expires_at")?;
    let created_at_str: String = row.try_get("created_at")?;

    Ok(Session {
        id: Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        user_id: Uuid::parse_str(&user_id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
        refresh_token: row.try_get("refresh_token")?,
        expires_at: parse_datetime(&expires_at_str)?,
        created_at: parse_datetime(&created_at_str)?,
        user_agent: row.try_get("user_agent")?,
        ip_address: row.try_get("ip_address")?,
    })
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_manager: Arc<JwtManager>,
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(refresh_token): Json<String>,
) -> Result<Json<AuthResponse>> {
    // Verify refresh token
    let claims = state.jwt_manager.verify_token(&refresh_token)?;

    // Check if it's a refresh token
    if claims.token_type != crate::utils::TokenType::Refresh {
        return Err(AppError::Unauthorized("Invalid token type".to_string()));
    }

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".to_string()))?;

    // Check if session exists and is valid - use string casts for UUID
    let session_row = sqlx::query(
        "SELECT id::text, user_id::text, refresh_token, expires_at, created_at, user_agent, ip_address
         FROM sessions WHERE refresh_token = $1 AND user_id = $2::uuid AND expires_at > NOW()"
    )
    .bind(&refresh_token)
    .bind(user_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid or expired refresh token".to_string()))?;

    let session = session_from_row(&session_row)?;

    // Get user - use string casts for UUID
    let user_row = sqlx::query(
        "SELECT id::text, email, password_hash, full_name, email_verified, auth_method::text, created_at, updated_at
         FROM users WHERE id = $1::uuid"
    )
    .bind(user_id.to_string())
    .fetch_one(&state.db)
    .await?;

    let user = user_from_row(&user_row)?;

    // Generate new tokens
    let new_access_token = state.jwt_manager.generate_access_token(user.id)?;
    let new_refresh_token = state.jwt_manager.generate_refresh_token(user.id)?;

    // Delete old refresh token and create new one - use string cast for UUID
    sqlx::query("DELETE FROM sessions WHERE id = $1::uuid")
        .bind(session.id.to_string())
        .execute(&state.db)
        .await?;

    let expires_at = Utc::now() + Duration::seconds(604800);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token, expires_at) VALUES ($1::uuid, $2, $3::timestamptz)"
    )
    .bind(user.id.to_string())
    .bind(&new_refresh_token)
    .bind(expires_at.to_rfc3339())
    .execute(&state.db)
    .await?;

    Ok(Json(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        user: UserResponse::from(user),
    }))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(refresh_token): Json<String>,
) -> Result<Json<serde_json::Value>> {
    // Delete the session
    sqlx::query("DELETE FROM sessions WHERE refresh_token = $1")
        .bind(&refresh_token)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

// Google OAuth structures
#[derive(Debug, Deserialize)]
pub struct GoogleAuthQuery {
    code: String,
    state: Option<String>,
}

#[derive(Debug, Serialize)]
struct GoogleTokenRequest {
    code: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    name: Option<String>,
    picture: Option<String>,
    email_verified: Option<bool>,
}

// Initiate Google OAuth flow
pub async fn google_auth_init() -> Result<Json<serde_json::Value>> {
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| AppError::Internal("GOOGLE_CLIENT_ID not configured".to_string()))?;

    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8000/api/auth/google/callback".to_string());

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=email%20profile&access_type=offline",
        google_client_id,
        urlencoding::encode(&redirect_uri)
    );

    Ok(Json(serde_json::json!({
        "url": auth_url
    })))
}

// Google OAuth callback handler
pub async fn google_auth_callback(
    State(state): State<AppState>,
    Query(params): Query<GoogleAuthQuery>,
) -> Result<Redirect> {
    let google_client_id = std::env::var("GOOGLE_CLIENT_ID")
        .map_err(|_| AppError::Internal("GOOGLE_CLIENT_ID not configured".to_string()))?;

    let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .map_err(|_| AppError::Internal("GOOGLE_CLIENT_SECRET not configured".to_string()))?;

    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8000/api/auth/google/callback".to_string());

    // Exchange authorization code for access token
    let client = reqwest::Client::new();
    let token_response = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", params.code.as_str()),
            ("client_id", &google_client_id),
            ("client_secret", &google_client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to exchange code: {}", e)))?
        .json::<GoogleTokenResponse>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse token response: {}", e)))?;

    // Get user info from Google
    let user_info = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&token_response.access_token)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get user info: {}", e)))?
        .json::<GoogleUserInfo>()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to parse user info: {}", e)))?;

    // Check if user exists - use string cast for enum
    let existing_user_row = sqlx::query(
        "SELECT id::text, email, password_hash, full_name, email_verified, auth_method::text, created_at, updated_at
         FROM users WHERE email = $1"
    )
    .bind(&user_info.email)
    .fetch_optional(&state.db)
    .await?;

    let (user, is_new_user) = match existing_user_row {
        Some(row) => (user_from_row(&row)?, false),
        None => {
            // Create new user with Google auth - use string cast for enum
            let new_user_row = sqlx::query(
                "INSERT INTO users (email, full_name, password_hash, auth_method)
                 VALUES ($1, $2, $3, $4::auth_method)
                 RETURNING id::text, email, password_hash, full_name, email_verified, auth_method::text, created_at, updated_at"
            )
            .bind(&user_info.email)
            .bind(user_info.name.as_deref().unwrap_or(""))
            .bind(Option::<String>::None) // No password for OAuth users
            .bind(AuthMethod::Google.as_str())
            .fetch_one(&state.db)
            .await?;
            (user_from_row(&new_user_row)?, true)
        }
    };

    // Auto-create wallet if user doesn't have one (for both new and existing users)
    // IMPORTANT: Double-check wallet doesn't exist to prevent duplicates
    let existing_wallet = sqlx::query(
        "SELECT id::text FROM wallets WHERE user_id = $1::uuid"
    )
    .bind(user.id.to_string())
    .fetch_optional(&state.db)
    .await?;

    if existing_wallet.is_none() {
        if is_new_user {
            tracing::info!("Creating wallet for new OAuth user {}", user.id);
        } else {
            tracing::info!("Creating wallet for existing OAuth user {} (no wallet found)", user.id);
        }

        // Generate 24-word BIP39 mnemonic (32 bytes of entropy)
        let mut entropy = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;

        let mnemonic_str = mnemonic.to_string();

        // Create wallet from mnemonic to get address
        let network = Network::MainNetwork;
        let wallet = crate::zcash::wallet::Wallet::from_mnemonic(&mnemonic, network)
            .map_err(|e| AppError::Internal(format!("Failed to create wallet: {}", e)))?;

        let address = wallet.get_address()
            .map_err(|e| AppError::Internal(format!("Failed to get address: {}", e)))?;

        // Get current block height for birthday (each wallet has its own birthday!)
        let lightwalletd_url = std::env::var("LIGHTWALLETD_MAINNET")
            .unwrap_or_else(|_| "https://na.zec.rocks:443".to_string());
        let mut lightwalletd_client = crate::zcash::lightwalletd::LightwalletdClient::new(lightwalletd_url);

        // Fetch current block height
        let birthday_height: i64 = match lightwalletd_client.connect().await {
            Ok(_) => {
                match lightwalletd_client.get_latest_block_height().await {
                    Ok(height) => {
                        tracing::info!("Setting OAuth wallet birthday to current height: {}", height);
                        height as i64
                    },
                    Err(e) => {
                        tracing::warn!("Failed to get block height for OAuth user, using Sapling activation: {}", e);
                        419200 // Sapling activation height as fallback
                    }
                }
            },
            Err(e) => {
                tracing::warn!("Failed to connect to lightwalletd for OAuth user, using Sapling activation: {}", e);
                419200 // Sapling activation height as fallback
            }
        };

        let wallet_id = Uuid::new_v4();

        // Store wallet with AWAIT to ensure completion - use UUID casts
        sqlx::query(
            "INSERT INTO wallets (id, user_id, encrypted_mnemonic, address, birthday_height, created_at)
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, NOW())"
        )
        .bind(wallet_id.to_string())
        .bind(user.id.to_string())
        .bind(&mnemonic_str) // TODO: ENCRYPT THIS IN PRODUCTION!
        .bind(&address)
        .bind(birthday_height)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create wallet for OAuth user {}: {:?}", user.id, e);
            AppError::Internal("Failed to create wallet".to_string())
        })?;

        tracing::info!("Successfully created wallet {} for OAuth user {}", wallet_id, user.id);

        // Also create Solana wallet
        match crate::solana::wallet::create_solana_wallet(&state.db, user.id).await {
            Ok((public_key, _)) => {
                tracing::info!("Successfully created Solana wallet for user {}: {}", user.id, public_key);
            }
            Err(e) => {
                tracing::error!("Failed to create Solana wallet for user {}: {:?}", user.id, e);
                // Don't fail the entire auth flow if Solana wallet creation fails
            }
        }
    } else {
        tracing::info!("Wallet already exists for OAuth user {}, skipping creation", user.id);
    }

    // Generate tokens
    let access_token = state.jwt_manager.generate_access_token(user.id)?;
    let refresh_token = state.jwt_manager.generate_refresh_token(user.id)?;

    // Store refresh token - use UUID cast
    let expires_at = Utc::now() + Duration::seconds(604800);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token, expires_at) VALUES ($1::uuid, $2, $3::timestamptz)"
    )
    .bind(user.id.to_string())
    .bind(&refresh_token)
    .bind(expires_at.to_rfc3339())
    .execute(&state.db)
    .await?;

    // Redirect to frontend with tokens
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let redirect_url = format!(
        "{}/auth/callback?access_token={}&refresh_token={}",
        frontend_url,
        access_token,
        refresh_token
    );

    Ok(Redirect::to(&redirect_url))
}

// Email/Password signup endpoint
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate request
    request.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Check if user already exists
    let existing_user = sqlx::query(
        "SELECT id::text FROM users WHERE email = $1"
    )
    .bind(&request.email)
    .fetch_optional(&state.db)
    .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Hash password with bcrypt
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // Create new user with email auth - use string casts
    let new_user_row = sqlx::query(
        "INSERT INTO users (email, full_name, password_hash, auth_method, email_verified)
         VALUES ($1, $2, $3, $4::auth_method, $5)
         RETURNING id::text, email, password_hash, full_name, email_verified, auth_method::text, created_at, updated_at"
    )
    .bind(&request.email)
    .bind(&request.full_name)
    .bind(&password_hash)
    .bind(AuthMethod::Email.as_str())
    .bind(false) // Email not verified yet
    .fetch_one(&state.db)
    .await?;

    let new_user = user_from_row(&new_user_row)?;

    tracing::info!("Created new email user: {}", new_user.id);

    // Auto-create Zcash wallet - use UUID cast
    let existing_wallet = sqlx::query(
        "SELECT id::text FROM wallets WHERE user_id = $1::uuid"
    )
    .bind(new_user.id.to_string())
    .fetch_optional(&state.db)
    .await?;

    if existing_wallet.is_none() {
        tracing::info!("Creating wallet for new email user {}", new_user.id);

        // Generate 24-word BIP39 mnemonic (32 bytes of entropy)
        let mut entropy = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;

        let mnemonic_str = mnemonic.to_string();

        // Create wallet from mnemonic to get address
        let network = Network::MainNetwork;
        let wallet = crate::zcash::wallet::Wallet::from_mnemonic(&mnemonic, network)
            .map_err(|e| AppError::Internal(format!("Failed to create wallet: {}", e)))?;

        let address = wallet.get_address()
            .map_err(|e| AppError::Internal(format!("Failed to get address: {}", e)))?;

        // Use a recent block height as birthday (skip lightwalletd to avoid timeout)
        // As of Dec 2025, Zcash mainnet is around block 3,154,000
        // Setting to 3,150,000 means only ~4000 blocks to scan for new wallets
        let birthday_height: i64 = 3150000;
        tracing::info!("Setting wallet birthday to recent height: {}", birthday_height);

        let wallet_id = Uuid::new_v4();

        // Use UUID casts for wallet insert
        sqlx::query(
            "INSERT INTO wallets (id, user_id, encrypted_mnemonic, address, birthday_height, created_at)
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, NOW())"
        )
        .bind(wallet_id.to_string())
        .bind(new_user.id.to_string())
        .bind(&mnemonic_str)
        .bind(&address)
        .bind(birthday_height)
        .execute(&state.db)
        .await?;

        tracing::info!("Successfully created wallet {} for email user {}", wallet_id, new_user.id);

        // Also create Solana wallet
        match crate::solana::wallet::create_solana_wallet(&state.db, new_user.id).await {
            Ok((public_key, _)) => {
                tracing::info!("Successfully created Solana wallet for user {}: {}", new_user.id, public_key);
            }
            Err(e) => {
                tracing::error!("Failed to create Solana wallet for user {}: {:?}", new_user.id, e);
            }
        }
    }

    // Generate tokens
    let access_token = state.jwt_manager.generate_access_token(new_user.id)?;
    let refresh_token = state.jwt_manager.generate_refresh_token(new_user.id)?;

    // Store refresh token - use UUID cast
    let expires_at = Utc::now() + Duration::seconds(604800);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token, expires_at) VALUES ($1::uuid, $2, $3::timestamptz)"
    )
    .bind(new_user.id.to_string())
    .bind(&refresh_token)
    .bind(expires_at.to_rfc3339())
    .execute(&state.db)
    .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserResponse::from(new_user),
    }))
}

// Email/Password login endpoint
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate request
    request.validate().map_err(|e| {
        AppError::Validation(format!("Validation error: {}", e))
    })?;

    // Get user by email - use string casts
    let user_row = sqlx::query(
        "SELECT id::text, email, password_hash, full_name, email_verified, auth_method::text, created_at::text, updated_at::text
         FROM users WHERE email = $1"
    )
    .bind(&request.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let user = user_from_row(&user_row)?;

    // Check if user registered with email/password (not Google OAuth)
    if user.auth_method != AuthMethod::Email {
        return Err(AppError::Validation(
            "This email is registered with Google. Please use Google Sign In.".to_string()
        ));
    }

    // Verify password
    let password_hash = user.password_hash.as_ref()
        .ok_or_else(|| AppError::Internal("Password hash not found".to_string()))?;

    let password_valid = bcrypt::verify(&request.password, password_hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate tokens
    let access_token = state.jwt_manager.generate_access_token(user.id)?;
    let refresh_token = state.jwt_manager.generate_refresh_token(user.id)?;

    // Store refresh token - use UUID cast
    let expires_at = Utc::now() + Duration::seconds(604800);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token, expires_at) VALUES ($1::uuid, $2, $3::timestamptz)"
    )
    .bind(user.id.to_string())
    .bind(&refresh_token)
    .bind(expires_at.to_rfc3339())
    .execute(&state.db)
    .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserResponse::from(user),
    }))
}
