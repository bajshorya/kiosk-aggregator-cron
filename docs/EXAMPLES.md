# Usage Examples

## Testing Individual Swaps

### Example 1: EVM Swap (Automatic)

```bash
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc
```

**Expected Output:**
```
2026-03-24T08:41:22.499Z  INFO Testing single swap: ethereum_sepolia:wbtc -> base_sepolia:wbtc
2026-03-24T08:41:22.502Z  INFO GET quote ethereum_sepolia:wbtc -> base_sepolia:wbtc (50000)
2026-03-24T08:41:22.924Z  INFO Quote received src=50000 dst=49855
2026-03-24T08:41:23.231Z  INFO Order submitted order_id=567ecd38...
2026-03-24T08:41:23.232Z  INFO EVM initiate_transaction: {"chain_id":11155111,...}
2026-03-24T08:41:38.478Z  INFO Poll poll=1 src_init= dst_redeem=
...

═══ Swap Test Result ═══
Pair      : ethereum_sepolia:wbtc -> base_sepolia:wbtc
Status    : Completed
Order ID  : 567ecd381cb3de194cfd4b46653c83cba666971adfcd310d2213d31deb2c0b5a
Duration  : 127s
Src Init  : 0x1234567890abcdef...
Dst Redeem: 0xabcdef1234567890...
```

### Example 2: Bitcoin Swap (Manual Deposit)

```bash
cargo run --release -- test-swap bitcoin_testnet:btc base_sepolia:wbtc
```

**Expected Output:**
```
2026-03-24T08:45:10.123Z  INFO Testing single swap: bitcoin_testnet:btc -> base_sepolia:wbtc
2026-03-24T08:45:10.456Z  INFO Quote received src=50000 dst=49850
2026-03-24T08:45:10.789Z  INFO Order submitted order_id=abc123...
2026-03-24T08:45:10.790Z  INFO ⚠️  [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to tb1pg2wu88w568358yv3ep3er7sdhjfy4symjc0jy5475g87yxwyqtytsvlq0ud
2026-03-24T08:45:25.123Z  INFO Poll poll=1 src_init= dst_redeem=
...

═══ Swap Test Result ═══
Pair      : bitcoin_testnet:btc -> base_sepolia:wbtc
Status    : Pending
Order ID  : abc123def456...
Duration  : 900s
Deposit   : tb1pg2wu88w568358yv3ep3er7sdhjfy4symjc0jy5475g87yxwyqtytsvlq0ud

⚠️  Manual deposit required. Send 50000 sats to the address above.
```

### Example 3: Solana Swap

```bash
cargo run --release -- test-swap solana_testnet:sol base_sepolia:wbtc
```

**Expected Output:**
```
2026-03-24T08:50:00.000Z  INFO Testing single swap: solana_testnet:sol -> base_sepolia:wbtc
2026-03-24T08:50:00.234Z  INFO Quote received src=350000000 dst=45020
2026-03-24T08:50:00.567Z  INFO Order submitted order_id=sol123...
2026-03-24T08:50:00.568Z  INFO Solana versioned_tx (432 chars)
2026-03-24T08:50:15.789Z  INFO Poll poll=1 src_init= dst_redeem=
...

═══ Swap Test Result ═══
Pair      : solana_testnet:sol -> base_sepolia:wbtc
Status    : Completed
Order ID  : sol123abc456...
Duration  : 145s
Src Init  : 5YmK8...
Dst Redeem: 0xdef456...
```

## Running All Swaps

### Example 4: Concurrent Execution

```bash
cargo run --release -- run-once
```

**Expected Output:**
```
2026-03-24T08:36:16.327Z  INFO === Starting CONCURRENT swap test run (16 pairs) ===
2026-03-24T08:36:16.327Z  INFO All swaps will start simultaneously!
2026-03-24T08:36:16.327Z  INFO Starting swap pair=ethereum_sepolia:wbtc -> base_sepolia:wbtc
2026-03-24T08:36:16.327Z  INFO Starting swap pair=base_sepolia:wbtc -> arbitrum_sepolia:wbtc
2026-03-24T08:36:16.327Z  INFO Starting swap pair=bitcoin_testnet:btc -> base_sepolia:wbtc
... (14 more swaps starting)

2026-03-24T08:36:17.022Z  INFO [DEPOSIT NEEDED] Send 1000000 litecoin_testnet:ltc to tltc1p...
2026-03-24T08:36:17.060Z  INFO [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to tb1pg2wu8...
2026-03-24T08:36:17.067Z  INFO [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to tb1pjzu4y...
2026-03-24T08:36:17.077Z  INFO [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to tb1pmjsfn...

... (All swaps polling concurrently)

2026-03-24T08:51:16.123Z  INFO ✅ Task finished pair=ethereum_sepolia:wbtc -> base_sepolia:wbtc status=Completed
2026-03-24T08:51:17.234Z  INFO ✅ Task finished pair=base_sepolia:wbtc -> arbitrum_sepolia:wbtc status=Completed
...

═══ Final Run Summary ═══
Run ID   : 4fb3fd66-718b-4975-8657-245c8e665455
Total    : 16
Completed: 12
Failed   : 2
Timed Out: 2
Pending  : 0

✅ ethereum_sepolia:wbtc -> base_sepolia:wbtc          | Completed    |
✅ base_sepolia:wbtc -> arbitrum_sepolia:wbtc          | Completed    |
⏰ base_sepolia:wbtc -> ethereum_sepolia:wbtc          | TimedOut     |
✅ arbitrum_sepolia:wbtc -> base_sepolia:wbtc          | Completed    |
✅ ethereum_sepolia:wbtc -> bitcoin_testnet:btc        | Completed    |
✅ base_sepolia:wbtc -> bitcoin_testnet:btc            | Completed    |
⏰ bitcoin_testnet:btc -> base_sepolia:wbtc            | TimedOut     | No deposit detected
⏰ bitcoin_testnet:btc -> ethereum_sepolia:wbtc        | TimedOut     | No deposit detected
⏰ bitcoin_testnet:btc -> arbitrum_sepolia:wbtc        | TimedOut     | No deposit detected
⏰ litecoin_testnet:ltc -> base_sepolia:wbtc           | TimedOut     | No deposit detected
✅ solana_testnet:sol -> bitcoin_testnet:btc           | Completed    |
✅ solana_testnet:sol -> base_sepolia:wbtc             | Completed    |
✅ starknet_sepolia:wbtc -> bitcoin_testnet:btc        | Completed    |
✅ starknet_sepolia:wbtc -> base_sepolia:wbtc          | Completed    |
❌ tron_shasta:wbtc -> arbitrum_sepolia:wbtc           | Failed       | API error: Failed to generate calldata
❌ tron_shasta:wbtc -> base_sepolia:wbtc               | Failed       | API error: Failed to generate calldata
```

## Listing Swaps

### Example 5: View All Available Pairs

```bash
cargo run --release -- list-swaps
```

**Output:**
```
═══ Available Swap Pairs (16) ═══

 1. ethereum_sepolia:wbtc -> base_sepolia:wbtc
 2. base_sepolia:wbtc -> arbitrum_sepolia:wbtc
 3. base_sepolia:wbtc -> ethereum_sepolia:wbtc
 4. arbitrum_sepolia:wbtc -> base_sepolia:wbtc
 5. ethereum_sepolia:wbtc -> bitcoin_testnet:btc
 6. base_sepolia:wbtc -> bitcoin_testnet:btc
 7. bitcoin_testnet:btc -> base_sepolia:wbtc ⚠️  DEPOSIT
 8. bitcoin_testnet:btc -> ethereum_sepolia:wbtc ⚠️  DEPOSIT
 9. bitcoin_testnet:btc -> arbitrum_sepolia:wbtc ⚠️  DEPOSIT
10. litecoin_testnet:ltc -> base_sepolia:wbtc ⚠️  DEPOSIT
11. solana_testnet:sol -> bitcoin_testnet:btc
12. solana_testnet:sol -> base_sepolia:wbtc
13. starknet_sepolia:wbtc -> bitcoin_testnet:btc
14. starknet_sepolia:wbtc -> base_sepolia:wbtc
15. tron_shasta:wbtc -> arbitrum_sepolia:wbtc
16. tron_shasta:wbtc -> base_sepolia:wbtc
```

## Viewing History

### Example 6: Check Past Runs

```bash
cargo run --release -- history
```

**Output:**
```
[2026-03-24 08:36 UTC] 4fb3fd66-718b-4975-8657-245c8e665455 | total=16 ✅=12 ❌=2 ⏰=2
[2026-03-24 07:30 UTC] 34c57669-134f-4df0-957b-f1ba29ad9923 | total=16 ✅=10 ❌=2 ⏰=4
[2026-03-24 06:00 UTC] 12345678-1234-1234-1234-123456789012 | total=16 ✅=14 ❌=0 ⏰=2
```

## Error Handling

### Example 7: Invalid Swap Pair

```bash
cargo run --release -- test-swap invalid:asset another:asset
```

**Output:**
```
Error: Swap pair not found: invalid:asset -> another:asset
Use 'list-swaps' to see available pairs
```

### Example 8: Missing Arguments

```bash
cargo run --release -- test-swap ethereum_sepolia:wbtc
```

**Output:**
```
Error: Usage: cargo run --release -- test-swap <from_asset> <to_asset>
Example: cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc
```

## Database Queries

### Example 9: Query Recent Swaps

```bash
sqlite3 garden_swaps.db "SELECT swap_pair, status, duration_secs FROM swap_records ORDER BY started_at DESC LIMIT 5;"
```

**Output:**
```
ethereum_sepolia:wbtc -> base_sepolia:wbtc|Completed|127
base_sepolia:wbtc -> arbitrum_sepolia:wbtc|Completed|145
bitcoin_testnet:btc -> base_sepolia:wbtc|TimedOut|900
solana_testnet:sol -> base_sepolia:wbtc|Completed|156
starknet_sepolia:wbtc -> bitcoin_testnet:btc|Completed|189
```

### Example 10: Count by Status

```bash
sqlite3 garden_swaps.db "SELECT status, COUNT(*) as count FROM swap_records GROUP BY status;"
```

**Output:**
```
Completed|12
Failed|2
TimedOut|4
```

## Tips

### Quick Test of EVM Swaps
```bash
# Test all EVM pairs quickly
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc
cargo run --release -- test-swap base_sepolia:wbtc arbitrum_sepolia:wbtc
cargo run --release -- test-swap arbitrum_sepolia:wbtc base_sepolia:wbtc
```

### Test with Manual Deposits
```bash
# Start the swap
cargo run --release -- test-swap bitcoin_testnet:btc base_sepolia:wbtc

# Copy the deposit address from output
# Send BTC from your wallet
# Wait for the swap to complete
```

### Monitor Live Progress
```bash
# In one terminal, run the swap
cargo run --release -- run-once

# In another terminal, watch the database
watch -n 5 'sqlite3 garden_swaps.db "SELECT swap_pair, status FROM swap_records WHERE run_id=(SELECT run_id FROM run_summaries ORDER BY started_at DESC LIMIT 1);"'
```
