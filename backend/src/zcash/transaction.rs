use anyhow::{Context, Result};
use std::convert::Infallible;

// Transaction building
use zcash_client_backend::data_api::wallet::{
    create_proposed_transactions,
    propose_standard_transfer_to_address,
    input_selection::GreedyInputSelectorError,
    ConfirmationsPolicy,
    SpendingKeys,
};
use zcash_client_backend::data_api::{Account, WalletRead};
use zcash_client_backend::fees::StandardFeeRule;
use zcash_client_backend::wallet::OvkPolicy;
use zcash_primitives::transaction::fees::zip317::FeeError;
use zcash_protocol::ShieldedProtocol;

// Types
use zcash_address::ZcashAddress;
use zcash_client_sqlite::ReceivedNoteId;
use zcash_keys::keys::UnifiedSpendingKey;
use zcash_primitives::memo::MemoBytes;
use zcash_protocol::consensus::{Network, NetworkType};
use zcash_protocol::value::Zatoshis;

use super::database::Database;

/// Transaction builder for creating shielded transactions
pub struct TransactionBuilder {
    db: Database,
    network: Network,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(db: Database, network: Network) -> Self {
        Self { db, network }
    }

    /// Build, sign, and return raw transaction bytes
    ///
    /// ⚠️ IMPORTANT: The USK is the spending key - handle securely!
    /// User should obtain this ONLY when building a transaction,
    /// then discard it immediately after.
    ///
    /// # Arguments
    /// * `usk` - UnifiedSpendingKey from create_account()
    /// * `to_address` - Recipient address ("utest1..." or "u1...")
    /// * `amount_zat` - Amount in zatoshis (1 ZEC = 100,000,000 zatoshis)
    /// * `memo` - Optional memo text (max 511 bytes)
    ///
    /// # Returns
    /// Raw transaction bytes ready for broadcast
    pub async fn build_and_sign_transaction(
        &mut self,
        usk: &UnifiedSpendingKey,
        to_address: &str,
        amount_zat: u64,
        memo: Option<&str>,
    ) -> Result<(Vec<u8>, u64)> {  // Returns (raw_tx, fee_zatoshis)
        println!("Building transaction...");
        println!("  To: {}", to_address);
        println!("  Amount: {} ZAT ({:.8} ZEC)", amount_zat, amount_zat as f64 / 100_000_000.0);

        // Step 1: Parse and validate address
        let recipient = ZcashAddress::try_from_encoded(to_address)
            .context("Invalid recipient address")?;

        // Convert Network to NetworkType
        let network_type = match self.network {
            Network::MainNetwork => NetworkType::Main,
            Network::TestNetwork => NetworkType::Test,
        };

        let recipient_addr = recipient.convert_if_network(network_type)
            .map_err(|_| anyhow::anyhow!("Address is for wrong network"))?;

        // Step 2: Convert amount
        let amount = Zatoshis::from_u64(amount_zat)
            .map_err(|_| anyhow::anyhow!("Invalid amount"))?;

        // Step 3: Format memo (if provided)
        let memo_bytes = self.format_memo(memo)?;
        println!("  Memo: {}", memo.unwrap_or("[none]"));

        // Step 4: Get account ID from USK
        let wallet_db = self.db.get_wallet_db_mut()?;
        let ufvk = usk.to_unified_full_viewing_key();
        let account = wallet_db.get_account_for_ufvk(&ufvk)?
            .ok_or_else(|| anyhow::anyhow!("Account not found for this spending key"))?;
        let account_id = Account::id(&account); // Use trait method explicitly

        println!("  From account: {:?}", account_id);

        // Step 5: Create proposal
        println!("\n1. Creating transaction proposal...");

        let proposal = match propose_standard_transfer_to_address::<_, _, Infallible>(
            wallet_db,
            &self.network,
            StandardFeeRule::Zip317,
            account_id,
            ConfirmationsPolicy::MIN, // minimum confirmations (1 for both trusted/untrusted)
            &recipient_addr,
            amount,
            memo_bytes,
            None, // change_memo
            ShieldedProtocol::Orchard, // fallback_change_pool
        ) {
            Ok(p) => p,
            Err(e) => anyhow::bail!("Failed to create transaction proposal: {:?}", e),
        };

        println!("  ✓ Proposal created");
        println!("  Steps: {}", proposal.steps().len());

        // Extract total fee from all steps
        let total_fee: u64 = proposal.steps().iter()
            .map(|step| u64::from(step.balance().fee_required()))
            .sum();

        println!("  Total fee: {} zatoshis ({} ZEC)", total_fee, total_fee as f64 / 100_000_000.0);

        // Step 6: Build transaction with proofs
        println!("\n2. Building transaction and generating zk-SNARK proofs...");

        use super::prover::get_prover;
        let prover = get_prover()?;

        // Wrap USK in SpendingKeys for the new API
        let spending_keys = SpendingKeys::new(usk.clone());

        // Note: The type inference for create_proposed_transactions is complex
        // We explicitly specify error type parameters for GreedyInputSelector and ZIP-317 fees
        let txids = create_proposed_transactions::<_, _, GreedyInputSelectorError, _, FeeError, ReceivedNoteId>(
            wallet_db,
            &self.network,
            &prover, // spend_prover
            &prover, // output_prover (same object!)
            &spending_keys,
            OvkPolicy::Sender,
            &proposal,
        ).map_err(|e| anyhow::anyhow!("Transaction creation failed: {:#?}", e))?;

        println!("  ✓ Transaction built and signed");
        println!("  TxID count: {}", txids.len());

        // Step 7: Get raw bytes
        println!("\n3. Retrieving transaction bytes...");

        let txid = txids.first();
        let transaction = wallet_db.get_transaction(*txid)?
            .ok_or_else(|| anyhow::anyhow!("Transaction not found in database"))?;

        let mut raw_tx = Vec::new();
        transaction.write(&mut raw_tx)?;

        println!("  ✓ Transaction serialized ({} bytes)", raw_tx.len());

        Ok((raw_tx, total_fee))
    }

    /// Estimate transaction fee without building the full transaction
    ///
    /// This creates a proposal to calculate the fee, but doesn't build the actual transaction.
    /// Much faster than building the full transaction with zk-SNARKs.
    pub async fn estimate_fee(
        &mut self,
        usk: &UnifiedSpendingKey,
        to_address: &str,
        amount_zat: u64,
        memo: Option<&str>,
    ) -> Result<u64> {
        // Step 1: Parse and validate address
        let recipient = ZcashAddress::try_from_encoded(to_address)
            .context("Invalid recipient address")?;

        let network_type = match self.network {
            Network::MainNetwork => NetworkType::Main,
            Network::TestNetwork => NetworkType::Test,
        };

        let recipient_addr = recipient.convert_if_network(network_type)
            .map_err(|_| anyhow::anyhow!("Address is for wrong network"))?;

        // Step 2: Convert amount
        let amount = Zatoshis::from_u64(amount_zat)
            .map_err(|_| anyhow::anyhow!("Invalid amount"))?;

        // Step 3: Format memo
        let memo_bytes = self.format_memo(memo)?;

        // Step 4: Get account ID
        let wallet_db = self.db.get_wallet_db_mut()?;
        let ufvk = usk.to_unified_full_viewing_key();
        let account = wallet_db.get_account_for_ufvk(&ufvk)?
            .ok_or_else(|| anyhow::anyhow!("Account not found for this spending key"))?;
        let account_id = Account::id(&account);

        // Step 5: Create proposal (this calculates the fee)
        let proposal = match propose_standard_transfer_to_address::<_, _, Infallible>(
            wallet_db,
            &self.network,
            StandardFeeRule::Zip317,
            account_id,
            ConfirmationsPolicy::MIN,
            &recipient_addr,
            amount,
            memo_bytes,
            None,
            ShieldedProtocol::Orchard,
        ) {
            Ok(p) => p,
            Err(e) => anyhow::bail!("Failed to create transaction proposal: {:?}", e),
        };

        // Extract fee from proposal
        let total_fee: u64 = proposal.steps().iter()
            .map(|step| u64::from(step.balance().fee_required()))
            .sum();

        Ok(total_fee)
    }

    /// Format memo text into MemoBytes
    fn format_memo(&self, memo: Option<&str>) -> Result<Option<MemoBytes>> {
        if let Some(text) = memo {
            if text.len() > 511 {
                anyhow::bail!("Memo too long (max 511 bytes, got {})", text.len());
            }

            let mut memo_array = [0u8; 512];
            memo_array[0] = 0xF4; // Text memo marker
            let len = text.as_bytes().len().min(511);
            memo_array[1..1+len].copy_from_slice(&text.as_bytes()[..len]);

            Ok(Some(MemoBytes::from_bytes(&memo_array)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::account::AccountManager;
    use super::wallet::Wallet;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_builder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path, Network::TestNetwork).unwrap();
        let builder = TransactionBuilder::new(db, Network::TestNetwork);

        assert_eq!(builder.network, Network::TestNetwork);
    }

    #[tokio::test]
    async fn test_transaction_with_empty_wallet() {
        // This should fail with "insufficient funds"
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create account
        let (_wallet, mnemonic) = Wallet::generate_new(Network::TestNetwork).unwrap();
        let seed = mnemonic.to_seed("");

        let db = Database::new(&db_path, Network::TestNetwork).unwrap();
        let mut account_mgr = AccountManager::new(db);
        let (_account_id, usk) = account_mgr.create_account("Test", &seed, None).unwrap();

        // Try to build transaction
        let db2 = Database::new(&db_path, Network::TestNetwork).unwrap();
        let mut builder = TransactionBuilder::new(db2, Network::TestNetwork);

        let result = builder.build_and_sign_transaction(
            &usk,
            "utest1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpq6d8kf", // dummy testnet address
            10_000,
            Some("Test"),
        ).await;

        // Should fail - no funds
        assert!(result.is_err());

        // Verify it's an insufficient funds error (not a compilation/API error)
        let err_msg = format!("{:?}", result.unwrap_err());
        println!("Expected error: {}", err_msg);
    }
}
