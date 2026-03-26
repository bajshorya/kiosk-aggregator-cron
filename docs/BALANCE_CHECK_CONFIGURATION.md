# Balance Check Configuration

Balance checking is now controlled via the `ENABLE_BALANCE_CHECK` environment variable in your `.env` file.

## Quick Setup

Add to your `.env` file:

```bash
# Enable balance checking (default: true)
ENABLE_BALANCE_CHECK=true
```

## Options

### Enabled (Default - Recommended)
```bash
ENABLE_BALANCE_CHECK=true
```

**What it does:**
- Checks EVM wallet balances before executing swaps
- Skips swaps with insufficient USDC, WBTC, or ETH
- Only checks Arbitrum, Base, and Ethereum Sepolia (fast)
- Assumes sufficient balance for Solana and other chains
- Uses 5-second timeout to prevent hanging

**Result:** 63/79 swaps attempted (16 skipped due to 0 USDC balance)

### Disabled
```bash
ENABLE_BALANCE_CHECK=false
```

**What it does:**
- Attempts all 79 swaps regardless of balance
- No RPC calls for balance checking (faster startup)
- Swaps may fail if insufficient balance

**Result:** All 79 swaps attempted (some may fail)

## How It Works

1. **EVM Chains Only**: Balance checks are performed only on:
   - Arbitrum Sepolia
   - Base Sepolia  
   - Ethereum Sepolia

2. **Tokens Checked**:
   - Native ETH (via `eth_getBalance`)
   - USDC (ERC20 token balance)
   - WBTC (ERC20 token balance)

3. **Non-EVM Chains**: Solana, Starknet, Tron, Sui, etc. are always attempted (no balance check)

4. **Timeout Protection**: Each balance check has a 5-second timeout to prevent hanging on slow RPCs

5. **Fallback Behavior**: If balance check fails or times out, the swap is still attempted

## Example Output

```
Balance checking: ENABLED
Checking balances for EVM chains only (fast check)...
⏭️  Skipping arbitrum_sepolia:usdc -> base_sepolia:usdc (insufficient balance)
⏭️  Skipping arbitrum_sepolia:usdc -> bitcoin_testnet:btc (insufficient balance)
...
✅ Balance check complete: 63/79 pairs will be attempted
```

## Usage

```bash
# Run with balance check enabled (default)
cargo run --release -- run-once

# Disable balance check in .env first
# Edit .env: ENABLE_BALANCE_CHECK=false
cargo run --release -- run-once

# Scheduler mode uses .env setting
cargo run --release
```

## Recommendation

**For Testnet**: Keep `ENABLE_BALANCE_CHECK=true` to conserve limited testnet tokens

**For Mainnet**: Consider `ENABLE_BALANCE_CHECK=true` to avoid failed transactions and wasted gas
