# Round-Trip Mode for Continuous Testing

## Problem
When testing 78 swap pairs, you quickly run out of testnet tokens because:
- You swap USDC → BTC, but now you have no USDC left
- You swap ETH → SOL, but now you have no ETH left
- You need to manually go to faucets or DEXs to get tokens back
- This is time-consuming and breaks the testing flow

## Solution: Round-Trip Mode
Round-trip mode automatically adds reverse swaps to bring tokens back to their original chains, allowing continuous testing without running out of tokens.

## How It Works

### Standard Mode (78 pairs)
```bash
# Run without round-trips
cargo run --release -- run-once
```
This tests all 78 one-way swap pairs.

### Round-Trip Mode (89 pairs)
```bash
# Enable round-trips in .env
ENABLE_ROUND_TRIPS=true

# Or set it temporarily
ENABLE_ROUND_TRIPS=true cargo run --release -- run-once
```
This tests 78 original pairs + 11 round-trip pairs to maintain balances.

## Round-Trip Pairs Added

The following 11 round-trip pairs are automatically added when enabled:

### USDC Round-Trips (EVM ↔ EVM)
1. `base_sepolia:usdc → arbitrum_sepolia:usdc` (15 USDC)
2. `ethereum_sepolia:usdc → arbitrum_sepolia:usdc` (15 USDC)
3. `arbitrum_sepolia:usdc → ethereum_sepolia:usdc` (15 USDC)

### Solana USDC Round-Trips
4. `solana_testnet:usdc → arbitrum_sepolia:usdc` (15 USDC)
5. `solana_testnet:usdc → base_sepolia:usdc` (15 USDC)
6. `solana_testnet:usdc → ethereum_sepolia:usdc` (15 USDC)

### WBTC Round-Trips
7. `ethereum_sepolia:wbtc → arbitrum_sepolia:wbtc` (0.00025 BTC)
8. `base_sepolia:wbtc → arbitrum_sepolia:wbtc` (0.00025 BTC)

### Bitcoin Round-Trips
9. `arbitrum_sepolia:wbtc → bitcoin_testnet:btc` (0.00025 BTC)
10. `base_sepolia:wbtc → bitcoin_testnet:btc` (0.00025 BTC)
11. `ethereum_sepolia:wbtc → bitcoin_testnet:btc` (0.00025 BTC)

## Benefits

1. **Continuous Testing**: Run tests repeatedly without running out of tokens
2. **Automated Balance Maintenance**: Tokens automatically flow back to source chains
3. **Reduced Manual Work**: No need to manually swap tokens back or visit faucets
4. **Better Coverage**: Tests both directions of swaps (A→B and B→A)

## Example Flow

### Without Round-Trips
```
Start: 100 USDC on Arbitrum
Test 1: Arbitrum USDC → Solana USDC ✅
Result: 0 USDC on Arbitrum, 99 USDC on Solana
Test 2: Arbitrum USDC → Base USDC ❌ (no USDC left!)
```

### With Round-Trips
```
Start: 100 USDC on Arbitrum
Test 1: Arbitrum USDC → Solana USDC ✅
Test 2: Solana USDC → Arbitrum USDC ✅ (round-trip)
Result: ~98 USDC on Arbitrum (minus fees)
Test 3: Arbitrum USDC → Base USDC ✅ (can continue!)
```

## Configuration

Add to your `.env` file:
```bash
# Enable round-trip mode
ENABLE_ROUND_TRIPS=true
```

Or run temporarily:
```bash
ENABLE_ROUND_TRIPS=true cargo run --release -- run-once
```

## Notes

- Round-trips use smaller amounts (15 USDC vs 30 USDC) to conserve tokens
- Only working chains are included (Arbitrum, Base, Ethereum, Solana)
- Bitcoin round-trips require manual deposits (as usual)
- Total test time increases by ~15% (11 extra swaps)

## Recommendations

- **For initial testing**: Use standard mode (78 pairs) to test all combinations once
- **For continuous testing**: Enable round-trip mode to maintain balances
- **For CI/CD**: Enable round-trip mode for automated testing pipelines
- **For production monitoring**: Use standard mode with real funds

## Cost Analysis

### Standard Mode (78 pairs)
- Total swaps: 78
- Estimated time: ~2 hours (with batching)
- Token depletion: High (one-way swaps)

### Round-Trip Mode (89 pairs)
- Total swaps: 89 (+14%)
- Estimated time: ~2.3 hours (with batching)
- Token depletion: Low (balanced swaps)

## Future Enhancements

Potential improvements:
1. Smart round-trip selection based on token balances
2. Automatic balance monitoring and round-trip triggering
3. Configurable round-trip percentage (e.g., swap back 50% of tokens)
4. Chain-specific round-trip strategies
