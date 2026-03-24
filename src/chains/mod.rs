pub mod evm_signer;
use crate::config::WalletConfig;
#[derive(Debug, Clone)]
pub struct ChainAsset {
    pub asset: String,
    pub default_from_amount: String,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct SwapPair {
    pub source: ChainAsset,
    pub destination: ChainAsset,
}

impl SwapPair {
    pub fn label(&self) -> String {
        format!("{} -> {}", self.source.asset, self.destination.asset)
    }
}

/// Returns swap pairs optimized for minimal spending (~$5 per swap)
/// Sequenced to minimize total cost by reusing received funds
pub fn all_swap_pairs(wallets: &WalletConfig) -> Vec<SwapPair> {
    let btc = &wallets.bitcoin_testnet_address;
    let ltc = &wallets.litecoin_testnet_address;
    let evm = &wallets.evm_address;
    let stark = &wallets.starknet_address;
    let sol = &wallets.solana_address;
    let tron = &wallets.tron_address;
    let _sui = &wallets.sui_address; // Disabled due to API error

    macro_rules! pair {
        ($fa:expr, $famt:expr, $fo:expr, $ta:expr, $to:expr) => {
            SwapPair {
                source: ChainAsset {
                    asset: $fa.to_string(),
                    default_from_amount: $famt.to_string(),
                    owner: $fo.clone(),
                },
                destination: ChainAsset {
                    asset: $ta.to_string(),
                    default_from_amount: String::new(),
                    owner: $to.clone(),
                },
            }
        };
    }

    // Amounts optimized for API minimum requirements:
    // BTC/WBTC: 50000 sats (minimum required by API, ~$50 at $100k BTC)
    // LTC: 1000000 sats (~$50 at $50 LTC)
    // SOL: 350000000 (~$50 at $140 SOL)
    // Note: API enforces minimum of 50000 to 1000000 range

    vec![
        // ═══ PHASE 1: Start with EVM assets (no manual deposit) ═══
        // Cost: $50 (you already have WBTC on EVM)
        pair!(
            "ethereum_sepolia:wbtc",
            "50000",
            evm,
            "base_sepolia:wbtc",
            evm
        ),
        // ═══ PHASE 2: Use received Base WBTC for more EVM swaps ═══
        // Cost: $0 (using received funds)
        pair!(
            "base_sepolia:wbtc",
            "50000",
            evm,
            "arbitrum_sepolia:wbtc",
            evm
        ),
        pair!(
            "base_sepolia:wbtc",
            "50000",
            evm,
            "ethereum_sepolia:wbtc",
            evm
        ),
        // ═══ PHASE 3: Use Arbitrum WBTC ═══
        // Cost: $0 (using received funds)
        pair!(
            "arbitrum_sepolia:wbtc",
            "50000",
            evm,
            "base_sepolia:wbtc",
            evm
        ),
        // ═══ PHASE 4: EVM to Bitcoin (test reverse flow) ═══
        // Cost: $0 (using received funds)
        pair!(
            "ethereum_sepolia:wbtc",
            "50000",
            evm,
            "bitcoin_testnet:btc",
            btc
        ),
        pair!(
            "base_sepolia:wbtc",
            "50000",
            evm,
            "bitcoin_testnet:btc",
            btc
        ),
        // ═══ PHASE 5: Bitcoin to EVM (requires manual deposit) ═══
        // Cost: $150 (3 × $50 BTC deposits)
        pair!(
            "bitcoin_testnet:btc",
            "50000",
            btc,
            "base_sepolia:wbtc",
            evm
        ),
        pair!(
            "bitcoin_testnet:btc",
            "50000",
            btc,
            "ethereum_sepolia:wbtc",
            evm
        ),
        pair!(
            "bitcoin_testnet:btc",
            "50000",
            btc,
            "arbitrum_sepolia:wbtc",
            evm
        ),
        // ═══ PHASE 6: Litecoin (requires manual deposit) ═══
        // Cost: $50 (1 × $50 LTC deposit)
        // pair!("litecoin_testnet:ltc", "1000000", ltc, "base_sepolia:wbtc", evm),

        // ═══ PHASE 7: Solana (requires manual deposit or existing balance) ═══
        // Cost: $100 (2 × $50 SOL)
        pair!(
            "solana_testnet:sol",
            "350000000",
            sol,
            "bitcoin_testnet:btc",
            btc
        ),
        pair!(
            "solana_testnet:sol",
            "350000000",
            sol,
            "base_sepolia:wbtc",
            evm
        ),
        // ═══ PHASE 8: Starknet (requires manual deposit or existing balance) ═══
        // Cost: $100 (2 × $50 WBTC on Starknet)
        pair!(
            "starknet_sepolia:wbtc",
            "50000",
            stark,
            "bitcoin_testnet:btc",
            btc
        ),
        pair!(
            "starknet_sepolia:wbtc",
            "50000",
            stark,
            "base_sepolia:wbtc",
            evm
        ),
        // ═══ PHASE 9: Tron (requires manual deposit or existing balance) ═══
        // Cost: $100 (2 × $50 WBTC on Tron)
        pair!(
            "tron_shasta:wbtc",
            "50000",
            tron,
            "arbitrum_sepolia:wbtc",
            evm
        ),
        pair!("tron_shasta:wbtc", "50000", tron, "base_sepolia:wbtc", evm),
        // ═══ PHASE 10: Sui (currently failing - skip or fix asset name) ═══
        // Cost: $100 (2 × $50 SUI) - DISABLED due to API error
        // pair!("sui_testnet:sui", "3000000000", sui, "bitcoin_testnet:btc", btc),
        // pair!("sui_testnet:sui", "3000000000", sui, "base_sepolia:wbtc", evm),
    ]
}

pub fn requires_manual_deposit(asset: &str) -> bool {
    asset.starts_with("bitcoin_") || asset.starts_with("litecoin_")
}
