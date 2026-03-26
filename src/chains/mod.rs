pub mod evm_signer;
pub mod solana_signer;
pub mod starknet_signer;
pub mod tron_signer;
pub mod sui_signer;
pub mod balance_checker;

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

/// Returns swap pairs optimized for minimal spending
/// Uses MINIMUM amounts required by Garden API to reduce costs
/// Includes round-trip pairs to maintain token balances for continuous testing
/// Set ENABLE_ROUND_TRIPS=true to automatically add reverse swaps for balance maintenance
pub fn all_swap_pairs(wallets: &WalletConfig) -> Vec<SwapPair> {
    let btc = &wallets.bitcoin_testnet_address;
    let _ltc = &wallets.litecoin_testnet_address;
    let evm = &wallets.evm_address;
    let stark = &wallets.starknet_address;
    let sol = &wallets.solana_address;
    let tron = &wallets.tron_address;
    let _sui = &wallets.sui_address;

    // Minimum amounts based on Garden API requirements:
    // - BTC/WBTC: 50,000 sats (0.0005 BTC) = ~$50
    // - USDC: 15,000,000 (15 USDC with 6 decimals) = ~$15
    // - ETH: 5,000,000,000,000,000 wei (0.005 ETH) = ~$15-20
    // - USDT: 15,000,000 (15 USDT with 6 decimals) = ~$15

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

    let mut pairs = vec![
        // Native token swaps (ETH to SOL) - minimum: 0.005 ETH
        pair!("ethereum_sepolia:eth", "5000000000000000", evm, "solana_testnet:sol", sol),
        
        // 1-3: WBTC swaps (minimum: 50,000 sats)
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "ethereum_sepolia:wbtc", evm),
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "alpen_signet:btc", btc),
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "alpen_testnet:sbtc", btc),
        
        // 4-12: USDC swaps from Arbitrum (minimum: 15,000,000 = 15 USDC)
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "base_sepolia:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc", btc),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "citrea_testnet:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "monad_testnet:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "tron_shasta:usdt", tron),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "xrpl_testnet:xrp", evm),
        
        // 13-22: Ethereum swaps (WBTC: 50k sats, USDC: 15M)
        pair!("ethereum_sepolia:wbtc", "50000", evm, "alpen_testnet:usdc", btc),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "base_sepolia:usdc", evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc", btc),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "citrea_testnet:usdc", evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "monad_testnet:usdc", evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "tron_shasta:usdt", tron),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "xrpl_testnet:xrp", evm),
        
        // 23-49: Alpen, Base, Bitcoin swaps (BTC: 50k sats, USDC: 15M)
        pair!("alpen_signet:btc", "50000", btc, "alpen_testnet:sbtc", btc),
        pair!("alpen_signet:btc", "50000", btc, "base_sepolia:usdc", evm),
        pair!("alpen_signet:btc", "50000", btc, "bitcoin_testnet:btc", btc),
        pair!("alpen_signet:btc", "50000", btc, "bnbchain_testnet:wbtc", evm),
        pair!("alpen_signet:btc", "50000", btc, "citrea_testnet:usdc", evm),
        pair!("alpen_signet:btc", "50000", btc, "monad_testnet:usdc", evm),
        pair!("alpen_signet:btc", "50000", btc, "solana_testnet:usdc", sol),
        pair!("alpen_signet:btc", "50000", btc, "starknet_sepolia:wbtc", stark),
        pair!("alpen_signet:btc", "50000", btc, "tron_shasta:usdt", tron),
        pair!("alpen_signet:btc", "50000", btc, "xrpl_testnet:xrp", evm),
        
        pair!("alpen_testnet:sbtc", "50000", btc, "base_sepolia:usdc", evm),
        pair!("alpen_testnet:usdc", "15000000", btc, "bitcoin_testnet:btc", btc),
        pair!("alpen_testnet:usdc", "15000000", btc, "bnbchain_testnet:wbtc", evm),
        pair!("alpen_testnet:usdc", "15000000", btc, "citrea_testnet:usdc", evm),
        pair!("alpen_testnet:usdc", "15000000", btc, "monad_testnet:usdc", evm),
        pair!("alpen_testnet:usdc", "15000000", btc, "solana_testnet:usdc", sol),
        pair!("alpen_testnet:usdc", "15000000", btc, "starknet_sepolia:wbtc", stark),
        pair!("alpen_testnet:usdc", "15000000", btc, "tron_shasta:usdt", tron),
        pair!("alpen_testnet:usdc", "15000000", btc, "xrpl_testnet:xrp", evm),
        
        pair!("base_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc", btc),
        pair!("base_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("base_sepolia:cbltc", "50000", evm, "citrea_testnet:usdc", evm),
        pair!("base_sepolia:usdc", "15000000", evm, "monad_testnet:usdc", evm),
        pair!("base_sepolia:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("base_sepolia:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        pair!("base_sepolia:usdc", "15000000", evm, "tron_shasta:usdt", tron),
        pair!("base_sepolia:usdc", "15000000", evm, "xrpl_testnet:xrp", evm),
        
        // 50-78: Bitcoin, BNB, Citrea, Monad, Solana, Starknet, Tron, XRPL swaps
        pair!("bitcoin_testnet:btc", "50000", btc, "bnbchain_testnet:wbtc", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "citrea_testnet:usdc", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "monad_testnet:usdc", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "solana_testnet:usdc", sol),
        pair!("bitcoin_testnet:btc", "50000", btc, "starknet_sepolia:wbtc", stark),
        pair!("bitcoin_testnet:btc", "50000", btc, "tron_shasta:usdt", tron),
        pair!("bitcoin_testnet:btc", "50000", btc, "xrpl_testnet:xrp", evm),
        
        pair!("bnbchain_testnet:wbtc", "50000", evm, "citrea_testnet:usdc", evm),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "monad_testnet:usdc", evm),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "solana_testnet:usdc", sol),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "starknet_sepolia:wbtc", stark),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "tron_shasta:usdt", tron),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "xrpl_testnet:xrp", evm),
        
        pair!("citrea_testnet:usdc", "15000000", evm, "monad_testnet:usdc", evm),
        pair!("citrea_testnet:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("citrea_testnet:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        pair!("citrea_testnet:usdc", "15000000", evm, "tron_shasta:usdt", tron),
        pair!("citrea_testnet:usdc", "15000000", evm, "xrpl_testnet:xrp", evm),
        
        pair!("monad_testnet:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("monad_testnet:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        pair!("monad_testnet:usdc", "15000000", evm, "tron_shasta:usdt", tron),
        pair!("monad_testnet:usdc", "15000000", evm, "xrpl_testnet:xrp", evm),
        
        pair!("solana_testnet:usdc", "50000000", sol, "starknet_sepolia:wbtc", stark),
        pair!("solana_testnet:usdc", "50000000", sol, "tron_shasta:usdt", tron),
        pair!("solana_testnet:usdc", "15000000", sol, "xrpl_testnet:xrp", evm),
        
        // Native token swap (SOL to ETH) - minimum: 0.1 SOL (100000000 lamports)
        pair!("solana_testnet:sol", "100000000", sol, "ethereum_sepolia:eth", evm),
        
        pair!("starknet_sepolia:wbtc", "50000", stark, "tron_shasta:usdt", tron),
        pair!("starknet_sepolia:wbtc", "50000", stark, "xrpl_testnet:xrp", evm),
        
        pair!("tron_shasta:usdt", "15000000", tron, "xrpl_testnet:xrp", evm),
        pair!("tron_shasta:wbtc", "50000", tron, "xrpl_testnet:xrp", evm),
    ];

    // Check if round-trip mode is enabled
    let enable_round_trips = std::env::var("ENABLE_ROUND_TRIPS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    if enable_round_trips {
        // Add round-trip pairs for continuous testing (swap back to maintain balances)
        // Focus on working chains: Arbitrum, Base, Ethereum, Solana
        // Using minimum amounts: USDC 15M, WBTC 50k sats, ETH 0.005
        let round_trips = vec![
            // Native token round-trip (SOL back to ETH)
            pair!("solana_testnet:sol", "100000000", sol, "ethereum_sepolia:eth", evm),
            
            // USDC round-trips between working EVM chains (15 USDC minimum)
            pair!("base_sepolia:usdc", "15000000", evm, "arbitrum_sepolia:usdc", evm),
            pair!("ethereum_sepolia:usdc", "15000000", evm, "arbitrum_sepolia:usdc", evm),
            pair!("arbitrum_sepolia:usdc", "15000000", evm, "ethereum_sepolia:usdc", evm),
            
            // Solana USDC round-trips (15 USDC minimum)
            pair!("solana_testnet:usdc", "15000000", sol, "arbitrum_sepolia:usdc", evm),
            pair!("solana_testnet:usdc", "15000000", sol, "base_sepolia:usdc", evm),
            pair!("solana_testnet:usdc", "15000000", sol, "ethereum_sepolia:usdc", evm),
            
            // WBTC round-trips (50k sats minimum)
            pair!("ethereum_sepolia:wbtc", "50000", evm, "arbitrum_sepolia:wbtc", evm),
            pair!("base_sepolia:wbtc", "50000", evm, "arbitrum_sepolia:wbtc", evm),
            
            // Bitcoin round-trips (50k sats minimum - to get BTC back from other chains)
            pair!("arbitrum_sepolia:wbtc", "50000", evm, "bitcoin_testnet:btc", btc),
            pair!("base_sepolia:wbtc", "50000", evm, "bitcoin_testnet:btc", btc),
            pair!("ethereum_sepolia:wbtc", "50000", evm, "bitcoin_testnet:btc", btc),
        ];

        let round_trip_count = round_trips.len();
        pairs.extend(round_trips);
        eprintln!("🔄 Round-trip mode enabled: {} total pairs (79 original + {} round-trips)", 
            pairs.len(), round_trip_count);
    } else {
        eprintln!("📋 Standard mode: {} pairs (set ENABLE_ROUND_TRIPS=true for round-trips)", pairs.len());
    }

    pairs
}

pub fn requires_manual_deposit(asset: &str) -> bool {
    asset.starts_with("bitcoin_") || asset.starts_with("litecoin_")
}
