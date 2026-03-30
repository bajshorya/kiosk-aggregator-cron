# Bitcoin Sequential Execution

## Overview

When running `cargo run --release -- run-once`, the system automatically detects Bitcoin swaps and executes them **sequentially** with 5-second delays, while all other swaps run **concurrently**.

## Why Sequential Execution?

### The UTXO Conflict Problem

Bitcoin uses UTXOs (Unspent Transaction Outputs) as inputs for transactions. When multiple swaps try to use the same UTXO simultaneously:

```
Time 0s: Swap 1 fetches UTXOs → sees 447,180 sats → uses it ✅
Time 0s: Swap 2 fetches UTXOs → sees 447,180 sats → tries to use it ❌ (double-spend)
Time 0s: Swap 3 fetches UTXOs → sees 447,180 sats → tries to use it ❌ (double-spend)
```

**Result**: Only the first swap succeeds. Others fail with "missing inputs" or "double-spend" errors.

### The Solution: Sequential + UTXO Reuse

```
Time 0s:  Swap 1 starts → uses 447,180 sats → broadcasts → creates 394,360 sats change ✅
Time 5s:  Swap 2 starts → sees 394,360 sats (mempool) → uses it → broadcasts ✅
Time 10s: Swap 3 starts → sees new change UTXO (mempool) → uses it → broadcasts ✅
```

**Result**: All swaps succeed by reusing change UTXOs from previous swaps!

## How It Works

### 1. Automatic Detection

The system automatically separates swap pairs:

```rust
let (bitcoin_pairs, other_pairs): (Vec<_>, Vec<_>) = executable_pairs
    .into_iter()
    .partition(|pair| {
        let chain = pair.source.asset.split(':').next().unwrap_or("");
        chain.starts_with("bitcoin_") || chain.starts_with("litecoin_")
    });
```

### 2. Concurrent Execution for Non-Bitcoin

All non-Bitcoin swaps (ETH, SOL, EVM chains) run concurrently:

```
✅ ethereum_sepolia:eth → solana_testnet:sol (concurrent)
✅ solana_testnet:sol → ethereum_sepolia:eth (concurrent)
✅ base_sepolia:usdc → arbitrum_sepolia:usdc (concurrent)
... (all running at the same time)
```

### 3. Sequential Execution for Bitcoin

Bitcoin swaps run one after another with 5-second delays:

```
🔶 Bitcoin swap 1/7: bitcoin_testnet:btc → solana_testnet:usdc
   ✅ Broadcasted: dbb4de3a...
   ⏱️  Waiting 5 seconds before next Bitcoin swap (UTXO reuse)...

🔶 Bitcoin swap 2/7: bitcoin_testnet:btc → starknet_sepolia:wbtc
   ✅ Broadcasted: a1b2c3d4...
   ⏱️  Waiting 5 seconds before next Bitcoin swap (UTXO reuse)...

🔶 Bitcoin swap 3/7: bitcoin_testnet:btc → citrea_testnet:usdc
   ✅ Broadcasted: e5f6g7h8...
```

## Benefits

1. **No UTXO Conflicts**: Each swap uses different UTXOs
2. **Automatic UTXO Reuse**: Change from previous swaps is immediately available
3. **Efficient Capital Use**: No need to wait for confirmations
4. **Realistic Testing**: Simulates real-world sequential swap scenarios
5. **Mixed Execution**: Bitcoin sequential + other chains concurrent = optimal speed

## Timing

### 5-Second Delay Rationale

The 5-second delay allows:
1. **Transaction broadcast** (~1-2 seconds)
2. **Mempool propagation** (~1-2 seconds)
3. **UTXO API update** (~1-2 seconds)
4. **Safety buffer** (~1 second)

This ensures the next swap can fetch and use the new change UTXO.

### Total Time Calculation

For `N` Bitcoin swaps:
- First swap: Immediate
- Subsequent swaps: 5 seconds delay each
- **Total delay**: `(N - 1) × 5 seconds`

Example with 7 Bitcoin swaps:
- Delays: `(7 - 1) × 5 = 30 seconds`
- Plus swap execution time: ~5-10 seconds per swap
- **Total**: ~65-100 seconds for all Bitcoin swaps

Meanwhile, all other swaps run concurrently and may finish faster!

## Logging

The system provides clear logging:

```
⚠️  Bitcoin UTXO conflict prevention: 7 Bitcoin swaps will run SEQUENTIALLY (5s delay between each)
✅ 73 non-Bitcoin swaps will run CONCURRENTLY

🔶 Bitcoin swap 1/7: bitcoin_testnet:btc -> solana_testnet:usdc
Found 12 UTXOs with total 1592093 sats (6 confirmed: 64826 sats, 6 unconfirmed: 1527267 sats)
Selected 1 UTXOs with total 447180 sats (need 52820 sats including fee)
  └─ 0 confirmed, 1 unconfirmed (mempool) UTXOs
  UTXO #1: 4a55485d...f6985197 vout=1 value=447180 sats [MEMPOOL]

⏱️  Waiting 5 seconds before next Bitcoin swap (UTXO reuse)...

🔶 Bitcoin swap 2/7: bitcoin_testnet:btc -> starknet_sepolia:wbtc
Found 13 UTXOs with total 1986453 sats (6 confirmed: 64826 sats, 7 unconfirmed: 1921627 sats)
Selected 1 UTXOs with total 394360 sats (need 52820 sats including fee)
  └─ 0 confirmed, 1 unconfirmed (mempool) UTXOs
  UTXO #1: dbb4de3a...605aafa0 vout=1 value=394360 sats [MEMPOOL]  ← Change from Swap 1!
```

## Testing

### Test All Swaps (Including Bitcoin)

```bash
cargo run --release -- run-once
```

This will:
- Run all non-Bitcoin swaps concurrently
- Run all Bitcoin swaps sequentially with 5s delays
- Show detailed UTXO usage logs

### Test Only Bitcoin Swaps

```bash
./test_bitcoin_utxo_reuse.sh
```

This runs 3 Bitcoin swaps sequentially to demonstrate UTXO reuse.

## Configuration

No configuration needed - sequential execution is automatic for Bitcoin swaps.

### Environment Variables

The behavior is controlled by existing settings:

```env
# Enable balance checking (filters out swaps with insufficient balance)
ENABLE_BALANCE_CHECK=false

# Enable round-trip swaps (adds reverse swap pairs)
ENABLE_ROUND_TRIPS=true
```

## Limitations

### 1. Bitcoin Swaps Take Longer

Sequential execution means Bitcoin swaps take longer than concurrent execution:
- 7 Bitcoin swaps: ~65-100 seconds
- vs. concurrent (if it worked): ~10-15 seconds

**Trade-off**: Reliability over speed. All swaps succeed instead of only the first one.

### 2. Not Truly Parallel

Bitcoin swaps don't benefit from parallel execution. However:
- Other chains (ETH, SOL, EVM) still run concurrently
- Overall test time is still much faster than running everything sequentially

### 3. API Dependency

The 5-second delay assumes mempool.space API updates quickly. If the API is slow:
- Increase delay to 10 seconds (requires code change)
- Or wait for API to catch up

## Technical Details

### Implementation

```rust
// Separate Bitcoin from other swaps
let (bitcoin_pairs, other_pairs): (Vec<_>, Vec<_>) = executable_pairs
    .into_iter()
    .partition(|pair| {
        let chain = pair.source.asset.split(':').next().unwrap_or("");
        chain.starts_with("bitcoin_") || chain.starts_with("litecoin_")
    });

// Spawn non-Bitcoin swaps concurrently
for pair in other_pairs {
    set.spawn(async move {
        runner.run_single_swap(&run_id, &pair).await
    });
}

// Spawn Bitcoin swaps sequentially
tokio::spawn(async move {
    for (idx, pair) in bitcoin_pairs.iter().enumerate() {
        if idx > 0 {
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        runner.run_single_swap(&run_id, pair).await;
    }
});
```

### Why Not Use a Lock?

An alternative approach would be to use a mutex/lock for Bitcoin swaps:

```rust
// ❌ This doesn't solve the problem
let bitcoin_lock = Arc::new(Mutex::new(()));
let _guard = bitcoin_lock.lock().await;
// ... execute swap
```

**Problem**: The lock only prevents concurrent execution, but doesn't ensure UTXO reuse. The 5-second delay is essential for:
1. Transaction broadcast
2. Mempool propagation
3. UTXO API update

## Related Documentation

- [Bitcoin UTXO Reuse](./BITCOIN_UTXO_REUSE.md)
- [Bitcoin Signer Implementation](./BITCOIN_SIGNER_IMPLEMENTATION.md)
- [Bitcoin Swap Integration](./BITCOIN_SWAP_INTEGRATION.md)
