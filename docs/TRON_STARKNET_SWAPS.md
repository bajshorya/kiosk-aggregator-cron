# Tron and Starknet Swap Integration

## Summary
Added Tron and Starknet to the Solana-centric orchestration strategy with minimum swap amounts (50 USDC).

## New Swap Pairs Added

### Phase 1: DISTRIBUTE (2 new pairs)
- `solana_testnet:usdc → tron_shasta:usdc` (50 USDC)
- `solana_testnet:usdc → starknet_sepolia:usdc` (50 USDC)

### Phase 2: TEST (4 new pairs)
- `tron_shasta:usdc → ethereum_sepolia:usdc` (50 USDC)
- `tron_shasta:usdc → base_sepolia:usdc` (50 USDC)
- `starknet_sepolia:usdc → ethereum_sepolia:usdc` (50 USDC)
- `starknet_sepolia:usdc → arbitrum_sepolia:usdc` (50 USDC)

### Phase 3: CONSOLIDATE (2 new pairs)
- `tron_shasta:usdc → solana_testnet:usdc` (50 USDC)
- `starknet_sepolia:usdc → solana_testnet:usdc` (50 USDC)

## Total Swap Pairs: 26
- **Before**: 18 pairs (Solana, EVM, Sui)
- **After**: 26 pairs (Solana, EVM, Sui, Tron, Starknet)
- **Added**: 8 new pairs

## Breakdown by Phase
- **DISTRIBUTE**: 7 pairs (was 5)
- **TEST**: 12 pairs (was 8)
- **CONSOLIDATE**: 7 pairs (was 5)

## Minimum Amounts Used
All swaps use the minimum required by Garden API:
- **USDC**: 50,000,000 (50 USDC with 6 decimals)
- This is the API minimum to reduce costs while testing

## Implementation Status

### ✅ Completed
- Swap pairs configured in `src/chains/mod.rs`
- Tron signer implemented (`src/chains/tron_signer.rs`)
- Starknet signer implemented (`src/chains/starknet_signer.rs`)
- Configuration loaded from `.env`

### ⚠️ Pending Integration
The signers exist but are not yet integrated into the swap runner:

**Tron**: 
- Error: "Tron signing not yet implemented" in `dispatch_initiation()`
- Signer ready: Uses secp256k1 ECDSA signing
- Needs: Integration into `src/scheduler/runner.rs`

**Starknet**:
- Error: "Starknet signing not yet implemented" in `dispatch_initiation()`
- Signer ready: Uses Pedersen hash + ECDSA signing
- Needs: Integration into `src/scheduler/runner.rs`

## Configuration Required

### .env Settings
```bash
# Tron Shasta Testnet
WALLET_TRON=TWbEz5ibiL6dreiLJ5oBF5CwDkw6Xfe6KX
TRON_PRIVATE_KEY=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
RPC_TRON_SHASTA=https://api.shasta.trongrid.io

# Starknet Sepolia
WALLET_STARKNET=0x00609190b1348bcc06da44d58c79709495c11a5a6f0b9e154e1209f2a17dd933
STARKNET_PRIVATE_KEY=0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
RPC_STARKNET_SEPOLIA=https://starknet-sepolia.public.blastapi.io
```

## Next Steps to Enable Tron/Starknet Swaps

### 1. Integrate Tron Signer
Add to `src/scheduler/runner.rs` in `dispatch_initiation()`:

```rust
c if c.starts_with("tron_") => {
    use crate::chains::tron_signer::TronSigner;
    
    let private_key = self.config.wallets.tron_private_key
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("TRON_PRIVATE_KEY not set"))?;
    
    let signer = TronSigner::new(private_key.clone())?;
    
    // Check for gasless transaction data
    if let Some(tx_data) = &order_result.tron_transaction {
        let signature = signer.sign_transaction(tx_data).await?;
        let order_id = &order_result.order_id;
        
        self.api.initiate_swap_gasless_tron(order_id, &signature).await?;
        Ok(format!("gasless-tron-{}", order_id))
    } else {
        Err(anyhow::anyhow!("No Tron transaction data in response"))
    }
}
```

### 2. Integrate Starknet Signer
Add to `src/scheduler/runner.rs` in `dispatch_initiation()`:

```rust
c if c.starts_with("starknet_") => {
    use crate::chains::starknet_signer::StarknetSigner;
    
    let private_key = self.config.wallets.starknet_private_key
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("STARKNET_PRIVATE_KEY not set"))?;
    
    let signer = StarknetSigner::new(private_key.clone())?;
    
    // Check for typed data (gasless)
    if let Some(typed_data) = &order_result.starknet_typed_data {
        let signature = signer.sign_typed_data(typed_data).await?;
        let order_id = &order_result.order_id;
        
        self.api.initiate_swap_gasless_starknet(order_id, &signature).await?;
        Ok(format!("gasless-starknet-{}", order_id))
    } else {
        Err(anyhow::anyhow!("No Starknet typed data in response"))
    }
}
```

### 3. Add API Methods
Add to `src/api/mod.rs`:

```rust
pub async fn initiate_swap_gasless_tron(&self, order_id: &str, signature: &str) -> Result<()>
pub async fn initiate_swap_gasless_starknet(&self, order_id: &str, signature: &[String]) -> Result<()>
```

### 4. Update Models
Add to `src/models/mod.rs` in `SubmitOrderResult`:

```rust
pub tron_transaction: Option<serde_json::Value>,
pub starknet_typed_data: Option<serde_json::Value>,
```

## Current Behavior

### When Running Swaps
- **Tron swaps**: Will fail with "Tron signing not yet implemented"
- **Starknet swaps**: Will fail with "Starknet signing not yet implemented"
- **Other chains**: Will work normally (Solana, EVM, Sui)

### Workaround
Until integration is complete:
1. Disable Tron/Starknet swaps by commenting them out
2. Or accept that they will fail and focus on working chains
3. Or complete the integration steps above

## Testing Commands

```bash
# Test Tron swap (will fail until integrated)
cargo run -- test-swap "solana_testnet:usdc" "tron_shasta:usdc"

# Test Starknet swap (will fail until integrated)
cargo run -- test-swap "solana_testnet:usdc" "starknet_sepolia:usdc"

# Test all swaps (Tron/Starknet will fail)
cargo run -- run-all

# List all 26 swap pairs
cargo run -- list-swaps
```

## Benefits of Adding These Chains

1. **Broader Coverage**: Tests more blockchain ecosystems
2. **Liquidity Distribution**: Spreads test liquidity across more chains
3. **Cross-Chain Testing**: More swap combinations to test
4. **Future Ready**: Infrastructure in place for when signers are integrated

## Cost Optimization

All swaps use minimum amounts:
- **50 USDC per swap** = ~$50 per swap
- **8 new swaps** = ~$400 total for full cycle
- **26 total swaps** = ~$1,300 for complete test run

Using minimum amounts keeps testing costs low while validating functionality.
