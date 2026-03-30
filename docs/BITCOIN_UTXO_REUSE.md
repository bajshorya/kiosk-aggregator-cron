# Bitcoin UTXO Reuse for Continuous Testing

## Overview

The swap tester automatically reuses unconfirmed UTXOs (change outputs in mempool) for subsequent Bitcoin swaps. This enables continuous testing without waiting for block confirmations between swaps.

## How It Works

### 1. Initial Swap
```
Your Wallet: 100,000 sats (confirmed)
Swap Amount: 50,000 sats
Fee: 10,000 sats
Change: 40,000 sats → returned to your wallet (unconfirmed, in mempool)
```

### 2. Subsequent Swap (Immediate)
```
Your Wallet: 40,000 sats (unconfirmed from previous swap)
Swap Amount: 50,000 sats
Status: ✅ Can proceed using unconfirmed UTXO
```

### 3. UTXO Selection Strategy

The system uses a **greedy algorithm** that:
- Fetches ALL UTXOs (confirmed + unconfirmed)
- Sorts by value (largest first)
- Selects UTXOs until sufficient balance is reached
- **Does NOT discriminate** between confirmed and unconfirmed UTXOs

## Benefits

1. **Continuous Testing**: No waiting between swaps
2. **Efficient Capital Use**: Reuse change immediately
3. **Realistic Testing**: Simulates high-frequency swap scenarios
4. **Mempool Awareness**: Tests real-world conditions

## Logging

The enhanced logging shows UTXO status:

```
Found 6 UTXOs with total 55047 sats (5 confirmed: 52253 sats, 1 unconfirmed: 2794 sats)
Selected 6 UTXOs with total 55047 sats (need 59620 sats including fee)
  └─ 5 confirmed, 1 unconfirmed (mempool) UTXOs
Estimated fee: 9620 sats (20 sats/vbyte)
  UTXO #1: ad9646c4...0bbc80df vout=5 value=12559 sats [CONFIRMED]
  UTXO #2: ad9646c4...0bbc80df vout=3 value=12558 sats [CONFIRMED]
  UTXO #3: ad9646c4...0bbc80df vout=6 value=12551 sats [CONFIRMED]
  UTXO #4: 8d23203f...3be7362f vout=0 value=11957 sats [CONFIRMED]
  UTXO #5: a693eb17...05b311d5 vout=1 value=2794 sats [MEMPOOL]
  UTXO #6: 754ae07b...4a007e76 vout=1 value=2628 sats [CONFIRMED]
```

## Testing

Run the UTXO reuse test script:

```bash
./test_bitcoin_utxo_reuse.sh
```

This will execute 3 consecutive Bitcoin swaps, demonstrating:
- Swap 1: Uses confirmed UTXOs
- Swap 2: Reuses unconfirmed change from Swap 1
- Swap 3: Reuses unconfirmed change from Swap 2

## Important Notes

### ⚠️ Concurrent Swaps
Running multiple Bitcoin swaps **simultaneously** will fail because:
- Multiple swaps try to use the same UTXO
- Only the first transaction will be accepted
- Others will fail with "double-spend" errors

**Solution**: Run Bitcoin swaps **sequentially** (one after another), not concurrently.

### ⚠️ Mempool Risks
Unconfirmed UTXOs can be:
- Replaced (RBF - Replace-By-Fee)
- Dropped from mempool (if fee too low)
- Delayed (network congestion)

For production use, consider adding a `min_confirmations` parameter.

### ⚠️ API Reliability
The mempool.space API can be slow or timeout. The system includes:
- 10-second timeout per request
- 2 retry attempts with delays
- Fallback URL support

## Configuration

No configuration needed - UTXO reuse is automatic.

To disable unconfirmed UTXO usage (future enhancement):
```env
# Not yet implemented
BITCOIN_MIN_CONFIRMATIONS=1  # Require at least 1 confirmation
```

## Technical Details

### UTXO Structure
```rust
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub script_pubkey: String,
    pub confirmed: bool,  // ← Tracks confirmation status
}
```

### Selection Algorithm
```rust
// 1. Fetch all UTXOs (confirmed + unconfirmed)
let all_utxos = provider.get_utxos(&wallet_address).await?;

// 2. Sort by value (largest first)
sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));

// 3. Select greedily until sufficient balance
for utxo in sorted_utxos {
    selected_utxos.push(utxo);
    selected_total += utxo.value;
    
    if selected_total >= amount + fee {
        break;  // ✅ Sufficient balance
    }
}
```

## Troubleshooting

### "Insufficient Bitcoin balance" Error
- Check if UTXOs are available: `./check_balances.sh`
- Verify mempool.space API is responding
- Check if previous swap's change is in mempool

### "Failed to fetch Bitcoin UTXOs" Error
- API timeout (mempool.space can be slow)
- Network connectivity issues
- Try again after a few seconds

### Double-Spend Errors
- Running concurrent Bitcoin swaps
- Solution: Run swaps sequentially

## Related Documentation

- [Bitcoin Signer Implementation](./BITCOIN_SIGNER_IMPLEMENTATION.md)
- [Bitcoin Swap Integration](./BITCOIN_SWAP_INTEGRATION.md)
- [Swap Test Status](./SWAP_TEST_STATUS.md)
