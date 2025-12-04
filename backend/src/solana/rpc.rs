use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Get Solana RPC URL from environment or use default
pub fn get_rpc_url() -> String {
    std::env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

/// Get SOL balance for a given address
pub async fn get_sol_balance(address: &str) -> Result<u64> {
    let rpc_url = get_rpc_url();
    let address = address.to_string();

    // Run the blocking RPC call in a separate thread
    tokio::task::spawn_blocking(move || {
        let rpc_client = RpcClient::new(rpc_url);

        // Parse the Solana address
        let pubkey = Pubkey::from_str(&address)
            .context("Invalid Solana address")?;

        // Get balance (in lamports)
        let balance = rpc_client
            .get_balance(&pubkey)
            .context("Failed to get balance from Solana RPC")?;

        Ok(balance)
    })
    .await
    .context("Failed to spawn blocking task")?
}

/// Get SOL balance in SOL (as f64) instead of lamports
pub async fn get_sol_balance_formatted(address: &str) -> Result<f64> {
    let lamports = get_sol_balance(address).await?;
    Ok(lamports as f64 / 1_000_000_000.0)
}

/// Check if an address is valid Solana address
pub fn is_valid_address(address: &str) -> bool {
    Pubkey::from_str(address).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_address() {
        // Valid Solana address
        assert!(is_valid_address("11111111111111111111111111111111"));

        // Invalid address
        assert!(!is_valid_address("invalid"));
        assert!(!is_valid_address(""));
    }

    #[test]
    fn test_balance_conversion() {
        let lamports: u64 = 1_000_000_000;
        let sol = lamports as f64 / 1_000_000_000.0;
        assert_eq!(sol, 1.0);

        let lamports: u64 = 500_000_000;
        let sol = lamports as f64 / 1_000_000_000.0;
        assert_eq!(sol, 0.5);
    }
}
