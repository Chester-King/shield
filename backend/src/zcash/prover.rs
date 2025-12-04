use anyhow::Result;
use zcash_proofs::prover::LocalTxProver;
use std::path::PathBuf;

use super::params::ensure_params;

/// Prover for generating zk-SNARK proofs for transactions
pub struct TransactionProver {
    prover: LocalTxProver,
}

impl TransactionProver {
    /// Create a new transaction prover with the given proving parameters
    pub fn new(params_dir: PathBuf) -> Result<Self> {
        println!("Initializing transaction prover...");
        println!("  Params directory: {}", params_dir.display());

        // Load the proving parameters
        let spend_path = params_dir.join("sapling-spend.params");
        let output_path = params_dir.join("sapling-output.params");

        let prover = LocalTxProver::new(&spend_path, &output_path);

        println!("✓ Prover initialized");

        Ok(Self { prover })
    }

    /// Get the underlying LocalTxProver
    ///
    /// This is used internally for proof generation during transaction building.
    pub fn get_local_prover(&self) -> &LocalTxProver {
        &self.prover
    }
}

/// Get a LocalTxProver for transaction building
///
/// This is a simple helper function that creates a prover using the
/// standard proving parameters location.
pub fn get_prover() -> Result<LocalTxProver> {
    let params_dir = ensure_params()?;
    let spend_path = params_dir.join("sapling-spend.params");
    let output_path = params_dir.join("sapling-output.params");

    Ok(LocalTxProver::new(&spend_path, &output_path))
}

#[cfg(all(test, feature = "disabled_tests"))]
mod tests {
    use super::*;
    use super::params::ensure_params;

    #[test]
    fn test_prover_initialization() {
        // Download/verify params if needed
        let params_dir = ensure_params().expect("Failed to ensure params");

        // Create prover
        let result = TransactionProver::new(params_dir);
        assert!(result.is_ok(), "Failed to initialize prover");
    }
}
