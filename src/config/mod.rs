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
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletConfig {
    pub bitcoin_testnet_address: String,
    pub litecoin_testnet_address: String,
    pub evm_address: String,
    pub evm_private_key: String,
    pub starknet_address: String,
    pub solana_address: String,
    pub tron_address: String,
    pub sui_address: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SchedulerConfig {
    pub cron: String,
    pub swap_timeout_secs: u64,
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub garden: GardenConfig,
    pub wallets: WalletConfig,
    pub scheduler: SchedulerConfig,
    pub database_url: String,
    pub rpc_urls: RpcUrlsConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        Ok(AppConfig {
            garden: GardenConfig {
                api_base_url: std::env::var("GARDEN_API_BASE_URL")
                    .unwrap_or_else(|_| "https://testnet.api.garden.finance".to_string()),
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
                solana_address: std::env::var("WALLET_SOLANA")
                    .unwrap_or_else(|_| "YH4btvqb4JBWSEJh22MuA231ekpJ5JqbBXQY1apJtKH".to_string()),
                tron_address: std::env::var("WALLET_TRON")
                    .unwrap_or_else(|_| "TWbEz5ibiL6dreiLJ5oBF5CwDkw6Xfe6KX".to_string()),
                sui_address: std::env::var("WALLET_SUI").unwrap_or_else(|_| {
                    "0x79a1582388c16d0ab85904f320eb0527481391a9b9ab4b2ab46adc4c2564f9d0".to_string()
                }),
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
            },
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "garden_swaps.db".to_string()),
        })
    }

    pub fn swap_timeout(&self) -> Duration {
        Duration::from_secs(self.scheduler.swap_timeout_secs)
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.scheduler.poll_interval_secs)
    }
}
