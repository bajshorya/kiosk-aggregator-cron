# Network Mode Configuration

## Overview
The system supports both testnet/devnet and mainnet modes for testing and production use. You can easily switch between them using the `NETWORK_MODE` environment variable.

## Configuration

### Setting Network Mode

Add to your `.env` file:

```bash
# For testing (default)
NETWORK_MODE=testnet

# For production (BE CAREFUL!)
NETWORK_MODE=mainnet
```

### What Changes with Network Mode

| Feature | Testnet Mode | Mainnet Mode |
|---------|-------------|--------------|
| **Garden API** | `https://testnet.api.garden.finance` | `https://api.garden.finance` |
| **Tokens** | Testnet tokens (no real value) | Real tokens (REAL VALUE!) |
| **Balance Checks** | Checks testnet token balances | Checks mainnet token balances |
| **RPC Endpoints** | Sepolia, Base Sepolia, Arbitrum Sepolia | Ethereum, Base, Arbitrum mainnet |
| **Risk** | ✅ Safe for testing | ⚠️ USES REAL MONEY |

## Testnet Mode (Default)

### Features
- Uses testnet endpoints automatically
- Checks balances on Sepolia testnets
- Safe for testing and development
- No risk of losing real funds

### Token Addresses (Testnet)

**Arbitrum Sepolia:**
- USDC: `0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d`
- WBTC: `0xb5ae9785349186069c48794a763db39ec756b1cf`

**Base Sepolia:**
- USDC: `0x036CbD53842c5426634e7929541eC2318f3dCF7e`
- WETH: `0x4200000000000000000000000000000000000006`

**Ethereum Sepolia:**
- USDC: `0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238`
- WBTC: `0x29f2D40B0605204364af54EC677bD022dA425d03`

### Usage

```bash
# Set testnet mode (or omit, it's the default)
echo "NETWORK_MODE=testnet" >> .env

# Run swaps
cargo run --release -- run-once
```

### Output
```
🌐 Network mode: Testnet
INFO Garden Swap Tester starting up
INFO Network mode: Testnet
INFO API base URL: https://testnet.api.garden.finance
```

## Mainnet Mode

### ⚠️ WARNING
**MAINNET MODE USES REAL TOKENS AND REAL MONEY!**
- All swaps will use real funds
- Failed transactions will cost real gas fees
- There is NO UNDO for mainnet transactions
- Only use mainnet mode when you're ready for production

### Features
- Uses mainnet endpoints
- Checks balances on mainnet
- Executes real swaps with real value
- Requires real ETH for gas fees

### Usage

```bash
# Set mainnet mode
echo "NETWORK_MODE=mainnet" >> .env

# Run swaps (BE CAREFUL!)
cargo run --release -- run-once
```

### Output
```
🌐 Network mode: Mainnet
INFO Garden Swap Tester starting up
INFO Network mode: Mainnet
INFO API base URL: https://api.garden.finance
```

## Balance Checking

### Testnet Balance Checks
- Checks ERC20 token balances on Sepolia testnets
- Fast checks (5-second timeout)
- Skips swaps with insufficient balance
- Assumes sufficient balance on timeout (to avoid blocking)

### How It Works

1. **EVM Chains** (Arbitrum, Base, Ethereum):
   - Queries token contract `balanceOf` function
   - Compares balance with required amount
   - Skips swap if balance < required

2. **Non-EVM Chains** (Solana, Bitcoin, etc.):
   - Skips balance check (not implemented)
   - Assumes sufficient balance
   - Swap will fail at execution if insufficient

### Example Output

```
INFO Checking balances for EVM chains only (fast check)...
INFO USDC balance: 0, required: 15000000, sufficient: false
INFO ⏭️  Skipping arbitrum_sepolia:usdc -> base_sepolia:usdc (insufficient balance)
INFO ✅ Balance check complete: 62/78 pairs will be attempted
```

## Switching Between Modes

### From Testnet to Mainnet

1. **Test thoroughly on testnet first!**
2. Update `.env`:
   ```bash
   NETWORK_MODE=mainnet
   ```
3. Update wallet addresses to mainnet addresses
4. Update RPC endpoints to mainnet RPCs
5. Ensure you have real ETH for gas
6. Start with small amounts!

### From Mainnet to Testnet

1. Update `.env`:
   ```bash
   NETWORK_MODE=testnet
   ```
2. Update wallet addresses to testnet addresses
3. Update RPC endpoints to testnet RPCs
4. Get testnet tokens from faucets

## Environment Variables

### Required for Both Modes

```bash
# Network mode
NETWORK_MODE=testnet  # or mainnet

# Garden App ID (same for both)
GARDEN_APP_ID=your_app_id_here

# Wallet private keys
WALLET_EVM_PRIVATE_KEY=your_private_key
SOLANA_PRIVATE_KEY=your_solana_key
```

### Optional Overrides

```bash
# Override API URL (if needed)
GARDEN_API_BASE_URL=https://custom.api.url

# Override RPC endpoints
RPC_ETHEREUM_SEPOLIA=https://your-rpc.com
RPC_ARBITRUM_SEPOLIA=https://your-rpc.com
RPC_BASE_SEPOLIA=https://your-rpc.com
```

## Best Practices

### For Testing (Testnet)
1. Always start with testnet mode
2. Test all swap combinations
3. Verify balance checking works
4. Check transaction confirmations
5. Monitor for errors

### For Production (Mainnet)
1. Test EVERYTHING on testnet first
2. Start with small amounts
3. Monitor closely
4. Have alerts set up
5. Keep private keys secure
6. Use hardware wallets if possible

## Troubleshooting

### Balance Check Failures

**Problem:** Balance checks timing out
```
WARN Balance check timed out for ethereum_sepolia:usdc, assuming sufficient
```

**Solution:**
- Use a faster RPC endpoint
- Increase timeout in `balance_checker.rs`
- Check RPC endpoint is responding

### Wrong Network

**Problem:** Swaps failing with "Invalid network"
```
ERROR Quote failed: Network mismatch
```

**Solution:**
- Check `NETWORK_MODE` matches your wallet addresses
- Verify RPC endpoints are for correct network
- Ensure Garden API URL matches network mode

### Token Address Not Found

**Problem:** Balance check fails with "Unknown token"
```
WARN Token address not found for usdc: Unknown token: usdc
```

**Solution:**
- Add token address to `balance_checker.rs`
- Check token symbol matches expected format
- Verify token exists on that network

## Summary

- **Testnet mode** (default): Safe for testing, uses testnet tokens
- **Mainnet mode**: Production use, uses real tokens (BE CAREFUL!)
- **Balance checking**: Automatically skips swaps with insufficient balance
- **Easy switching**: Just change `NETWORK_MODE` in `.env`
- **Safety first**: Always test on testnet before mainnet!
