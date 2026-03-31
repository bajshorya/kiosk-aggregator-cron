# Discord Bot Notification Format

## Overview

The Discord bot now supports enhanced swap notification messages with status-specific formatting, matching the format from modern Discord bot implementations.

## Notification Types

### 1. Failed Trade (❌)
- **Color**: Red (#E74C3C)
- **Icon**: ❌
- **Title**: "❌ Swap Report — Failed Trade"
- **Use Case**: General swap failures

### 2. Insufficient Funds (💸)
- **Color**: Orange (#E67E22)
- **Icon**: 💸
- **Title**: "💸 Swap Report — Insufficient Funds"
- **Use Case**: When wallet doesn't have enough balance

### 3. Liquidity Error (🌊)
- **Color**: Purple (#9B59B6)
- **Icon**: 🌊
- **Title**: "🌊 Swap Report — Liquidity Error"
- **Use Case**: When there's insufficient liquidity in the pool

### 4. In Processing (⏳)
- **Color**: Blue (#3498DB)
- **Icon**: ⏳
- **Title**: "⏳ Swap Report — In Processing"
- **Use Case**: When swap is pending or being processed

### 5. Completed (✅)
- **Color**: Green (#2ECC71)
- **Icon**: ✅
- **Title**: "✅ Swap Report — Trade Completed"
- **Use Case**: Successful swap completion

### 6. Timeout (⏰)
- **Color**: Orange (#FFA500)
- **Icon**: ⏰
- **Title**: "⏰ Swap Report — Timeout"
- **Use Case**: When swap exceeds time limit

## Message Structure

Each notification embed includes:

1. **Status Header**: Emoji + status label (e.g., "❌ Failure trade")
2. **Order ID**: Displayed in code block for easy copying
3. **Order Pair**: Shows the swap direction (from → to)
4. **Reason**: Detailed error or status message
5. **Warnings** (optional): Additional context or suggestions

## Example Output

```
❌ Swap Report — Failed Trade

❌ Failure trade

• Order ID
`order_id_12345`

• Order Pair
`arbitrum_sepolia:0xe918...3fb07 → monad_testnet:0xe99d...5867`

• Reason
Invalid assets found for swap from `0xe918a5a...3fb07` to `0xe99d8a...5867`

⚠️ Warnings
• Low balance of `0xbcdad29...fe4f` in base_sepolia
• 💰 Make sure your wallet has enough testnet tokens
```

## Automatic Status Detection

The bot automatically categorizes errors and selects the appropriate notification type:

- **Insufficient Balance** → `insufficient_funds` type
- **Liquidity Issues** → `liquidity_error` type
- **Pending/Processing** → `in_processing` type
- **Timeout** → `timeout` type
- **Other Failures** → `failed` type

## Implementation

The notification format is implemented in `src/discord/commands/test_swap.rs`:

- `build_swap_notification_embed()` - Creates status-specific embeds
- `categorize_error()` - Analyzes error messages and provides helpful tips
- `parse_swap_output_to_embed()` - Parses CLI output and generates appropriate notifications

## Usage

The bot automatically uses the new notification format for:
- Failed swaps with error messages
- Timed out swaps
- Swaps with insufficient funds
- Swaps with liquidity errors

For successful swaps and batch operations, the original detailed format is still used.

## Color Reference

| Status | Hex Color | RGB |
|--------|-----------|-----|
| Failed | #E74C3C | (231, 76, 60) |
| Insufficient Funds | #E67E22 | (230, 126, 34) |
| Liquidity Error | #9B59B6 | (155, 89, 182) |
| In Processing | #3498DB | (52, 152, 219) |
| Completed | #2ECC71 | (46, 204, 113) |
| Timeout | #FFA500 | (255, 165, 0) |

## Related Files

- `src/discord/commands/test_swap.rs` - Main implementation
- `docs/DISCORD_BOT.md` - General Discord bot documentation
- `docs/DISCORD_BOT_IMPROVEMENTS.md` - Bot improvement history
