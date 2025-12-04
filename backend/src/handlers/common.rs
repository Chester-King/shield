use crate::middleware::{AppError, Result};
use crate::zcash::{database, lightwalletd};
use bip39::Mnemonic;
use sqlx::{PgPool, Row};
use std::env;
use std::path::PathBuf;
use uuid::Uuid;
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_protocol::consensus::Network;
use zip32::AccountId;

/// Conversion constant: 1 ZEC = 100,000,000 zatoshis
pub const ZATOSHIS_PER_ZEC: f64 = 100_000_000.0;

/// Wallet configuration loaded from PostgreSQL
pub struct WalletConfig {
    pub mnemonic: Mnemonic,
    pub seed: Vec<u8>,
    pub birthday_height: u32,
    pub address: Option<String>,
    pub network: Network,
    pub db_path: PathBuf,
}

/// Load wallet configuration from PostgreSQL
pub async fn load_wallet_config(
    db: &PgPool,
    user_id: Uuid,
    include_address: bool,
) -> Result<WalletConfig> {
    // Get wallet info from PostgreSQL - use string cast for UUID since sqlx uuid feature disabled
    let (encrypted_mnemonic, birthday_height, address) = if include_address {
        let row = sqlx::query(
            "SELECT encrypted_mnemonic, birthday_height, address FROM wallets WHERE user_id = $1::uuid"
        )
        .bind(user_id.to_string())
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Wallet not found".to_string()))?;

        let encrypted_mnemonic: String = row.get("encrypted_mnemonic");
        let birthday_height: i64 = row.get("birthday_height");
        let address: String = row.get("address");
        (encrypted_mnemonic, birthday_height, Some(address))
    } else {
        let row = sqlx::query(
            "SELECT encrypted_mnemonic, birthday_height FROM wallets WHERE user_id = $1::uuid"
        )
        .bind(user_id.to_string())
        .fetch_optional(db)
        .await?
        .ok_or_else(|| AppError::NotFound("Wallet not found".to_string()))?;

        let encrypted_mnemonic: String = row.get("encrypted_mnemonic");
        let birthday_height: i64 = row.get("birthday_height");
        (encrypted_mnemonic, birthday_height, None)
    };

    // Parse mnemonic
    let mnemonic = Mnemonic::parse(&encrypted_mnemonic)
        .map_err(|e| AppError::Internal(format!("Failed to parse mnemonic: {}", e)))?;

    let seed = mnemonic.to_seed("");
    let birthday_height_u32 = birthday_height as u32;

    // Get network from environment
    let network = get_network();

    // Setup per-user wallet database path
    let data_dir = PathBuf::from("./wallet_data");
    std::fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join(format!("wallet_{}.db", user_id));

    Ok(WalletConfig {
        mnemonic,
        seed: seed.to_vec(),
        birthday_height: birthday_height_u32,
        address,
        network,
        db_path,
    })
}

/// Get network configuration from environment
pub fn get_network() -> Network {
    let network_str = env::var("ZCASH_NETWORK").unwrap_or_else(|_| "mainnet".to_string());
    match network_str.to_lowercase().as_str() {
        "testnet" => Network::TestNetwork,
        _ => Network::MainNetwork,
    }
}

/// Get lightwalletd URL for the given network
pub fn get_lightwalletd_url(network: Network) -> String {
    match network {
        Network::MainNetwork => {
            env::var("LIGHTWALLETD_MAINNET")
                .unwrap_or_else(|_| "https://na.zec.rocks:443".to_string())
        }
        Network::TestNetwork => {
            env::var("LIGHTWALLETD_TESTNET")
                .unwrap_or_else(|_| "https://testnet.zec.rocks:443".to_string())
        }
    }
}

/// Connect to lightwalletd server
pub async fn connect_lightwalletd(network: Network) -> Result<lightwalletd::LightwalletdClient> {
    let url = get_lightwalletd_url(network);
    tracing::info!("Connecting to lightwalletd: {}", url);

    let mut client = lightwalletd::LightwalletdClient::new(url);
    client
        .connect()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to connect to lightwalletd: {}", e)))?;

    tracing::info!("Connected to lightwalletd");
    Ok(client)
}

/// Open or create wallet database
pub fn open_wallet_database(
    db_path: &std::path::Path,
    network: Network,
) -> Result<database::Database> {
    database::Database::new(db_path, network)
        .map_err(|e| AppError::Internal(format!("Failed to open database: {}", e)))
}

/// Derive unified spending key from seed
pub fn derive_spending_key(seed: &[u8], network: Network) -> Result<UnifiedSpendingKey> {
    UnifiedSpendingKey::from_seed(&network, seed, AccountId::try_from(0).unwrap())
        .map_err(|e| AppError::Internal(format!("Failed to derive key: {:?}", e)))
}

/// Convert ZEC to zatoshis
pub fn zec_to_zatoshis(zec: f64) -> u64 {
    (zec * ZATOSHIS_PER_ZEC) as u64
}

/// Convert zatoshis to ZEC
pub fn zatoshis_to_zec(zatoshis: u64) -> f64 {
    zatoshis as f64 / ZATOSHIS_PER_ZEC
}

/// Get block explorer URL for a transaction
pub fn get_explorer_url(network: Network, txid: &str) -> String {
    match network {
        Network::MainNetwork => {
            format!("https://mainnet.zcashexplorer.app/transactions/{}", txid)
        }
        Network::TestNetwork => {
            format!("https://testnet.zcashexplorer.app/transactions/{}", txid)
        }
    }
}
