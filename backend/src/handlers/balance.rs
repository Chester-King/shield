use crate::middleware::{AppError, Result};
use crate::zcash::{account, database, lightwalletd, scanner};
use axum::{extract::State, Json};
use bip39::Mnemonic;
use once_cell::sync::Lazy;
use rand::rngs::OsRng;
use rusqlite::Connection as SqliteConnection;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use zcash_client_sqlite::{util::SystemClock, WalletDb};
use zcash_protocol::consensus::Network;

// Global mutex map for per-user database access to prevent concurrent initialization
static USER_DB_LOCKS: Lazy<Mutex<HashMap<Uuid, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct BalanceState {
    pub db: PgPool,
}

#[derive(Serialize, Deserialize)]
pub struct GetBalanceRequest {
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct BalanceResponse {
    pub balance_zec: String,
    pub synced: bool,
    pub last_synced_height: Option<i64>,
    pub blocks_scanned: Option<usize>,
    pub notes_found: Option<usize>,
    pub chain_tip: Option<u64>,
}

/// Get wallet balance for a user
/// Performs full blockchain scanning and returns actual balance
#[axum::debug_handler]
pub async fn get_balance(
    State(state): State<BalanceState>,
    Json(payload): Json<GetBalanceRequest>,
) -> Result<Json<BalanceResponse>> {
    tracing::info!("Balance check requested for user {}", payload.user_id);

    // Acquire per-user lock to prevent concurrent database access
    let user_lock = {
        let mut locks = USER_DB_LOCKS.lock().await;
        locks
            .entry(payload.user_id)
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    };
    let _guard = user_lock.lock().await;
    tracing::info!("Acquired database lock for user {}", payload.user_id);

    // Get wallet info from PostgreSQL - use string cast for UUID
    let row = sqlx::query(
        "SELECT encrypted_mnemonic, birthday_height FROM wallets WHERE user_id = $1::uuid"
    )
    .bind(payload.user_id.to_string())
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Wallet not found".to_string()))?;

    let encrypted_mnemonic: String = row.get("encrypted_mnemonic");
    let birthday_height_i64: i64 = row.get("birthday_height");

    // Parse mnemonic (currently stored unencrypted - TODO: encrypt in production)
    let mnemonic = Mnemonic::parse(&encrypted_mnemonic)
        .map_err(|e| AppError::Internal(format!("Failed to parse mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");
    let birthday_height = birthday_height_i64 as u32;  // Convert i64 to u32

    // Get network from environment
    let network_str = env::var("ZCASH_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    let network = match network_str.to_lowercase().as_str() {
        "testnet" => Network::TestNetwork,
        _ => Network::MainNetwork,
    };

    tracing::info!("Network: {:?}, Birthday height: {}", network, birthday_height);

    // Setup per-user wallet database path
    let data_dir = PathBuf::from("./wallet_data");
    std::fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join(format!("wallet_{}.db", payload.user_id));

    tracing::info!("Using wallet database: {:?}", db_path);

    // Step 1: Connect to lightwalletd
    let lightwalletd_url = match network {
        Network::MainNetwork => {
            env::var("LIGHTWALLETD_MAINNET").unwrap_or_else(|_| "https://na.zec.rocks:443".to_string())
        }
        Network::TestNetwork => {
            env::var("LIGHTWALLETD_TESTNET").unwrap_or_else(|_| "https://testnet.zec.rocks:443".to_string())
        }
    };

    tracing::info!("Connecting to lightwalletd: {}", lightwalletd_url);
    let mut client = lightwalletd::LightwalletdClient::new(lightwalletd_url);

    client.connect().await
        .map_err(|e| AppError::Internal(format!("Failed to connect to lightwalletd: {}", e)))?;

    tracing::info!("Connected to lightwalletd");

    // Step 2: Initialize per-user wallet database
    // Check if database exists before deciding initialization strategy
    let db_exists = db_path.exists();
    tracing::info!("Database exists: {}", db_exists);

    let mut db = if db_exists {
        // Try to open existing database without running migrations
        match database::Database::open_existing(&db_path, network) {
            Ok(d) => d,
            Err(e) => {
                tracing::warn!("Failed to open existing database, will reinitialize: {}", e);
                database::Database::new(&db_path, network)
                    .map_err(|e| AppError::Internal(format!("Failed to initialize database: {}", e)))?
            }
        }
    } else {
        // New database - run full initialization
        database::Database::new(&db_path, network)
            .map_err(|e| AppError::Internal(format!("Failed to initialize database: {}", e)))?
    };

    // Step 3: Check if account exists, create if needed
    let has_accounts = match SqliteConnection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get::<_, i64>(0)) {
                Ok(count) => {
                    tracing::info!("Found {} existing account(s)", count);
                    count > 0
                }
                Err(_) => false,
            }
        }
        Err(_) => false,
    };

    // Create account if none exists
    if !has_accounts {
        tracing::info!("Creating new account with birthday height {}", birthday_height);

        let mut account_mgr = account::AccountManager::new(db);
        db = match account_mgr
            .create_account("Primary", &seed, &client, Some(birthday_height))
            .await
        {
            Ok((account_id, _usk)) => {
                tracing::info!("Account created: {:?}", account_id);
                // Use open_existing since DB is now initialized
                database::Database::open_existing(&db_path, network)
                    .map_err(|e| AppError::Internal(format!("Failed to reopen database: {}", e)))?
            }
            Err(e) => {
                return Err(AppError::Internal(format!("Failed to create account: {}", e)));
            }
        };
    } else {
        tracing::info!("Using existing account(s)");
    }

    // Get chain tip
    let chain_tip = client
        .get_latest_block_height()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to get block height: {}", e)))?;

    tracing::info!("Chain tip: {}", chain_tip);

    // Step 4: Scan blockchain
    tracing::info!("Starting blockchain scan...");

    // Drop db before scanner takes ownership
    drop(db);

    // Create wallet_db for scanner
    let wallet_db = WalletDb::<SqliteConnection, Network, SystemClock, OsRng>::for_path(
        &db_path,
        network,
        SystemClock,
        OsRng,
    )
    .map_err(|e| {
        AppError::Internal(format!("Failed to open wallet database for scanning: {:?}", e))
    })?;

    // Create scanner with database path for checkpoint management
    let mut scanner = scanner::BlockchainScanner::new_with_path(wallet_db, client, network, db_path.clone());

    // Run the scan
    let scan_result = scanner.scan_from_birthday().await.map_err(|e| {
        AppError::Internal(format!("Scan failed: {}", e))
    })?;

    tracing::info!(
        "Scan complete! Blocks scanned: {}, Notes found: {}",
        scan_result.blocks_scanned,
        scan_result.notes_discovered
    );

    // Step 5: Get balance from database
    tracing::info!("Calculating balance from database...");

    // Query balance directly from SQLite database
    // Sum UNSPENT notes from BOTH Sapling and Orchard pools
    let balance_zatoshis: i64 = match SqliteConnection::open(&db_path) {
        Ok(conn) => {
            // Query Sapling unspent notes
            let sapling_balance: i64 = conn.query_row(
                "SELECT COALESCE(SUM(srn.value), 0)
                 FROM sapling_received_notes srn
                 LEFT JOIN sapling_received_note_spends srns
                   ON srn.id = srns.sapling_received_note_id
                 WHERE srns.sapling_received_note_id IS NULL",
                [],
                |row| row.get(0),
            ).unwrap_or(0);
            tracing::info!("Sapling balance: {} zatoshis", sapling_balance);

            // Query Orchard unspent notes (if table exists)
            let orchard_balance: i64 = conn.query_row(
                "SELECT COALESCE(SUM(orn.value), 0)
                 FROM orchard_received_notes orn
                 LEFT JOIN orchard_received_note_spends orns
                   ON orn.id = orns.orchard_received_note_id
                 WHERE orns.orchard_received_note_id IS NULL",
                [],
                |row| row.get(0),
            ).unwrap_or_else(|e| {
                tracing::debug!("Orchard balance query (may not exist): {:?}", e);
                0
            });
            tracing::info!("Orchard balance: {} zatoshis", orchard_balance);

            let total = sapling_balance + orchard_balance;
            tracing::info!("Total balance: {} zatoshis (Sapling: {}, Orchard: {})",
                          total, sapling_balance, orchard_balance);
            total
        }
        Err(e) => {
            tracing::warn!("Failed to open database: {:?}", e);
            0
        }
    };

    let balance_f64 = balance_zatoshis as f64 / 100_000_000.0;
    let balance_zec = format!("{:.8}", balance_f64);

    tracing::info!("Balance: {} ZEC", balance_f64);

    // Step 6: Sync SQLite data to PostgreSQL (in background)
    let db_path_bg = db_path.clone();
    let user_id_bg = payload.user_id;
    let pg_pool_bg = state.db.clone();
    tokio::spawn(async move {
        if let Err(e) = sync_blockchain_data_to_postgres(&db_path_bg, user_id_bg, &pg_pool_bg).await {
            tracing::error!("Failed to sync blockchain data: {:?}", e);
        }
    });

    // Update sync status in PostgreSQL
    sqlx::query(
        "UPDATE wallets SET last_synced_at = NOW(), last_synced_height = $1 WHERE user_id = $2::uuid"
    )
    .bind(chain_tip as i64)
    .bind(payload.user_id.to_string())
    .execute(&state.db)
    .await?;

    Ok(Json(BalanceResponse {
        balance_zec,
        synced: true,
        last_synced_height: Some(chain_tip as i64),
        blocks_scanned: Some(scan_result.blocks_scanned),
        notes_found: Some(scan_result.notes_discovered),
        chain_tip: Some(chain_tip),
    }))
}

// Data structures for passing SQLite data across thread boundary
#[derive(Debug, Clone)]
struct TxData {
    txid: String,
    mined_height: Option<i64>,
    tx_index: Option<i32>,
    created: Option<String>,
    fee: Option<i64>,
}

#[derive(Debug, Clone)]
struct NoteData {
    txid: String,
    note_index: i32,
    value: i64,
    memo: Option<Vec<u8>>,
    is_change: bool,
    spent_tx_hex: Option<String>,
}

#[derive(Debug, Clone)]
struct SentData {
    txid: String,
    to_address: Option<String>,
    value: i64,
    memo: Option<Vec<u8>>,
}

/// Sync blockchain data from SQLite to PostgreSQL
/// This reads transactions and notes from the per-user SQLite database
/// and stores them in the centralized PostgreSQL database
async fn sync_blockchain_data_to_postgres(
    db_path: &PathBuf,
    user_id: Uuid,
    pg_pool: &PgPool,
) -> Result<()> {
    let db_path_clone = db_path.clone();

    // Step 1: Read ALL data from SQLite in a blocking task
    let (tx_data, note_data, sent_data) = tokio::task::spawn_blocking(move || -> std::result::Result<(Vec<TxData>, Vec<NoteData>, Vec<SentData>), AppError> {
        let conn = SqliteConnection::open(&db_path_clone)
            .map_err(|e| AppError::Internal(format!("Failed to open SQLite: {}", e)))?;

        // Read transactions
        let mut tx_vec = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT hex(txid), mined_height, tx_index, created, fee
             FROM transactions
             ORDER BY id_tx"
        ).map_err(|e| AppError::Internal(format!("Failed to prepare statement: {}", e)))?;

        let tx_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, Option<i32>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<i64>>(4)?,
            ))
        }).map_err(|e| AppError::Internal(format!("Failed to query transactions: {}", e)))?;

        for tx in tx_rows {
            let (txid, mined_height, tx_index, created, fee) = tx
                .map_err(|e| AppError::Internal(format!("Failed to read transaction: {}", e)))?;

            tx_vec.push(TxData {
                txid,
                mined_height,
                tx_index,
                created,
                fee,
            });
        }

        // Read received notes
        let mut note_vec = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT t.hex_txid, srn.output_index, srn.value, srn.memo, srn.is_change,
                    spent.spent_tx_hex
             FROM sapling_received_notes srn
             JOIN (SELECT id_tx, hex(txid) as hex_txid FROM transactions) t
                  ON srn.tx = t.id_tx
             LEFT JOIN (
                 SELECT sapling_received_note_id, hex(t.txid) as spent_tx_hex
                 FROM sapling_received_note_spends srns
                 JOIN transactions t ON srns.transaction_id = t.id_tx
             ) spent ON srn.id = spent.sapling_received_note_id"
        ).map_err(|e| AppError::Internal(format!("Failed to prepare notes statement: {}", e)))?;

        let note_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<Vec<u8>>>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        }).map_err(|e| AppError::Internal(format!("Failed to query notes: {}", e)))?;

        for note in note_rows {
            let (txid, note_index, value, memo, is_change, spent_tx_hex) = note
                .map_err(|e| AppError::Internal(format!("Failed to read note: {}", e)))?;

            note_vec.push(NoteData {
                txid,
                note_index,
                value,
                memo,
                is_change: is_change != 0,
                spent_tx_hex,
            });
        }

        // Read sent notes
        let mut sent_vec = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT t.hex_txid, sn.to_address, sn.value, sn.memo
             FROM sent_notes sn
             JOIN (SELECT id_tx, hex(txid) as hex_txid FROM transactions) t
                  ON sn.tx = t.id_tx"
        ).map_err(|e| AppError::Internal(format!("Failed to prepare sent notes statement: {}", e)))?;

        let sent_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<Vec<u8>>>(3)?,
            ))
        }).map_err(|e| AppError::Internal(format!("Failed to query sent notes: {}", e)))?;

        for sent in sent_rows {
            let (txid, to_address, value, memo) = sent
                .map_err(|e| AppError::Internal(format!("Failed to read sent note: {}", e)))?;

            sent_vec.push(SentData {
                txid,
                to_address,
                value,
                memo,
            });
        }

        Ok((tx_vec, note_vec, sent_vec))
    })
    .await
    .map_err(|e| AppError::Internal(format!("Failed to read SQLite data: {}", e)))??;

    // Step 2: Now insert all data into PostgreSQL (async operations are OK here)

    // Insert transactions
    for tx in tx_data {
        let created_at = tx.created.and_then(|s| chrono::DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f%#z").ok());

        sqlx::query(
            "INSERT INTO transactions (user_id, txid, block_height, tx_index, created_at, fee_zatoshis)
             VALUES ($1::uuid, $2, $3, $4, $5::timestamptz, $6)
             ON CONFLICT (user_id, txid)
             DO UPDATE SET
                block_height = EXCLUDED.block_height,
                tx_index = EXCLUDED.tx_index,
                created_at = EXCLUDED.created_at,
                fee_zatoshis = EXCLUDED.fee_zatoshis"
        )
        .bind(user_id.to_string())
        .bind(&tx.txid)
        .bind(tx.mined_height)
        .bind(tx.tx_index)
        .bind(created_at.map(|d| d.to_rfc3339()))
        .bind(tx.fee)
        .execute(pg_pool)
        .await?;
    }

    // Insert received notes
    for note in note_data {
        let tx_row = sqlx::query(
            "SELECT id FROM transactions WHERE user_id = $1::uuid AND txid = $2"
        )
        .bind(user_id.to_string())
        .bind(&note.txid)
        .fetch_optional(pg_pool)
        .await?;

        if let Some(tx) = tx_row {
            let tx_id: i64 = tx.get("id");
            let spent_in_tx_id = if let Some(spent_txid) = &note.spent_tx_hex {
                let result = sqlx::query(
                    "SELECT id FROM transactions WHERE user_id = $1::uuid AND txid = $2"
                )
                .bind(user_id.to_string())
                .bind(spent_txid)
                .fetch_optional(pg_pool)
                .await?;
                result.map(|r| r.get::<i64, _>("id"))
            } else {
                None
            };

            sqlx::query(
                "INSERT INTO received_notes (user_id, transaction_id, note_index, value_zatoshis, memo, is_change, spent_in_tx_id)
                 VALUES ($1::uuid, $2, $3, $4, $5, $6, $7)
                 ON CONFLICT (user_id, transaction_id, note_index)
                 DO UPDATE SET
                    value_zatoshis = EXCLUDED.value_zatoshis,
                    memo = EXCLUDED.memo,
                    is_change = EXCLUDED.is_change,
                    spent_in_tx_id = EXCLUDED.spent_in_tx_id"
            )
            .bind(user_id.to_string())
            .bind(tx_id)
            .bind(note.note_index)
            .bind(note.value)
            .bind(&note.memo)
            .bind(note.is_change)
            .bind(spent_in_tx_id)
            .execute(pg_pool)
            .await?;
        }
    }

    // Insert sent notes
    for sent in sent_data {
        let tx_row = sqlx::query(
            "SELECT id FROM transactions WHERE user_id = $1::uuid AND txid = $2"
        )
        .bind(user_id.to_string())
        .bind(&sent.txid)
        .fetch_optional(pg_pool)
        .await?;

        if let Some(tx) = tx_row {
            let tx_id: i64 = tx.get("id");
            let memo_str = sent.memo.as_ref().and_then(|bytes| {
                String::from_utf8(bytes.iter().filter(|&&b| b != 0).copied().collect()).ok()
            });

            sqlx::query(
                "INSERT INTO sent_notes (user_id, transaction_id, to_address, value_zatoshis, memo)
                 VALUES ($1::uuid, $2, $3, $4, $5)
                 ON CONFLICT DO NOTHING"
            )
            .bind(user_id.to_string())
            .bind(tx_id)
            .bind(sent.to_address.as_deref().unwrap_or_default())
            .bind(sent.value)
            .bind(memo_str)
            .execute(pg_pool)
            .await?;
        }
    }

    tracing::info!("Blockchain data synced to PostgreSQL successfully");
    Ok(())
}
