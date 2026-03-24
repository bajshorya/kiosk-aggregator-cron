# Quick Start Guide

## Installation

```bash
git clone <repository-url>
cd kiosk-aggregator-cron
cargo build --release
```

## Configuration

Copy `.env.example` to `.env` and update with your wallet addresses:

```bash
cp .env.example .env
# Edit .env with your wallet addresses
```

## Commands

### 1. List All Available Swaps

```bash
cargo run --release -- list-swaps
```

**Output:**
```
═══ Available Swap Pairs (16) ═══

 1. ethereum_sepolia:wbtc -> base_sepolia:wbtc
 2. base_sepolia:wbtc -> arbitrum_sepolia:wbtc
 ...
 7. bitcoin_testnet:btc -> base_sepolia:wbtc ⚠️  DEPOSIT
```

### 2. Test a Single Swap

```bash
cargo run --release -- test-swap <from_asset> <to_asset>
```

**Examples:**

```bash
# Test EVM swap (automatic)
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc

# Test Bitcoin swap (requires deposit)
cargo run --release -- test-swap bitcoin_testnet:btc base_sepolia:wbtc

# Test Solana swap
cargo run --release -- test-swap solana_testnet:sol base_sepolia:wbtc
```

**Output:**
```
═══ Swap Test Result ═══
Pair      : ethereum_sepolia:wbtc -> base_sepolia:wbtc
Status    : Completed
Order ID  : 567ecd381cb3de194cfd4b46653c83cba666971adfcd310d2213d31deb2c0b5a
Duration  : 127s
Src Init  : 0x1234...
Dst Redeem: 0x5678...
```

### 3. Run All Swaps (Concurrent)

```bash
cargo run --release -- run-once
```

All 16 swaps will start simultaneously and run in parallel.

**Output:**
```
═══ Final Run Summary ═══
Run ID   : 4fb3fd66-718b-4975-8657-245c8e665455
Total    : 16
Completed: 12
Failed   : 2
Timed Out: 2
```

### 4. View History

```bash
cargo run --release -- history
```

**Output:**
```
[2026-03-24 08:36 UTC] 4fb3fd66... | total=16 ✅=12 ❌=2 ⏰=2
[2026-03-24 07:30 UTC] 34c57669... | total=16 ✅=10 ❌=2 ⏰=4
```

### 5. Scheduler Mode (Continuous)

```bash
cargo run --release
# or
cargo run --release -- scheduler
```

Runs continuously based on the cron schedule in `.env`.

## Common Swap Pairs

### EVM Swaps (Automatic)
```bash
# Ethereum → Base
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc

# Base → Arbitrum
cargo run --release -- test-swap base_sepolia:wbtc arbitrum_sepolia:wbtc

# Arbitrum → Base
cargo run --release -- test-swap arbitrum_sepolia:wbtc base_sepolia:wbtc
```

### Bitcoin Swaps (Requires Deposit)
```bash
# Bitcoin → Base
cargo run --release -- test-swap bitcoin_testnet:btc base_sepolia:wbtc

# Base → Bitcoin
cargo run --release -- test-swap base_sepolia:wbtc bitcoin_testnet:btc
```

### Solana Swaps
```bash
# Solana → Bitcoin
cargo run --release -- test-swap solana_testnet:sol bitcoin_testnet:btc

# Solana → Base
cargo run --release -- test-swap solana_testnet:sol base_sepolia:wbtc
```

### Starknet Swaps
```bash
# Starknet → Bitcoin
cargo run --release -- test-swap starknet_sepolia:wbtc bitcoin_testnet:btc

# Starknet → Base
cargo run --release -- test-swap starknet_sepolia:wbtc base_sepolia:wbtc
```

## Tips

### For Manual Deposit Swaps

When you see:
```
⚠️ [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to tb1p...
```

1. Copy the address
2. Send from your Bitcoin testnet wallet
3. Wait for confirmations
4. The swap will auto-detect and continue

### For EVM Swaps on Testnet

1. Connect MetaMask to the testnet
2. Check for pending transactions
3. Approve and sign
4. Wait for confirmations

### Checking Swap Status

While a swap is running, you can check the database:

```bash
sqlite3 garden_swaps.db "SELECT swap_pair, status, order_id FROM swap_records ORDER BY started_at DESC LIMIT 5;"
```

## Troubleshooting

### "Swap pair not found"

Make sure you're using the exact asset names from `list-swaps`:

```bash
# ✅ Correct
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc

# ❌ Wrong
cargo run --release -- test-swap ethereum:wbtc base:wbtc
```

### Swap Times Out

This is normal on testnet. Swaps timeout after 15 minutes if no transaction is detected. Check:
- MetaMask for pending transactions
- Your wallet has testnet ETH for gas
- For Bitcoin/Litecoin, you sent the deposit

### API Errors

Wait a few minutes and retry. The API may be rate limiting or experiencing issues.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check [COST_OPTIMIZATION.md](COST_OPTIMIZATION.md) for spending strategy
- Review [FIXES_SUMMARY.md](FIXES_SUMMARY.md) for technical details
