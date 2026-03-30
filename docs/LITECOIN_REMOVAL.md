# Litecoin Support Removal

## Summary

All Litecoin references have been removed from the codebase because Garden Finance does not support Litecoin on testnet yet.

## Changes Made

### 1. Configuration (`src/config/mod.rs`)
- Removed `litecoin_testnet_address` field from `WalletConfig` struct
- Removed Litecoin address loading from environment variables

### 2. Chain Logic (`src/chains/mod.rs`)
- Removed `_ltc` variable from `all_swap_pairs()` function
- Updated `requires_manual_deposit()` to only check for Bitcoin (removed Litecoin check)

### 3. Swap Runner (`src/scheduler/runner.rs`)
- Updated Bitcoin swap partitioning logic to only check for `bitcoin_` prefix (removed `litecoin_` check)
- Bitcoin swaps now run sequentially with UTXO reuse, Litecoin no longer included

### 4. Environment Files
- **`.env`**: Removed `WALLET_LITECOIN_TESTNET` variable and commented-out reference
- **`.env.example`**: Removed Litecoin wallet address configuration section

### 5. Documentation (`README.md`)
- Removed Litecoin from supported chains table
- Updated feature list to remove Litecoin references
- Removed "Bitcoin/Litecoin Swaps" section from manual deposits
- Updated cost optimization section to remove Litecoin costs
- Updated automated execution documentation

## Impact

### Before Removal
- 80+ swap pairs including Litecoin routes
- Litecoin wallet configuration required
- Manual deposit handling for Litecoin swaps

### After Removal
- 80 swap pairs (Litecoin routes removed)
- No Litecoin wallet configuration needed
- Cleaner codebase focused on supported chains

## Supported Chains (After Removal)

1. **Bitcoin Testnet** - Fully automated with UTXO reuse
2. **Ethereum Sepolia** - EVM with gasless support
3. **Base Sepolia** - EVM with gasless support
4. **Arbitrum Sepolia** - EVM with gasless support
5. **Alpen Testnet** - EVM chain
6. **BNB Chain Testnet** - EVM chain
7. **Citrea Testnet** - EVM chain
8. **Monad Testnet** - EVM chain
9. **XRPL Testnet** - EVM-compatible
10. **Solana Testnet** - Versioned transactions with gasless
11. **Starknet Sepolia** - Typed data signing
12. **Tron Shasta** - EVM-compatible

## Future Considerations

If Garden Finance adds Litecoin support to testnet in the future, the following would need to be restored:

1. Add `litecoin_testnet_address` back to `WalletConfig`
2. Add Litecoin swap pairs to `all_swap_pairs()`
3. Update `requires_manual_deposit()` to include Litecoin
4. Add Litecoin to sequential execution logic (similar to Bitcoin)
5. Update documentation and environment files

## Verification

Build completed successfully after removal:
```bash
cargo build --release
# Finished `release` profile [optimized] target(s) in 16.86s
```

All Litecoin references have been cleanly removed without breaking existing functionality.
