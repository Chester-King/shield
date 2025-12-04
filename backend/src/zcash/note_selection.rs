use anyhow::Result;
use rusqlite::Connection;
use zcash_client_sqlite::WalletDb;
use zcash_client_sqlite::util::SystemClock;
use zcash_protocol::consensus::Network;
use rand::rngs::OsRng;

/// Note selector for choosing which notes to spend in a transaction
pub struct NoteSelector {
    wallet_db: WalletDb<Connection, Network, SystemClock, OsRng>,
}

impl NoteSelector {
    /// Create a new note selector
    pub fn new(wallet_db: WalletDb<Connection, Network, SystemClock, OsRng>) -> Self {
        Self { wallet_db }
    }

    /// Select notes to cover the target amount plus fees
    ///
    /// Uses a greedy selection strategy:
    /// 1. Sort notes by value (largest first)
    /// 2. Select notes until we have enough to cover amount + fees
    /// 3. Calculate change if any
    pub fn select_notes(
        &self,
        target_amount: u64,
        fee: u64,
    ) -> Result<NoteSelectionResult> {
        println!("Selecting notes for transaction...");
        println!("  Target amount: {} ZAT", target_amount);

        let _total_needed = target_amount + fee;

        // TODO: Query wallet database for spendable notes using WalletRead trait
        // This requires:
        // 1. Get all unspent notes from the wallet
        // 2. Filter for notes that are confirmed (sufficient confirmations)
        // 3. Sort by value (largest first)
        // 4. Select notes greedily until we have enough

        // For POC, return empty result - notes will be populated after scanning
        let result = NoteSelectionResult {
            selected_notes: vec![],
            total_selected: 0,
            change_amount: 0,
        };

        println!("  Selected {} notes", result.selected_notes.len());
        println!("  Total: {} ZAT", result.total_selected);
        println!("  Change: {} ZAT", result.change_amount);

        Ok(result)
    }

    /// Get the total spendable balance
    pub fn get_spendable_balance(&self) -> Result<u64> {
        // TODO: Query wallet database for total spendable balance using WalletRead trait
        // This is the sum of all confirmed unspent notes
        Ok(0)
    }
}

/// Result of note selection
#[derive(Debug, Clone)]
pub struct NoteSelectionResult {
    pub selected_notes: Vec<SelectedNote>,
    pub total_selected: u64,
    pub change_amount: u64,
}

/// A selected note to spend
#[derive(Debug, Clone)]
pub struct SelectedNote {
    pub value: u64,
    pub note_id: u64,
}

#[cfg(all(test, feature = "disabled_tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use super::database::Database;

    #[test]
    fn test_note_selector_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        let database = Database::new(db_path.clone(), Network::TestNetwork);
        let wallet_db = database.init().unwrap();

        let selector = NoteSelector::new(wallet_db);

        // Test that we can create a selector
        let balance = selector.get_spendable_balance().unwrap();
        assert_eq!(balance, 0); // Empty wallet has 0 balance
    }

    #[test]
    fn test_note_selection_empty_wallet() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        let database = Database::new(db_path.clone(), Network::TestNetwork);
        let wallet_db = database.init().unwrap();

        let selector = NoteSelector::new(wallet_db);

        // Try to select notes from empty wallet
        let result = selector.select_notes(100_000, 10_000).unwrap();
        assert_eq!(result.selected_notes.len(), 0);
        assert_eq!(result.total_selected, 0);
    }
}
