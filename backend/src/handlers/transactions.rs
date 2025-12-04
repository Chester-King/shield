use crate::middleware::Result;
use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionsState {
    pub db: PgPool,
}

#[derive(Serialize, Deserialize)]
pub struct GetTransactionsRequest {
    pub user_id: Uuid,
    pub page: Option<i64>,      // Page number (0-indexed)
    pub page_size: Option<i64>, // Number of items per page (default: 20, max: 100)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub txid: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub block_height: Option<i64>,
    pub amount_zec: String,
    pub direction: TransactionDirection,
    pub memo: Option<String>,
    pub fee_zec: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    Received,
    Sent,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
    pub has_more: bool,
}

/// Get transaction history for a user
/// Returns list of all transactions (sent and received) with details
#[axum::debug_handler]
pub async fn get_transactions(
    State(state): State<TransactionsState>,
    Json(payload): Json<GetTransactionsRequest>,
) -> Result<Json<TransactionsResponse>> {
    let page = payload.page.unwrap_or(0).max(0);
    let page_size = payload.page_size.unwrap_or(20).min(100).max(1);
    let offset = page * page_size;

    tracing::info!(
        "Transaction history requested for user {} (page: {}, size: {})",
        payload.user_id,
        page,
        page_size
    );

    // First, get total count
    let total_count_result = sqlx::query("SELECT COUNT(DISTINCT txid) as count FROM transactions WHERE user_id = $1::uuid")
        .bind(payload.user_id.to_string())
        .fetch_one(&state.db)
        .await?;

    let total_count = total_count_result.get::<Option<i64>, _>("count").unwrap_or(0);

    // Query to get paginated transactions with their notes
    // We need to determine direction based on whether the transaction has sent_notes
    let tx_records = sqlx::query(
        r#"
        WITH tx_summary AS (
            SELECT
                t.id,
                t.user_id,
                t.txid,
                t.created_at,
                t.block_height,
                t.fee_zatoshis,
                CAST(COALESCE(SUM(CASE WHEN rn.is_change = false AND rn.spent_in_tx_id IS NULL
                                  THEN rn.value_zatoshis ELSE 0 END), 0) AS BIGINT) as received_value,
                CAST(COALESCE(SUM(CASE WHEN sn.id IS NOT NULL
                                  THEN sn.value_zatoshis ELSE 0 END), 0) AS BIGINT) as sent_value,
                COUNT(DISTINCT sn.id) as sent_count,
                COUNT(DISTINCT CASE WHEN rn.is_change = false THEN rn.id END) as received_count
            FROM transactions t
            LEFT JOIN received_notes rn ON rn.transaction_id = t.id AND rn.user_id = t.user_id
            LEFT JOIN sent_notes sn ON sn.transaction_id = t.id AND sn.user_id = t.user_id
            WHERE t.user_id = $1::uuid
            GROUP BY t.id, t.user_id, t.txid, t.created_at, t.block_height, t.fee_zatoshis
        )
        SELECT
            ts.txid,
            ts.created_at,
            ts.block_height,
            ts.fee_zatoshis,
            ts.received_value,
            ts.sent_value,
            ts.sent_count,
            ts.received_count,
            sn.memo as sent_memo
        FROM tx_summary ts
        LEFT JOIN sent_notes sn ON sn.transaction_id = ts.id AND sn.user_id = ts.user_id
        ORDER BY ts.block_height DESC NULLS LAST, ts.created_at DESC NULLS LAST
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(payload.user_id.to_string())
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let mut transactions: Vec<Transaction> = Vec::new();
    let mut seen_txids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for record in tx_records {
        let txid: String = record.get("txid");

        // Skip if we've already processed this txid (due to multiple sent_notes)
        if seen_txids.contains(&txid) {
            continue;
        }
        seen_txids.insert(txid.clone());

        let sent_count: i64 = record.get::<Option<i64>, _>("sent_count").unwrap_or(0);
        let _received_count: i64 = record.get::<Option<i64>, _>("received_count").unwrap_or(0);
        let sent_value: i64 = record.get::<Option<i64>, _>("sent_value").unwrap_or(0);
        let received_value: i64 = record.get::<Option<i64>, _>("received_value").unwrap_or(0);

        // Determine direction:
        // - If we sent notes, it's a SENT transaction
        // - If we only received notes, it's a RECEIVED transaction
        let (direction, amount_zatoshis) = if sent_count > 0 {
            // This is a sent transaction
            // Amount is what we sent (excluding fee)
            (TransactionDirection::Sent, sent_value)
        } else {
            // This is a received transaction
            // Amount is what we received (excluding change notes)
            (TransactionDirection::Received, received_value)
        };

        let amount_zec = format!("{:.8}", amount_zatoshis as f64 / 100_000_000.0);

        let fee_zatoshis: Option<i64> = record.get("fee_zatoshis");
        let fee_zec = fee_zatoshis.map(|fee| {
            format!("{:.8}", fee as f64 / 100_000_000.0)
        });

        // Parse memo if present
        let memo: Option<String> = record.get("sent_memo");

        // Get created_at as String and parse it
        let created_at_str: Option<String> = record.get("created_at");
        let timestamp = created_at_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });

        transactions.push(Transaction {
            txid,
            timestamp,
            block_height: record.get("block_height"),
            amount_zec,
            direction,
            memo,
            fee_zec,
        });
    }

    let has_more = (offset + transactions.len() as i64) < total_count;

    tracing::info!(
        "Found {} transactions for user {} (total: {}, has_more: {})",
        transactions.len(),
        payload.user_id,
        total_count,
        has_more
    );

    Ok(Json(TransactionsResponse {
        transactions,
        total_count,
        page,
        page_size,
        has_more,
    }))
}
