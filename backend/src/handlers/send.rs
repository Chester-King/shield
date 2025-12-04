use crate::handlers::common::{
    connect_lightwalletd, derive_spending_key, get_explorer_url, get_lightwalletd_url,
    load_wallet_config, open_wallet_database, zatoshis_to_zec, zec_to_zatoshis,
};
use crate::middleware::{AppError, Result};
use crate::zcash::{account, lightwalletd, scanner, transaction};
use axum::{extract::State, Json};
use rand::rngs::OsRng;
use rusqlite::Connection as SqliteConnection;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use zcash_client_sqlite::{util::SystemClock, WalletDb};
use zcash_protocol::consensus::Network;

#[derive(Clone)]
pub struct SendState {
    pub db: PgPool,
}

#[derive(Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub user_id: Uuid,
    pub to_address: String,
    pub amount_zec: f64,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub txid: String,
    pub from_address: String,
    pub to_address: String,
    pub amount_zec: f64,
    pub fee_zec: f64,
    pub explorer_url: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct EstimateFeeRequest {
    pub user_id: Uuid,
    pub to_address: String,
    pub amount_zec: f64,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EstimateFeeResponse {
    pub estimated_fee_zec: f64,
    pub total_zec: f64,
}

/// Send ZEC transaction
/// Scans blockchain, builds and signs transaction, then broadcasts it
#[axum::debug_handler]
pub async fn send_transaction(
    State(state): State<SendState>,
    Json(payload): Json<SendTransactionRequest>,
) -> Result<Json<SendTransactionResponse>> {
    tracing::info!(
        "Send transaction requested for user {} to {} amount {}",
        payload.user_id,
        payload.to_address,
        payload.amount_zec
    );

    // Load wallet configuration
    let config = load_wallet_config(&state.db, payload.user_id, true).await?;

    tracing::info!(
        "Network: {:?}, Birthday height: {}",
        config.network,
        config.birthday_height
    );
    tracing::info!("Using wallet database: {:?}", config.db_path);

    // Connect to lightwalletd
    let client = connect_lightwalletd(config.network).await?;

    // Initialize per-user wallet database
    let mut db = open_wallet_database(&config.db_path, config.network)?;

    // Check if account exists, create if needed
    let has_accounts = check_account_exists(&config.db_path)?;

    if !has_accounts {
        tracing::info!(
            "Creating new account with birthday height {}",
            config.birthday_height
        );

        let mut account_mgr = account::AccountManager::new(db);
        db = match account_mgr
            .create_account("Primary", &config.seed, &client, Some(config.birthday_height))
            .await
        {
            Ok((account_id, _usk)) => {
                tracing::info!("Account created: {:?}", account_id);
                open_wallet_database(&config.db_path, config.network)?
            }
            Err(e) => {
                return Err(AppError::Internal(format!("Failed to create account: {}", e)));
            }
        };
    } else {
        tracing::info!("Using existing account(s)");
    }

    // Scan blockchain to find spendable funds
    tracing::info!("Scanning blockchain for spendable funds...");

    // Drop db before scanner takes ownership
    drop(db);

    // Scan blockchain with checkpoint conflict handling
    scan_blockchain_with_retry(
        &config.db_path,
        config.network,
        &config.seed,
        config.birthday_height,
        payload.user_id,
        &state.db,
    )
    .await?;

    tracing::info!("Blockchain scanned successfully");

    // Derive USK for signing
    tracing::info!("Preparing signing key...");
    let usk = derive_spending_key(&config.seed, config.network)?;

    // Build and sign transaction
    tracing::info!("Building and signing transaction...");

    let db = open_wallet_database(&config.db_path, config.network)?;
    let mut tx_builder = transaction::TransactionBuilder::new(db, config.network);

    let amount_zatoshis = zec_to_zatoshis(payload.amount_zec);

    let (raw_tx, fee_zatoshis) = tx_builder
        .build_and_sign_transaction(
            &usk,
            &payload.to_address,
            amount_zatoshis,
            payload.memo.as_deref(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to build transaction: {}", e)))?;

    let fee_zec = zatoshis_to_zec(fee_zatoshis);
    tracing::info!(
        "Transaction built ({} bytes, fee: {} ZEC)",
        raw_tx.len(),
        fee_zec
    );

    // Broadcast transaction
    tracing::info!("Broadcasting transaction...");

    // Reconnect to lightwalletd for broadcasting
    let lightwalletd_url = get_lightwalletd_url(config.network);
    let mut client = lightwalletd::LightwalletdClient::new(lightwalletd_url);
    client
        .connect()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to lightwalletd: {}", e)))?;

    let response = client
        .send_transaction(raw_tx)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to broadcast transaction: {}", e)))?;

    // The txid is in error_message field (confusing API)
    let txid = hex::encode(&response.error_message);

    tracing::info!("Transaction broadcast! TxID: {}", txid);

    // Create block explorer URL
    let explorer_url = get_explorer_url(config.network, &txid);

    let from_address = config
        .address
        .ok_or_else(|| AppError::Internal("Missing wallet address".to_string()))?;

    Ok(Json(SendTransactionResponse {
        txid: txid.clone(),
        from_address: from_address.clone(),
        to_address: payload.to_address.clone(),
        amount_zec: payload.amount_zec,
        fee_zec,
        explorer_url: explorer_url.clone(),
        message: format!(
            "Transaction sent successfully!\n\n\
            Transaction Details:\n\
            • TxID: {}\n\
            • From: {}\n\
            • To: {}\n\
            • Amount: {} ZEC\n\
            • Fee: {} ZEC\n\
            • Memo: {}\n\n\
            Track on explorer:\n\
            {}",
            txid,
            from_address,
            payload.to_address,
            payload.amount_zec,
            fee_zec,
            payload.memo.as_deref().unwrap_or("(none)"),
            explorer_url
        ),
    }))
}

/// Estimate transaction fee before sending
/// This is much faster than building the full transaction as it skips zk-SNARK generation
#[axum::debug_handler]
pub async fn estimate_fee(
    State(state): State<SendState>,
    Json(payload): Json<EstimateFeeRequest>,
) -> Result<Json<EstimateFeeResponse>> {
    tracing::info!(
        "Fee estimation requested for user {} to {} amount {}",
        payload.user_id,
        payload.to_address,
        payload.amount_zec
    );

    // Load wallet configuration
    let config = load_wallet_config(&state.db, payload.user_id, false).await?;

    // Derive USK
    let usk = derive_spending_key(&config.seed, config.network)?;

    // Open database
    let db = open_wallet_database(&config.db_path, config.network)?;

    // Estimate fee
    let mut tx_builder = transaction::TransactionBuilder::new(db, config.network);
    let amount_zatoshis = zec_to_zatoshis(payload.amount_zec);

    let fee_zatoshis = tx_builder
        .estimate_fee(
            &usk,
            &payload.to_address,
            amount_zatoshis,
            payload.memo.as_deref(),
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to estimate fee: {}", e)))?;

    let fee_zec = zatoshis_to_zec(fee_zatoshis);
    let total_zec = payload.amount_zec + fee_zec;

    tracing::info!("Estimated fee: {} ZEC (total: {} ZEC)", fee_zec, total_zec);

    Ok(Json(EstimateFeeResponse {
        estimated_fee_zec: fee_zec,
        total_zec,
    }))
}

/// Check if account exists in wallet database
fn check_account_exists(db_path: &std::path::Path) -> Result<bool> {
    match SqliteConnection::open(db_path) {
        Ok(conn) => {
            match conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| {
                row.get::<_, i64>(0)
            }) {
                Ok(count) => {
                    tracing::info!("Found {} existing account(s)", count);
                    Ok(count > 0)
                }
                Err(_) => Ok(false),
            }
        }
        Err(_) => Ok(false),
    }
}

/// Scan blockchain with automatic retry on checkpoint conflict
async fn scan_blockchain_with_retry(
    db_path: &std::path::Path,
    network: Network,
    seed: &[u8],
    birthday_height: u32,
    user_id: Uuid,
    pg_pool: &PgPool,
) -> Result<()> {
    // Create wallet_db for scanner
    let wallet_db = WalletDb::<SqliteConnection, Network, SystemClock, OsRng>::for_path(
        db_path, network, SystemClock, OsRng,
    )
    .map_err(|e| {
        AppError::Internal(format!(
            "Failed to open wallet database for scanning: {:?}",
            e
        ))
    })?;

    // Create scanner WITH db_path so checkpoint clearing works
    let client = connect_lightwalletd(network).await?;
    let mut scanner = scanner::BlockchainScanner::new_with_path(
        wallet_db, client, network, db_path.to_path_buf()
    );

    // Try to scan, if checkpoint conflict occurs, delete DB and retry
    let scan_result = scanner.scan_from_birthday().await;
    if let Err(ref e) = scan_result {
        let error_msg = format!("{}", e);
        if error_msg.contains("CheckpointConflict") {
            tracing::warn!(
                "Checkpoint conflict detected, deleting corrupted database and retrying..."
            );

            // Drop scanner to release the database
            drop(scanner);

            // Delete the corrupted database file
            if db_path.exists() {
                std::fs::remove_file(db_path).map_err(|e| {
                    AppError::Internal(format!("Failed to delete corrupted database: {}", e))
                })?;
                tracing::info!("Deleted corrupted database file");
            }

            // Clear PostgreSQL transaction data to avoid conflicts
            clear_transaction_data(pg_pool, user_id).await?;

            // Recreate database and account
            let db = open_wallet_database(db_path, network)?;
            let mut account_mgr = account::AccountManager::new(db);
            let client_retry = connect_lightwalletd(network).await?;

            account_mgr
                .create_account("Primary", seed, &client_retry, Some(birthday_height))
                .await
                .map_err(|e| {
                    AppError::Internal(format!("Failed to recreate account: {}", e))
                })?;

            drop(account_mgr);

            // Retry scan with fresh database - use new_with_path for checkpoint management
            let wallet_db_retry =
                WalletDb::<SqliteConnection, Network, SystemClock, OsRng>::for_path(
                    db_path, network, SystemClock, OsRng,
                )
                .map_err(|e| {
                    AppError::Internal(format!("Failed to reopen wallet database: {:?}", e))
                })?;

            let client_retry2 = connect_lightwalletd(network).await?;
            let mut scanner_retry = scanner::BlockchainScanner::new_with_path(
                wallet_db_retry, client_retry2, network, db_path.to_path_buf()
            );
            scanner_retry.scan_from_birthday().await.map_err(|e| {
                AppError::Internal(format!("Failed to scan blockchain after retry: {}", e))
            })?;

            tracing::info!("Successfully scanned after clearing corrupted state");
        } else {
            return Err(AppError::Internal(format!(
                "Failed to scan blockchain: {}",
                e
            )));
        }
    }

    Ok(())
}

/// Clear transaction data from PostgreSQL
async fn clear_transaction_data(pg_pool: &PgPool, user_id: Uuid) -> Result<()> {
    tracing::info!("Clearing stale transaction data from PostgreSQL...");

    sqlx::query("DELETE FROM sent_notes WHERE user_id = $1::uuid")
        .bind(user_id.to_string())
        .execute(pg_pool)
        .await?;

    sqlx::query("DELETE FROM received_notes WHERE user_id = $1::uuid")
        .bind(user_id.to_string())
        .execute(pg_pool)
        .await?;

    sqlx::query("DELETE FROM transactions WHERE user_id = $1::uuid")
        .bind(user_id.to_string())
        .execute(pg_pool)
        .await?;

    tracing::info!("Cleared stale PostgreSQL data");
    Ok(())
}
