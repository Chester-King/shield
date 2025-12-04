use std::env;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub network: Network,
    pub lightwalletd_url: String,
    pub database_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Testnet,
    Mainnet,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let network_str = env::var("ZCASH_NETWORK")
            .unwrap_or_else(|_| "test".to_string());

        let network = match network_str.as_str() {
            "test" | "testnet" => Network::Testnet,
            "main" | "mainnet" => Network::Mainnet,
            _ => anyhow::bail!("Invalid ZCASH_NETWORK: {}", network_str),
        };

        let lightwalletd_url = env::var("LIGHTWALLETD_URL")
            .unwrap_or_else(|_| "testnet.lightwalletd.com:9067".to_string());

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:wallet.db".to_string());

        Ok(Config {
            network,
            lightwalletd_url,
            database_url,
        })
    }
}
