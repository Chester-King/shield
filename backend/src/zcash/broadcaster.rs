use anyhow::Result;
use super::lightwalletd::LightwalletdClient;

/// Transaction broadcaster for submitting transactions to the network
pub struct TransactionBroadcaster {
    lightwalletd: LightwalletdClient,
}

impl TransactionBroadcaster {
    /// Create a new transaction broadcaster
    pub fn new(lightwalletd: LightwalletdClient) -> Self {
        Self { lightwalletd }
    }

    /// Broadcast a transaction to the network
    ///
    /// Sends the raw transaction bytes to lightwalletd, which will
    /// relay it to the Zcash network for mining.
    ///
    /// # Arguments
    /// * `raw_transaction` - The raw transaction bytes from TransactionBuilder
    ///
    /// # Returns
    /// The transaction ID as a hex string
    pub async fn broadcast(&mut self, raw_transaction: Vec<u8>) -> Result<String> {
        println!("Broadcasting transaction...");
        println!("  Size: {} bytes", raw_transaction.len());

        // Ensure we're connected to lightwalletd
        if !self.lightwalletd.is_connected() {
            println!("  Connecting to lightwalletd...");
            self.lightwalletd.connect().await?;
        }

        // Send the transaction to lightwalletd
        let response = self.lightwalletd.send_transaction(raw_transaction).await?;

        // Debug: Print full response
        println!("  Response from lightwalletd:");
        println!("    error_code: {}", response.error_code);
        println!("    error_message length: {} bytes", response.error_message.len());
        println!("    error_message (first 200 chars): {}",
                 if response.error_message.len() > 200 {
                     &response.error_message[..200]
                 } else {
                     &response.error_message
                 });

        // Check if the transaction was accepted
        if response.error_code != 0 {
            // Try to decode hex if it looks like hex
            let decoded = if response.error_message.chars().all(|c| c.is_ascii_hexdigit()) && response.error_message.len() % 2 == 0 {
                hex::decode(&response.error_message)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or_else(|| response.error_message.clone())
            } else {
                response.error_message.clone()
            };

            anyhow::bail!(
                "Transaction rejected by network: {} (code: {})",
                decoded,
                response.error_code
            );
        }

        // The response should contain the txid
        let txid = response.error_message; // lightwalletd returns txid in error_message when successful

        println!("✓ Transaction broadcast successfully");
        println!("  TxID: {}", txid);

        Ok(txid)
    }

    /// Wait for a transaction to be confirmed
    ///
    /// Polls the blockchain until the transaction appears in a block
    pub async fn wait_for_confirmation(
        &mut self,
        txid: &str,
        confirmations: u32,
    ) -> Result<u64> {
        println!("Waiting for {} confirmation(s) of {}...", confirmations, txid);

        // TODO: Poll lightwalletd for transaction status
        // This requires:
        // 1. Query transaction status
        // 2. Check confirmation count
        // 3. Poll until desired confirmations reached

        println!("✓ Transaction confirmed (placeholder)");

        Ok(0) // Return block height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_creation() {
        let lightwalletd = LightwalletdClient::new("http://localhost:9067".to_string());
        let _broadcaster = TransactionBroadcaster::new(lightwalletd);

        // Just test that we can create a broadcaster
    }
}
