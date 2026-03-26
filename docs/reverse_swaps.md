# Manual Reverse Swaps to Refill Chains

Since you've been testing swaps and tokens have moved to different chains, here are the manual commands to bring tokens back to refill your source chains.

## Current Situation
Based on your recent swaps:
- Arbitrum USDC → Solana USDC (30 USDC moved to Solana)
- Arbitrum WBTC → Ethereum WBTC (0.0005 BTC moved to Ethereum)
- Arbitrum WBTC → Bitcoin (0.0005 BTC moved to Bitcoin)

## Reverse Swap Commands

### 1. Bring Solana USDC back to Arbitrum
```bash
# Solana → Arbitrum requires 50 USDC minimum
curl -X POST "https://testnet.api.garden.finance/v2/orders" \
  -H "garden-app-id: f242ea49332293424c96c562a6ef575a819908c878134dcb4fce424dc84ec796" \
  -H "Content-Type: application/json" \
  -d '{
    "source": {
      "asset": "solana_testnet:usdc",
      "owner": "YH4btvqb4JBWSEJh22MuA231ekpJ5JqbBXQY1apJtKH",
      "amount": "50000000"
    },
    "destination": {
      "asset": "arbitrum_sepolia:usdc",
      "owner": "0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406",
      "amount": "49500000"
    }
  }'
```

### 2. Bring Base USDC back to Arbitrum
```bash
# Base → Arbitrum requires 25-35 USDC
curl -X POST "https://testnet.api.garden.finance/v2/orders" \
  -H "garden-app-id: f242ea49332293424c96c562a6ef575a819908c878134dcb4fce424dc84ec796" \
  -H "Content-Type: application/json" \
  -d '{
    "source": {
      "asset": "base_sepolia:usdc",
      "owner": "0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406",
      "amount": "25000000"
    },
    "destination": {
      "asset": "arbitrum_sepolia:usdc",
      "owner": "0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406",
      "amount": "24750000"
    }
  }'
```

### 3. Bring Ethereum USDC back to Arbitrum
```bash
# Ethereum → Arbitrum requires 15 USDC minimum
curl -X POST "https://testnet.api.garden.finance/v2/orders" \
  -H "garden-app-id: f242ea49332293424c96c562a6ef575a819908c878134dcb4fce424dc84ec796" \
  -H "Content-Type: application/json" \
  -d '{
    "source": {
      "asset": "ethereum_sepolia:usdc",
      "owner": "0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406",
      "amount": "15000000"
    },
    "destination": {
      "asset": "arbitrum_sepolia:usdc",
      "owner": "0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406",
      "amount": "14850000"
    }
  }'
```

## Better Approach: Enable Round-Trip Mode

Instead of manually running reverse swaps, enable round-trip mode which automatically includes reverse swaps:

```bash
# Add to .env file
ENABLE_ROUND_TRIPS=true

# Then run swaps
cargo run --release -- run-once
```

This will run 89 swaps (78 original + 11 round-trips) that automatically bring tokens back to source chains.

## Quick Check: Do You Need Reverse Swaps?

Check your current balances:
```bash
./check_balances.sh
```

If you still have enough ETH/USDC on Arbitrum, Base, and Ethereum, you might not need reverse swaps yet!

## Recommendation

For continuous testing without running out of tokens:
1. Enable `ENABLE_ROUND_TRIPS=true` in your `.env` file
2. Run `cargo run --release -- run-once`
3. The system will automatically swap tokens back to maintain balances
