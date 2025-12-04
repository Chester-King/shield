use anyhow::Result;
use bip39::Mnemonic;
use rand::Rng;
use zcash_keys::keys::{UnifiedSpendingKey, UnifiedAddressRequest, ReceiverRequirement};
use zcash_protocol::consensus::{Network, TestNetwork, MainNetwork};
use zip32::AccountId;

/// Represents a Zcash wallet with keys
pub struct Wallet {
    spending_key: UnifiedSpendingKey,
    network: Network,
}

impl Wallet {
    /// Generate a new wallet with a random mnemonic
    /// Returns: (Wallet, Mnemonic) - wallet instance and the 24-word phrase
    pub fn generate_new(network: Network) -> Result<(Self, Mnemonic)> {
        // Generate 24-word BIP39 mnemonic (32 bytes of entropy)
        let mut rng = rand::thread_rng();
        let entropy: [u8; 32] = rng.gen();
        let mnemonic = Mnemonic::from_entropy(&entropy)?;

        // Derive wallet from this mnemonic
        let wallet = Self::from_mnemonic(&mnemonic, network)?;

        Ok((wallet, mnemonic))
    }

    /// Restore wallet from existing mnemonic
    ///
    /// # Rust Concepts:
    /// - `&Mnemonic` = borrowed reference (we don't take ownership)
    /// - `network: Network` = owned value (we copy it into struct)
    ///
    /// # Steps:
    /// 1. Convert mnemonic to 512-bit seed (BIP39 standard)
    /// 2. Derive spending key using ZIP 32 (Zcash's key derivation)
    /// 3. Create wallet with the key
    pub fn from_mnemonic(mnemonic: &Mnemonic, network: Network) -> Result<Self> {
        // Step 1: Convert mnemonic to seed
        // BIP39 standard: mnemonic → 512-bit seed using PBKDF2
        // The empty string "" is the passphrase (optional, we don't use one)
        let seed = mnemonic.to_seed("");

        // Step 2: Derive Unified Spending Key from seed
        // ZIP 32 path: m/32'/133'/0'
        // - 32' = purpose (ZIP 32)
        // - 133' = coin_type (Zcash)
        // - 0' = account 0 (first account)
        let account_id = AccountId::try_from(0)
            .map_err(|e| anyhow::anyhow!("Invalid account ID: {:?}", e))?;

        let spending_key = match network {
            Network::TestNetwork => {
                UnifiedSpendingKey::from_seed(
                    &TestNetwork,
                    &seed[..],
                    account_id,
                )
                .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?
            }
            Network::MainNetwork => {
                UnifiedSpendingKey::from_seed(
                    &MainNetwork,
                    &seed[..],
                    account_id,
                )
                .map_err(|e| anyhow::anyhow!("Failed to derive spending key: {:?}", e))?
            }
        };

        // Step 3: Create wallet
        let wallet = Wallet {
            spending_key,
            network,
        };

        Ok(wallet)
    }

    /// Get a reference to the spending key
    ///
    /// Returns the UnifiedSpendingKey for use in database initialization
    /// and transaction signing.
    pub fn spending_key(&self) -> &UnifiedSpendingKey {
        &self.spending_key
    }

    /// Get the network this wallet is configured for
    pub fn network(&self) -> Network {
        self.network.clone()
    }

    /// Get the unified address for this wallet
    pub fn get_address(&self) -> Result<String> {
        let ufvk = self.spending_key.to_unified_full_viewing_key();

        // Request address with Sapling (required) and optionally Orchard
        // Matches Zashi wallet configuration - NEAR Intents accepts shielded unified addresses
        use ReceiverRequirement::*;
        let request = UnifiedAddressRequest::unsafe_custom(Allow, Require, Omit);

        let (ua, _diversifier_index) = ufvk.default_address(request)
            .map_err(|e| anyhow::anyhow!("Failed to generate address: {:?}", e))?;

        let address_str = match self.network {
            Network::TestNetwork => ua.encode(&TestNetwork),
            Network::MainNetwork => ua.encode(&MainNetwork),
        };

        Ok(address_str)
    }

    /// Get the transparent address for this wallet
    /// Returns a unified address that includes both shielded (Sapling) and transparent receivers
    /// This address can receive both shielded and transparent ZEC
    pub fn get_transparent_address(&self) -> Result<String> {
        use zcash_keys::keys::UnifiedAddressRequest;
        use zcash_keys::keys::ReceiverRequirement::*;

        let ufvk = self.spending_key.to_unified_full_viewing_key();

        // Request address with Sapling (required) and Transparent (required)
        // Note: At least one shielded receiver is required by zcash_keys
        let request = UnifiedAddressRequest::unsafe_custom(Allow, Require, Require);

        let (ua, _diversifier_index) = ufvk.default_address(request)
            .map_err(|e| anyhow::anyhow!("Failed to generate transparent address: {:?}", e))?;

        let address_str = match self.network {
            Network::TestNetwork => ua.encode(&TestNetwork),
            Network::MainNetwork => ua.encode(&MainNetwork),
        };

        Ok(address_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Language;

    #[test]
    fn test_generate_wallet() {
        let result = Wallet::generate_new(Network::TestNetwork);
        assert!(result.is_ok());

        let (_wallet, mnemonic) = result.unwrap();
        println!("Generated mnemonic: {}", mnemonic);

        let mnemonic_str = mnemonic.to_string();
        let words: Vec<&str> = mnemonic_str.split_whitespace().collect();
        assert_eq!(words.len(), 24);
    }

    #[test]
    fn test_restore_wallet() {
        let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        let mnemonic = Mnemonic::parse_in(Language::English, test_mnemonic)
            .expect("Failed to parse mnemonic");

        let result = Wallet::from_mnemonic(&mnemonic, Network::TestNetwork);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_address() {
        let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

        let mnemonic = Mnemonic::parse_in(Language::English, test_mnemonic)
            .expect("Failed to parse mnemonic");

        let wallet = Wallet::from_mnemonic(&mnemonic, Network::TestNetwork)
            .expect("Failed to create wallet");

        let address = wallet.get_address().expect("Failed to get address");

        println!("Generated address: {}", address);
        assert!(address.starts_with("utest1"));
        assert!(address.len() > 100);
    }
}
