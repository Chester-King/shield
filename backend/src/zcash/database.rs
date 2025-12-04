use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use zcash_client_sqlite::WalletDb;
use zcash_client_sqlite::util::SystemClock;
use zcash_client_sqlite::wallet::init::init_wallet_db;
use rand::rngs::OsRng;
use rusqlite::Connection;

// Use Network from zcash_protocol v0.5 - the same version that zcash_client_sqlite uses
pub use zcash_protocol::consensus::Network;

/// Wallet database manager
pub struct Database {
    db_path: PathBuf,
    network: Network,
    wallet_db: Option<WalletDb<Connection, Network, SystemClock, OsRng>>,
}

impl Database {
    /// Create a new database manager and initialize it
    pub fn new(db_path: impl AsRef<Path>, network: Network) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        // Check if database already exists and is initialized
        let db_exists = db_path.exists();

        // Create WalletDb with all 4 type parameters
        let mut wallet_db = WalletDb::for_path(
            &db_path,
            network.clone(),
            SystemClock,
            OsRng,
        ).context("Failed to open database")?;

        // Only initialize the database schema if it's a new database
        if !db_exists {
            init_wallet_db(&mut wallet_db, None)
                .context("Failed to initialize database schema")?;
        } else {
            // For existing databases, still run init to handle any pending migrations
            // but catch errors silently if the schema is already up to date
            if let Err(e) = init_wallet_db(&mut wallet_db, None) {
                // Log but don't fail if migrations already applied
                tracing::debug!("Database init (possibly already initialized): {:?}", e);
            }
        }

        Ok(Self {
            db_path,
            network,
            wallet_db: Some(wallet_db),
        })
    }

    /// Open an existing database without running migrations
    /// Use this for read operations on databases that are already initialized
    pub fn open_existing(db_path: impl AsRef<Path>, network: Network) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        if !db_path.exists() {
            anyhow::bail!("Database does not exist at {:?}", db_path);
        }

        // Open WalletDb without running migrations
        let wallet_db = WalletDb::for_path(
            &db_path,
            network.clone(),
            SystemClock,
            OsRng,
        ).context("Failed to open existing database")?;

        Ok(Self {
            db_path,
            network,
            wallet_db: Some(wallet_db),
        })
    }

    /// Initialize or open the wallet database (deprecated - use new())
    pub fn init(&self) -> Result<WalletDb<Connection, Network, SystemClock, OsRng>> {
        println!("Initializing wallet database...");
        println!("  Path: {}", self.db_path.display());
        println!("  Network: {}", match self.network {
            Network::MainNetwork => "mainnet",
            Network::TestNetwork => "testnet",
        });

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create database directory")?;
        }

        // Create WalletDb with all 4 type parameters
        let mut db = WalletDb::for_path(
            &self.db_path,
            self.network.clone(),
            SystemClock,
            OsRng,
        ).context("Failed to open database")?;

        // Initialize the database schema
        // Note: We pass None for seed here since we'll add accounts separately
        init_wallet_db(&mut db, None)
            .context("Failed to initialize database schema")?;

        println!("✓ Database initialized successfully");

        Ok(db)
    }

    /// Get the database path
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Get the network
    pub fn network(&self) -> Network {
        self.network.clone()
    }

    /// Get a reference to the wallet database
    pub fn get_wallet_db(&self) -> Result<&WalletDb<Connection, Network, SystemClock, OsRng>> {
        self.wallet_db.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
    }

    /// Get a mutable reference to the wallet database
    pub fn get_wallet_db_mut(&mut self) -> Result<&mut WalletDb<Connection, Network, SystemClock, OsRng>> {
        self.wallet_db.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))
    }
}

/// Get the default database directory for Shield wallets
pub fn default_db_dir() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?
        .join("Shield");

    Ok(dir)
}

/// Get the default database path for a given network
pub fn default_db_path(network: &Network) -> Result<PathBuf> {
    let dir = default_db_dir()?;
    let filename = match network {
        Network::MainNetwork => "wallet_mainnet.db",
        Network::TestNetwork => "wallet_testnet.db",
    };

    Ok(dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_init_testnet() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        let result = Database::new(&db_path, Network::TestNetwork);

        if let Err(e) = &result {
            panic!("Failed to initialize database: {}", e);
        }
        assert!(result.is_ok());
        assert!(db_path.exists(), "Database file was not created");
    }

    #[test]
    fn test_database_init_mainnet() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        let result = Database::new(&db_path, Network::MainNetwork);

        if let Err(e) = &result {
            panic!("Failed to initialize database: {}", e);
        }
        assert!(result.is_ok());
        assert!(db_path.exists(), "Database file was not created");
    }

    #[test]
    fn test_default_paths() {
        let testnet_path = default_db_path(&Network::TestNetwork).unwrap();
        let mainnet_path = default_db_path(&Network::MainNetwork).unwrap();

        assert!(testnet_path.to_string_lossy().contains("wallet_testnet.db"));
        assert!(mainnet_path.to_string_lossy().contains("wallet_mainnet.db"));
        assert_ne!(testnet_path, mainnet_path, "Testnet and mainnet should use different files");
    }
}
