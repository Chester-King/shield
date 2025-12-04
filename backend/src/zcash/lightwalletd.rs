use anyhow::{Result, Context};
use tonic::transport::Channel;
use tonic::Streaming;
use zcash_client_backend::proto::service::compact_tx_streamer_client::CompactTxStreamerClient;
use zcash_client_backend::proto::service::{ChainSpec, BlockRange, BlockId, RawTransaction, SendResponse, TreeState};
use zcash_client_backend::proto::compact_formats::CompactBlock;

pub struct LightwalletdClient {
    endpoint: String,
    client: Option<CompactTxStreamerClient<Channel>>,
}

impl LightwalletdClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            client: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        // Check if we need TLS
        let use_tls = self.endpoint.starts_with("https://");

        let channel = if use_tls {
            // Parse the domain from the endpoint for TLS config
            let domain = self.endpoint
                .trim_start_matches("https://")
                .split(':')
                .next()
                .ok_or_else(|| anyhow::anyhow!("Invalid endpoint format"))?;

            // Configure TLS with native system roots (enabled via Cargo.toml feature)
            let tls = tonic::transport::ClientTlsConfig::new()
                .domain_name(domain)
                .with_native_roots();

            Channel::from_shared(self.endpoint.clone())?
                .tls_config(tls)?
                .connect_timeout(std::time::Duration::from_secs(30))
                .timeout(std::time::Duration::from_secs(600))  // 10 minutes for large downloads
                .connect()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", self.endpoint, e))?
        } else {
            // No TLS for local development
            Channel::from_shared(self.endpoint.clone())?
                .connect_timeout(std::time::Duration::from_secs(30))
                .timeout(std::time::Duration::from_secs(600))  // 10 minutes for large downloads
                .connect()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", self.endpoint, e))?
        };

        let client = CompactTxStreamerClient::new(channel);
        self.client = Some(client);
        Ok(())
    }

    pub async fn get_latest_block_height(&self) -> Result<u64> {
        if self.client.is_none() {
            anyhow::bail!("Not connected. Call connect() first.");
        }

        let mut client = self.client.clone().unwrap();
        let request = tonic::Request::new(ChainSpec {});

        let response = client.get_latest_block(request).await?;
        let block_id = response.into_inner();

        Ok(block_id.height)
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// Stream a range of compact blocks from the server
    ///
    /// Returns a stream of CompactBlock messages that can be iterated over
    pub async fn get_block_range(&self, start_height: u64, end_height: u64) -> Result<Streaming<CompactBlock>> {
        if self.client.is_none() {
            anyhow::bail!("Not connected. Call connect() first.");
        }

        let mut client = self.client.clone().unwrap();

        let block_range = BlockRange {
            start: Some(BlockId {
                height: start_height,
                hash: vec![],
            }),
            end: Some(BlockId {
                height: end_height,
                hash: vec![],
            }),
        };

        let request = tonic::Request::new(block_range);
        let response = client.get_block_range(request).await
            .context(format!("Failed to get block range {}-{}", start_height, end_height))?;

        Ok(response.into_inner())
    }

    /// Send a transaction to the Zcash network
    ///
    /// Broadcasts the raw transaction bytes to lightwalletd, which relays it to the network
    pub async fn send_transaction(&self, raw_tx: Vec<u8>) -> Result<SendResponse> {
        if self.client.is_none() {
            anyhow::bail!("Not connected. Call connect() first.");
        }

        let mut client = self.client.clone().unwrap();

        let raw_transaction = RawTransaction {
            data: raw_tx.clone(),
            height: 0, // Height is not required for sending
        };

        let request = tonic::Request::new(raw_transaction);
        let response = client.send_transaction(request).await
            .context("Failed to send transaction")?;

        Ok(response.into_inner())
    }

    /// Get the tree state at a specific block height
    ///
    /// Returns the Sapling and Orchard note commitment tree state at the given height.
    /// This is essential for initializing wallet scanning from a specific birthday height.
    pub async fn get_tree_state(&self, height: u64) -> Result<TreeState> {
        if self.client.is_none() {
            anyhow::bail!("Not connected. Call connect() first.");
        }

        let mut client = self.client.clone().unwrap();

        let block_id = BlockId {
            height,
            hash: vec![],
        };

        let request = tonic::Request::new(block_id);
        let response = client.get_tree_state(request).await
            .context(format!("Failed to get tree state at height {}", height))?;

        Ok(response.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LightwalletdClient::new("http://localhost:9067".to_string());
        assert!(!client.is_connected());
    }

    #[tokio::test]
    async fn test_connect_to_zec_rocks_mainnet() {
        let mut client = LightwalletdClient::new("https://na.zec.rocks:443".to_string());

        let result = client.connect().await;

        if let Err(e) = &result {
            println!("Connection error: {}", e);
        } else {
            println!("Successfully connected to zec.rocks mainnet!");
        }

        assert!(result.is_ok(), "Failed to connect to na.zec.rocks:443");
        assert!(client.is_connected());
    }

    #[tokio::test]
    async fn test_connect_to_eu_zec_rocks() {
        let mut client = LightwalletdClient::new("https://eu.zec.rocks:443".to_string());

        let result = client.connect().await;

        if let Err(e) = &result {
            println!("EU connection error: {}", e);
        } else {
            println!("Successfully connected to EU zec.rocks server!");
        }

        assert!(result.is_ok(), "Failed to connect to eu.zec.rocks:443");
        assert!(client.is_connected());
    }

    #[tokio::test]
    async fn test_connect_to_localhost() {
        let mut client = LightwalletdClient::new("http://localhost:9067".to_string());

        let result = client.connect().await;

        if let Err(e) = &result {
            println!("Local connection error: {}", e);
        } else {
            println!("Successfully connected to local lightwalletd!");
        }

        assert!(result.is_ok(), "Failed to connect to localhost:9067");
        assert!(client.is_connected());

        // Try to get block height
        let height = client.get_latest_block_height().await;
        if let Ok(h) = height {
            println!("Latest block height: {}", h);
            assert!(h > 0, "Block height should be greater than 0");
        } else {
            println!("Failed to get block height: {:?}", height);
        }
    }
}
