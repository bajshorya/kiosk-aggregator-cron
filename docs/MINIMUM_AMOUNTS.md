# Minimum Swap Amounts

## Overview
All swap pairs now use the **minimum amounts required by Garden API** to reduce costs and conserve testnet tokens.

## Minimum Amount Requirements

Based on Garden API validation and testing, the minimum amounts are:

| Asset Type | Minimum Amount | Decimals | USD Value | Notes |
|------------|---------------|----------|-----------|-------|
| **BTC/WBTC** | 50,000 sats | 8 | ~$50 | Bitcoin, WBTC on all chains |
| **USDC** | 15,000,000 | 6 | ~$15 | USDC on all chains |
| **USDT** | 15,000,000 | 6 | ~$15 | Tron USDT |
| **ETH** | 5,000,000,000,000,000 wei | 18 | ~$15-20 | Native ETH (0.005 ETH) |
| **SOL USDC** | 50,000,000 | 6 | ~$50 | Solana USDC (higher minimum) |

## API Validation Errors

The Garden API returns specific error messages when amounts are out of range:

```json
{
  "status": "Error",
  "error": "Exact output quote error : expected amount to be within the range of X to Y"
}
```

### Observed Ranges from API

| Swap Type | Minimum | Maximum | Example |
|-----------|---------|---------|---------|
| USDC → WBTC | 15,000,000 | 50,000,000 | Base USDC → Arbitrum WBTC |
| USDC → USDC | 25,000,000 | 35,000,000 | Some EVM pairs |
| SOL USDC → WBTC | 50,000,000 | 100,000,000 | Solana USDC → Starknet WBTC |
| ETH swaps | 5,000,000,000,000,000 | 1,000,000,000,000,000,000 | Native ETH swaps |

## Implementation

All 78 swap pairs have been updated to use minimum amounts:

```rust
// BTC/WBTC swaps: 50,000 sats
pair!("arbitrum_sepolia:wbtc", "50000", evm, "ethereum_sepolia:wbtc", evm),

// USDC swaps: 15,000,000 (15 USDC)
pair!("arbitrum_sepolia:usdc", "15000000", evm, "base_sepolia:usdc", evm),

// Solana USDC: 50,000,000 (50 USDC) - higher minimum
pair!("solana_testnet:usdc", "50000000", sol, "starknet_sepolia:wbtc", stark),
```

## Cost Savings

### Before (30 USDC per swap)
- 78 swaps × $30 = **$2,340 total**
- Depletes testnet tokens quickly
- Requires frequent faucet visits

### After (15 USDC per swap)
- 78 swaps × $15 = **$1,170 total**
- **50% cost reduction**
- Longer testing cycles before needing more tokens

### With Round-Trips Enabled
- 89 swaps (78 + 11 round-trips)
- Total cost: ~$1,335
- **Maintains token balances** for continuous testing
- No need to manually swap tokens back

## Testing Results

### Successful Swaps with Minimum Amounts
✅ Arbitrum USDC (15M) → Solana USDC - **Completed in 91s**
✅ Arbitrum USDC (15M) → Base USDC - **Working**
✅ Solana USDC (50M) → Arbitrum USDC - **Working**

### Failed Swaps (API Limitations)
❌ Solana USDC (30M) → Starknet WBTC - "expected 50000000 to 100000000"
❌ Some XRPL pairs - "insufficient liquidity"
❌ Some Monad pairs - "insufficient liquidity"

## Recommendations

1. **Use minimum amounts** for all testing to conserve tokens
2. **Enable round-trip mode** (`ENABLE_ROUND_TRIPS=true`) for continuous testing
3. **Monitor API errors** - if you see "expected amount to be within range", adjust accordingly
4. **Batch testing** - run swaps in batches of 10 to avoid overwhelming the system

## Environment Variable

No configuration needed - minimum amounts are now hardcoded based on API requirements.

For round-trip mode:
```bash
ENABLE_ROUND_TRIPS=true cargo run --release -- run-once
```

## Future Improvements

1. **Dynamic amount detection** - query API for min/max amounts per pair
2. **Smart amount selection** - automatically use minimum for each pair
3. **Balance-aware amounts** - adjust amounts based on available balances
4. **Cost tracking** - track total USD spent across all swaps
