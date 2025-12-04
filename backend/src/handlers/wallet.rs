use crate::middleware::{AppError, Result};
use crate::handlers::AppState;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use bip39::Mnemonic;
use rand::RngCore;
use sqlx::Row;
use zcash_protocol::consensus::Network;

#[derive(Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct CreateWalletResponse {
    pub wallet_id: Uuid,
    pub address: String,
    pub mnemonic: String, // SECURITY: In production, encrypt this or return only once!
}

#[derive(Serialize, Deserialize)]
pub struct GetAddressRequest {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct AddressResponse {
    pub address: String,
}

/// Create a new Zcash wallet for a user
#[axum::debug_handler]
pub async fn create_wallet(
    State(state): State<AppState>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<CreateWalletResponse>> {
    // Check if user already has a wallet
    let existing_wallet = sqlx::query("SELECT id FROM wallets WHERE user_id = $1::uuid")
        .bind(payload.user_id.to_string())
        .fetch_optional(&state.db)
        .await?;

    if existing_wallet.is_some() {
        return Err(AppError::Conflict("User already has a wallet".to_string()));
    }

    // Generate 24-word BIP39 mnemonic (32 bytes of entropy)
    // Use OsRng directly instead of thread_rng() since it's Send-safe
    let mut entropy = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy)
        .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;

    let mnemonic_str = mnemonic.to_string();

    // Create wallet from mnemonic to get address
    let network = Network::MainNetwork; // TODO: Make this configurable
    let wallet = crate::zcash::wallet::Wallet::from_mnemonic(&mnemonic, network)
        .map_err(|e| AppError::Internal(format!("Failed to create wallet: {}", e)))?;

    let address = wallet.get_address()
        .map_err(|e| AppError::Internal(format!("Failed to get address: {}", e)))?;

    // Get current block height for birthday optimization
    // For now, use a recent mainnet height (update this regularly)
    let birthday_height: i64 = 3135000; // Dec 2024 height

    // Store wallet in database
    let wallet_id = Uuid::new_v4();

    // SECURITY WARNING: In production, ENCRYPT the mnemonic before storing!
    sqlx::query(
        "INSERT INTO wallets (id, user_id, encrypted_mnemonic, address, birthday_height, created_at)
         VALUES ($1::uuid, $2::uuid, $3, $4, $5, NOW())"
    )
    .bind(wallet_id.to_string())
    .bind(payload.user_id.to_string())
    .bind(&mnemonic_str) // TODO: ENCRYPT THIS IN PRODUCTION!
    .bind(&address)
    .bind(birthday_height)
    .execute(&state.db)
    .await?;

    Ok(Json(CreateWalletResponse {
        wallet_id,
        address,
        mnemonic: mnemonic_str,
    }))
}

/// Get wallet address for a user
#[axum::debug_handler]
pub async fn get_address(
    State(state): State<AppState>,
    Json(payload): Json<GetAddressRequest>,
) -> Result<Json<AddressResponse>> {
    let wallet_record = sqlx::query("SELECT address FROM wallets WHERE user_id = $1::uuid")
        .bind(payload.user_id.to_string())
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Wallet not found".to_string()))?;

    Ok(Json(AddressResponse {
        address: wallet_record.get("address"),
    }))
}

/// Check if user has a wallet
pub async fn has_wallet(
    State(state): State<AppState>,
    user_id: Uuid,
) -> Result<Json<serde_json::Value>> {
    let wallet_exists = sqlx::query("SELECT EXISTS(SELECT 1 FROM wallets WHERE user_id = $1::uuid) as exists")
        .bind(user_id.to_string())
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!({
        "has_wallet": wallet_exists.get::<Option<bool>, _>("exists").unwrap_or(false)
    })))
}
