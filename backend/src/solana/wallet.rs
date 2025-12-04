use anyhow::{Context, Result};
use solana_sdk::signature::{Keypair, Signer};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Create a new Solana wallet for a user
pub async fn create_solana_wallet(db: &PgPool, user_id: Uuid) -> Result<(String, Vec<u8>)> {
    // Generate new Solana keypair
    let keypair = Keypair::new();

    // Get public key (Solana address)
    let public_key = keypair.pubkey().to_string();

    // Get keypair bytes (64 bytes: 32-byte secret key + 32-byte public key)
    let keypair_bytes = keypair.to_bytes().to_vec();

    // Store in database (unencrypted for now - encryption will be added later)
    sqlx::query(
        r#"
        INSERT INTO solana_wallets (user_id, encrypted_keypair, public_key)
        VALUES ($1::uuid, $2, $3)
        ON CONFLICT (user_id) DO UPDATE
        SET encrypted_keypair = EXCLUDED.encrypted_keypair,
            public_key = EXCLUDED.public_key,
            updated_at = NOW()
        "#
    )
    .bind(user_id.to_string())
    .bind(keypair_bytes.clone())
    .bind(public_key.clone())
    .execute(db)
    .await
    .context("Failed to insert Solana wallet into database")?;

    tracing::info!("Created Solana wallet for user {}: {}", user_id, public_key);

    Ok((public_key, keypair_bytes))
}

/// Get Solana wallet for a user
pub async fn get_solana_wallet(db: &PgPool, user_id: Uuid) -> Result<Option<(String, Vec<u8>)>> {
    let wallet = sqlx::query(
        r#"
        SELECT public_key, encrypted_keypair
        FROM solana_wallets
        WHERE user_id = $1::uuid
        "#
    )
    .bind(user_id.to_string())
    .fetch_optional(db)
    .await
    .context("Failed to fetch Solana wallet from database")?;

    Ok(wallet.map(|row| (row.get("public_key"), row.get("encrypted_keypair"))))
}

/// Load keypair from bytes
pub fn keypair_from_bytes(bytes: &[u8]) -> Result<Keypair> {
    let keypair = Keypair::from_bytes(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize keypair: {:?}", e))?;
    Ok(keypair)
}

/// Get public key from wallet without loading full keypair
pub async fn get_public_key(db: &PgPool, user_id: Uuid) -> Result<Option<String>> {
    let result = sqlx::query(
        r#"
        SELECT public_key
        FROM solana_wallets
        WHERE user_id = $1::uuid
        "#
    )
    .bind(user_id.to_string())
    .fetch_optional(db)
    .await
    .context("Failed to fetch Solana public key")?;

    Ok(result.map(|row| row.get("public_key")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = Keypair::new();
        let bytes = keypair.to_bytes();

        // Verify we can reconstruct the keypair
        let restored = Keypair::from_bytes(&bytes).unwrap();
        assert_eq!(keypair.pubkey(), restored.pubkey());
    }

    #[test]
    fn test_keypair_serialization() {
        let keypair = Keypair::new();
        let bytes = keypair.to_bytes().to_vec();

        // Verify bytes are correct length (64 bytes)
        assert_eq!(bytes.len(), 64);

        // Verify we can deserialize
        let restored = keypair_from_bytes(&bytes).unwrap();
        assert_eq!(keypair.pubkey(), restored.pubkey());
    }
}
