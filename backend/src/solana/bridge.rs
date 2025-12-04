use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use sqlx::{PgPool, Row};
use std::str::FromStr;
use uuid::Uuid;

const NEAR_INTENTS_API_URL: &str = "https://1click.chaindefuser.com";

#[derive(Debug, Serialize)]
struct QuoteRequest {
    dry: bool,
    #[serde(rename = "swapType")]
    swap_type: String,
    #[serde(rename = "slippageTolerance")]
    slippage_tolerance: i32,
    #[serde(rename = "originAsset")]
    origin_asset: String,
    #[serde(rename = "depositType")]
    deposit_type: String,
    #[serde(rename = "destinationAsset")]
    destination_asset: String,
    amount: String,
    #[serde(rename = "refundTo")]
    refund_to: String,
    #[serde(rename = "refundType")]
    refund_type: String,
    recipient: String,
    #[serde(rename = "recipientType")]
    recipient_type: String,
    deadline: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BridgeQuote {
    pub amount_in: String,
    pub amount_in_formatted: String,
    pub amount_out: String,
    pub amount_out_formatted: String,
    pub deposit_address: String,
    pub time_estimate: i64,
}

/// Get JWT token from environment
fn get_jwt_token() -> Option<String> {
    std::env::var("NEAR_INTENTS_JWT").ok()
}

/// Get bridge quote from NEAR Intents for SOL → ZEC swap
pub async fn get_bridge_quote(
    amount_lamports: u64,
    refund_address: &str,
    recipient_address: &str,
) -> Result<BridgeQuote> {
    let client = Client::new();
    let url = format!("{}/v0/quote", NEAR_INTENTS_API_URL);

    let deadline = chrono::Utc::now() + chrono::Duration::hours(24);

    let quote_request = QuoteRequest {
        dry: false, // Real swap
        swap_type: "EXACT_INPUT".to_string(),
        slippage_tolerance: 100, // 1%
        origin_asset: "nep141:sol.omft.near".to_string(),
        deposit_type: "ORIGIN_CHAIN".to_string(),
        destination_asset: "nep141:zec.omft.near".to_string(),
        amount: amount_lamports.to_string(),
        refund_to: refund_address.to_string(),
        refund_type: "ORIGIN_CHAIN".to_string(),
        recipient: recipient_address.to_string(),
        recipient_type: "DESTINATION_CHAIN".to_string(),
        deadline: deadline.to_rfc3339(),
    };

    tracing::info!("NEAR Intents API URL: {}", url);
    tracing::debug!("Quote request: {:?}", quote_request);

    let jwt_token = get_jwt_token();
    tracing::info!("JWT token present: {}", jwt_token.is_some());

    let mut request = client.post(&url).json(&quote_request);

    if let Some(token) = jwt_token {
        request = request.header("Authorization", format!("Bearer {}", token));
        tracing::debug!("Added Authorization header");
    } else {
        tracing::warn!("No NEAR_INTENTS_JWT found in environment");
    }

    tracing::info!("Sending request to NEAR Intents API...");
    let response = request.send().await.context("Failed to send quote request")?;
    let status = response.status();
    tracing::info!("NEAR Intents API response status: {}", status);

    if !status.is_success() {
        let error_text = response.text().await?;
        tracing::error!("NEAR Intents API error ({}): {}", status, error_text);
        anyhow::bail!("Quote request failed ({}): {}", status, error_text);
    }

    let quote_response: Value = response.json().await?;

    // Extract quote data from nested structure
    let quote = quote_response
        .get("quote")
        .context("No quote object in response")?;

    let bridge_quote = BridgeQuote {
        amount_in: quote["amountIn"]
            .as_str()
            .unwrap_or("0")
            .to_string(),
        amount_in_formatted: quote["amountInFormatted"]
            .as_str()
            .unwrap_or("0")
            .to_string(),
        amount_out: quote["amountOut"]
            .as_str()
            .unwrap_or("0")
            .to_string(),
        amount_out_formatted: quote["amountOutFormatted"]
            .as_str()
            .unwrap_or("0")
            .to_string(),
        deposit_address: quote["depositAddress"]
            .as_str()
            .context("No deposit address in quote")?
            .to_string(),
        time_estimate: quote["timeEstimate"]
            .as_i64()
            .unwrap_or(180),
    };

    Ok(bridge_quote)
}

/// Execute bridge transaction by sending SOL to NEAR Intents deposit address
pub async fn execute_bridge(
    keypair: &Keypair,
    deposit_address: &str,
    amount_lamports: u64,
) -> Result<String> {
    let rpc_url = std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
    let rpc_client = RpcClient::new(rpc_url);

    let to_pubkey = Pubkey::from_str(deposit_address)
        .context("Invalid deposit address")?;

    // Create transfer instruction
    let instruction = system_instruction::transfer(
        &keypair.pubkey(),
        &to_pubkey,
        amount_lamports,
    );

    // Get recent blockhash
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .context("Failed to get latest blockhash")?;

    // Create transaction
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    // Send transaction
    let signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .context("Failed to send transaction")?;

    Ok(signature.to_string())
}

/// Get bridge transaction status from NEAR Intents
pub async fn get_bridge_status(deposit_address: &str) -> Result<Value> {
    let client = Client::new();
    let url = format!("{}/v0/status", NEAR_INTENTS_API_URL);

    let mut request = client
        .get(&url)
        .query(&[("depositAddress", deposit_address)]);

    if let Some(token) = get_jwt_token() {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request.send().await.context("Failed to check status")?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("Status check failed: {}", error_text);
    }

    let status: Value = response.json().await?;
    Ok(status)
}

/// Create a bridge transaction record in the database
pub async fn create_bridge_transaction(
    db: &PgPool,
    user_id: Uuid,
    amount_lamports: i64,
    expected_zec_zatoshis: i64,
    deposit_address: &str,
    refund_address: &str,
    recipient_address: &str,
) -> Result<Uuid> {
    let result = sqlx::query(
        r#"
        INSERT INTO bridge_transactions (
            user_id,
            amount_sol_lamports,
            expected_zec_zatoshis,
            deposit_address,
            refund_address,
            recipient_address,
            status
        )
        VALUES ($1::uuid, $2, $3, $4, $5, $6, 'PENDING')
        RETURNING id::text
        "#
    )
    .bind(user_id.to_string())
    .bind(amount_lamports)
    .bind(expected_zec_zatoshis)
    .bind(deposit_address)
    .bind(refund_address)
    .bind(recipient_address)
    .fetch_one(db)
    .await
    .context("Failed to create bridge transaction record")?;

    let id_str: String = result.get("id");
    Uuid::parse_str(&id_str).context("Failed to parse bridge transaction id")
}

/// Update bridge transaction with Solana transaction signature
pub async fn update_bridge_tx_signature(
    db: &PgPool,
    bridge_tx_id: Uuid,
    solana_signature: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE bridge_transactions
        SET solana_tx_signature = $1,
            status = 'PROCESSING'
        WHERE id = $2::uuid
        "#
    )
    .bind(solana_signature)
    .bind(bridge_tx_id.to_string())
    .execute(db)
    .await
    .context("Failed to update bridge transaction signature")?;

    Ok(())
}

/// Update bridge transaction status
pub async fn update_bridge_status(
    db: &PgPool,
    bridge_tx_id: Uuid,
    status: &str,
    zec_tx_hash: Option<&str>,
    actual_zec_zatoshis: Option<i64>,
    error_message: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE bridge_transactions
        SET status = $1,
            zec_tx_hash = COALESCE($2, zec_tx_hash),
            actual_zec_zatoshis = COALESCE($3, actual_zec_zatoshis),
            error_message = COALESCE($4, error_message),
            completed_at = CASE
                WHEN $1 IN ('SUCCESS', 'FAILED', 'REFUNDED') THEN NOW()
                ELSE completed_at
            END
        WHERE id = $5::uuid
        "#
    )
    .bind(status)
    .bind(zec_tx_hash)
    .bind(actual_zec_zatoshis)
    .bind(error_message)
    .bind(bridge_tx_id.to_string())
    .execute(db)
    .await
    .context("Failed to update bridge transaction status")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamports_to_sol() {
        let lamports: u64 = 50_000_000;
        let sol = lamports as f64 / 1_000_000_000.0;
        assert_eq!(sol, 0.05);
    }
}
