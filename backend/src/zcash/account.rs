use anyhow::{Result, Context};
use rusqlite::Connection;
use secrecy::SecretVec;
use zcash_client_backend::data_api::{AccountBirthday, WalletRead, WalletWrite};
use zcash_client_sqlite::{AccountUuid, wallet::Account};
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_protocol::consensus::{Network, Parameters};

use super::database::Database;
use super::lightwalletd::LightwalletdClient;

/// Account manager for creating and managing Zcash accounts
pub struct AccountManager {
    db: Database,
}

impl AccountManager {
    /// Create a new account manager
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Initialize a new account from a seed phrase
    ///
    /// # Arguments
    /// * `account_name` - Human-readable name for the account
    /// * `seed` - BIP39 seed bytes (typically 32 bytes from mnemonic)
    /// * `lightwalletd` - Client for fetching tree state from lightwalletd
    /// * `birthday_height` - Block height to start scanning from (None = Sapling activation)
    ///
    /// # Returns
    /// * AccountUuid and the derived UnifiedSpendingKey
    pub async fn create_account(
        &mut self,
        account_name: &str,
        seed: &[u8],
        lightwalletd: &LightwalletdClient,
        birthday_height: Option<u32>,
    ) -> Result<(AccountUuid, UnifiedSpendingKey)> {
        let network = self.db.network();

        // Create AccountBirthday with proper tree state
        //
        // PRODUCTION NOTE: We fetch tree state at birthday_height - 1 (not birthday_height itself).
        // This is the correct way per zcash-devtool and Discord guidance.
        //
        // Key insight: "You need the tree state as of the end of the block prior to the
        // birthday height, so h - 1" - from Zcash Discord
        let effective_birthday = birthday_height.unwrap_or_else(|| {
            // Use Sapling activation height if no birthday specified
            u32::from(network.activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling).unwrap())
        });

        println!("  Creating account with birthday height: {}", effective_birthday);

        // Fetch tree state from the block BEFORE the birthday height
        // This is critical - we need tree state at (height - 1), not (height)
        let tree_state_height = effective_birthday.saturating_sub(1);
        println!("  Fetching tree state at height: {} (birthday - 1)", tree_state_height);

        let tree_state = lightwalletd.get_tree_state(tree_state_height as u64).await
            .context(format!("Failed to fetch tree state at height {}", tree_state_height))?;

        println!("  ✓ Fetched tree state from lightwalletd");

        // Create birthday from tree state
        let birthday = AccountBirthday::from_treestate(tree_state, None)
            .map_err(|_| anyhow::anyhow!("Failed to create birthday from tree state"))?;

        // Wrap seed in SecretVec for secure handling
        let seed_secret = SecretVec::new(seed.to_vec());

        // WORKAROUND: Clear all existing checkpoints BEFORE creating account
        // This prevents checkpoint conflicts where schema-initialized empty checkpoints
        // conflict with the tree state that will be inserted by create_account.
        // The create_account call will properly set up checkpoints with the birthday tree state.
        let db_path = self.db.path();
        if let Ok(conn) = Connection::open(db_path) {
            let _ = conn.execute("DELETE FROM sapling_tree_checkpoints", []);
            let _ = conn.execute("DELETE FROM sapling_tree_checkpoint_marks_removed", []);
            let _ = conn.execute("DELETE FROM orchard_tree_checkpoints", []);
            let _ = conn.execute("DELETE FROM orchard_tree_checkpoint_marks_removed", []);
            println!("  ✓ Cleared existing checkpoints before account creation");
        }

        // Get mutable database handle
        let wallet_db = self.db.get_wallet_db_mut()?;

        // Create account using WalletWrite trait
        let (account_id, usk) = wallet_db.create_account(
            account_name,
            &seed_secret,
            &birthday,
            None, // key_source (optional metadata)
        )?;

        println!("  ✓ Account created with proper tree state initialization");

        Ok((account_id, usk))
    }

    /// Import an existing account by HD derivation path
    ///
    /// Similar to create_account but allows importing accounts that were
    /// created elsewhere with the same seed
    pub async fn import_account_hd(
        &mut self,
        account_name: &str,
        seed: &[u8],
        lightwalletd: &LightwalletdClient,
        account_index: u32,
        birthday_height: Option<u32>,
    ) -> Result<(Account, UnifiedSpendingKey)> {
        let network = self.db.network();

        // Create AccountBirthday with proper tree state (same logic as create_account)
        let effective_birthday = birthday_height.unwrap_or_else(|| {
            u32::from(network.activation_height(zcash_protocol::consensus::NetworkUpgrade::Sapling).unwrap())
        });

        println!("  Importing account with birthday height: {}", effective_birthday);

        let tree_state_height = effective_birthday.saturating_sub(1);
        println!("  Fetching tree state at height: {} (birthday - 1)", tree_state_height);

        let tree_state = lightwalletd.get_tree_state(tree_state_height as u64).await
            .context(format!("Failed to fetch tree state at height {}", tree_state_height))?;

        println!("  ✓ Fetched tree state from lightwalletd");

        let birthday = AccountBirthday::from_treestate(tree_state, None)
            .map_err(|_| anyhow::anyhow!("Failed to create birthday from tree state"))?;

        let seed_secret = SecretVec::new(seed.to_vec());
        let wallet_db = self.db.get_wallet_db_mut()?;

        // Import with specific account index
        let (account_id, usk) = wallet_db.import_account_hd(
            account_name,
            &seed_secret,
            zip32::AccountId::try_from(account_index)
                .map_err(|_| anyhow::anyhow!("Invalid account index"))?,
            &birthday,
            None,
        )?;

        println!("  ✓ Account imported with proper tree state initialization");

        Ok((account_id, usk))
    }

    /// List all account IDs in the database
    pub fn list_account_ids(&self) -> Result<Vec<AccountUuid>> {
        let wallet_db = self.db.get_wallet_db()?;
        let account_ids = wallet_db.get_account_ids()?;
        Ok(account_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::wallet::Wallet;
    use tempfile::TempDir;

    #[test]
    fn test_create_account() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        // Create a wallet with mnemonic
        let (_wallet, mnemonic) = Wallet::generate_new(Network::TestNetwork).unwrap();
        let seed = mnemonic.to_seed("");

        // Initialize database
        let db = Database::new(&db_path, Network::TestNetwork).unwrap();
        let mut account_mgr = AccountManager::new(db);

        // Create account
        let result = account_mgr.create_account("Test Account", &seed, None);
        assert!(result.is_ok());

        let (account_id, _usk) = result.unwrap();
        println!("Created account: {:?}", account_id);
        println!("USK derived successfully");

        // Verify account is in database
        let accounts = account_mgr.list_account_ids().unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[test]
    fn test_import_account_hd() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_import.db");

        let (_wallet, mnemonic) = Wallet::generate_new(Network::TestNetwork).unwrap();
        let seed = mnemonic.to_seed("");

        let db = Database::new(&db_path, Network::TestNetwork).unwrap();
        let mut account_mgr = AccountManager::new(db);

        // Import account with specific index
        let result = account_mgr.import_account_hd("Imported Account", &seed, 0, None);
        assert!(result.is_ok());

        let accounts = account_mgr.list_account_ids().unwrap();
        assert_eq!(accounts.len(), 1);
    }
}
