use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey};
use serde_json::Value;
use tracing::info;

/// Sui signer for gasless transactions
/// Uses Ed25519 keypair for signing PTBs (Programmable Transaction Blocks)
#[allow(dead_code)]
pub struct SuiSigner {
    signing_key: SigningKey,
}

#[allow(dead_code)]
impl SuiSigner {
    pub fn new(private_key: String) -> Result<Self> {
        let key_bytes = if private_key.starts_with("suiprivkey1") {
            // Bech32-encoded Sui private key
            info!("Decoding Bech32-encoded Sui private key");
            Self::decode_bech32_key(&private_key)?
        } else {
            // Hex-encoded private key
            let key_str = private_key.trim_start_matches("0x");
            hex::decode(key_str)
                .context("Failed to decode Sui private key hex")?
        };
        
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Invalid Sui private key length: expected 32 bytes, got {}",
                key_bytes.len()
            ));
        }
        
        // Create Ed25519 signing key
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        let signing_key = SigningKey::from_bytes(&key_array);
        
        Ok(Self { signing_key })
    }

    /// Decode Bech32-encoded Sui private key (format: suiprivkey1...)
    fn decode_bech32_key(encoded: &str) -> Result<Vec<u8>> {
        use bech32::FromBase32;
        
        let (hrp, data, _variant) = bech32::decode(encoded)
            .context("Failed to decode Bech32 Sui private key")?;
        
        if hrp != "suiprivkey" {
            return Err(anyhow::anyhow!(
                "Invalid Sui private key HRP: expected 'suiprivkey', got '{}'",
                hrp
            ));
        }
        
        let bytes = Vec::<u8>::from_base32(&data)
            .context("Failed to convert Bech32 data to bytes")?;
        
        Ok(bytes)
    }

    /// Sign Sui PTB (Programmable Transaction Block) for gasless initiation
    /// Sui uses Ed25519 signatures
    pub async fn sign_transaction(&self, tx_bytes: &str) -> Result<String> {
        info!("Signing Sui PTB for gasless initiation");
        
        // Decode transaction bytes from base64
        use base64::{engine::general_purpose, Engine as _};
        let tx_data = general_purpose::STANDARD.decode(tx_bytes)
            .context("Failed to decode Sui transaction bytes from base64")?;
        
        // Sign the transaction data
        let signature: Signature = self.signing_key.sign(&tx_data);
        
        // Encode signature as base64
        let sig_base64 = general_purpose::STANDARD.encode(signature.to_bytes());
        
        info!("Sui signature generated (base64): {}", sig_base64);
        
        Ok(sig_base64)
    }

    /// Execute a signed transaction on Sui network (non-gasless fallback)
    pub async fn execute_transaction(
        &self,
        tx_bytes: &str,
        signature: &str,
        rpc_url: &str,
    ) -> Result<String> {
        info!("Executing Sui transaction via RPC (non-gasless)");
        
        // Create HTTP client
        let client = reqwest::Client::new();
        
        // Prepare RPC request for sui_executeTransactionBlock
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sui_executeTransactionBlock",
            "params": [
                tx_bytes,
                [signature],
                {
                    "showInput": true,
                    "showRawInput": false,
                    "showEffects": true,
                    "showEvents": true,
                    "showObjectChanges": false,
                    "showBalanceChanges": false
                },
                "WaitForLocalExecution"
            ]
        });
        
        // Send request
        let response = client
            .post(rpc_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send Sui RPC request")?;
        
        let status = response.status();
        let body = response.text().await?;
        
        if !status.is_success() {
            return Err(anyhow::anyhow!("Sui RPC error {}: {}", status, body));
        }
        
        // Parse response
        let result: Value = serde_json::from_str(&body)
            .context("Failed to parse Sui response")?;
        
        // Extract transaction digest
        let digest = result
            .get("result")
            .and_then(|r| r.get("digest"))
            .and_then(|d| d.as_str())
            .ok_or_else(|| anyhow::anyhow!("No digest in Sui response"))?;
        
        info!("Sui transaction executed: {}", digest);
        
        Ok(digest.to_string())
    }

    /// Get Sui address from public key
    pub fn get_address(&self) -> Result<String> {
        // Get public key
        let public_key = self.signing_key.verifying_key();
        let pubkey_bytes = public_key.to_bytes();
        
        // Sui address is derived from: BLAKE2b(flag || pubkey)[0..32]
        // where flag = 0x00 for Ed25519
        // Note: For now we use SHA256 as a placeholder since blake2 crate needs to be added
        // TODO: Add blake2 = "0.10" to Cargo.toml and use proper BLAKE2b hashing
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(&[0x00]); // Ed25519 flag
        hasher.update(&pubkey_bytes);
        let hash = hasher.finalize();
        
        // Take first 32 bytes and encode as hex with 0x prefix
        let address = format!("0x{}", hex::encode(&hash[..32]));
        
        Ok(address)
    }
}

/* 
Implementation Notes from gardenjs.md:

1. Sui uses Ed25519 keypairs for signing
2. Transaction format: PTB (Programmable Transaction Block)
   - Serialized as base64 bytes
   - Example: "AAACAgEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE..."

3. Signing process:
   - Deserialize PTB bytes
   - Sign with Ed25519 keypair
   - Return signature as base64

4. Gasless endpoint: PATCH /v2/orders/{orderId}?action=initiate
   Body: { signature: "base64_signature" }

5. Non-gasless: Execute PTB directly
   - Use sui_executeTransactionBlock RPC method
   - Requires: transaction bytes + signature + public key

6. Dependencies needed:
   - sui-sdk = "0.2"
   - sui-types = "0.2"
   - Add to Cargo.toml:
     sui-sdk = { version = "0.2", features = ["full"] }

7. Sui address format:
   - Hex encoded (0x prefix)
   - 32 bytes (64 hex characters)
   - Example: 0x5c438715b7dcc02d12ab92449153a1e5ade2301620d5bf60aa748f006726d369

8. Chain: sui (mainnet) or sui_testnet

9. Key generation from private key:
   - Use Ed25519Keypair::from_bytes()
   - Private key is 32 bytes
   - Can derive from seed phrase or raw bytes

10. Transaction structure from API:
    {
      transaction: "base64_ptb_bytes",
      // For gasless:
      transaction_gasless: "base64_ptb_bytes_without_gas"
    }

11. Wallet Standard integration:
    - Sui uses @mysten/wallet-standard
    - Feature: 'sui:signAndExecuteTransaction'
    - Returns: { digest, effects, events }
*/
