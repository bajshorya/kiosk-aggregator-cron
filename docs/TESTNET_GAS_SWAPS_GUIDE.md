# Testnet Gas-Based Swaps Guide

## Overview

Since gasless is not enabled, we'll use traditional gas-based swaps where you pay transaction fees.

## Requirements

### 1. Testnet Tokens Needed

#### For EVM Chains (Ethereum, Base, Arbitrum)
You need testnet ETH for gas fees:

**Ethereum Sepolia:**
- Faucet: https://sepoliafaucet.com/
- Alternative: https://www.alchemy.com/faucets/ethereum-sepolia
- Amount needed: ~0.01 ETH per swap

**Base Sepolia:**
- Faucet: https://www.coinbase.com/faucets/base-ethereum-sepolia-faucet
- Amount needed: ~0.001 ETH per swap

**Arbitrum Sepolia:**
- Faucet: https://faucet.quicknode.com/arbitrum/sepolia
- Amount needed: ~0.001 ETH per swap

**Your EVM Wallet:** `0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406`

#### For Solana
You need testnet SOL for transaction fees:

**Solana Testnet:**
- Faucet: https://faucet.solana.com/
- Command: `solana airdrop 2 5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny --url testnet`
- Amount needed: ~0.01 SOL per swap

**Your Solana Wallet:** `5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny`

#### For Bitcoin/Litecoin
Manual deposit (no gas fees, but requires manual sending):

**Bitcoin Testnet:**
- Faucet: https://testnet-faucet.com/btc-testnet/
- Your address: `tb1qypyl87pagnmcted8d8gnmxxps7dyqstllhhe9z`

## How It Works

### EVM Chains
1. Get quote from Garden API
2. Create order
3. **Sign and broadcast transaction to RPC** (pays gas)
4. Wait for confirmation
5. Garden completes the swap

### Solana
1. Get quote from Garden API
2. Create order
3. **Sign and broadcast transaction to Solana RPC** (pays gas)
4. Wait for confirmation
5. Garden completes the swap

### Bitcoin/Litecoin (UTXO)
1. Get quote from Garden API
2. Create order
3. **Manually send BTC/LTC to deposit address**
4. Wait for confirmation
5. Garden completes the swap

## Testing Steps

### Step 1: Get Testnet Tokens

```bash
# Get Ethereum Sepolia ETH
# Visit: https://sepoliafaucet.com/
# Enter: 0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406

# Get Solana testnet SOL
solana airdrop 2 5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny --url testnet

# Or use web faucet: https://faucet.solana.com/
```

### Step 2: Verify Balances

```bash
# Check Ethereum Sepolia balance
# Visit: https://sepolia.etherscan.io/address/0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406

# Check Solana balance
solana balance 5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny --url testnet
```

### Step 3: Run Test Swaps

```bash
# Set Solana private key
$env:SOLANA_PRIVATE_KEY="3Nb6qpea1cgbCVqYAGPMJZqg4KXd9BSgnY9shsXYYoBFEduSFtJifoJs3cWznouL3q3isMhVW3kt4ntDcaZijEJM"

# Test EVM to EVM (requires ETH for gas)
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc

# Test Solana to EVM (requires SOL for gas)
cargo run --release -- test-swap solana_testnet:sol ethereum_sepolia:wbtc

# Test EVM to Solana (requires ETH for gas)
cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol
```

## Current Implementation Status

### ✅ Working (with gas)
- **EVM → EVM**: Ethereum ↔ Base ↔ Arbitrum
- **Solana → Solana**: SOL swaps (if liquidity available)
- **Bitcoin/Litecoin**: Manual deposit swaps

### ⚠️ May Not Work
- **Solana non-gasless**: API may reject (needs testing with actual SOL balance)
- **Low liquidity pairs**: Some testnet pairs have no liquidity

## Troubleshooting

### "Failed to send transaction"
**Cause**: Insufficient gas or RPC issue
**Solution**: 
1. Check you have enough testnet ETH/SOL
2. Try a different RPC endpoint
3. Wait and retry (RPC might be congested)

### "Insufficient liquidity"
**Cause**: No liquidity providers for this pair on testnet
**Solution**:
1. Try a different swap pair
2. Use mainnet instead
3. Wait for testnet liquidity to be added

### "Missing signature" (Solana)
**Cause**: Solana non-gasless might not be supported
**Solution**:
1. Contact Garden Finance to enable gasless
2. Or test on mainnet where gasless might be enabled

## Cost Estimates

### Testnet (Free)
- EVM gas: ~0.001-0.01 ETH per swap (free from faucets)
- Solana fees: ~0.000005 SOL per transaction (free from faucets)
- Bitcoin/Litecoin: Free (manual deposit)

### Mainnet (Real Cost)
- EVM gas: $1-5 per swap (depending on network)
- Solana fees: ~$0.0001 per transaction
- Bitcoin/Litecoin: Network fees apply

## Next Steps

1. **Get testnet tokens** from faucets
2. **Test EVM swaps** (most likely to work)
3. **Test Solana swaps** (may need gasless)
4. **Contact Garden Finance** to enable gasless for production

## Configuration

Your current `.env` is already configured correctly:

```bash
GARDEN_API_BASE_URL=https://testnet.api.garden.finance
GARDEN_APP_ID=79702d04cb63391922f2e1471afe4743b0e3ba71f260e3c2117aa36e7fb74a9c

WALLET_EVM=0x55D246A6c46C4371ffcb6e18cD1D10Eca2d6F406
WALLET_EVM_PRIVATE_KEY=92796a4469a152563fa7790aca17caad6ecdeea7c20740e06538de01f3a64566

WALLET_SOLANA=5ymiDmUNjH1ehfVaR9W95kNXyKuaLsVPGBS7dXLP41ny
SOLANA_PRIVATE_KEY=3Nb6qpea1cgbCVqYAGPMJZqg4KXd9BSgnY9shsXYYoBFEduSFtJifoJs3cWznouL3q3isMhVW3kt4ntDcaZijEJM

RPC_ETHEREUM_SEPOLIA=https://rpc.sepolia.org
RPC_SOLANA_TESTNET=https://api.testnet.solana.com
```

## Summary

The code is ready for gas-based swaps. You just need:
1. ✅ Testnet ETH in your EVM wallet
2. ✅ Testnet SOL in your Solana wallet
3. ✅ Run the test commands

Once you have the tokens, the swaps should work automatically!
