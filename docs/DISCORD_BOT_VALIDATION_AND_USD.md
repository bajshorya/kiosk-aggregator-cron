# Discord Bot - Swap Pair Validation & USD Amounts

## Overview

The Discord bot now includes two important features:
1. **Swap Pair Validation**: Checks if the swap pair is valid before executing
2. **USD Amount Input**: Accept amounts in USD instead of smallest units

## Feature 1: Swap Pair Validation

### What It Does

Before executing a swap, the bot validates:
- ✅ Both assets are in the supported list
- ✅ Assets are on different chains (can't swap on same chain)
- ✅ Proper format: `chain:token`

### User Experience

**Invalid Swap Attempt:**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:ethereum_sepolia:wbtc
```

**Bot Response:**
```
❌ Invalid Swap Pair

The swap from `ethereum_sepolia:eth` to `ethereum_sepolia:wbtc` is not supported.

Possible issues:
• Assets must be on different chains
• Both assets must be in the supported list
• Check the asset format: `chain:token` (e.g., `ethereum_sepolia:eth`)

Use the autocomplete dropdown to see valid options.
```

**Valid Swap:**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
```

Bot proceeds with the swap ✅

### Validation Rules

1. **Both assets must be supported:**
   - Checks against the list of 20+ supported assets
   - Rejects unknown chains or tokens

2. **Different chains required:**
   - `ethereum_sepolia:eth` → `base_sepolia:wbtc` ✅ (different chains)
   - `ethereum_sepolia:eth` → `ethereum_sepolia:wbtc` ❌ (same chain)

3. **Proper format:**
   - Must be `chain:token` format
   - Case-sensitive

### Benefits

- **Prevents errors**: Catches invalid swaps before execution
- **Saves time**: No need to wait for swap to fail
- **Clear feedback**: Explains why the swap is invalid
- **Better UX**: Users know immediately if something is wrong

---

## Feature 2: USD Amount Input

### What Changed

**Before:**
```
amount: Optional custom amount in smallest units
```
Users had to calculate:
- BTC: satoshis (1 BTC = 100,000,000 sats)
- ETH: wei (1 ETH = 1,000,000,000,000,000,000 wei)
- SOL: lamports (1 SOL = 1,000,000,000 lamports)

**After:**
```
amount_usd: Amount in USD (e.g., 50 for $50)
```
Users just enter the USD value!

### Usage Examples

**Swap $50 worth of ETH:**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc amount_usd:50
```

**Swap $100 worth of BTC:**
```
/test-swap from_asset:bitcoin_testnet:btc to_asset:ethereum_sepolia:wbtc amount_usd:100
```

**Swap $25 worth of SOL:**
```
/test-swap from_asset:solana_testnet:sol to_asset:arbitrum_sepolia:usdc amount_usd:25
```

**Default amount (no USD specified):**
```
/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
```
Uses the default minimum amount

### Conversion Logic

The bot automatically converts USD to smallest units:

```rust
USD Amount → Token Amount → Smallest Units
```

**Example: $50 of ETH**
1. ETH price: ~$3,500
2. Token amount: $50 / $3,500 = 0.0142857 ETH
3. Smallest units: 0.0142857 × 10^18 = 14,285,700,000,000,000 wei

### Supported Tokens & Prices

| Token | Approx Price | Decimals | Example: $50 USD |
|-------|-------------|----------|------------------|
| BTC/WBTC | $100,000 | 8 | 50,000 sats |
| ETH | $3,500 | 18 | ~14.3 × 10^15 wei |
| SOL | $150 | 9 | ~333 × 10^6 lamports |
| USDC/USDT | $1 | 6 | 50,000,000 (50 USDC) |
| XRP | $0.50 | 6 | 100,000,000 (100 XRP) |

**Note:** Prices are approximate and hardcoded for testnet. In production, you'd fetch real-time prices from an API.

### Conversion Function

```rust
fn convert_usd_to_smallest_units(asset: &str, usd_amount: f64) -> u64 {
    let token = asset.split(':').nth(1).unwrap_or("");
    
    // Get price per token
    let price_per_token = match token {
        "btc" | "wbtc" => 100000.0,
        "eth" => 3500.0,
        "sol" => 150.0,
        "usdc" | "usdt" => 1.0,
        _ => 1.0,
    };
    
    // Get decimals
    let decimals = match token {
        "btc" | "wbtc" => 8,
        "eth" => 18,
        "sol" => 9,
        "usdc" | "usdt" => 6,
        _ => 6,
    };
    
    // Calculate
    let token_amount = usd_amount / price_per_token;
    let smallest_units = token_amount * 10_f64.powi(decimals);
    
    smallest_units.max(1.0) as u64
}
```

### Benefits

- **User-friendly**: No need to calculate smallest units
- **Intuitive**: Everyone understands USD amounts
- **Consistent**: Same unit across all tokens
- **Flexible**: Still works without amount (uses defaults)

---

## Technical Implementation

### Validation Function

```rust
fn is_valid_swap_pair(from_asset: &str, to_asset: &str) -> bool {
    // Check if both assets are supported
    let supported_assets = vec![
        "bitcoin_testnet:btc",
        "ethereum_sepolia:eth",
        // ... more assets
    ];
    
    if !supported_assets.contains(&from_asset) || 
       !supported_assets.contains(&to_asset) {
        return false;
    }
    
    // Extract chain names
    let from_chain = from_asset.split(':').next().unwrap_or("");
    let to_chain = to_asset.split(':').next().unwrap_or("");
    
    // Assets must be on different chains
    if from_chain == to_chain {
        return false;
    }
    
    true
}
```

### Command Signature

```rust
#[poise::command(slash_command, prefix_command, rename = "test-swap")]
pub async fn test_swap(
    ctx: Context<'_>,
    #[description = "From asset (e.g., ethereum_sepolia:eth)"]
    #[autocomplete = "autocomplete_asset"]
    from_asset: String,
    
    #[description = "To asset (e.g., base_sepolia:wbtc)"]
    #[autocomplete = "autocomplete_asset"]
    to_asset: String,
    
    #[description = "Amount in USD (e.g., 50 for $50)"] 
    amount_usd: Option<f64>,
) -> Result<(), Error>
```

### Execution Flow

```
1. User enters command with USD amount
   ↓
2. Validate swap pair
   ↓
3. If invalid → Show error message and stop
   ↓
4. If valid → Convert USD to smallest units
   ↓
5. Execute swap with converted amount
   ↓
6. Show result in Discord embed
```

---

## Examples

### Example 1: Valid Swap with USD

**Command:**
```
/test-swap 
  from_asset:ethereum_sepolia:eth 
  to_asset:base_sepolia:wbtc 
  amount_usd:50
```

**What Happens:**
1. ✅ Validation passes (different chains, both supported)
2. 💱 Converts $50 to ~14.3 × 10^15 wei
3. 🚀 Executes swap
4. 📊 Shows result in embed

---

### Example 2: Invalid Swap (Same Chain)

**Command:**
```
/test-swap 
  from_asset:ethereum_sepolia:eth 
  to_asset:ethereum_sepolia:wbtc
```

**What Happens:**
1. ❌ Validation fails (same chain)
2. 💬 Shows error message
3. 🛑 Stops execution (no swap attempted)

**Error Message:**
```
❌ Invalid Swap Pair

The swap from `ethereum_sepolia:eth` to `ethereum_sepolia:wbtc` is not supported.

Possible issues:
• Assets must be on different chains
• Both assets must be in the supported list
• Check the asset format: `chain:token`

Use the autocomplete dropdown to see valid options.
```

---

### Example 3: Invalid Asset

**Command:**
```
/test-swap 
  from_asset:ethereum_sepolia:doge 
  to_asset:base_sepolia:wbtc
```

**What Happens:**
1. ❌ Validation fails (doge not supported)
2. 💬 Shows error message
3. 🛑 Stops execution

---

### Example 4: Default Amount

**Command:**
```
/test-swap 
  from_asset:ethereum_sepolia:eth 
  to_asset:base_sepolia:wbtc
```

**What Happens:**
1. ✅ Validation passes
2. ⚙️ No USD amount provided, uses default
3. 🚀 Executes swap with minimum amount
4. 📊 Shows result

---

## Testing

### Test Validation

1. **Test same chain (should fail):**
   ```
   /test-swap from_asset:ethereum_sepolia:eth to_asset:ethereum_sepolia:wbtc
   ```
   Expected: Error message

2. **Test invalid asset (should fail):**
   ```
   /test-swap from_asset:ethereum_sepolia:doge to_asset:base_sepolia:wbtc
   ```
   Expected: Error message

3. **Test valid pair (should work):**
   ```
   /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
   ```
   Expected: Swap executes

### Test USD Conversion

1. **Test with $50:**
   ```
   /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc amount_usd:50
   ```
   Check logs for: "Converting $50 USD to ... smallest units"

2. **Test with $100:**
   ```
   /test-swap from_asset:solana_testnet:sol to_asset:arbitrum_sepolia:usdc amount_usd:100
   ```

3. **Test without amount:**
   ```
   /test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc
   ```
   Should use default amount

---

## Future Enhancements

### Real-time Price Feeds

Instead of hardcoded prices, fetch from APIs:

```rust
async fn get_token_price(token: &str) -> f64 {
    // Fetch from CoinGecko, CoinMarketCap, etc.
    match reqwest::get(&format!("https://api.coingecko.com/.../{}", token))
        .await?
        .json::<PriceResponse>()
        .await
    {
        Ok(response) => response.usd,
        Err(_) => get_fallback_price(token),
    }
}
```

### Slippage Protection

Add slippage parameter:

```rust
#[description = "Max slippage % (e.g., 1 for 1%)"] 
slippage: Option<f64>,
```

### Price Display

Show estimated output:

```
💰 Swapping ~$50 USD worth of ETH
📊 Estimated output: ~0.0005 WBTC ($50)
⚠️ Actual amount may vary due to price changes
```

### Historical Prices

Use historical prices for better accuracy:

```rust
// Get price at time of swap initiation
let price_at_swap = get_historical_price(token, swap_timestamp).await?;
```

---

## Troubleshooting

### Issue: Validation fails for valid pair

**Check:**
1. Asset format: `chain:token` (lowercase)
2. Both assets in supported list
3. Different chains

**Solution:**
Use autocomplete to select assets

### Issue: USD conversion seems wrong

**Cause:** Hardcoded prices may be outdated

**Solution:**
- Prices are approximate for testnet
- For production, implement real-time price feeds
- Check conversion in logs

### Issue: Amount too small

**Cause:** USD amount results in < 1 smallest unit

**Solution:**
- Increase USD amount
- Minimum is 1 smallest unit
- For expensive tokens (BTC), use higher USD amounts

---

## Summary

✅ **Swap Pair Validation**
- Prevents invalid swaps
- Clear error messages
- Saves time and gas

✅ **USD Amount Input**
- User-friendly
- No need to calculate smallest units
- Works across all tokens

✅ **Better UX**
- Immediate feedback
- Intuitive interface
- Professional error handling

---

**Status**: ✅ Implemented and tested  
**Version**: 1.5.0  
**Date**: 2026-03-30
