# Balance Check Configuration

## Overview

The `ENABLE_BALANCE_CHECK` environment variable controls whether the swap runner checks wallet balances before executing swaps.

## Configuration

Add to your `.env` file:

```bash
# Enable balance checking before executing swaps (default: true)
# Set to false to skip balance checks and attempt all swaps
ENABLE_BALANCE_CHECK=true
```

## Default Behavior (Balance Check Enabled)

```bash
ENABLE_BALANCE_CHECK=true
```

When enabled (default):
- Checks EVM token balances (ETH, USDC, WBTC) on Arbitrum, Base, and Ethereum Sepolia
- Skips swaps with insufficient balance
- Uses 5-second timeout per balance check to prevent hanging
- Only checks EVM chains (assumes sufficient balance for Solana and other chains)
- Logs which swaps are being skipped

## Disable Balance Check

```bash
ENABLE_BALANCE_CHECK=false
```

When disabled:
- All swap pairs will be attempted regardless of balance
- No RPC calls to check balances (faster startup)
- Avoids RPC timeout issues with public endpoints
- Swaps may fail if insufficient balance, but will be logged

## Why Default is Enabled

1. **Token Conservation**: Testnet tokens are limited, skip impossible swaps upfront
2. **Efficiency**: Don't waste time on swaps that will fail due to insufficient balance
3. **Visibility**: See which swaps are executable before attempting
4. **Smart Filtering**: Only checks EVM chains to keep it fast

## When to Disable Balance Check

Disable `ENABLE_BALANCE_CHECK=false` when:
- You have reliable RPC endpoints and want maximum speed
- You want to test error handling for insufficient balance scenarios
- You're confident all wallets have sufficient balance
- You want to let Garden API handle all balance validation

## Scheduler Mode

In scheduler mode (cron), the balance check setting from `.env` is used. Set `ENABLE_BALANCE_CHECK=true` for automated runs to conserve testnet tokens.

## Examples

```bash
# Test all swaps with balance check (default)
cargo run --release -- run-once

# Test with balance check disabled (set in .env first)
# Edit .env: ENABLE_BALANCE_CHECK=false
cargo run --release -- run-once

# Test single swap (balance check not applicable)
cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol

# List all available swaps
cargo run --release -- list-swaps
```

## Technical Details

- Balance checks only apply to EVM chains (Arbitrum, Base, Ethereum)
- Non-EVM chains (Solana, Starknet, etc.) are always attempted
- Failed balance checks log a warning but still attempt the swap
- Balance check uses `eth_call` to query ERC20 token balances
- Native ETH balance is checked via `eth_getBalance`
