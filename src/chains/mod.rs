pub mod evm_signer;
pub mod solana_signer;
pub mod starknet_signer;
pub mod tron_signer;
pub mod sui_signer;
pub mod balance_checker;
pub mod bitcoin_signer;
pub mod bitcoin_provider;

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
    let _btc = &wallets.bitcoin_testnet_address;
    let _ltc = &wallets.litecoin_testnet_address;
    let evm = &wallets.evm_address;
    let _stark = &wallets.starknet_address;
    let sol = &wallets.solana_address;
    let _tron = &wallets.tron_address;
    let sui = &wallets.sui_address;
    let _xrpl = &wallets.xrpl_address;
    let _alpen = &wallets.alpen_address;

    // Minimum amounts based on Garden API requirements:
    // - BTC/WBTC: 50,000 sats (0.0005 BTC) = ~$50
    // - USDC/USDT: 50,000,000 (50 with 6 decimals) = ~$50
    // - ETH: 5,000,000,000,000,000 wei (0.005 ETH) = ~$15-20
    // - SOL: 100,000,000 lamports (0.1 SOL) = ~$20-30
    // - XRP: 50,000,000 (50 XRP with 6 decimals) = ~$100
    
    // Extra amounts for return swaps to cover gas fees:
    // - USDC back to Solana: 55 USDC (10% extra for gas)
    // - WBTC back to Solana: 55,000 sats (10% extra)

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

    // ═══════════════════════════════════════════════════════════════
    // SOLANA-CENTRIC SWAP ORCHESTRATION (COMPREHENSIVE)
    // ═══════════════════════════════════════════════════════════════
    // Strategy: Use Solana as liquidity hub for continuous testing
    // 
    // Phase 1: DISTRIBUTE - Swap from Solana to ALL other chains
    // Phase 2: TEST - Cross-chain swaps between distributed chains  
    // Phase 3: CONSOLIDATE - Swap everything back to Solana (with extra for gas)
    //
    // This allows continuous testing with the same initial liquidity
    // ═══════════════════════════════════════════════════════════════

    let pairs = vec![
        // ═══════════════════════════════════════════════════════════════
        // PHASE 1: DISTRIBUTE LIQUIDITY (Solana → All Chains)
        // ═══════════════════════════════════════════════════════════════
        
        // Solana → EVM chains (USDC)
        pair!("solana_testnet:usdc", "50000000", sol, "ethereum_sepolia:usdc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "base_sepolia:usdc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "arbitrum_sepolia:usdc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "bnbchain_testnet:wbtc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "citrea_testnet:usdc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "monad_testnet:usdc", evm),
        
        // Solana → Non-EVM chains
        pair!("solana_testnet:usdc", "50000000", sol, "tron_shasta:usdt", _tron),
        pair!("solana_testnet:usdc", "50000000", sol, "starknet_sepolia:wbtc", _stark),
        pair!("solana_testnet:usdc", "50000000", sol, "xrpl_testnet:xrp", _xrpl),
        pair!("solana_testnet:usdc", "50000000", sol, "alpen_testnet:usdc", _alpen),
        
        // Solana → Bitcoin chains (manual deposit required)
        pair!("solana_testnet:usdc", "50000000", sol, "bitcoin_testnet:btc", _btc),
        pair!("solana_testnet:usdc", "50000000", sol, "alpen_signet:btc", _alpen),
        
        // ═══════════════════════════════════════════════════════════════
        // PHASE 2: TEST SWAPS (Cross-Chain Between Distributed Chains)
        // ═══════════════════════════════════════════════════════════════
        
        // Arbitrum → Other chains
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "ethereum_sepolia:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "base_sepolia:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "citrea_testnet:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "monad_testnet:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "starknet_sepolia:wbtc", _stark),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "tron_shasta:usdt", _tron),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "xrpl_testnet:xrp", _xrpl),
        pair!("arbitrum_sepolia:usdc", "50000000", evm, "bitcoin_testnet:btc", _btc),
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "ethereum_sepolia:wbtc", evm),
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "alpen_signet:btc", _alpen),
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "alpen_testnet:sbtc", _alpen),
        
        // Ethereum → Other chains
        pair!("ethereum_sepolia:usdc", "50000000", evm, "base_sepolia:usdc", evm),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "arbitrum_sepolia:usdc", evm),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "citrea_testnet:usdc", evm),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "monad_testnet:usdc", evm),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "starknet_sepolia:wbtc", _stark),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "tron_shasta:usdt", _tron),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "xrpl_testnet:xrp", _xrpl),
        pair!("ethereum_sepolia:usdc", "50000000", evm, "bitcoin_testnet:btc", _btc),
        pair!("ethereum_sepolia:wbtc", "50000", evm, "alpen_testnet:usdc", _alpen),
        
        // Base → Other chains
        pair!("base_sepolia:usdc", "50000000", evm, "ethereum_sepolia:usdc", evm),
        pair!("base_sepolia:usdc", "50000000", evm, "arbitrum_sepolia:usdc", evm),
        pair!("base_sepolia:usdc", "50000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("base_sepolia:usdc", "50000000", evm, "monad_testnet:usdc", evm),
        pair!("base_sepolia:usdc", "50000000", evm, "starknet_sepolia:wbtc", _stark),
        pair!("base_sepolia:usdc", "50000000", evm, "tron_shasta:usdt", _tron),
        pair!("base_sepolia:usdc", "50000000", evm, "xrpl_testnet:xrp", _xrpl),
        pair!("base_sepolia:usdc", "50000000", evm, "bitcoin_testnet:btc", _btc),
        pair!("base_sepolia:cbltc", "50000", evm, "citrea_testnet:usdc", evm),
        
        // Alpen Signet → Other chains
        pair!("alpen_signet:btc", "50000", _alpen, "alpen_testnet:sbtc", _alpen),
        pair!("alpen_signet:btc", "50000", _alpen, "base_sepolia:usdc", evm),
        pair!("alpen_signet:btc", "50000", _alpen, "bitcoin_testnet:btc", _btc),
        pair!("alpen_signet:btc", "50000", _alpen, "bnbchain_testnet:wbtc", evm),
        pair!("alpen_signet:btc", "50000", _alpen, "citrea_testnet:usdc", evm),
        pair!("alpen_signet:btc", "50000", _alpen, "monad_testnet:usdc", evm),
        pair!("alpen_signet:btc", "50000", _alpen, "starknet_sepolia:wbtc", _stark),
        pair!("alpen_signet:btc", "50000", _alpen, "tron_shasta:usdt", _tron),
        pair!("alpen_signet:btc", "50000", _alpen, "xrpl_testnet:xrp", _xrpl),
        
        // Alpen Testnet → Other chains
        pair!("alpen_testnet:sbtc", "50000", _alpen, "base_sepolia:usdc", evm),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "bitcoin_testnet:btc", _btc),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "bnbchain_testnet:wbtc", evm),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "citrea_testnet:usdc", evm),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "monad_testnet:usdc", evm),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "starknet_sepolia:wbtc", _stark),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "tron_shasta:usdt", _tron),
        pair!("alpen_testnet:usdc", "50000000", _alpen, "xrpl_testnet:xrp", _xrpl),
        
        // Bitcoin → Other chains
        pair!("bitcoin_testnet:btc", "50000", _btc, "bnbchain_testnet:wbtc", evm),
        pair!("bitcoin_testnet:btc", "50000", _btc, "citrea_testnet:usdc", evm),
        pair!("bitcoin_testnet:btc", "50000", _btc, "monad_testnet:usdc", evm),
        pair!("bitcoin_testnet:btc", "50000", _btc, "starknet_sepolia:wbtc", _stark),
        pair!("bitcoin_testnet:btc", "50000", _btc, "tron_shasta:usdt", _tron),
        pair!("bitcoin_testnet:btc", "50000", _btc, "xrpl_testnet:xrp", _xrpl),
        
        // BNB Chain → Other chains
        pair!("bnbchain_testnet:wbtc", "50000", evm, "citrea_testnet:usdc", evm),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "monad_testnet:usdc", evm),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "starknet_sepolia:wbtc", _stark),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "tron_shasta:usdt", _tron),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "xrpl_testnet:xrp", _xrpl),
        
        // Citrea → Other chains
        pair!("citrea_testnet:usdc", "50000000", evm, "monad_testnet:usdc", evm),
        pair!("citrea_testnet:usdc", "50000000", evm, "starknet_sepolia:wbtc", _stark),
        pair!("citrea_testnet:usdc", "50000000", evm, "tron_shasta:usdt", _tron),
        pair!("citrea_testnet:usdc", "50000000", evm, "xrpl_testnet:xrp", _xrpl),
        
        // Monad → Other chains
        pair!("monad_testnet:usdc", "50000000", evm, "starknet_sepolia:wbtc", _stark),
        pair!("monad_testnet:usdc", "50000000", evm, "tron_shasta:usdt", _tron),
        pair!("monad_testnet:usdc", "50000000", evm, "xrpl_testnet:xrp", _xrpl),
        
        // Starknet → Other chains
        pair!("starknet_sepolia:wbtc", "50000", _stark, "tron_shasta:usdt", _tron),
        pair!("starknet_sepolia:wbtc", "50000", _stark, "xrpl_testnet:xrp", _xrpl),
        
        // Tron → Other chains
        pair!("tron_shasta:usdt", "50000000", _tron, "xrpl_testnet:xrp", _xrpl),
        pair!("tron_shasta:wbtc", "50000", _tron, "xrpl_testnet:xrp", _xrpl),
        
        // ═══════════════════════════════════════════════════════════════
        // PHASE 3: CONSOLIDATE LIQUIDITY (All Chains → Solana)
        // ═══════════════════════════════════════════════════════════════
        // Return all liquidity back to Solana with EXTRA for gas fees
        
        // EVM chains → Solana (55 USDC = 10% extra for gas)
        pair!("ethereum_sepolia:usdc", "55000000", evm, "solana_testnet:usdc", sol),
        pair!("base_sepolia:usdc", "55000000", evm, "solana_testnet:usdc", sol),
        pair!("arbitrum_sepolia:usdc", "55000000", evm, "solana_testnet:usdc", sol),
        pair!("bnbchain_testnet:wbtc", "55000", evm, "solana_testnet:usdc", sol),
        pair!("citrea_testnet:usdc", "55000000", evm, "solana_testnet:usdc", sol),
        pair!("monad_testnet:usdc", "55000000", evm, "solana_testnet:usdc", sol),
        
        // Non-EVM chains → Solana (55 USDC/WBTC = 10% extra for gas)
        pair!("tron_shasta:usdt", "55000000", _tron, "solana_testnet:usdc", sol),
        pair!("starknet_sepolia:wbtc", "55000", _stark, "solana_testnet:usdc", sol),
        pair!("xrpl_testnet:xrp", "55000000", _xrpl, "solana_testnet:usdc", sol),
        pair!("alpen_testnet:usdc", "55000000", _alpen, "solana_testnet:usdc", sol),
        
        // Bitcoin chains → Solana (55,000 sats = 10% extra for gas)
        pair!("bitcoin_testnet:btc", "55000", _btc, "solana_testnet:usdc", sol),
        pair!("alpen_signet:btc", "55000", _alpen, "solana_testnet:usdc", sol),
    ];

    // Check if round-trip mode is enabled
    let enable_round_trips = std::env::var("ENABLE_ROUND_TRIPS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

    if enable_round_trips {
        eprintln!("🔄 Round-trip mode enabled: {} total pairs", pairs.len());
    } else {
        eprintln!("📋 Solana-centric mode: {} pairs (DISTRIBUTE → TEST → CONSOLIDATE)", pairs.len());
        eprintln!("   Chains: Solana (hub), EVM (6), Tron, Starknet, XRP, Alpen, Bitcoin");
        eprintln!("   💡 Return swaps include 10% extra to cover gas fees");
    }

    pairs
}

pub fn requires_manual_deposit(asset: &str) -> bool {
    asset.starts_with("bitcoin_") || asset.starts_with("litecoin_")
}
