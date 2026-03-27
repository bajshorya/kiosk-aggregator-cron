use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GardenConfig {
    pub api_base_url: String,
    pub app_id: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcUrlsConfig {
    pub ethereum_sepolia: String,
    pub base_sepolia: String,
    pub arbitrum_sepolia: String,
    pub solana_testnet: String,
    pub starknet_sepolia: String,
    pub tron_shasta: String,
    pub sui_testnet: String,
    pub bitcoin_testnet: String,
    pub bnbchain_testnet: String,
    pub citrea_testnet: String,
    pub monad_testnet: String,
    pub xrpl_testnet: String,
    pub alpen_testnet: String,
    pub alpen_signet: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletConfig {
    pub bitcoin_testnet_address: String,
    pub litecoin_testnet_address: String,
    pub evm_address: String,
    pub evm_private_key: String,
    pub starknet_address: String,
    pub starknet_private_key: Option<String>,
    pub solana_address: String,
    pub solana_private_key: Option<String>,
    pub tron_address: String,
    pub tron_private_key: Option<String>,
    pub sui_address: String,
    pub sui_private_key: Option<String>,
    pub bitcoin_testnet_private_key: Option<String>,
    pub xrpl_address: String,
    pub xrpl_private_key: Option<String>,
    pub alpen_address: String,
    pub alpen_private_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SchedulerConfig {
    pub cron: String,
    pub swap_timeout_secs: u64,
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
    Testnet,
    Mainnet,
}

impl Default for NetworkMode {
    fn default() -> Self {
        NetworkMode::Testnet
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub garden: GardenConfig,
    pub wallets: WalletConfig,
    pub scheduler: SchedulerConfig,
    pub database_url: String,
    pub rpc_urls: RpcUrlsConfig,
    pub network_mode: NetworkMode,
    pub enable_balance_check: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        // Debug: Check if SOLANA_PRIVATE_KEY is in environment
        let solana_key = std::env::var("SOLANA_PRIVATE_KEY").ok();
        eprintln!("DEBUG: SOLANA_PRIVATE_KEY loaded: is_some={}, len={}", 
            solana_key.is_some(),
            solana_key.as_ref().map(|s| s.len()).unwrap_or(0)
        );
        
        let network_mode = std::env::var("NETWORK_MODE")
            .unwrap_or_else(|_| "testnet".to_string())
            .to_lowercase();
        
        let network_mode = match network_mode.as_str() {
            "mainnet" => NetworkMode::Mainnet,
            _ => NetworkMode::Testnet,
        };

        eprintln!("🌐 Network mode: {:?}", network_mode);
        
        Ok(AppConfig {
            garden: GardenConfig {
                api_base_url: std::env::var("GARDEN_API_BASE_URL")
                    .unwrap_or_else(|_| {
                        match network_mode {
                            NetworkMode::Mainnet => "https://api.garden.finance".to_string(),
                            NetworkMode::Testnet => "https://testnet.api.garden.finance".to_string(),
                        }
                    }),
                app_id: std::env::var("GARDEN_APP_ID").unwrap_or_else(|_| {
                    "f242ea49332293424c96c562a6ef575a819908c878134dcb4fce424dc84ec796".to_string()
                }),
            },
            wallets: WalletConfig {
                bitcoin_testnet_address: std::env::var("WALLET_BITCOIN_TESTNET").unwrap_or_else(
                    |_| {
                        "tb1p4pr78swsn60y4ushe05v28mqpqppxxkfkxu2wun5jw6duc8unj3sjrh4gd".to_string()
                    },
                ),
                evm_private_key: std::env::var("WALLET_EVM_PRIVATE_KEY")
                    .expect("WALLET_EVM_PRIVATE_KEY must be set"),
                litecoin_testnet_address: std::env::var("WALLET_LITECOIN_TESTNET")
                    .unwrap_or_else(|_| "tltc1qycexnc7fjqh2x4dnaht6gumcjxdzkdpjnlxe4s".to_string()),
                evm_address: std::env::var("WALLET_EVM")
                    .unwrap_or_else(|_| "0x004Cc75ACF4132Fc08cB6a252E767804F303F729".to_string()),
                starknet_address: std::env::var("WALLET_STARKNET").unwrap_or_else(|_| {
                    "0x00609190b1348bcc06da44d58c79709495c11a5a6f0b9e154e1209f2a17dd933".to_string()
                }),
                starknet_private_key: std::env::var("STARKNET_PRIVATE_KEY").ok(),
                solana_address: std::env::var("WALLET_SOLANA")
                    .unwrap_or_else(|_| "YH4btvqb4JBWSEJh22MuA231ekpJ5JqbBXQY1apJtKH".to_string()),
                solana_private_key: std::env::var("SOLANA_PRIVATE_KEY").ok(),
                tron_address: std::env::var("WALLET_TRON")
                    .unwrap_or_else(|_| "TWbEz5ibiL6dreiLJ5oBF5CwDkw6Xfe6KX".to_string()),
                tron_private_key: std::env::var("TRON_PRIVATE_KEY").ok(),
                sui_address: std::env::var("WALLET_SUI").unwrap_or_else(|_| {
                    "0x79a1582388c16d0ab85904f320eb0527481391a9b9ab4b2ab46adc4c2564f9d0".to_string()
                }),
                sui_private_key: std::env::var("SUI_PRIVATE_KEY").ok(),
                bitcoin_testnet_private_key: std::env::var("BITCOIN_TESTNET_PRIVATE_KEY").ok(),
                xrpl_address: std::env::var("WALLET_XRPL")
                    .unwrap_or_else(|_| "rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY".to_string()),
                xrpl_private_key: std::env::var("XRPL_PRIVATE_KEY").ok(),
                alpen_address: std::env::var("WALLET_ALPEN")
                    .unwrap_or_else(|_| "tb1qalpen1234567890abcdefghijklmnopqrstuvwxyz".to_string()),
                alpen_private_key: std::env::var("ALPEN_PRIVATE_KEY").ok(),
            },
            scheduler: SchedulerConfig {
                cron: std::env::var("SCHEDULER_CRON")
                    .unwrap_or_else(|_| "0 0 */5 * * *".to_string()),
                swap_timeout_secs: std::env::var("SWAP_TIMEOUT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(900),
                poll_interval_secs: std::env::var("POLL_INTERVAL_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(15),
            },
            rpc_urls: RpcUrlsConfig {
                ethereum_sepolia: std::env::var("RPC_ETHEREUM_SEPOLIA")
                    .unwrap_or_else(|_| "https://rpc.sepolia.org".to_string()),
                base_sepolia: std::env::var("RPC_BASE_SEPOLIA")
                    .unwrap_or_else(|_| "https://sepolia.base.org".to_string()),
                arbitrum_sepolia: std::env::var("RPC_ARBITRUM_SEPOLIA")
                    .unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string()),
                solana_testnet: std::env::var("RPC_SOLANA_TESTNET")
                    .unwrap_or_else(|_| "https://api.testnet.solana.com".to_string()),
                starknet_sepolia: std::env::var("RPC_STARKNET_SEPOLIA")
                    .unwrap_or_else(|_| "https://starknet-sepolia.public.blastapi.io".to_string()),
                tron_shasta: std::env::var("RPC_TRON_SHASTA")
                    .unwrap_or_else(|_| "https://api.shasta.trongrid.io".to_string()),
                sui_testnet: std::env::var("RPC_SUI_TESTNET")
                    .unwrap_or_else(|_| "https://fullnode.testnet.sui.io:443".to_string()),
                bitcoin_testnet: std::env::var("RPC_BITCOIN_TESTNET")
                    .unwrap_or_else(|_| "https://blockstream.info/testnet/api".to_string()),
                bnbchain_testnet: std::env::var("RPC_BNBCHAIN_TESTNET")
                    .unwrap_or_else(|_| "https://data-seed-prebsc-1-s1.binance.org:8545".to_string()),
                citrea_testnet: std::env::var("RPC_CITREA_TESTNET")
                    .unwrap_or_else(|_| "https://rpc.testnet.citrea.xyz".to_string()),
                monad_testnet: std::env::var("RPC_MONAD_TESTNET")
                    .unwrap_or_else(|_| "https://testnet.monad.xyz".to_string()),
                xrpl_testnet: std::env::var("RPC_XRPL_TESTNET")
                    .unwrap_or_else(|_| "https://s.altnet.rippletest.net:51234".to_string()),
                alpen_testnet: std::env::var("RPC_ALPEN_TESTNET")
                    .unwrap_or_else(|_| "https://rpc.testnet.alpen.network".to_string()),
                alpen_signet: std::env::var("RPC_ALPEN_SIGNET")
                    .unwrap_or_else(|_| "https://rpc.signet.alpen.network".to_string()),
            },
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "garden_swaps.db".to_string()),
            network_mode,
            enable_balance_check: std::env::var("ENABLE_BALANCE_CHECK")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true), // Default to true
        })
    }

    pub fn swap_timeout(&self) -> Duration {
        Duration::from_secs(self.scheduler.swap_timeout_secs)
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.scheduler.poll_interval_secs)
    }
}
