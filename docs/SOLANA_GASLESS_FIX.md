# Solana Swap Initiation Fix

## Problem
Solana swaps were not being initiated. The code needed to sign and send transactions to initiate the swap on the source chain.

## Solution
After investigating the Garden Finance API, the correct approach for Solana testnet is:

1. Create order via POST `/v2/orders` - returns `versioned_tx`
2. Sign the transaction locally with user's wallet
3. Send the signed transaction directly to Solana RPC

Note: The gasless PATCH endpoint (`versioned_tx_gasless`) is not available on testnet for Solana.

## Changes Made

### 1. Environment Configuration
- Added `SOLANA_PRIVATE_KEY` to `.env` file
- Ensured the private key is loaded correctly in `AppConfig::from_env()`

### 2. Solana Signer (`src/chains/solana_signer.rs`)
The existing `sign_and_send()` method handles:
- Decoding the base64 versioned transaction from Garden API
- Signing with the user's keypair
- Sending to Solana RPC with proper configuration (skip_preflight, retries)

### 3. Swap Runner (`src/scheduler/runner.rs`)
Updated Solana initiation flow in `dispatch_initiation()`:
- Uses `versioned_tx` from the order response
- Signs and sends directly to Solana RPC
- Returns the transaction signature for tracking

## Testing
```bash
# Set the environment variable and run test
$env:SOLANA_PRIVATE_KEY="<your_private_key>"
cargo run --release -- test-swap solana_testnet:sol ethereum_sepolia:wbtc
```

## Results
- Transaction successfully signed and sent to Solana
- Transaction signature returned: `2JB1hMprfswhmFr7xQ3VkpJJWiGj3XgBMwA7K1WkKMuXP9dBFeNfkypEFGCKWvBie3Hg93LKUrtSb6KwsGcoAeub`
- Swap initiation is now working correctly

## References
- [Garden API - Create Order](https://docs.garden.finance/api-reference/endpoint/orders/create-order)
- [Garden API - Order Lifecycle](https://docs.garden.finance/developers/core/)
- [Solana Transaction Documentation](https://docs.solana.com/developing/programming-model/transactions)
