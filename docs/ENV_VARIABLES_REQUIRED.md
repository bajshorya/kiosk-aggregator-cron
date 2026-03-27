# Required Environment Variables for All Swap Pairs

## Summary
To run all 106 swap pairs with Solana orchestration, you need to configure wallet addresses and RPC endpoints for all supported chains.

## Total Swap Pairs: 106
- **DISTRIBUTE**: 13 pairs (Solana → All chains)
- **TEST**: 80 pairs (Cross-chain swaps)
- **CONSOLIDATE**: 13 pairs (All chains → Solana with 10% extra for gas)

---

## Wallet Addresses Required

### Already Configured ✅
```bash
# EVM Chains (Ethereum, Base, Arbitrum, BNB, Citrea, Monad)
WALLET_EVM=0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406
WALLET_EVM_PRIVATE_KEY=92796a4469a152563fa7790aca17caad6ecdeea7c20740e06538de01f3a64566

# Solana
WALLET_SOLANA=5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny
SOLANA_PRIVATE_KEY=3Nb6qpea1cgbCVqYAGPMJZqg4KXd9BSgnY9shsXYYoBFEduSFtJifoJs3cWznouL3q3isMhVW3kt4ntDcaZijEJM

# Bitcoin Testnet
WALLET_BITCOIN_TESTNET=tb1qypyl87pagnmcted8d8gnmxxps7dyqstllhhe9z

# Starknet
WALLET_STARKNET=0x00609190b1348bcc06da44d58c79709495c11a5a6f0b9e154e1209f2a17dd933
STARKNET_PRIVATE_KEY=0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# Tron
WALLET_TRON=TWbEz5ibiL6dreiLJ5oBF5CwDkw6Xfe6KX
TRON_PRIVATE_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# Sui
WALLET_SUI=0x60408691622dd5d95a4ee8d149fb0803f2be062f255f285c64e44a6695ac76aa
SUI_PRIVATE_KEY=suiprivkey1qzpt6km8jp5hhykmexed683ehqyk5thu6a3vw2k7ehlpj9s7kteq7fsct2f
```

### Need to Add ⚠️
```bash
# XRP Ledger Testnet
WALLET_XRPL=rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY
XRPL_PRIVATE_KEY=<your_xrpl_private_key_here>

# Alpen Network (Bitcoin Layer 2)
WALLET_ALPEN=tb1qalpen1234567890abcdefghijklmnopqrstuvwxyz
ALPEN_PRIVATE_KEY=<your_alpen_private_key_here>
```

---

## RPC Endpoints Required

### Already Configured ✅
```bash
# Ethereum Sepolia
RPC_ETHEREUM_SEPOLIA=https://ethereum-sepolia-rpc.publicnode.com

# Base Sepolia
RPC_BASE_SEPOLIA=https://sepolia.base.org

# Arbitrum Sepolia
RPC_ARBITRUM_SEPOLIA=https://sepolia-rollup.arbitrum.io/rpc

# Solana Devnet
RPC_SOLANA_TESTNET=https://api.devnet.solana.com

# Starknet Sepolia
RPC_STARKNET_SEPOLIA=https://starknet-sepolia.public.blastapi.io

# Tron Shasta
RPC_TRON_SHASTA=https://api.shasta.trongrid.io

# Sui Testnet
RPC_SUI_TESTNET=https://fullnode.testnet.sui.io:443

# Bitcoin Testnet
RPC_BITCOIN_TESTNET=https://blockstream.info/testnet/api
```

### Need to Add ⚠️
```bash
# BNB Chain Testnet
RPC_BNBCHAIN_TESTNET=https://data-seed-prebsc-1-s1.binance.org:8545

# Citrea Testnet (Bitcoin ZK Rollup)
RPC_CITREA_TESTNET=https://rpc.testnet.citrea.xyz

# Monad Testnet
RPC_MONAD_TESTNET=https://testnet.monad.xyz

# XRP Ledger Testnet
RPC_XRPL_TESTNET=https://s.altnet.rippletest.net:51234

# Alpen Testnet
RPC_ALPEN_TESTNET=https://rpc.testnet.alpen.network

# Alpen Signet
RPC_ALPEN_SIGNET=https://rpc.signet.alpen.network
```

---

## Complete .env Template

Add these to your `.env` file:

```bash
# ═══════════════════════════════════════════════════════════════
# NEW WALLET ADDRESSES (Add these)
# ═══════════════════════════════════════════════════════════════

# XRP Ledger Testnet Address (r...)
WALLET_XRPL=rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY

# Alpen Network Address (tb1... for testnet)
WALLET_ALPEN=tb1qalpen1234567890abcdefghijklmnopqrstuvwxyz

# ═══════════════════════════════════════════════════════════════
# NEW PRIVATE KEYS (Add these - Keep secret!)
# ═══════════════════════════════════════════════════════════════

# XRP Ledger Private Key (hex format)
XRPL_PRIVATE_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# Alpen Private Key (hex format)
ALPEN_PRIVATE_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

# ═══════════════════════════════════════════════════════════════
# NEW RPC ENDPOINTS (Add these)
# ═══════════════════════════════════════════════════════════════

# BNB Chain Testnet RPC
RPC_BNBCHAIN_TESTNET=https://data-seed-prebsc-1-s1.binance.org:8545

# Citrea Testnet RPC (Bitcoin ZK Rollup)
RPC_CITREA_TESTNET=https://rpc.testnet.citrea.xyz

# Monad Testnet RPC
RPC_MONAD_TESTNET=https://testnet.monad.xyz

# XRP Ledger Testnet RPC
RPC_XRPL_TESTNET=https://s.altnet.rippletest.net:51234

# Alpen Testnet RPC
RPC_ALPEN_TESTNET=https://rpc.testnet.alpen.network

# Alpen Signet RPC
RPC_ALPEN_SIGNET=https://rpc.signet.alpen.network
```

---

## How to Get Wallet Addresses

### XRP Ledger (XRPL)
1. Use XRP Testnet Faucet: https://xrpl.org/xrp-testnet-faucet.html
2. Or create wallet with xrpl.js library
3. Format: Starts with 'r' (e.g., rPEPPER7kfTD9w2To4CQk6UCfuHM9c6GDY)

### Alpen Network
1. Visit Alpen Network documentation: https://docs.alpen.network
2. Create testnet wallet
3. Format: Bitcoin testnet address (tb1...)

### BNB Chain, Citrea, Monad
- Use same EVM address as Ethereum (WALLET_EVM)
- These are EVM-compatible chains

---

## Swap Amount Details

### Standard Amounts (50 units)
- USDC: 50,000,000 (50 USDC with 6 decimals)
- USDT: 50,000,000 (50 USDT with 6 decimals)
- WBTC: 50,000 sats (0.0005 BTC)
- BTC: 50,000 sats (0.0005 BTC)
- XRP: 50,000,000 (50 XRP with 6 decimals)

### Return Amounts (55 units - 10% extra for gas)
- USDC back to Solana: 55,000,000 (55 USDC)
- WBTC back to Solana: 55,000 sats (0.00055 BTC)
- USDT back to Solana: 55,000,000 (55 USDT)
- XRP back to Solana: 55,000,000 (55 XRP)

The extra 10% ensures there's enough to cover gas fees when consolidating back to Solana.

---

## Chain Support Status

| Chain | Status | Signer | Notes |
|-------|--------|--------|-------|
| Solana | ✅ Working | Yes | Hub chain, gasless |
| Ethereum | ✅ Working | Yes | EVM, gasless |
| Base | ✅ Working | Yes | EVM, gasless |
| Arbitrum | ✅ Working | Yes | EVM, gasless |
| Sui | ✅ Working | Yes | Gasless |
| Bitcoin | ⚠️ Manual | Partial | Manual deposit required |
| Tron | ⚠️ Pending | Yes | Needs integration |
| Starknet | ⚠️ Pending | Yes | Needs integration |
| BNB Chain | ✅ Working | Yes | EVM, uses WALLET_EVM |
| Citrea | ✅ Working | Yes | EVM, uses WALLET_EVM |
| Monad | ✅ Working | Yes | EVM, uses WALLET_EVM |
| XRPL | ⚠️ Pending | No | Needs signer implementation |
| Alpen | ⚠️ Pending | No | Needs signer implementation |

---

## Priority Order for Adding Chains

### High Priority (EVM chains - use existing signer)
1. ✅ BNB Chain - Just add RPC URL
2. ✅ Citrea - Just add RPC URL
3. ✅ Monad - Just add RPC URL

### Medium Priority (Signers exist, need integration)
4. ⚠️ Tron - Integrate existing signer
5. ⚠️ Starknet - Integrate existing signer

### Low Priority (Need new signers)
6. ⚠️ XRPL - Create signer + integrate
7. ⚠️ Alpen - Create signer + integrate

---

## Testing Strategy

### Phase 1: Test EVM Chains (BNB, Citrea, Monad)
```bash
# Add RPC URLs to .env
# Test swaps
cargo run -- test-swap "solana_testnet:usdc" "bnbchain_testnet:wbtc"
cargo run -- test-swap "solana_testnet:usdc" "citrea_testnet:usdc"
cargo run -- test-swap "solana_testnet:usdc" "monad_testnet:usdc"
```

### Phase 2: Integrate Tron & Starknet
```bash
# Complete signer integration
# Test swaps
cargo run -- test-swap "solana_testnet:usdc" "tron_shasta:usdt"
cargo run -- test-swap "solana_testnet:usdc" "starknet_sepolia:wbtc"
```

### Phase 3: Add XRPL & Alpen
```bash
# Create signers
# Add wallet addresses
# Test swaps
cargo run -- test-swap "solana_testnet:usdc" "xrpl_testnet:xrp"
cargo run -- test-swap "solana_testnet:usdc" "alpen_testnet:usdc"
```

---

## Cost Estimate

### Per Swap
- Minimum: 50 USDC/WBTC/XRP = ~$50 per swap
- Return: 55 USDC/WBTC/XRP = ~$55 per swap

### Full Cycle (106 swaps)
- **DISTRIBUTE**: 13 × $50 = $650
- **TEST**: 80 × $50 = $4,000
- **CONSOLIDATE**: 13 × $55 = $715
- **TOTAL**: ~$5,365 per complete test cycle

### Working Chains Only (EVM + Solana + Sui)
- Approximately 40-50 swaps
- Cost: ~$2,000-2,500 per cycle

---

## Summary

### To Add Now:
1. ✅ BNB Chain RPC URL (EVM - works immediately)
2. ✅ Citrea RPC URL (EVM - works immediately)
3. ✅ Monad RPC URL (EVM - works immediately)

### To Add Later:
4. ⚠️ XRPL wallet address + private key + RPC (needs signer)
5. ⚠️ Alpen wallet address + private key + RPC (needs signer)
6. ⚠️ Complete Tron integration (signer exists)
7. ⚠️ Complete Starknet integration (signer exists)

### Quick Win:
Add the 3 EVM chain RPC URLs to immediately enable 30+ additional swap pairs!
