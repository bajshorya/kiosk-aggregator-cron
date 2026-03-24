# Cost-Optimized Swap Testing Strategy

## Overview
All swaps use **API minimum amounts (~$50 USD)** and execute **sequentially** in an optimized order to minimize your total spending.

## API Minimum Requirements

The Garden API enforces a minimum swap amount of **50,000 to 1,000,000 sats** for BTC/WBTC.

| Asset | Amount | USD Value |
|-------|--------|-----------|
| BTC | 50,000 sats | ~$50 |
| WBTC | 50,000 sats | ~$50 |
| LTC | 1,000,000 sats | ~$50 |
| SOL | 350,000,000 lamports | ~$50 |

## Execution Strategy: Sequential with Fund Reuse

### Phase 1: EVM Swaps (Start Here - No Manual Deposit)
**Cost: $50** (you need WBTC on Ethereum Sepolia to start)

1. `ethereum_sepolia:wbtc → base_sepolia:wbtc` ($50)

### Phase 2: Reuse Received Base WBTC
**Cost: $0** (using funds from Phase 1)

2. `base_sepolia:wbtc → arbitrum_sepolia:wbtc` ($0)
3. `base_sepolia:wbtc → ethereum_sepolia:wbtc` ($0)

### Phase 3: Reuse Arbitrum WBTC
**Cost: $0** (using funds from Phase 2)

4. `arbitrum_sepolia:wbtc → base_sepolia:wbtc` ($0)

### Phase 4: EVM to Bitcoin
**Cost: $0** (using existing EVM WBTC)

5. `ethereum_sepolia:wbtc → bitcoin_testnet:btc` ($0)
6. `base_sepolia:wbtc → bitcoin_testnet:btc` ($0)

### Phase 5: Bitcoin to EVM (Manual Deposits Required)
**Cost: $150** (3 deposits × $50)

7. `bitcoin_testnet:btc → base_sepolia:wbtc` ($50) ⚠️ DEPOSIT NEEDED
8. `bitcoin_testnet:btc → ethereum_sepolia:wbtc` ($50) ⚠️ DEPOSIT NEEDED
9. `bitcoin_testnet:btc → arbitrum_sepolia:wbtc` ($50) ⚠️ DEPOSIT NEEDED

### Phase 6: Litecoin (Manual Deposit Required)
**Cost: $50** (1 deposit × $50)

10. `litecoin_testnet:ltc → base_sepolia:wbtc` ($50) ⚠️ DEPOSIT NEEDED

### Phase 7: Solana (Requires SOL Balance)
**Cost: $100** (2 swaps × $50)

11. `solana_testnet:sol → bitcoin_testnet:btc` ($50)
12. `solana_testnet:sol → base_sepolia:wbtc` ($50)

### Phase 8: Starknet (Requires WBTC on Starknet)
**Cost: $100** (2 swaps × $50)

13. `starknet_sepolia:wbtc → bitcoin_testnet:btc` ($50)
14. `starknet_sepolia:wbtc → base_sepolia:wbtc` ($50)

### Phase 9: Tron (Requires WBTC on Tron)
**Cost: $100** (2 swaps × $50)

15. `tron_shasta:wbtc → arbitrum_sepolia:wbtc` ($50)
16. `tron_shasta:wbtc → base_sepolia:wbtc` ($50)

### Phase 10: Sui (DISABLED)
**Cost: $0** (skipped due to API error)

~~17. `sui_testnet:sui → bitcoin_testnet:btc`~~
~~18. `sui_testnet:sui → base_sepolia:wbtc`~~

## Total Cost Breakdown

### Minimum Required Deposits:
- **EVM WBTC** (Ethereum Sepolia): $50 (to start)
- **Bitcoin**: $150 (3 deposits)
- **Litecoin**: $50 (1 deposit)
- **Solana**: $100 (2 swaps)
- **Starknet WBTC**: $100 (2 swaps)
- **Tron WBTC**: $100 (2 swaps)

### Total: **$550 USD**

### Savings from Fund Reuse:
- **6 EVM swaps** use received funds ($0 additional cost)
- **Potential savings**: $300 (6 × $50)

## Sequential Execution Benefits

1. **Predictable Order**: Swaps execute one at a time in the order listed
2. **Fund Reuse**: Later swaps can use funds received from earlier swaps
3. **Easy Monitoring**: Clear progress tracking (swap 1/16, 2/16, etc.)
4. **Manual Deposit Control**: You can deposit BTC/LTC when prompted
5. **Lower Risk**: If one swap fails, you can stop before spending more

## How to Run

```bash
# Run all swaps sequentially
cargo run --release -- run-once
```

The app will:
1. Execute swaps one at a time in the optimized order
2. Wait for each swap to complete before starting the next
3. Show clear progress: "Starting swap 5/16: ..."
4. Prompt you when manual deposits are needed
5. Continue automatically for EVM/Solana/Starknet/Tron swaps

## Manual Deposit Instructions

When you see:
```
⚠️ [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to tb1p...
```

You need to:
1. Copy the deposit address
2. Send the exact amount from your Bitcoin wallet
3. Wait for the swap to detect the deposit and complete
4. The next swap will start automatically

## Monitoring Progress

Each swap shows:
- ✅ Completed (funds received)
- ❌ Failed (error occurred)
- ⏰ Timed out (took too long)
- ↩️ Refunded (swap reversed)

## Database Tracking

All swaps are recorded in `garden_swaps.db`:
- Real-time status updates
- Transaction hashes
- Duration tracking
- Error messages

View history:
```bash
cargo run --release -- history
```

