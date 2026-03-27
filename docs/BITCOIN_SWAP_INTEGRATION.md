# Bitcoin Swap Integration - Complete Implementation

## Overview

Bitcoin swaps are now **fully integrated** and can execute automatically when a Bitcoin private key (WIF format) is configured.

## What Was Implemented

### 1. Bitcoin Provider (`src/chains/bitcoin_provider.rs`)
- **UTXO Fetching**: Get unspent transaction outputs for an address
- **Balance Checking**: Query Bitcoin balance
- **Transaction Broadcasting**: Submit signed transactions to the network
- **Fee Estimation**: Get current network fee rates
- **Fee Calculation**: Estimate transaction fees based on inputs/outputs

**RPC Endpoint**: Uses Blockstream.info API (free, no API key required)

### 2. Bitcoin Signer (`src/chains/bitcoin_signer.rs`)
- **WIF Private Key Support**: Initialize from Wallet Import Format
- **P2WPKH Addresses**: Native SegWit (bech32) address generation
- **Transaction Building**: Construct Bitcoin transactions with UTXOs
- **Transaction Signing**: Sign with SegWit v0 signatures
- **Change Handling**: Automatic change output calculation

### 3. Configuration (`src/config/mod.rs`)
Added Bitcoin configuration fields:
- `bitcoin_testnet_private_key: Option<String>` - WIF private key
- `bitcoin_testnet: String` - RPC endpoint URL

### 4. Swap Runner Integration (`src/scheduler/runner.rs`)
Added automatic Bitcoin transaction initiation in `dispatch_initiation()`:
1. Check if Bitcoin private key is configured
2. Get deposit address from Garden API order
3. Fetch UTXOs from Bitcoin provider
4. Calculate fees and verify sufficient balance
5. Build and sign transaction
6. Broadcast to Bitcoin network
7. Return transaction ID

## Configuration

### Required Environment Variables

Add to your `.env` file:

```bash
# Bitcoin Testnet Private Key (WIF format)
# Get from: Electrum wallet > Export Private Keys
# Or: Bitcoin Core > dumpprivkey <address>
# Format: Testnet WIF starts with 'c' or '9'
BITCOIN_TESTNET_PRIVATE_KEY=cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy

# Bitcoin Testnet RPC (optional, defaults to Blockstream)
RPC_BITCOIN_TESTNET=https://blockstream.info/testnet/api
```

### Getting Your Bitcoin WIF Private Key

#### From Electrum Wallet:
1. Open Electrum
2. Wallet → Private Keys → Export
3. Copy the WIF string (starts with `c` for testnet)

#### From Bitcoin Core:
```bash
bitcoin-cli -testnet dumpprivkey <your_address>
```

#### Generate New Testnet Wallet:
```bash
# Using Bitcoin Core
bitcoin-cli -testnet getnewaddress
bitcoin-cli -testnet dumpprivkey <address>
```

## How It Works

### Automatic Bitcoin Swap Flow

1. **Quote & Order Creation**
   - Get quote from Garden API
   - Create order with Bitcoin as source asset

2. **UTXO Fetching**
   - Query Blockstream API for confirmed UTXOs
   - Calculate total available balance

3. **Fee Estimation**
   - Get current network fee rate (sats/vbyte)
   - Calculate transaction fee based on UTXO count
   - Typical: ~140 vbytes for 1 input, 2 outputs

4. **Transaction Building**
   - Add inputs from UTXOs
   - Add output to Garden deposit address
   - Add change output back to wallet (if > 546 sats)

5. **Transaction Signing**
   - Sign each input with P2WPKH witness
   - Create proper SegWit v0 signatures

6. **Broadcasting**
   - Submit signed transaction to Bitcoin network
   - Return transaction ID to Garden API

7. **Monitoring**
   - Poll Garden API for swap status
   - Wait for Bitcoin confirmations
   - Complete when destination chain receives funds

## Supported Bitcoin Swap Pairs

Total: **16 Bitcoin swap pairs** (from `src/chains/mod.rs`)

### Bitcoin → Other Chains:
- `bitcoin_testnet:btc → bnbchain_testnet:wbtc`
- `bitcoin_testnet:btc → citrea_testnet:usdc`
- `bitcoin_testnet:btc → monad_testnet:usdc`
- `bitcoin_testnet:btc → solana_testnet:usdc`
- `bitcoin_testnet:btc → starknet_sepolia:wbtc`
- `bitcoin_testnet:btc → tron_shasta:usdt`
- `bitcoin_testnet:btc → xrpl_testnet:xrp`

### Other Chains → Bitcoin:
- `arbitrum_sepolia:usdc → bitcoin_testnet:btc`
- `ethereum_sepolia:usdc → bitcoin_testnet:btc`
- `alpen_signet:btc → bitcoin_testnet:btc`
- `alpen_testnet:usdc → bitcoin_testnet:btc`
- `base_sepolia:usdc → bitcoin_testnet:btc`

### Round-Trip Swaps (when `ENABLE_ROUND_TRIPS=true`):
- `arbitrum_sepolia:wbtc → bitcoin_testnet:btc`
- `base_sepolia:wbtc → bitcoin_testnet:btc`
- `ethereum_sepolia:wbtc → bitcoin_testnet:btc`

**Minimum Amount**: 50,000 sats (0.0005 BTC) ≈ $50

## Testing

### Test a Bitcoin Swap

```bash
# Make sure you have testnet BTC and WIF key configured
cargo run --release -- test-swap bitcoin_testnet:btc solana_testnet:usdc
```

### Check Bitcoin Balance

```bash
# Using Blockstream API
curl https://blockstream.info/testnet/api/address/<your_address>
```

### Get Testnet Bitcoin

Faucets:
- https://testnet-faucet.com/btc-testnet/
- https://coinfaucet.eu/en/btc-testnet/
- https://bitcoinfaucet.uo1.net/

## Technical Details

### Transaction Structure

```
Version: 2
Inputs: [UTXO1, UTXO2, ...]
Outputs:
  - Recipient: <deposit_address> (amount)
  - Change: <wallet_address> (total - amount - fee)
Witnesses: [signature + pubkey for each input]
```

### Fee Calculation

```rust
// P2WPKH transaction size estimation
vbytes = 10.5 + (num_inputs * 68) + (num_outputs * 31)
fee = vbytes * fee_rate_sats_per_vbyte
```

Example:
- 1 input, 2 outputs: ~140 vbytes
- At 10 sats/vbyte: ~1,400 sats fee

### Address Format

- **Testnet**: `tb1q...` (bech32, P2WPKH)
- **Mainnet**: `bc1q...` (bech32, P2WPKH)

## Behavior Changes

### Before This Implementation:
```
Bitcoin swaps → Manual deposit required
User must manually send BTC to deposit address
Logs show: "⚠️ MANUAL DEPOSIT REQUIRED"
```

### After This Implementation:
```
Bitcoin swaps → Automatic if WIF key configured
System fetches UTXOs, builds tx, signs, broadcasts
Logs show: "Bitcoin transaction broadcasted: <txid>"

If no WIF key → Falls back to manual deposit
```

## Error Handling

### Common Errors:

1. **No Private Key**
   ```
   Error: BITCOIN_TESTNET_PRIVATE_KEY not set in .env
   Solution: Add your WIF private key to .env
   ```

2. **No UTXOs**
   ```
   Error: No UTXOs available for Bitcoin address
   Solution: Send testnet BTC to your address
   ```

3. **Insufficient Balance**
   ```
   Error: Insufficient Bitcoin balance: have X sats, need Y sats
   Solution: Get more testnet BTC from faucets
   ```

4. **Invalid WIF**
   ```
   Error: Failed to parse Bitcoin WIF private key
   Solution: Check WIF format (testnet starts with 'c' or '9')
   ```

## Security Considerations

1. **Private Key Storage**
   - WIF keys in `.env` (gitignored)
   - Never commit private keys to git
   - Use testnet keys for development

2. **Network Validation**
   - WIF includes network info (testnet vs mainnet)
   - Prevents accidental mainnet transactions

3. **Fee Validation**
   - Checks fee doesn't exceed amount
   - Prevents overpaying fees

4. **UTXO Confirmation**
   - Only uses confirmed UTXOs
   - Prevents double-spend issues

5. **Change Output**
   - Enforces 546 sat dust limit
   - Prevents unspendable outputs

## Limitations

### Current Implementation:
- ✅ P2WPKH (native SegWit) only
- ✅ Testnet support
- ✅ Automatic UTXO selection (uses all available)
- ✅ Fee estimation from network

### Not Yet Implemented:
- ❌ Mainnet support (easy to add)
- ❌ P2SH, P2PKH, P2TR addresses
- ❌ Custom UTXO selection
- ❌ RBF (Replace-By-Fee)
- ❌ CPFP (Child Pays For Parent)
- ❌ Multi-signature wallets

## Future Enhancements

1. **Mainnet Support**
   - Add `BITCOIN_MAINNET_PRIVATE_KEY`
   - Add mainnet RPC endpoint
   - Update network detection logic

2. **Advanced Fee Control**
   - Custom fee rate selection
   - Fee bumping (RBF)
   - Batch transactions

3. **UTXO Management**
   - Smart UTXO selection
   - UTXO consolidation
   - Coin control

4. **Additional Address Types**
   - P2SH support
   - Taproot (P2TR) support
   - Legacy (P2PKH) support

## Conclusion

Bitcoin swaps are now **fully automated** when a WIF private key is configured. The implementation follows Bitcoin best practices and integrates seamlessly with the existing swap flow.

**Status**: ✅ Ready for testing on Bitcoin testnet
