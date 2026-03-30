# Swap Execution Order

## Overview

Swap pairs are now organized into 6 phases for optimal execution order. This ensures that liquidity flows efficiently from fast chains (EVM) to slower chains (Bitcoin), maximizing success rates and minimizing failures.

## Execution Phases

### Phase 1: EVM Chains (Execute First)
**Why first:** Fastest execution, most reliable, highest liquidity

- **EVM → EVM swaps** (33 pairs)
  - Ethereum Sepolia ↔ Base, Arbitrum, Alpen, BNB Chain, Citrea, Monad, XRPL
  - Arbitrum Sepolia ↔ Base, BNB Chain, Citrea, Monad, XRPL
  - Base Sepolia ↔ BNB Chain, Citrea, Monad, XRPL
  - BNB Chain ↔ Citrea, Monad, XRPL
  - Citrea ↔ Monad, XRPL
  - Monad ↔ XRPL

**Benefits:**
- Instant execution (no block confirmations needed)
- Gasless transactions available
- High success rate
- Provides liquidity for subsequent phases

### Phase 2: EVM → Solana (Execute Second)
**Why second:** Use EVM liquidity to fund Solana swaps

- **EVM → Solana USDC** (7 pairs)
  - Arbitrum, Ethereum, Alpen, Base, BNB Chain, Citrea, Monad → Solana USDC

**Benefits:**
- Solana receives USDC from multiple EVM chains
- Prepares Solana for Phase 4 outbound swaps
- Gasless transactions available on Solana

### Phase 3: EVM → Starknet & Tron (Execute Third)
**Why third:** Use EVM liquidity for other chains

- **EVM → Starknet WBTC** (7 pairs)
- **EVM → Tron USDT** (7 pairs)

**Benefits:**
- Funds Starknet and Tron for Phase 5 swaps
- Diversifies liquidity across chains

### Phase 4: Solana Swaps (Execute Fourth)
**Why fourth:** After receiving from EVM in Phase 2

- **Solana SOL → Ethereum ETH** (1 pair)
- **Solana USDC → Starknet, Tron, XRPL** (3 pairs)

**Benefits:**
- Uses USDC received from Phase 2
- Gasless transactions available
- Fast execution

### Phase 5: Starknet & Tron Swaps (Execute Fifth)
**Why fifth:** After receiving from EVM in Phase 3

- **Starknet WBTC → Tron, XRPL** (2 pairs)
- **Tron USDT/WBTC → XRPL** (2 pairs)

**Benefits:**
- Uses tokens received from Phase 3
- Completes cross-chain liquidity flow

### Phase 6: Bitcoin Swaps (Execute Last - Sequential)
**Why last:** Slowest, requires UTXO management, sequential execution

- **EVM → Bitcoin** (4 pairs)
  - Arbitrum, Ethereum, Alpen, Base → Bitcoin BTC
  
- **Bitcoin → EVM** (4 pairs)
  - Bitcoin BTC → BNB Chain, Citrea, Monad, XRPL
  
- **Bitcoin → Solana** (1 pair)
  - Bitcoin BTC → Solana USDC
  
- **Bitcoin → Starknet & Tron** (2 pairs)
  - Bitcoin BTC → Starknet WBTC, Tron USDT

- **Alpen Signet (Bitcoin-style)** (11 pairs)
  - Treated as Bitcoin for execution order

**Special Handling:**
- Bitcoin swaps run **sequentially** (not concurrently)
- 5-second delay between each Bitcoin swap
- Enables UTXO reuse from mempool
- Prevents UTXO conflicts

## Total Swap Pairs

- **Phase 1 (EVM):** 33 pairs
- **Phase 2 (EVM → Solana):** 7 pairs
- **Phase 3 (EVM → Starknet/Tron):** 14 pairs
- **Phase 4 (Solana):** 4 pairs
- **Phase 5 (Starknet/Tron):** 4 pairs
- **Phase 6 (Bitcoin):** 22 pairs

**Total:** 84 pairs (standard mode)

## Execution Strategy

### Concurrent Execution
All non-Bitcoin swaps (Phases 1-5) execute **concurrently** for maximum speed:
- 62 swaps start simultaneously
- No waiting between phases
- Phases are logical groupings, not sequential steps

### Sequential Execution
Bitcoin swaps (Phase 6) execute **sequentially**:
- One Bitcoin swap at a time
- 5-second delay between each
- Allows UTXO reuse from mempool
- Prevents double-spend errors

## Benefits of This Order

1. **Liquidity Flow:** EVM → Solana → Bitcoin ensures each phase has funds
2. **Speed:** Fast chains execute first, slow chains last
3. **Success Rate:** High-reliability swaps complete before risky ones
4. **UTXO Management:** Bitcoin swaps run last with proper sequencing
5. **Gas Efficiency:** Gasless transactions prioritized on EVM and Solana

## Example Execution Timeline

```
Time 0s:  Start all EVM swaps (Phase 1-3) + Solana swaps (Phase 4-5)
          ├─ 33 EVM → EVM swaps
          ├─ 7 EVM → Solana swaps
          ├─ 14 EVM → Starknet/Tron swaps
          ├─ 4 Solana swaps
          └─ 4 Starknet/Tron swaps
          
Time 0s:  Start first Bitcoin swap (Phase 6)
Time 5s:  Start second Bitcoin swap
Time 10s: Start third Bitcoin swap
...
Time 105s: Complete last Bitcoin swap (22 swaps × 5s = 110s)
```

## Configuration

The execution order is hardcoded in `src/chains/mod.rs` in the `all_swap_pairs()` function. The phases are clearly marked with comments for easy maintenance.

## Round-Trip Mode

When `ENABLE_ROUND_TRIPS=true`, an additional 12 round-trip pairs are added:
- Solana ↔ Ethereum (ETH/SOL)
- EVM ↔ EVM (USDC, WBTC)
- EVM ↔ Bitcoin (WBTC/BTC)

These maintain token balances for continuous testing.

## See Also

- [Bitcoin Sequential Execution](BITCOIN_SEQUENTIAL_EXECUTION.md) - Details on Bitcoin UTXO management
- [Bitcoin UTXO Reuse](BITCOIN_UTXO_REUSE.md) - How mempool UTXOs are reused
- [Gasless Implementation](GASLESS_IMPLEMENTATION_STATUS.md) - Gasless transaction support
