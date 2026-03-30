# Litecoin Support Removal

## Overview
All Litecoin references have been removed from the codebase because Garden Finance does not support Litecoin on testnet yet.

## Changes Made

### 1. Swap Pairs (`src/chains/mod.rs`)
- Removed all Litecoin swap pairs from `all_swap_pairs()` function
- Updated `requires_manual_deposit()` to include `alpen_signet` (Bitcoin-style chain)
- Removed unused `_sui` variable

### 2. Swap Runner (`src/scheduler/runner.rs`)
- Updated Bitcoin swap dispatch logic to only check for `bitcoin_` prefix (removed `litecoin_` check)
- Fixed `alpen_testnet` matching to use prefix match instead of exact match
- Bitcoin swaps run sequentially with UTXO reuse, Litecoin no longer included

### 3. Environment Files
- **`.env`**: Commented out Sui wallet configuration (not currently used)
- **`.env.example`**: Removed Litecoin wallet address configuration section

### 4. Configuration (`src/config/mod.rs`)
- No changes needed - `WalletConfig` struct doesn't have `litecoin_testnet_address` field

## Testing
After removal, the system supports:
- Bitcoin Testnet (with automatic UTXO reuse)
- Alpen Signet (Bitcoin-style, manual deposit)
- All EVM chains (Ethereum, Base, Arbitrum, Alpen, BNB, Citrea, Monad, XRPL)
- Solana Testnet
- Starknet Sepolia
- Tron Shasta

## Future Restoration
If Garden Finance adds Litecoin support to testnet in the future, the following would need to be added:

1. Add Litecoin swap pairs to `all_swap_pairs()` in `src/chains/mod.rs`
2. Update `requires_manual_deposit()` to include Litecoin if it requires manual deposits
3. Add Litecoin dispatch logic to `dispatch_initiation()` in `src/scheduler/runner.rs`
4. Add `WALLET_LITECOIN_TESTNET` to `.env` and `.env.example`
5. Optionally add `litecoin_testnet_address` to `WalletConfig` struct

## Verification
Run the following to verify the changes:
```bash
# Check for any remaining Litecoin references
grep -r "litecoin" src/

# Build the project
cargo build --release

# Test a swap
cargo run --release -- test-swap ethereum_sepolia:eth solana_testnet:sol
```
