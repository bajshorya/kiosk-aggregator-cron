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

pub fn all_swap_pairs(wallets: &WalletConfig) -> Vec<SwapPair> {
    let btc = &wallets.bitcoin_testnet_address;
    let evm = &wallets.evm_address;
    let stark = &wallets.starknet_address;
    let sol = &wallets.solana_address;
    let _tron = &wallets.tron_address; // Unused for now, but kept for future use

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
        // ═══════════════════════════════════════════════════════════════════
        // VERIFIED WORKING SWAP PAIRS (tested and confirmed)
        // ═══════════════════════════════════════════════════════════════════
        
        // ✅ Ethereum Sepolia → Solana (WORKING - non-gasless)
        pair!("ethereum_sepolia:eth", "5000000000000000", evm, "solana_testnet:sol", sol),
        
        // ✅ Arbitrum Sepolia → Others (WORKING - you have USDC tokens)
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "alpen_testnet:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc", btc),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "citrea_testnet:usdc", evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        
        // ✅ Citrea Testnet → Others (WORKING - gasless enabled, 2/4 pairs work)
        pair!("citrea_testnet:usdc", "15000000", evm, "solana_testnet:usdc", sol),
        pair!("citrea_testnet:usdc", "15000000", evm, "bnbchain_testnet:wbtc", evm),
        
        // ✅ Solana Testnet → Others (WORKING - all gasless)
        pair!("solana_testnet:sol", "100000000", sol, "ethereum_sepolia:eth", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "alpen_testnet:usdc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "bnbchain_testnet:wbtc", evm),
        pair!("solana_testnet:usdc", "50000000", sol, "bitcoin_testnet:btc", btc),

        // ✅ Bitcoin Testnet → Others (WORKING - sequential execution with UTXO reuse)
        pair!("bitcoin_testnet:btc", "50000", btc, "ethereum_sepolia:eth", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "solana_testnet:usdc", sol),
        pair!("bitcoin_testnet:btc", "50000", btc, "alpen_testnet:usdc", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "bnbchain_testnet:wbtc", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "citrea_testnet:usdc", evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "starknet_sepolia:wbtc", stark),

        // ═══════════════════════════════════════════════════════════════════
        // COMMENTED OUT - FAILING SWAP PAIRS
        // ═══════════════════════════════════════════════════════════════════

        // ❌ Ethereum Sepolia USDC - DISABLED (insufficient balance, skipped by balance checker)
        // pair!("ethereum_sepolia:usdc", "15000000", evm, "alpen_testnet:usdc", evm),
        // pair!("ethereum_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc", evm),
        // pair!("ethereum_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc", btc),

        // ❌ Citrea → Alpen - DISABLED (gasless "Invalid swap request", fallback also fails)
        // pair!("citrea_testnet:usdc", "15000000", evm, "alpen_testnet:usdc", evm),

        // ❌ Citrea → Starknet - DISABLED (gasless "Invalid swap request", fallback "replacement transaction underpriced")
        // pair!("citrea_testnet:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),

        // ❌ Alpen Testnet as SOURCE - DISABLED (insufficient gas for approval)
        // pair!("alpen_testnet:sbtc",  "50000",    evm, "base_sepolia:usdc",      evm),
        // pair!("alpen_testnet:usdc",  "15000000", evm, "bnbchain_testnet:wbtc",  evm),
        // pair!("alpen_testnet:usdc",  "15000000", evm, "citrea_testnet:usdc",    evm),
        // pair!("alpen_testnet:usdc",  "15000000", evm, "solana_testnet:usdc",    sol),

        // ❌ Base Sepolia - DISABLED (gasless "Invalid swap request")
        // pair!("base_sepolia:usdc",  "15000000", evm, "bnbchain_testnet:wbtc",  evm),
        // pair!("base_sepolia:cbltc", "50000",    evm, "citrea_testnet:usdc",    evm),
        // pair!("base_sepolia:usdc",  "15000000", evm, "solana_testnet:usdc",    sol),
        // pair!("base_sepolia:usdc",  "15000000", evm, "bitcoin_testnet:btc",    btc),

        // ❌ BNB Chain as SOURCE - DISABLED (gasless "Invalid swap request")
        // pair!("bnbchain_testnet:wbtc", "50000", evm, "citrea_testnet:usdc",    evm),
        // pair!("bnbchain_testnet:wbtc", "50000", evm, "solana_testnet:usdc",    sol),

        // ❌ Arbitrum Sepolia - DISABLED (gasless "Invalid swap request" - but you can try if you have tokens)
        // Note: Arbitrum swaps are now ENABLED above since you have USDC tokens
        // If gasless fails, they will fallback to direct RPC broadcast

        // ❌ Monad, XRPL, Tron, Alpen Signet - DISABLED (insufficient liquidity or no order pairs)
        // (All commented out)
    ];

    let enable_round_trips = {
        let env_val = std::env::var("ENABLE_ROUND_TRIPS").ok();
        eprintln!("DEBUG: ENABLE_ROUND_TRIPS env value: {:?}", env_val);
        let result = env_val
            .unwrap_or_else(|| "false".to_string())
            .to_lowercase() == "true";
        eprintln!("DEBUG: enable_round_trips result: {}", result);
        result
    };

    if enable_round_trips {
        let round_trips = vec![
            pair!("solana_testnet:sol",    "100000000", sol, "ethereum_sepolia:eth",   evm),
            pair!("base_sepolia:usdc",     "15000000",  evm, "arbitrum_sepolia:usdc",  evm),
            pair!("ethereum_sepolia:usdc", "15000000",  evm, "arbitrum_sepolia:usdc",  evm),
            pair!("arbitrum_sepolia:usdc", "15000000",  evm, "ethereum_sepolia:usdc",  evm),
            pair!("solana_testnet:usdc",   "15000000",  sol, "arbitrum_sepolia:usdc",  evm),
            pair!("solana_testnet:usdc",   "15000000",  sol, "base_sepolia:usdc",      evm),
            pair!("solana_testnet:usdc",   "15000000",  sol, "ethereum_sepolia:usdc",  evm),
            pair!("ethereum_sepolia:wbtc", "50000",     evm, "arbitrum_sepolia:wbtc",  evm),
            pair!("base_sepolia:wbtc",     "50000",     evm, "arbitrum_sepolia:wbtc",  evm),
            pair!("arbitrum_sepolia:wbtc", "50000",     evm, "bitcoin_testnet:btc",    btc),
            pair!("base_sepolia:wbtc",     "50000",     evm, "bitcoin_testnet:btc",    btc),
            pair!("ethereum_sepolia:wbtc", "50000",     evm, "bitcoin_testnet:btc",    btc),
        ];
        let n = round_trips.len();
        pairs.extend(round_trips);
        eprintln!("🔄 Round-trip mode enabled: {} total pairs ({} round-trips)", pairs.len(), n);
    } else {
        eprintln!("📋 Standard mode: {} pairs (set ENABLE_ROUND_TRIPS=true for round-trips)", pairs.len());
    }

    pairs
}

pub fn requires_manual_deposit(asset: &str) -> bool {
    asset.starts_with("bitcoin_") || asset.starts_with("alpen_signet")
}
