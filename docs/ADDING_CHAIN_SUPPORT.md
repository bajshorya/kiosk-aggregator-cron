# Adding Full Chain Support

This guide explains how to add complete support for Starknet, Tron, Sui, and other chains based on the gardenjs.md reference.

## Current Status

### ✅ Fully Implemented
- **EVM Chains** (Ethereum, Base, Arbitrum): Gasless + Non-gasless
- **Solana**: Gasless + Non-gasless
- **Bitcoin/Litecoin**: Manual deposit (no signing needed)

### 🚧 Partially Implemented (Stubs Created)
- **Starknet**: Signer stub created (`src/chains/starknet_signer.rs`)
- **Tron**: Signer stub created (`src/chains/tron_signer.rs`)
- **Sui**: Signer stub created (`src/chains/sui_signer.rs`)

## Starknet Implementation

### Dependencies Required
Add to `Cargo.toml`:
```toml
starknet = "0.9"
starknet-crypto = "0.6"
starknet-providers = "0.9"
starknet-accounts = "0.9"
```

### Signing Flow
1. **Typed Data Structure**:
```rust
// Domain
{
    name: "HTLC",
    version: "2",
    chainId: "0x534e5f5345504f4c4941", // Starknet Sepolia
    revision: "ACTIVE"
}

// Message (Initiate)
{
    redeemer: ContractAddress,
    amount: u256,
    timelock: u128,
    secretHash: u128[], // Array of u128 values
    verifyingContract: ContractAddress
}
```

2. **Signature Format**:
   - Returns array of field elements: `["0x...", "0x..."]`
   - Must format using `formatStarknetSignature()` logic
   - Converts signature object `{r, s}` to array `[r, s]`

3. **Implementation Steps**:
```rust
use starknet::accounts::{Account, SingleOwnerAccount};
use starknet::core::types::{FieldElement, TypedData};
use starknet::signers::{LocalWallet, SigningKey};

pub async fn sign_typed_data(&self, typed_data: &Value) -> Result<Vec<String>> {
    // 1. Parse typed data
    let typed: TypedData = serde_json::from_value(typed_data.clone())?;
    
    // 2. Create signer from private key
    let signing_key = SigningKey::from_secret_scalar(
        FieldElement::from_hex_be(&self.private_key)?
    );
    let signer = LocalWallet::from(signing_key);
    
    // 3. Sign typed data
    let signature = signer.sign_typed_data(&typed).await?;
    
    // 4. Format as array of hex strings
    let sig_array = vec![
        format!("0x{:x}", signature.r),
        format!("0x{:x}", signature.s),
    ];
    
    Ok(sig_array)
}
```

4. **Gasless Endpoint**:
```rust
// PATCH /v2/orders/{orderId}?action=initiate
// Body: { signature: ["0x...", "0x..."] }
```

### Non-Gasless Flow
For non-gasless, need to:
1. Approve token spending (if ERC20)
2. Call `initiate()` on HTLC contract
3. Wait for transaction confirmation

## Tron Implementation

### Dependencies Required
Tron doesn't have an official Rust SDK. Options:
1. **Custom Implementation**: Use `secp256k1` for signing
2. **HTTP API**: Use Tron's HTTP API for signing
3. **FFI**: Call TronWeb via Node.js FFI

### Recommended: Custom Implementation
Add to `Cargo.toml`:
```toml
secp256k1 = { version = "0.28", features = ["recovery"] }
sha3 = "0.10"
bs58 = "0.5"
```

### Signing Flow
1. **Transaction Structure**:
```json
{
  "raw_data": {
    "contract": [{
      "parameter": {
        "value": {
          "data": "0x...",
          "owner_address": "41...",
          "contract_address": "41..."
        }
      }
    }],
    "timestamp": 1234567890,
    "expiration": 1234567890
  },
  "raw_data_hex": "0x..."
}
```

2. **Signing Process**:
```rust
use secp256k1::{Secp256k1, SecretKey, Message};
use sha3::{Digest, Keccak256};

pub async fn sign_transaction(&self, tx_data: &Value) -> Result<String> {
    // 1. Get raw_data_hex
    let raw_data_hex = tx_data["raw_data_hex"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing raw_data_hex"))?;
    
    // 2. Hash the raw data
    let raw_bytes = hex::decode(raw_data_hex.trim_start_matches("0x"))?;
    let hash = Keccak256::digest(&raw_bytes);
    
    // 3. Sign with secp256k1
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(
        &hex::decode(&self.private_key)?
    )?;
    let message = Message::from_slice(&hash)?;
    let signature = secp.sign_ecdsa(&message, &secret_key);
    
    // 4. Format signature
    let sig_bytes = signature.serialize_compact();
    Ok(hex::encode(sig_bytes))
}
```

3. **Gasless Endpoint**:
```rust
// PATCH /v2/orders/{orderId}?action=initiate
// Body: { signature: "0x..." }
```

### Address Format
- Base58 encoded (starts with 'T' for testnet, '4' for mainnet)
- Convert to hex (41... prefix) for contract calls
- Example: `TQa4rN2Vayesv5vmMAXzP5QA1PnPwD46w6` → `41...`

## Sui Implementation

### Dependencies Required
Add to `Cargo.toml`:
```toml
sui-sdk = { version = "0.2", features = ["full"] }
sui-types = "0.2"
sui-keys = "0.2"
```

### Signing Flow
1. **Transaction Format**:
   - PTB (Programmable Transaction Block) as base64 bytes
   - Example: `"AAACAgEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE..."`

2. **Signing Process**:
```rust
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_types::crypto::{Ed25519SuiSignature, Signature};
use sui_types::transaction::TransactionData;

pub async fn sign_transaction(&self, tx_bytes: &str) -> Result<String> {
    // 1. Decode transaction bytes
    let tx_data = TransactionData::from_base64(tx_bytes)?;
    
    // 2. Create Ed25519 keypair from private key
    let keypair = Ed25519KeyPair::from_bytes(
        &hex::decode(&self.private_key)?
    )?;
    
    // 3. Sign transaction
    let signature = Signature::new_secure(
        &tx_data,
        &keypair
    );
    
    // 4. Encode signature as base64
    Ok(signature.encode_base64())
}
```

3. **Gasless Endpoint**:
```rust
// PATCH /v2/orders/{orderId}?action=initiate
// Body: { signature: "base64_signature" }
```

### Non-Gasless Flow
```rust
use sui_sdk::SuiClient;

pub async fn execute_transaction(
    &self,
    tx_bytes: &str,
    signature: &str,
    rpc_url: &str,
) -> Result<String> {
    let client = SuiClient::new(rpc_url).await?;
    
    let response = client
        .quorum_driver()
        .execute_transaction_block(
            tx_bytes,
            vec![signature],
            None,
            None,
        )
        .await?;
    
    Ok(response.digest.to_string())
}
```

## Integration Steps

### 1. Update `src/scheduler/runner.rs`

Add chain detection and routing:
```rust
fn detect_chain_type(asset: &str) -> ChainType {
    if asset.starts_with("starknet_") {
        ChainType::Starknet
    } else if asset.starts_with("tron_") {
        ChainType::Tron
    } else if asset.starts_with("sui_") {
        ChainType::Sui
    } else if asset.starts_with("ethereum_") || asset.starts_with("base_") || asset.starts_with("arbitrum_") {
        ChainType::Evm
    } else if asset.starts_with("solana_") {
        ChainType::Solana
    } else if asset.starts_with("bitcoin_") || asset.starts_with("litecoin_") {
        ChainType::ManualDeposit
    } else {
        ChainType::Unknown
    }
}
```

### 2. Update `dispatch_initiation()` Method

```rust
async fn dispatch_initiation(&self, order: &Order, pair: &SwapPair) -> Result<String> {
    let chain_type = detect_chain_type(&pair.source.asset);
    
    match chain_type {
        ChainType::Evm => {
            // Existing EVM logic
        }
        ChainType::Solana => {
            // Existing Solana logic
        }
        ChainType::Starknet => {
            if let Some(typed_data) = &order.typed_data {
                // Gasless Starknet
                let signer = StarknetSigner::new(self.config.wallets.starknet_private_key.clone())?;
                let signature = signer.sign_typed_data(typed_data).await?;
                self.api.initiate_swap_gasless_starknet(&order.order_id, &signature).await?;
            } else {
                // Non-gasless Starknet
                let signer = StarknetSigner::new(self.config.wallets.starknet_private_key.clone())?;
                let tx_hash = signer.send_transaction(&order.initiate_transaction, &self.config.starknet_rpc_url).await?;
                return Ok(tx_hash);
            }
        }
        ChainType::Tron => {
            // Similar to Starknet
        }
        ChainType::Sui => {
            // Similar to Starknet
        }
        ChainType::ManualDeposit => {
            // Existing Bitcoin/Litecoin logic
        }
        ChainType::Unknown => {
            return Err(anyhow::anyhow!("Unknown chain type"));
        }
    }
    
    Ok(String::new())
}
```

### 3. Add Gasless Endpoints to `src/api/mod.rs`

```rust
pub async fn initiate_swap_gasless_starknet(
    &self,
    order_id: &str,
    signature: &[String],
) -> Result<()> {
    let url = format!(
        "{}/v2/orders/{}?action=initiate",
        self.config.api_base_url, order_id
    );
    
    let payload = serde_json::json!({
        "signature": signature
    });
    
    let (hk, hv) = self.app_id_header();
    
    let resp = self
        .client
        .patch(&url)
        .header(hk, &hv)
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await?;
    
    // Handle response...
    Ok(())
}

pub async fn initiate_swap_gasless_tron(
    &self,
    order_id: &str,
    signature: &str,
) -> Result<()> {
    // Similar to Starknet but signature is a single string
}

pub async fn initiate_swap_gasless_sui(
    &self,
    order_id: &str,
    signature: &str,
) -> Result<()> {
    // Similar to Tron
}
```

### 4. Update Configuration

Add to `.env`:
```env
# Starknet
STARKNET_PRIVATE_KEY=0x...
STARKNET_RPC_URL=https://starknet-sepolia.public.blastapi.io

# Tron
TRON_PRIVATE_KEY=...
TRON_RPC_URL=https://api.shasta.trongrid.io

# Sui
SUI_PRIVATE_KEY=...
SUI_RPC_URL=https://fullnode.testnet.sui.io:443
```

Add to `src/config/mod.rs`:
```rust
pub struct WalletConfig {
    // Existing fields...
    pub starknet_private_key: String,
    pub tron_private_key: String,
    pub sui_private_key: String,
}

pub struct GardenConfig {
    // Existing fields...
    pub starknet_rpc_url: String,
    pub tron_rpc_url: String,
    pub sui_rpc_url: String,
}
```

## Testing

### Starknet Test
```bash
cargo run --release -- test-swap starknet_sepolia:wbtc base_sepolia:wbtc
```

### Tron Test
```bash
cargo run --release -- test-swap tron_shasta:wbtc base_sepolia:wbtc
```

### Sui Test
```bash
cargo run --release -- test-swap sui_testnet:sui bitcoin_testnet:btc
```

## Troubleshooting

### Starknet Issues
- **Invalid signature format**: Ensure signature is array `["0x...", "0x..."]`
- **Wrong chain ID**: Use correct chain ID for testnet vs mainnet
- **Contract not found**: Verify HTLC contract address

### Tron Issues
- **Address format**: Convert between Base58 and hex (41... prefix)
- **Signature format**: Ensure proper ECDSA signature encoding
- **Energy/Bandwidth**: Non-gasless requires TRX for fees

### Sui Issues
- **PTB deserialization**: Ensure correct base64 decoding
- **Ed25519 key format**: Verify private key is 32 bytes
- **Gas object**: Non-gasless requires gas object selection

## References

- **gardenjs.md**: Complete reference implementation (38k lines)
- **Starknet Docs**: https://docs.starknet.io/
- **Tron Docs**: https://developers.tron.network/
- **Sui Docs**: https://docs.sui.io/
- **Garden Finance API**: https://docs.garden.finance/
