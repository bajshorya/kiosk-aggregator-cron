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

pub fn all_swap_pairs(wallets: &WalletConfig) -> Vec<SwapPair> {
    let btc = &wallets.bitcoin_testnet_address;
    let ltc = &wallets.litecoin_testnet_address;
    let evm = &wallets.evm_address;
    let stark = &wallets.starknet_address;
    let sol = &wallets.solana_address;
    let tron = &wallets.tron_address;
    let sui = &wallets.sui_address;

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

    vec![
        // Bitcoin → EVM
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
        // Litecoin → EVM
        pair!(
            "litecoin_testnet:ltc",
            "1000000",
            ltc,
            "base_sepolia:wbtc",
            evm
        ),
        // EVM → Bitcoin
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
        // Fully automatic EVM <-> EVM swaps - no manual deposit needed
        pair!(
            "ethereum_sepolia:wbtc",
            "50000",
            evm,
            "base_sepolia:wbtc",
            evm
        ),
        pair!(
            "base_sepolia:wbtc",
            "50000",
            evm,
            "ethereum_sepolia:wbtc",
            evm
        ),
        pair!(
            "base_sepolia:wbtc",
            "50000",
            evm,
            "arbitrum_sepolia:wbtc",
            evm
        ),
        pair!(
            "arbitrum_sepolia:wbtc",
            "50000",
            evm,
            "base_sepolia:wbtc",
            evm
        ),
        // Tron → EVM
        pair!(
            "tron_shasta:wbtc",
            "50000",
            tron,
            "arbitrum_sepolia:wbtc",
            evm
        ),
        pair!("tron_shasta:wbtc", "50000", tron, "base_sepolia:wbtc", evm),
        // Solana → BTC / EVM
        pair!(
            "solana_testnet:sol",
            "354608265",
            sol,
            "bitcoin_testnet:btc",
            btc
        ),
        pair!(
            "solana_testnet:sol",
            "354608265",
            sol,
            "base_sepolia:wbtc",
            evm
        ),
        // Starknet → BTC / EVM
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
        // Sui → BTC / EVM
        pair!(
            "sui_testnet:sui",
            "3000000000",
            sui,
            "bitcoin_testnet:btc",
            btc
        ),
        pair!(
            "sui_testnet:sui",
            "3000000000",
            sui,
            "base_sepolia:wbtc",
            evm
        ),
    ]
}

pub fn requires_manual_deposit(asset: &str) -> bool {
    asset.starts_with("bitcoin_") || asset.starts_with("litecoin_")
}
