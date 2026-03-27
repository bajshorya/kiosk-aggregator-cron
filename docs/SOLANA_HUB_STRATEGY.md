# Solana Hub Liquidity Strategy

## Overview

This document describes the Solana-centric swap orchestration strategy that enables continuous testing with minimal initial capital.

## Strategy

### Core Concept
Use Solana testnet as a **liquidity hub** - all funds start on Solana, get distributed to other chains for testing, then return to Solana for the next cycle.

### Three-Phase Execution

```
┌─────────────────────────────────────────────────────────────┐
│                    PHASE 1: DISTRIBUTE                      │
│              (Solana → Other Chains)                        │
├─────────────────────────────────────────────────────────────┤
│  • SOL → ETH (Ethereum)                                     │
│  • USDC → USDC (Ethereum, Base, Arbitrum)                  │
│                                                             │
│  Result: Liquidity distributed across test chains          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                     PHASE 2: TEST                           │
│              (Cross-Chain Swaps)                            │
├─────────────────────────────────────────────────────────────┤
│  • Ethereum ↔ Base                                          │
│  • Ethereum ↔ Arbitrum                                      │
│  • Base ↔ Arbitrum                                          │
│  • ETH ↔ ETH (native token swaps)                          │
│                                                             │
│  Result: Cross-chain functionality validated                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                   PHASE 3: CONSOLIDATE                      │
│              (Other Chains → Solana)                        │
├─────────────────────────────────────────────────────────────┤
│  • USDC (Ethereum, Base, Arbitrum) → USDC (Solana)         │
│  • ETH (Ethereum) → SOL (Solana)                            │
│                                                             │
│  Result: All liquidity back on Solana, ready for next run  │
└─────────────────────────────────────────────────────────────┘
```

## Swap Pairs

### Phase 1: Distribute (4 swaps)
1. `solana_testnet:sol` → `ethereum_sepolia:eth` (0.1 SOL)
2. `solana_testnet:usdc` → `ethereum_sepolia:usdc` (15 USDC)
3. `solana_testnet:usdc` → `base_sepolia:usdc` (15 USDC)
4. `solana_testnet:usdc` → `arbitrum_sepolia:usdc` (15 USDC)

### Phase 2: Test (5 swaps)
5. `ethereum_sepolia:usdc` → `base_sepolia:usdc` (15 USDC)
6. `ethereum_sepolia:usdc` → `arbitrum_sepolia:usdc` (15 USDC)
7. `base_sepolia:usdc` → `arbitrum_sepolia:usdc` (15 USDC)
8. `arbitrum_sepolia:usdc` → `base_sepolia:usdc` (15 USDC)
9. `ethereum_sepolia:eth` → `base_sepolia:eth` (0.005 ETH)

### Phase 3: Consolidate (4 swaps)
10. `ethereum_sepolia:usdc` → `solana_testnet:usdc` (15 USDC)
11. `base_sepolia:usdc` → `solana_testnet:usdc` (15 USDC)
12. `arbitrum_sepolia:usdc` → `solana_testnet:usdc` (15 USDC)
13. `ethereum_sepolia:eth` → `solana_testnet:sol` (0.005 ETH)

**Total: 13 swap pairs**

## Initial Requirements

### Minimum Balances Needed on Solana Testnet
- **SOL**: 0.2 SOL (200M lamports)
  - 0.1 SOL for swaps
  - 0.1 SOL for transaction fees
  
- **USDC**: 50 USDC (50M with 6 decimals)
  - 45 USDC for swaps (3 × 15 USDC)
  - 5 USDC buffer for fees

### How to Get Testnet Tokens

#### Solana Devnet SOL
```bash
solana airdrop 2 <your-solana-address> --url https://api.devnet.solana.com
```

#### Solana Devnet USDC
Use a testnet faucet or swap some SOL for USDC on Solana devnet.

## Benefits

1. **Capital Efficiency**: Start with ~$50 worth of testnet tokens, test 13 swap pairs
2. **Continuous Testing**: After Phase 3, you're back to the starting state
3. **No Manual Deposits**: All swaps are automated (no Bitcoin manual deposits)
4. **Fast Execution**: Solana transactions are fast, reducing wait times
5. **Scalable**: Easy to add more chains by extending the hub-and-spoke model

## Round-Trip Mode

Enable `ENABLE_ROUND_TRIPS=true` in `.env` to add 5 additional test swaps:

- More EVM cross-chain tests
- Additional native token swaps
- Extended Solana distribution paths

**Total with round-trips: 18 swap pairs**

## Usage

### Run All Swaps (Sequential Phases)
```bash
cargo run --release -- run-once
```

The system automatically:
1. Checks Solana balance
2. Executes Phase 1 (distribute)
3. Executes Phase 2 (test)
4. Executes Phase 3 (consolidate)
5. Reports results

### Check Current Balances
```bash
# Check Solana balance
solana balance <your-address> --url https://api.devnet.solana.com

# Check USDC balance (use Solana explorer)
# https://explorer.solana.com/address/<your-address>?cluster=devnet
```

### View Test History
```bash
cargo run --release -- history
```

## Troubleshooting

### "Insufficient balance" errors
- Check your Solana SOL and USDC balances
- Request more from faucets
- Reduce swap amounts in `src/chains/mod.rs`

### Swaps timing out
- Testnet can be slow during high usage
- Wait and retry
- Check transaction status on explorers

### Phase 3 fails to consolidate
- Some liquidity may be stuck on other chains
- Manually swap back to Solana using Garden Finance UI
- Or wait for next test cycle

## Future Enhancements

1. **Dynamic Amount Calculation**: Automatically adjust swap amounts based on available balance
2. **Multi-Hub Support**: Add Ethereum as a secondary hub
3. **Parallel Phase Execution**: Run independent swaps within each phase concurrently
4. **Auto-Rebalancing**: Automatically detect and fix liquidity imbalances

## Monitoring

The system logs each phase:
```
📋 Solana-centric mode: 13 pairs (DISTRIBUTE → TEST → CONSOLIDATE)
Starting batch 1/2 (10 swaps)
✅ Phase 1 complete: 4/4 swaps successful
Starting batch 2/2 (3 swaps)
✅ Phase 2 complete: 5/5 swaps successful
✅ Phase 3 complete: 4/4 swaps successful
```

Check the database for detailed results:
```bash
sqlite3 garden_swaps.db "SELECT swap_pair, status, duration_secs FROM swap_records ORDER BY started_at DESC LIMIT 20;"
```
