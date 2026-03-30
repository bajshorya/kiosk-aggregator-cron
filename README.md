# Garden Swap Tester

A comprehensive testing tool for the [Garden Finance](https://garden.finance) cross-chain Bitcoin bridge. Tests all supported swap pairs across multiple blockchain networks including Bitcoin, Litecoin, EVM chains, Solana, Starknet, and Tron.

## Features

- ✅ **Concurrent Swap Execution** - All swaps run simultaneously for maximum efficiency
- ✅ **Multiple Chains** - Tests Bitcoin, EVM chains, Solana, Starknet, and Tron
- ✅ **Real-time Database Tracking** - SQLite with WAL mode for concurrent access
- ✅ **Chain-Specific Handling** - Automatic detection of deposit requirements
- ✅ **Comprehensive Logging** - Detailed progress tracking with transaction hashes
- ✅ **Multiple Execution Modes** - Run all, test single swap, scheduler, or view history
- ✅ **Discord Bot Integration** - Execute swap tests remotely via Discord slash commands
- ✅ **Testnet Support** - Fully configured for testnet environments

## Supported Chains

| Chain | Asset | Type | Notes |
|-------|-------|------|-------|
| Bitcoin Testnet | BTC | UTXO | Automatic execution with private key |
| Ethereum Sepolia | ETH, WBTC, USDC | EVM | Automatic execution |
| Base Sepolia | WBTC, USDC | EVM | Automatic execution |
| Arbitrum Sepolia | WBTC, USDC | EVM | Automatic execution |
| Alpen Testnet | sBTC, USDC | EVM | Automatic execution |
| BNB Chain Testnet | WBTC | EVM | Automatic execution |
| Citrea Testnet | USDC | EVM | Automatic execution |
| Monad Testnet | USDC | EVM | Automatic execution |
| XRPL Testnet | XRP | EVM | Automatic execution |
| Solana Testnet | SOL, USDC | Solana | Versioned transactions |
| Starknet Sepolia | WBTC | Starknet | Typed data signing |
| Tron Shasta | USDT, WBTC | Tron | EVM-compatible |

## Installation

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Git

### Clone and Build

```bash
git clone <repository-url>
cd kiosk-aggregator-cron
cargo build --release
```

## Configuration

Create a `.env` file in the project root:

```env
# Garden API Configuration
GARDEN_API_BASE_URL=https://testnet.api.garden.finance
GARDEN_APP_ID=your_app_id_here

# Wallet Addresses (Testnet)
WALLET_BITCOIN_TESTNET=tb1q...
WALLET_EVM=0x...
WALLET_STARKNET=0x...
WALLET_SOLANA=...
WALLET_TRON=T...
WALLET_SUI=0x...

# Scheduler Settings
SCHEDULER_CRON=0 0 */5 * * *    # Every 5 hours
SWAP_TIMEOUT_SECS=900           # 15 minutes
POLL_INTERVAL_SECS=15           # Check every 15 seconds

# Database
DATABASE_URL=garden_swaps.db
```

## Usage

### 1. Run All Swaps (Concurrent)

Execute all 16 swap pairs simultaneously:

```bash
cargo run --release -- run-once
```

**Output:**
```
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
...
```

### 2. Test Single Swap

Test a specific swap pair:

```bash
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc
```

**Output:**
```
═══ Swap Test Result ═══
Pair      : ethereum_sepolia:wbtc -> base_sepolia:wbtc
Status    : Completed
Order ID  : 4af1fe7045bf7456b72db4df3ec2fe5ad7e50cab889fcd1173ec4bea6477f3f8
Duration  : 127s
Src Init  : 0x1234...
Dst Redeem: 0x5678...
```

### 3. List Available Swaps

See all configured swap pairs:

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
 ...
```

### 4. View History

Check previous test runs:

```bash
cargo run --release -- history
```

**Output:**
```
[2026-03-24 08:36 UTC] 4fb3fd66-718b-4975-8657-245c8e665455 | total=16 ✅=12 ❌=2 ⏰=2
[2026-03-24 07:30 UTC] 34c57669-134f-4df0-957b-f1ba29ad9923 | total=16 ✅=10 ❌=2 ⏰=4
```

### 5. Scheduler Mode (Default)

Run continuously on a cron schedule:

```bash
cargo run --release
# or
cargo run --release -- scheduler
```

The scheduler will execute all swaps according to the `SCHEDULER_CRON` setting in `.env`.

### 6. Discord Bot Mode

Run the Discord bot to execute swap tests remotely via Discord slash commands:

```bash
cargo run -- discord-bot
```

See [Discord Bot Quick Start](docs/DISCORD_BOT_QUICK_START.md) for setup instructions.

**Available Discord Commands:**
- `/ping` - Health check
- `/test-swap from_asset:ethereum_sepolia:eth to_asset:base_sepolia:wbtc` - Test single swap
- `/test-swap-all` - Run all swaps in batch mode

## Swap Amounts

All swaps use the Garden API minimum amounts:

| Asset | Amount | USD Value (approx) |
|-------|--------|-------------------|
| BTC | 50,000 sats | ~$50 |
| WBTC | 50,000 sats | ~$50 |
| LTC | 1,000,000 sats | ~$50 |
| SOL | 350,000,000 lamports | ~$50 |

## Automated Execution

All swaps are now fully automated when you configure the appropriate private keys in your `.env` file:

### Bitcoin Swaps

Bitcoin swaps are fully automated when you set `BITCOIN_TESTNET_PRIVATE_KEY` in your `.env` file. The system will:
1. Automatically fetch available UTXOs from your wallet
2. Select the optimal UTXOs for the transaction
3. Build and sign the transaction
4. Broadcast it to the Bitcoin network

No manual deposits required!

### EVM Swaps

EVM swaps are automatic with your `WALLET_EVM_PRIVATE_KEY`. The system handles:
1. Transaction signing
2. Gas estimation
3. Broadcasting to the network
4. Gasless transactions when available

### Solana Swaps

Solana swaps are automatic with your `SOLANA_PRIVATE_KEY`. The system supports:
1. Versioned transactions
2. Gasless transactions when available
3. Automatic signing and broadcasting

## Database

All swap data is stored in SQLite (`garden_swaps.db`):

- **swap_records** - Individual swap details with TX hashes
- **run_summaries** - Aggregate statistics per run

### Query Examples

```bash
# View all swaps
sqlite3 garden_swaps.db "SELECT swap_pair, status, duration_secs FROM swap_records ORDER BY started_at DESC LIMIT 10;"

# Count by status
sqlite3 garden_swaps.db "SELECT status, COUNT(*) FROM swap_records GROUP BY status;"
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Garden Swap Tester                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐      ┌──────────────┐                   │
│  │   CLI Args   │─────▶│  Main Loop   │                   │
│  └──────────────┘      └──────┬───────┘                   │
│                               │                             │
│                               ▼                             │
│                    ┌──────────────────┐                    │
│                    │   SwapRunner     │                    │
│                    └────────┬─────────┘                    │
│                             │                               │
│              ┌──────────────┼──────────────┐              │
│              ▼              ▼              ▼              │
│         ┌────────┐    ┌────────┐    ┌────────┐          │
│         │ Task 1 │    │ Task 2 │... │ Task N │          │
│         └───┬────┘    └───┬────┘    └───┬────┘          │
│             │             │             │                 │
│             ▼             ▼             ▼                 │
│      ┌──────────────────────────────────────┐            │
│      │      Garden Finance API              │            │
│      │  (Quote → Submit → Poll Status)      │            │
│      └──────────────────────────────────────┘            │
│                             │                             │
│                             ▼                             │
│                    ┌─────────────────┐                   │
│                    │  SQLite DB      │                   │
│                    │  (WAL Mode)     │                   │
│                    └─────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Swaps Timing Out

**Cause:** Testnet swaps often require manual wallet interaction.

**Solution:**
- Check MetaMask for pending transactions
- Ensure you have testnet ETH for gas
- For Bitcoin/Litecoin, send the deposit manually

### API Errors

**Cause:** Rate limiting or API issues.

**Solution:**
- Wait a few minutes and retry
- Check the Garden Finance status page
- Verify your `GARDEN_APP_ID` is correct

### Database Locked

**Cause:** Multiple processes accessing the database.

**Solution:**
- Stop all running instances
- WAL mode should prevent this, but restart if needed

### Compilation Errors

**Cause:** Missing dependencies or wrong Rust version.

**Solution:**
```bash
rustup update
cargo clean
cargo build --release
```

## Development

### Project Structure

```
kiosk-aggregator-cron/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── api/mod.rs           # Garden API client
│   ├── chains/mod.rs        # Swap pair definitions
│   ├── config/mod.rs        # Configuration management
│   ├── db/mod.rs            # SQLite database
│   ├── models/              # Data structures
│   │   ├── mod.rs
│   │   ├── order.rs
│   │   ├── quote.rs
│   │   └── swap_result.rs
│   └── scheduler/           # Swap execution
│       ├── mod.rs
│       └── runner.rs
├── Cargo.toml               # Dependencies
├── .env                     # Configuration
└── README.md                # This file
```

### Adding New Swap Pairs

Edit `src/chains/mod.rs`:

```rust
vec![
    // Add your new pair
    pair!("new_chain:asset", "amount", wallet, "dest_chain:asset", dest_wallet),
    // ... existing pairs
]
```

### Running Tests

```bash
cargo test
```

## Cost Optimization

The swap pairs are ordered to minimize spending:

1. **EVM swaps** - Automatic execution, can reuse received funds
2. **Bitcoin swaps** - Automatic with UTXO reuse for continuous testing
3. **Solana swaps** - Automatic with gasless support
4. **Starknet swaps** - Automatic with typed data signing
5. **Tron swaps** - Automatic execution

All swaps use testnet tokens with minimal amounts for testing.

## Documentation

- **[Quick Start Guide](docs/QUICK_START.md)** - Get started in 5 minutes
- **[Discord Bot Quick Start](docs/DISCORD_BOT_QUICK_START.md)** - Set up Discord bot in 5 minutes
- **[Discord Bot Documentation](docs/DISCORD_BOT.md)** - Complete Discord bot guide
- **[Usage Examples](docs/EXAMPLES.md)** - Common use cases and commands
- **[Cost Optimization](docs/COST_OPTIMIZATION.md)** - Minimize spending strategy
- **[Implementation Details](docs/IMPLEMENTATION_COMPLETE.md)** - Technical overview
- **[Fixes Summary](docs/FIXES_SUMMARY.md)** - All bugs fixed
- **[Test Results](docs/TEST_RESULTS.md)** - Initial test results

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

[Your License Here]

## Support

- **Documentation:** [Garden Finance Docs](https://docs.garden.finance)
- **API Reference:** [Garden API](https://testnet.api.garden.finance)
- **Issues:** [GitHub Issues](your-repo-url/issues)

## Acknowledgments

Built for testing the [Garden Finance](https://garden.finance) cross-chain Bitcoin bridge.
