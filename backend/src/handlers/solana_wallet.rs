use crate::{
    middleware::{AppError, Result},
    solana::{bridge, rpc, wallet},
};
use axum::{extract::Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GetBalanceRequest {
    pub user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct GetBalanceResponse {
    pub balance_lamports: u64,
    pub balance_sol: f64,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct BridgeQuoteRequest {
    pub amount_lamports: u64,
    pub recipient_zcash_address: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeQuoteResponse {
    pub amount_in: String,
    pub amount_in_formatted: String,
    pub amount_out: String,
    pub amount_out_formatted: String,
    pub deposit_address: String,
    pub time_estimate: i64,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteBridgeRequest {
    pub amount_lamports: u64,
    pub recipient_zcash_address: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteBridgeResponse {
    pub bridge_tx_id: Uuid,
    pub solana_signature: String,
    pub deposit_address: String,
    pub expected_zec: String,
}

#[derive(Debug, Deserialize)]
pub struct BridgeStatusRequest {
    pub deposit_address: String,
}

/// Get Solana wallet balance
pub async fn get_balance(
    Extension(user_id): Extension<Uuid>,
    Extension(db): Extension<PgPool>,
    Json(request): Json<GetBalanceRequest>,
) -> Result<Json<GetBalanceResponse>> {
    // Verify user is requesting their own balance
    if user_id != request.user_id {
        return Err(AppError::Unauthorized(
            "Cannot access other user's balance".to_string(),
        ));
    }

    // Get user's Solana public key
    let public_key = wallet::get_public_key(&db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Solana wallet not found".to_string()))?;

    // Get balance from Solana RPC
    let balance_lamports = rpc::get_sol_balance(&public_key).await?;
    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;

    Ok(Json(GetBalanceResponse {
        balance_lamports,
        balance_sol,
        address: public_key,
    }))
}

/// Get bridge quote for SOL → ZEC swap
pub async fn get_bridge_quote(
    Extension(user_id): Extension<Uuid>,
    Extension(db): Extension<PgPool>,
    Json(request): Json<BridgeQuoteRequest>,
) -> Result<Json<BridgeQuoteResponse>> {
    tracing::info!("Bridge quote requested - amount: {} lamports, user: {}", request.amount_lamports, user_id);

    // Get user's Solana wallet (for refund address)
    let (public_key, _) = wallet::get_solana_wallet(&db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Solana wallet not found".to_string()))?;

    tracing::info!("Calling NEAR Intents API for quote - refund: {}, recipient: {}", public_key, request.recipient_zcash_address);

    // Get quote from NEAR Intents
    let quote = bridge::get_bridge_quote(
        request.amount_lamports,
        &public_key,
        &request.recipient_zcash_address,
    )
    .await
    .map_err(|e| {
        tracing::error!("Bridge quote failed: {:?}", e);
        AppError::Internal(format!("Failed to get bridge quote: {}", e))
    })?;

    Ok(Json(BridgeQuoteResponse {
        amount_in: quote.amount_in,
        amount_in_formatted: quote.amount_in_formatted,
        amount_out: quote.amount_out,
        amount_out_formatted: quote.amount_out_formatted,
        deposit_address: quote.deposit_address,
        time_estimate: quote.time_estimate,
    }))
}

/// Execute bridge transaction (send SOL to NEAR Intents)
pub async fn execute_bridge(
    Extension(user_id): Extension<Uuid>,
    Extension(db): Extension<PgPool>,
    Json(request): Json<ExecuteBridgeRequest>,
) -> Result<Json<ExecuteBridgeResponse>> {
    // Get user's Solana wallet
    let (public_key, keypair_bytes) = wallet::get_solana_wallet(&db, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Solana wallet not found".to_string()))?;

    // Reconstruct keypair from bytes
    let keypair = wallet::keypair_from_bytes(&keypair_bytes)?;

    // Get quote first to get deposit address
    let quote = bridge::get_bridge_quote(
        request.amount_lamports,
        &public_key,
        &request.recipient_zcash_address,
    )
    .await?;

    // Create bridge transaction record in database
    let expected_zec_zatoshis = quote
        .amount_out
        .parse::<i64>()
        .unwrap_or(0);

    let bridge_tx_id = bridge::create_bridge_transaction(
        &db,
        user_id,
        request.amount_lamports as i64,
        expected_zec_zatoshis,
        &quote.deposit_address,
        &public_key,
        &request.recipient_zcash_address,
    )
    .await?;

    // Execute the SOL transfer
    let solana_signature = bridge::execute_bridge(
        &keypair,
        &quote.deposit_address,
        request.amount_lamports,
    )
    .await?;

    // Update bridge transaction with signature
    bridge::update_bridge_tx_signature(&db, bridge_tx_id, &solana_signature).await?;

    Ok(Json(ExecuteBridgeResponse {
        bridge_tx_id,
        solana_signature,
        deposit_address: quote.deposit_address,
        expected_zec: quote.amount_out_formatted,
    }))
}

/// Get bridge transaction status
pub async fn get_bridge_status(
    Extension(user_id): Extension<Uuid>,
    Extension(db): Extension<PgPool>,
    Json(request): Json<BridgeStatusRequest>,
) -> Result<Json<serde_json::Value>> {
    // Verify this deposit address belongs to user's transaction
    let tx = sqlx::query(
        r#"
        SELECT id, status
        FROM bridge_transactions
        WHERE user_id = $1::uuid AND deposit_address = $2
        "#
    )
    .bind(user_id.to_string())
    .bind(&request.deposit_address)
    .fetch_optional(&db)
    .await?
    .ok_or_else(|| AppError::NotFound("Bridge transaction not found".to_string()))?;

    // Get status from NEAR Intents
    let status = bridge::get_bridge_status(&request.deposit_address).await?;

    // Update database if status changed
    if let Some(status_str) = status.get("status").and_then(|s| s.as_str()) {
        let zec_tx_hash = status
            .get("swapDetails")
            .and_then(|sd| sd.get("destinationChainTxHashes"))
            .and_then(|hashes| hashes.get(0))
            .and_then(|h| h.get("hash"))
            .and_then(|h| h.as_str());

        let actual_zec = status
            .get("swapDetails")
            .and_then(|sd| sd.get("amountOut"))
            .and_then(|a| a.as_str())
            .and_then(|s| s.parse::<i64>().ok());

        // Get UUID as String and parse it
        let tx_id_str: String = tx.get("id");
        let tx_id = Uuid::parse_str(&tx_id_str)
            .map_err(|e| AppError::Internal(format!("Invalid UUID: {}", e)))?;
        bridge::update_bridge_status(
            &db,
            tx_id,
            status_str,
            zec_tx_hash,
            actual_zec,
            None,
        )
        .await?;
    }

    Ok(Json(status))
}
