use sqlx::{PgPool, Row};
use bip39::Mnemonic;
use zcash_keys::keys::{UnifiedSpendingKey, UnifiedAddressRequest, ReceiverRequirement};
use zip32::AccountId;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url).await?;

    // Fetch all wallets without transparent addresses
    // Use raw query and manually parse results since sqlx doesn't have uuid feature
    let rows = sqlx::query(
        "SELECT id::text, encrypted_mnemonic FROM wallets WHERE transparent_address IS NULL"
    )
    .fetch_all(&pool)
    .await?;

    println!("Found {} wallets without transparent addresses", rows.len());

    for row in rows {
        let id_str: String = row.get(0);
        let encrypted_mnemonic: String = row.get(1);
        let wallet_id = Uuid::parse_str(&id_str)?;

        println!("Processing wallet {}...", wallet_id);

        // Parse mnemonic
        let mnemonic = Mnemonic::parse(&encrypted_mnemonic)
            .map_err(|e| format!("Failed to parse mnemonic: {:?}", e))?;

        // Derive wallet from mnemonic (same logic as in wallet.rs)
        let seed = mnemonic.to_seed("");
        let account_id = AccountId::try_from(0)
            .map_err(|e| format!("Invalid account ID: {:?}", e))?;

        let spending_key = UnifiedSpendingKey::from_seed(
            &zcash_protocol::consensus::MainNetwork,
            &seed[..],
            account_id,
        )
        .map_err(|e| format!("Failed to derive spending key: {:?}", e))?;

        // Generate unified address that includes sapling and transparent
        let ufvk = spending_key.to_unified_full_viewing_key();

        // Debug: Check if sapling key exists
        if ufvk.sapling().is_none() {
            eprintln!("WARNING: Wallet has no Sapling key!");
        }

        let request = UnifiedAddressRequest::custom(
            ReceiverRequirement::Allow,   // Orchard: Allow (include if available)
            ReceiverRequirement::Require, // Sapling: Require
            ReceiverRequirement::Require  // Transparent: Require
        )
        .map_err(|e| format!("Failed to create address request: {:?}", e))?;
        let (ua, _diversifier_index) = ufvk.default_address(request)
            .map_err(|e| format!("Failed to generate address: {:?}", e))?;

        // Encode as unified address (will contain sapling + transparent receivers)
        let transparent_address = ua.encode(&zcash_protocol::consensus::MainNetwork);

        // Update database - use id::uuid cast since we're binding as string
        sqlx::query(
            "UPDATE wallets SET transparent_address = $1 WHERE id = $2::uuid"
        )
        .bind(&transparent_address)
        .bind(&id_str)
        .execute(&pool)
        .await?;

        println!("  ✓ Updated wallet {} with transparent address: {}", wallet_id, transparent_address);
    }

    println!("\nMigration complete!");

    Ok(())
}
