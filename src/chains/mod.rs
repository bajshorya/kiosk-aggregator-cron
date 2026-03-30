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
    let tron = &wallets.tron_address;
    let _sui = &wallets.sui_address;

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
        // ── ETH native ──────────────────────────────────────────────────────
        pair!("ethereum_sepolia:eth", "5000000000000000", evm, "solana_testnet:sol", sol),

        // ── Arbitrum WBTC ───────────────────────────────────────────────────
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "ethereum_sepolia:wbtc", evm),
        // alpen_signet is a Bitcoin-style chain → btc destination owner
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "alpen_signet:btc",    btc),
        // alpen_testnet is an EVM chain → evm destination owner
        pair!("arbitrum_sepolia:wbtc", "50000", evm, "alpen_testnet:sbtc",  evm),

        // ── Arbitrum USDC ───────────────────────────────────────────────────
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "base_sepolia:usdc",       evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc",     btc),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc",   evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "citrea_testnet:usdc",     evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "monad_testnet:usdc",      evm),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "solana_testnet:usdc",     sol),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "starknet_sepolia:wbtc",   stark),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "tron_shasta:usdt",        tron),
        pair!("arbitrum_sepolia:usdc", "15000000", evm, "xrpl_testnet:xrp",        evm),

        // ── Ethereum WBTC / USDC ─────────────────────────────────────────────
        // alpen_testnet destination is EVM
        pair!("ethereum_sepolia:wbtc", "50000",    evm, "alpen_testnet:usdc",      evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "base_sepolia:usdc",       evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "bitcoin_testnet:btc",     btc),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "bnbchain_testnet:wbtc",   evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "citrea_testnet:usdc",     evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "monad_testnet:usdc",      evm),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "solana_testnet:usdc",     sol),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "starknet_sepolia:wbtc",   stark),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "tron_shasta:usdt",        tron),
        pair!("ethereum_sepolia:usdc", "15000000", evm, "xrpl_testnet:xrp",        evm),

        // ── Alpen Signet (Bitcoin-style chain, btc owner) ────────────────────
        // alpen_testnet:sbtc destination is EVM
        pair!("alpen_signet:btc", "50000", btc, "alpen_testnet:sbtc",      evm),
        pair!("alpen_signet:btc", "50000", btc, "base_sepolia:usdc",       evm),
        pair!("alpen_signet:btc", "50000", btc, "bitcoin_testnet:btc",     btc),
        pair!("alpen_signet:btc", "50000", btc, "bnbchain_testnet:wbtc",   evm),
        pair!("alpen_signet:btc", "50000", btc, "citrea_testnet:usdc",     evm),
        pair!("alpen_signet:btc", "50000", btc, "monad_testnet:usdc",      evm),
        pair!("alpen_signet:btc", "50000", btc, "solana_testnet:usdc",     sol),
        pair!("alpen_signet:btc", "50000", btc, "starknet_sepolia:wbtc",   stark),
        pair!("alpen_signet:btc", "50000", btc, "tron_shasta:usdt",        tron),
        pair!("alpen_signet:btc", "50000", btc, "xrpl_testnet:xrp",        evm),

        // ── Alpen Testnet (EVM chain, evm owner) ─────────────────────────────
        pair!("alpen_testnet:sbtc",  "50000",    evm, "base_sepolia:usdc",      evm),
        pair!("alpen_testnet:usdc",  "15000000", evm, "bitcoin_testnet:btc",    btc),
        pair!("alpen_testnet:usdc",  "15000000", evm, "bnbchain_testnet:wbtc",  evm),
        pair!("alpen_testnet:usdc",  "15000000", evm, "citrea_testnet:usdc",    evm),
        pair!("alpen_testnet:usdc",  "15000000", evm, "monad_testnet:usdc",     evm),
        pair!("alpen_testnet:usdc",  "15000000", evm, "solana_testnet:usdc",    sol),
        pair!("alpen_testnet:usdc",  "15000000", evm, "starknet_sepolia:wbtc",  stark),
        pair!("alpen_testnet:usdc",  "15000000", evm, "tron_shasta:usdt",       tron),
        pair!("alpen_testnet:usdc",  "15000000", evm, "xrpl_testnet:xrp",       evm),

        // ── Base Sepolia ─────────────────────────────────────────────────────
        pair!("base_sepolia:usdc",  "15000000", evm, "bitcoin_testnet:btc",    btc),
        pair!("base_sepolia:usdc",  "15000000", evm, "bnbchain_testnet:wbtc",  evm),
        pair!("base_sepolia:cbltc", "50000",    evm, "citrea_testnet:usdc",    evm),
        pair!("base_sepolia:usdc",  "15000000", evm, "monad_testnet:usdc",     evm),
        pair!("base_sepolia:usdc",  "15000000", evm, "solana_testnet:usdc",    sol),
        pair!("base_sepolia:usdc",  "15000000", evm, "starknet_sepolia:wbtc",  stark),
        pair!("base_sepolia:usdc",  "15000000", evm, "tron_shasta:usdt",       tron),
        pair!("base_sepolia:usdc",  "15000000", evm, "xrpl_testnet:xrp",       evm),

        // ── Bitcoin Testnet ──────────────────────────────────────────────────
        pair!("bitcoin_testnet:btc", "50000", btc, "bnbchain_testnet:wbtc",  evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "citrea_testnet:usdc",    evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "monad_testnet:usdc",     evm),
        pair!("bitcoin_testnet:btc", "50000", btc, "solana_testnet:usdc",    sol),
        pair!("bitcoin_testnet:btc", "50000", btc, "starknet_sepolia:wbtc",  stark),
        pair!("bitcoin_testnet:btc", "50000", btc, "tron_shasta:usdt",       tron),
        pair!("bitcoin_testnet:btc", "50000", btc, "xrpl_testnet:xrp",       evm),

        // ── BNB Chain Testnet ────────────────────────────────────────────────
        pair!("bnbchain_testnet:wbtc", "50000", evm, "citrea_testnet:usdc",    evm),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "monad_testnet:usdc",     evm),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "solana_testnet:usdc",    sol),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "starknet_sepolia:wbtc",  stark),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "tron_shasta:usdt",       tron),
        pair!("bnbchain_testnet:wbtc", "50000", evm, "xrpl_testnet:xrp",       evm),

        // ── Citrea Testnet ───────────────────────────────────────────────────
        pair!("citrea_testnet:usdc", "15000000", evm, "monad_testnet:usdc",     evm),
        pair!("citrea_testnet:usdc", "15000000", evm, "solana_testnet:usdc",    sol),
        pair!("citrea_testnet:usdc", "15000000", evm, "starknet_sepolia:wbtc",  stark),
        pair!("citrea_testnet:usdc", "15000000", evm, "tron_shasta:usdt",       tron),
        pair!("citrea_testnet:usdc", "15000000", evm, "xrpl_testnet:xrp",       evm),

        // ── Monad Testnet ────────────────────────────────────────────────────
        pair!("monad_testnet:usdc", "15000000", evm, "solana_testnet:usdc",   sol),
        pair!("monad_testnet:usdc", "15000000", evm, "starknet_sepolia:wbtc", stark),
        pair!("monad_testnet:usdc", "15000000", evm, "tron_shasta:usdt",      tron),
        pair!("monad_testnet:usdc", "15000000", evm, "xrpl_testnet:xrp",      evm),

        // ── Solana USDC ──────────────────────────────────────────────────────
        pair!("solana_testnet:usdc", "50000000", sol, "starknet_sepolia:wbtc", stark),
        pair!("solana_testnet:usdc", "50000000", sol, "tron_shasta:usdt",      tron),
        pair!("solana_testnet:usdc", "15000000", sol, "xrpl_testnet:xrp",      evm),

        // ── SOL native ───────────────────────────────────────────────────────
        pair!("solana_testnet:sol", "100000000", sol, "ethereum_sepolia:eth", evm),

        // ── Starknet ─────────────────────────────────────────────────────────
        pair!("starknet_sepolia:wbtc", "50000", stark, "tron_shasta:usdt",  tron),
        pair!("starknet_sepolia:wbtc", "50000", stark, "xrpl_testnet:xrp",  evm),

        // ── Tron ─────────────────────────────────────────────────────────────
        pair!("tron_shasta:usdt", "15000000", tron, "xrpl_testnet:xrp", evm),
        pair!("tron_shasta:wbtc", "50000",    tron, "xrpl_testnet:xrp", evm),
    ];

    let enable_round_trips = std::env::var("ENABLE_ROUND_TRIPS")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase() == "true";

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
    asset.starts_with("bitcoin_")
}