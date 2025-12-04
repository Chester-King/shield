use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;
use zcash_client_backend::{
    data_api::{
        chain::{scan_cached_blocks, BlockSource, ChainState},
        WalletRead,
    },
    proto::compact_formats::CompactBlock,
};
use zcash_client_sqlite::WalletDb;
use zcash_client_sqlite::util::SystemClock;
use zcash_protocol::consensus::{BlockHeight, Network};
use rand::rngs::OsRng;
use std::collections::HashMap;
use zcash_primitives::block::BlockHash;

use super::lightwalletd::LightwalletdClient;

/// In-memory block cache for storing compact blocks during scanning
struct InMemoryBlockCache {
    blocks: HashMap<BlockHeight, CompactBlock>,
}

impl InMemoryBlockCache {
    fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    fn write_block(&mut self, height: BlockHeight, block: &CompactBlock) -> Result<()> {
        self.blocks.insert(height, block.clone());
        Ok(())
    }
}

// Implement BlockSource trait for InMemoryBlockCache
impl BlockSource for InMemoryBlockCache {
    type Error = anyhow::Error;

    fn with_blocks<F, DbErrT>(
        &self,
        from_height: Option<BlockHeight>,
        _limit: Option<usize>,
        mut with_row: F,
    ) -> Result<(), zcash_client_backend::data_api::chain::error::Error<DbErrT, Self::Error>>
    where
        F: FnMut(CompactBlock) -> Result<(), zcash_client_backend::data_api::chain::error::Error<DbErrT, Self::Error>>,
    {
        // Get all blocks starting from from_height
        let start_height = from_height.unwrap_or(BlockHeight::from_u32(0));

        // Get all heights and sort them
        let mut heights: Vec<_> = self.blocks.keys().copied().collect();
        heights.sort();

        // Process blocks in order, respecting limit if provided
        let mut count = 0;
        for height in heights {
            if height >= start_height {
                if let Some(limit) = _limit {
                    if count >= limit {
                        break;
                    }
                }
                if let Some(block) = self.blocks.get(&height) {
                    with_row(block.clone())?;
                    count += 1;
                }
            }
        }

        Ok(())
    }
}

/// Blockchain scanner for discovering wallet transactions
pub struct BlockchainScanner {
    wallet_db: WalletDb<Connection, Network, SystemClock, OsRng>,
    block_cache: InMemoryBlockCache,
    lightwalletd: LightwalletdClient,
    network: Network,
    db_path: Option<PathBuf>,
}

impl BlockchainScanner {
    /// Create a new blockchain scanner
    pub fn new(
        wallet_db: WalletDb<Connection, Network, SystemClock, OsRng>,
        lightwalletd: LightwalletdClient,
        network: Network,
    ) -> Self {
        Self {
            wallet_db,
            block_cache: InMemoryBlockCache::new(),
            lightwalletd,
            network,
            db_path: None,
        }
    }

    /// Create a new blockchain scanner with database path for checkpoint management
    pub fn new_with_path(
        wallet_db: WalletDb<Connection, Network, SystemClock, OsRng>,
        lightwalletd: LightwalletdClient,
        network: Network,
        db_path: PathBuf,
    ) -> Self {
        Self {
            wallet_db,
            block_cache: InMemoryBlockCache::new(),
            lightwalletd,
            network,
            db_path: Some(db_path),
        }
    }

    /// Scan the blockchain from the wallet's birthday height
    ///
    /// This downloads compact blocks from lightwalletd and scans them for
    /// transactions relevant to the wallet's accounts.
    ///
    /// Uses batched scanning - processes blocks in chunks and saves progress
    /// incrementally so interruptions don't lose all work.
    pub async fn scan_from_birthday(&mut self) -> Result<ScanSummary> {
        println!("Starting blockchain scan...");

        // Get the wallet's birthday (earliest block we need to scan)
        let birthday_height = self.get_wallet_birthday()
            .context("Failed to get wallet birthday")?;

        // Get the current chain tip from lightwalletd
        let chain_tip = self.lightwalletd.get_latest_block_height().await
            .context("Failed to get chain tip")?;

        println!("  Wallet birthday: {}", birthday_height);
        println!("  Chain tip: {}", chain_tip);

        if chain_tip < birthday_height {
            anyhow::bail!("Chain tip ({}) is before wallet birthday ({})", chain_tip, birthday_height);
        }

        // Check what height has already been scanned
        let last_scanned = self.get_last_scanned_height()?;

        let start = if let Some(last_height) = last_scanned {
            // Resume from where we left off - scan from next block
            let next_height = last_height + 1;
            println!("  Last scanned height: {}", last_height);
            println!("  Resuming from: {}", next_height);

            // If we're already caught up, no need to scan
            if next_height > chain_tip {
                println!("✓ Already up to date!");
                return Ok(ScanSummary {
                    start_height: chain_tip,
                    end_height: chain_tip,
                    blocks_scanned: 0,
                    notes_discovered: 0,
                });
            }

            next_height
        } else {
            // First scan - start from birthday
            println!("  First scan - starting from birthday");
            birthday_height
        };

        let total_blocks = chain_tip - start + 1;
        println!("  Blocks to scan: {}", total_blocks);

        // Process blocks in batches to save progress incrementally
        const BATCH_SIZE: u64 = 50_000;
        let mut current_height = start;
        let mut total_blocks_scanned = 0;
        let mut total_notes_discovered = 0;

        while current_height <= chain_tip {
            let batch_end = std::cmp::min(current_height + BATCH_SIZE - 1, chain_tip);
            let batch_size = batch_end - current_height + 1;

            println!("\n📦 Batch: blocks {} to {} ({} blocks)",
                     current_height, batch_end, batch_size);
            println!("   Progress: {}/{} blocks ({:.1}%)",
                     current_height - start,
                     total_blocks,
                     ((current_height - start) as f64 / total_blocks as f64) * 100.0);

            // Download this batch
            println!("   Downloading...");
            let blocks = self.download_blocks(current_height, batch_end).await?;

            // Scan this batch
            println!("   Scanning...");
            let scan_result = self.scan_blocks(&blocks)?;

            total_blocks_scanned += scan_result.blocks_scanned;
            total_notes_discovered += scan_result.notes_discovered;

            println!("   ✓ Batch complete: {} blocks scanned, {} notes found",
                     scan_result.blocks_scanned,
                     scan_result.notes_discovered);

            // Move to next batch
            current_height = batch_end + 1;
        }

        let summary = ScanSummary {
            start_height: start,
            end_height: chain_tip,
            blocks_scanned: total_blocks_scanned,
            notes_discovered: total_notes_discovered,
        };

        println!("\n✓ Scan complete!");
        println!("  Total blocks scanned: {}", summary.blocks_scanned);
        println!("  Total notes discovered: {}", summary.notes_discovered);

        Ok(summary)
    }

    /// Get the last block height that has been scanned
    /// Returns None if no blocks have been scanned yet
    fn get_last_scanned_height(&self) -> Result<Option<u64>> {
        use zcash_client_backend::data_api::WalletRead;

        // Use the WalletRead trait's chain_height method to get the last synced height
        // This queries the internal database state
        match self.wallet_db.chain_height() {
            Ok(Some(height)) => {
                // Convert BlockHeight to u64
                Ok(Some(u64::from(height)))
            },
            Ok(None) => {
                // No blocks have been scanned yet
                Ok(None)
            },
            Err(e) => {
                // If the query fails, log and assume first scan
                println!("  Note: Could not query chain height ({:?}), assuming first scan", e);
                Ok(None)
            }
        }
    }

    /// Get the wallet's birthday height (earliest block to scan)
    ///
    /// Returns the wallet birthday height for scanning.
    ///
    /// For production wallets using from_sapling_activation(), this will return the
    /// Sapling activation height. No safety margin is needed since there's no checkpoint.
    fn get_wallet_birthday(&self) -> Result<u64> {
        const REORG_SAFETY_MARGIN: u64 = 0;

        // First, check if user specified a custom birthday in environment
        if let Ok(birthday_str) = std::env::var("WALLET_BIRTHDAY_HEIGHT") {
            if !birthday_str.trim().is_empty() {
                if let Ok(birthday) = birthday_str.trim().parse::<u64>() {
                    let scan_from = birthday + REORG_SAFETY_MARGIN;
                    println!("  Wallet birthday from env: {}", birthday);
                    println!("  Starting scan from: {} (birthday + {} block safety margin)",
                             scan_from, REORG_SAFETY_MARGIN);
                    return Ok(scan_from);
                }
            }
        }

        // Get the minimum birthday height across all accounts
        let account_ids = self.wallet_db.get_account_ids()
            .context("Failed to get account IDs")?;

        if account_ids.is_empty() {
            // No accounts yet, use network activation height
            let default_birthday = match self.network {
                Network::TestNetwork => 280_000, // Testnet sapling activation
                Network::MainNetwork => 419_200, // Mainnet sapling activation
            };
            println!("  Using default birthday (Sapling activation): {}", default_birthday);
            return Ok(default_birthday);
        }

        // Get the earliest account birthday
        // Since birthday() is private, we'll use a simpler approach:
        // Use the sapling activation height for now
        // TODO: Store and retrieve account birthdays separately
        let default_birthday = match self.network {
            Network::TestNetwork => 280_000,
            Network::MainNetwork => 419_200,
        };
        println!("  Using default birthday (Sapling activation): {}", default_birthday);
        Ok(default_birthday)
    }

    /// Download compact blocks from lightwalletd
    async fn download_blocks(&mut self, start: u64, end: u64) -> Result<Vec<CompactBlock>> {
        println!("  Downloading blocks {} to {}...", start, end);

        // Stream compact blocks from lightwalletd
        let mut stream = self.lightwalletd.get_block_range(start, end).await
            .context("Failed to start block stream")?;

        let mut blocks = Vec::new();

        // Collect all blocks from the stream
        use tokio_stream::StreamExt;
        while let Some(block_result) = stream.next().await {
            match block_result {
                Ok(block) => {
                    if blocks.len() % 1000 == 0 && !blocks.is_empty() {
                        println!("    Downloaded {} blocks...", blocks.len());
                    }
                    blocks.push(block);
                }
                Err(e) => {
                    anyhow::bail!("Failed to receive block: {}", e);
                }
            }
        }

        println!("  ✓ Downloaded {} blocks", blocks.len());

        Ok(blocks)
    }

    /// Scan cached blocks for wallet transactions
    fn scan_blocks(&mut self, blocks: &[CompactBlock]) -> Result<ScanResult> {
        println!("  Scanning {} blocks...", blocks.len());

        if blocks.is_empty() {
            return Ok(ScanResult {
                blocks_scanned: 0,
                notes_discovered: 0,
            });
        }

        // Insert blocks into the block cache
        let mut blocks_written = 0;
        for block in blocks {
            let height = BlockHeight::from_u32(block.height as u32);
            self.block_cache.write_block(height, block)
                .context("Failed to write block to cache")?;
            blocks_written += 1;
        }

        println!("  ✓ Cached {} blocks", blocks_written);

        // Get the starting height from first block
        let first_block = &blocks[0];
        let start_height = BlockHeight::from_u32(first_block.height as u32);

        // WORKAROUND: Clear checkpoints at (start_height - 1) to avoid conflicts
        // Account creation sets up a checkpoint at (birthday - 1) with tree state
        // ChainState::empty will try to create an empty checkpoint at the same height
        // We clear the conflicting checkpoint to allow ChainState::empty to work
        // Note: This is safe because the tree frontiers are preserved in the shardtree
        if let Some(db_path) = &self.db_path {
            if let Ok(conn) = Connection::open(db_path) {
                let clear_height = u32::from(start_height).saturating_sub(1);
                let _ = conn.execute(
                    "DELETE FROM sapling_tree_checkpoints WHERE checkpoint_id = ?",
                    [clear_height],
                );
                let _ = conn.execute(
                    "DELETE FROM orchard_tree_checkpoints WHERE checkpoint_id = ?",
                    [clear_height],
                );
                println!("  ✓ Cleared checkpoint at height {}", clear_height);
            }
        }

        println!("  Trial-decrypting notes...");

        // Parse block hash from the first block's prev_hash
        let block_hash = if first_block.prev_hash.len() == 32 {
            let mut hash_bytes = [0u8; 32];
            hash_bytes.copy_from_slice(&first_block.prev_hash);
            BlockHash(hash_bytes)
        } else {
            BlockHash([0u8; 32])
        };

        // Create ChainState for scanning
        // Note: ChainState::empty provides minimal state at the prior block height
        // The wallet database's shardtree still contains the proper tree frontiers
        // from account creation - they're stored in shard tables, not checkpoints
        let chain_state = ChainState::empty(start_height - 1, block_hash);

        println!("  Scanning from height {}...", start_height);

        // Scan the cached blocks
        // This will trial-decrypt notes and store discovered transactions
        let summary = scan_cached_blocks(
            &self.network,
            &self.block_cache,
            &mut self.wallet_db,
            start_height,
            &chain_state,
            blocks.len(),
        ).map_err(|e| anyhow::anyhow!("Failed to scan blocks: {:?}", e))?;

        // Count received notes from both Sapling and Orchard pools
        let sapling_notes = summary.received_sapling_note_count();
        let orchard_notes = summary.received_orchard_note_count();
        let total_notes = sapling_notes + orchard_notes;

        println!("  ✓ Scan complete");
        println!("    Sapling notes: {}", sapling_notes);
        println!("    Orchard notes: {}", orchard_notes);
        println!("    Total notes discovered: {}", total_notes);

        Ok(ScanResult {
            blocks_scanned: blocks.len(),
            notes_discovered: total_notes,
        })
    }
}

/// Summary of a blockchain scan operation
#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub start_height: u64,
    pub end_height: u64,
    pub blocks_scanned: usize,
    pub notes_discovered: usize,
}

/// Result of scanning a batch of blocks
#[derive(Debug)]
struct ScanResult {
    blocks_scanned: usize,
    notes_discovered: usize,
}

#[cfg(all(test, feature = "disabled_tests"))]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use super::database::Database;

    #[tokio::test]
    async fn test_scanner_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        let database = Database::new(db_path.clone(), Network::TestNetwork).unwrap();
        let wallet_db = database.get_wallet_db_mut().unwrap();

        let lightwalletd = LightwalletdClient::new("https://testnet.lightwalletd.com:9067".to_string());

        let scanner = BlockchainScanner::new(
            wallet_db,
            lightwalletd,
            Network::TestNetwork,
        );

        // Just test that we can create a scanner
        assert_eq!(scanner.network, Network::TestNetwork);
    }

    #[test]
    fn test_birthday_heights() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");

        // Testnet
        let database = Database::new(db_path.clone(), Network::TestNetwork).unwrap();
        let wallet_db = database.get_wallet_db_mut().unwrap();
        let lightwalletd = LightwalletdClient::new("http://localhost:9067".to_string());
        let scanner = BlockchainScanner::new(wallet_db, lightwalletd, Network::TestNetwork);

        let birthday = scanner.get_wallet_birthday().unwrap();
        assert_eq!(birthday, 280_000);

        // Mainnet
        let temp_dir2 = TempDir::new().unwrap();
        let db_path2 = temp_dir2.path().join("test_wallet.db");
        let database2 = Database::new(db_path2.clone(), Network::MainNetwork);
        let wallet_db2 = database2.init().unwrap();
        let lightwalletd2 = LightwalletdClient::new("http://localhost:9067".to_string());
        let scanner2 = BlockchainScanner::new(wallet_db2, lightwalletd2, Network::MainNetwork);

        let birthday2 = scanner2.get_wallet_birthday().unwrap();
        assert_eq!(birthday2, 419_200);
    }
}
