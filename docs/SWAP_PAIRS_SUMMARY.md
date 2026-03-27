# Complete Swap Pairs Summary

## Total: 26 Swap Pairs

### Chains Included
- **Solana Testnet** (Hub chain)
- **Ethereum Sepolia** (EVM)
- **Base Sepolia** (EVM)
- **Arbitrum Sepolia** (EVM)
- **Sui Testnet**
- **Tron Shasta** (Testnet)
- **Starknet Sepolia**

---

## PHASE 1: DISTRIBUTE (7 pairs)
Distribute liquidity from Solana to other chains

1. `solana_testnet:sol → ethereum_sepolia:eth` (100M lamports = 0.1 SOL)
2. `solana_testnet:usdc → ethereum_sepolia:usdc` (50 USDC)
3. `solana_testnet:usdc → base_sepolia:usdc` (50 USDC)
4. `solana_testnet:usdc → arbitrum_sepolia:usdc` (50 USDC)
5. `solana_testnet:usdc → sui_testnet:usdc` (50 USDC)
6. `solana_testnet:usdc → tron_shasta:usdc` (50 USDC) ⚠️
7. `solana_testnet:usdc → starknet_sepolia:usdc` (50 USDC) ⚠️

---

## PHASE 2: TEST (12 pairs)
Cross-chain swaps between distributed chains

### EVM ↔ EVM (6 pairs)
8. `ethereum_sepolia:usdc → base_sepolia:usdc` (50 USDC)
9. `ethereum_sepolia:usdc → arbitrum_sepolia:usdc` (50 USDC)
10. `base_sepolia:usdc → arbitrum_sepolia:usdc` (50 USDC)
11. `arbitrum_sepolia:usdc → base_sepolia:usdc` (50 USDC)
12. `arbitrum_sepolia:usdc → ethereum_sepolia:usdc` (50 USDC)
13. `base_sepolia:usdc → ethereum_sepolia:usdc` (50 USDC)

### Sui → EVM (2 pairs)
14. `sui_testnet:usdc → ethereum_sepolia:usdc` (50 USDC)
15. `sui_testnet:usdc → base_sepolia:usdc` (50 USDC)

### Tron → EVM (2 pairs) ⚠️
16. `tron_shasta:usdc → ethereum_sepolia:usdc` (50 USDC)
17. `tron_shasta:usdc → base_sepolia:usdc` (50 USDC)

### Starknet → EVM (2 pairs) ⚠️
18. `starknet_sepolia:usdc → ethereum_sepolia:usdc` (50 USDC)
19. `starknet_sepolia:usdc → arbitrum_sepolia:usdc` (50 USDC)

---

## PHASE 3: CONSOLIDATE (7 pairs)
Return all liquidity back to Solana

20. `ethereum_sepolia:usdc → solana_testnet:usdc` (50 USDC)
21. `base_sepolia:usdc → solana_testnet:usdc` (50 USDC)
22. `arbitrum_sepolia:usdc → solana_testnet:usdc` (50 USDC)
23. `sui_testnet:usdc → solana_testnet:usdc` (50 USDC)
24. `tron_shasta:usdc → solana_testnet:usdc` (50 USDC) ⚠️
25. `starknet_sepolia:usdc → solana_testnet:usdc` (50 USDC) ⚠️
26. `ethereum_sepolia:eth → solana_testnet:sol` (0.005 ETH)

---

## Implementation Status

### ✅ Fully Implemented (18 pairs)
- Solana swaps (gasless)
- EVM swaps (gasless with EIP-712)
- Sui swaps (gasless with Ed25519)

### ⚠️ Configured but Not Integrated (8 pairs)
- Tron swaps (signer exists, needs runner integration)
- Starknet swaps (signer exists, needs runner integration)

---

## Minimum Amounts

All swaps use API minimum requirements:

| Asset | Amount | Decimals | Display | USD Value |
|-------|--------|----------|---------|-----------|
| USDC | 50,000,000 | 6 | 50 USDC | ~$50 |
| SOL | 100,000,000 | 9 | 0.1 SOL | ~$20-30 |
| ETH | 5,000,000,000,000,000 | 18 | 0.005 ETH | ~$15-20 |

---

## Cost Estimate

### Per Phase
- **DISTRIBUTE**: 7 swaps × $50 = ~$350
- **TEST**: 12 swaps × $50 = ~$600
- **CONSOLIDATE**: 7 swaps × $50 = ~$350

### Total
- **All 26 swaps**: ~$1,300 per complete cycle
- **Working swaps only (18)**: ~$900 per cycle

---

## Testing Strategy

### 1. Test Working Chains First
```bash
# Test Solana → EVM
cargo run -- test-swap "solana_testnet:usdc" "ethereum_sepolia:usdc"

# Test EVM → EVM
cargo run -- test-swap "ethereum_sepolia:usdc" "base_sepolia:usdc"

# Test Sui
cargo run -- test-swap "solana_testnet:usdc" "sui_testnet:usdc"
```

### 2. Run All Working Swaps
```bash
# This will attempt all 26, but Tron/Starknet will fail
cargo run -- run-all
```

### 3. After Tron/Starknet Integration
```bash
# Test Tron
cargo run -- test-swap "solana_testnet:usdc" "tron_shasta:usdc"

# Test Starknet
cargo run -- test-swap "solana_testnet:usdc" "starknet_sepolia:usdc"

# Run all 26 swaps
cargo run -- run-all
```

---

## Liquidity Flow

```
                    ┌─────────────────┐
                    │  SOLANA (HUB)   │
                    │   SOL + USDC    │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
    ┌───────▼──────┐  ┌─────▼─────┐  ┌──────▼──────┐
    │   ETHEREUM   │  │    BASE   │  │  ARBITRUM   │
    │     USDC     │  │    USDC   │  │    USDC     │
    └───────┬──────┘  └─────┬─────┘  └──────┬──────┘
            │                │                │
            └────────────────┼────────────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
    ┌───────▼──────┐  ┌─────▼─────┐  ┌──────▼──────┐
    │     SUI      │  │   TRON ⚠️  │  │ STARKNET ⚠️ │
    │    USDC      │  │   USDC    │  │    USDC     │
    └───────┬──────┘  └─────┬─────┘  └──────┬──────┘
            │                │                │
            └────────────────┼────────────────┘
                             │
                    ┌────────▼────────┐
                    │  SOLANA (HUB)   │
                    │   SOL + USDC    │
                    └─────────────────┘
```

---

## Benefits

1. **Continuous Testing**: Liquidity returns to Solana for repeated cycles
2. **Multi-Chain Coverage**: Tests 7 different blockchain ecosystems
3. **Cost Optimized**: Uses minimum amounts to reduce expenses
4. **Scalable**: Easy to add more chains or swap pairs
5. **Automated**: Most swaps run automatically (18/26)

---

## Legend

- ✅ = Fully implemented and working
- ⚠️ = Configured but needs integration
- 🔄 = Round-trip enabled (optional mode)
