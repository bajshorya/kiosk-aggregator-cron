# Garden Finance Gasless Implementation Status

## Overview
Implementing proper gasless swap initiation for Garden Finance API according to their documentation.

## Key Findings

### 1. EVM Chains (Ethereum, Base, Arbitrum)
- **Correct Approach**: EIP-712 signature via gasless endpoint
- **Implementation**: ✅ Completed
  - Sign `typed_data` with `eth_signTypedData_v4`
  - POST signature to `/v2/orders/{id}?action=initiate`
  - No ETH needed for gas
- **Status**: Signature generated successfully, but getting "Invalid swap request" error
- **Next Steps**: 
  - Verify order needs to be in "Matched" state before initiation
  - Check if there's a timing issue or additional validation needed

### 2. Solana
- **Correct Approach**: Sign versioned transaction and submit via gasless endpoint
- **Implementation**: ✅ Completed
  - Sign `versioned_tx` bytes
  - POST to `/v2/orders/{id}?action=initiate` with `serialized_tx`
- **Status**: Getting "Missing signature" error
- **Issue**: `versioned_tx_gasless=false` in API response suggests gasless not available on testnet
- **Workaround Attempted**: Direct RPC send - transaction sent but not recognized by Garden

### 3. Bitcoin/Litecoin
- **Approach**: Manual deposit to provided address
- **Status**: Working as expected

## API Response Fields by Chain

### EVM Response
```json
{
  "typed_data": {...},           // ✅ Present
  "initiate_transaction": {...}, // Present but not used in gasless flow
  "versioned_tx": false,
  "versioned_tx_gasless": false
}
```

### Solana Response  
```json
{
  "versioned_tx": "base64...",    // ✅ Present
  "versioned_tx_gasless": false,  // ❌ Not available on testnet
  "typed_data": false,
  "initiate_transaction": false
}
```

## Code Changes Made

### 1. EVM Signer (`src/chains/evm_signer.rs`)
- Removed raw transaction broadcasting
- Added `sign_typed_data()` method for EIP-712 signing
- Returns hex signature for gasless endpoint

### 2. Solana Signer (`src/chains/solana_signer.rs`)
- Added `sign_transaction()` method
- Returns base64-encoded signed transaction
- Kept `sign_and_send()` for direct RPC (not used in gasless flow)

### 3. API Client (`src/api/mod.rs`)
- Added `initiate_swap_gasless_evm()` for EVM signature submission
- Added `initiate_swap_gasless()` for Solana serialized_tx submission

### 4. Swap Runner (`src/scheduler/runner.rs`)
- Updated `dispatch_initiation()` to use gasless flow
- EVM: Signs typed_data and submits signature
- Solana: Signs versioned_tx and submits serialized_tx

## Current Errors

### EVM: "Invalid swap request"
```
PATCH https://testnet.api.garden.finance/v2/orders/{id}?action=initiate
Body: {"signature": "0x..."}
Response: 400 Bad Request - {"status":"Error","error":"Invalid swap request"}
```

**Possible Causes**:
1. Order not in correct state (needs to be "Matched")
2. Signature format issue
3. Missing additional fields in request
4. Timing issue - need to wait after order creation

### Solana: "Missing signature"
```
PATCH https://testnet.api.garden.finance/v2/orders/{id}?action=initiate
Body: {"serialized_tx": "base64..."}
Response: 400 Bad Request - {"status":"Error","error":"Missing signature"}
```

**Possible Causes**:
1. Gasless endpoint not available for Solana on testnet
2. Different payload structure expected
3. Should use different field name

## Testing Commands

### EVM Swap Test
```bash
cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc
```

### Solana Swap Test  
```bash
$env:SOLANA_PRIVATE_KEY="<key>"
cargo run --release -- test-swap solana_testnet:sol ethereum_sepolia:wbtc
```

## References
- [Garden API - Gasless Endpoint](https://docs.garden.finance/api-reference/endpoint/orders/patch)
- [Garden API - Order Lifecycle](https://docs.garden.finance/developers/core/)
- [Garden API - Create Order](https://docs.garden.finance/api-reference/endpoint/orders/create-order)

## Next Actions
1. Check if orders need to be in "Matched" state before initiation
2. Contact Garden Finance support for clarification on testnet gasless support
3. Test on mainnet where gasless features may be fully enabled
4. Consider fallback to manual deposit flow for chains where gasless isn't available
