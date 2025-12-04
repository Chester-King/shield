use crate::{
    middleware::{AppError, Result},
    models::user::{User, UserResponse},
};
use axum::{extract::Extension, Json};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Helper to parse DateTime string from database
fn parse_datetime(s: &str) -> std::result::Result<DateTime<Utc>, sqlx::Error> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            chrono::DateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S%.f%#z")
                .map(|dt| dt.with_timezone(&Utc))
        })
        .map_err(|e| sqlx::Error::Decode(Box::new(e)))
}

/// Helper to parse User from a database row (since sqlx uuid/chrono features are disabled)
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
        auth_method: crate::models::user::AuthMethod::from_str(&auth_method_str),
        created_at: parse_datetime(&created_at_str)?,
        updated_at: parse_datetime(&updated_at_str)?,
    })
}

pub async fn get_me(
    Extension(user_id): Extension<Uuid>,
    Extension(db): Extension<PgPool>,
) -> Result<Json<UserResponse>> {
    let user_row = sqlx::query(
        "SELECT id::text, email, password_hash, full_name, email_verified, auth_method::text, created_at::text, updated_at::text
         FROM users WHERE id = $1::uuid"
    )
        .bind(user_id.to_string())
        .fetch_optional(&db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let user = user_from_row(&user_row)?;

    // Fetch wallet address for this user
    let wallet_data = sqlx::query("SELECT address FROM wallets WHERE user_id = $1::uuid")
        .bind(user_id.to_string())
        .fetch_optional(&db)
        .await?;

    let wallet_address = wallet_data.map(|row| row.get("address"));

    // Fetch Solana wallet address
    let solana_data = sqlx::query("SELECT public_key FROM solana_wallets WHERE user_id = $1::uuid")
        .bind(user_id.to_string())
        .fetch_optional(&db)
        .await?;

    let solana_address = solana_data.map(|row| row.get("public_key"));

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        email_verified: user.email_verified,
        created_at: user.created_at,
        wallet_address,
        solana_address,
    }))
}
