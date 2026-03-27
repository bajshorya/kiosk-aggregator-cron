use anyhow::{Context, Result};
use serde_json::Value;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};
use tracing::info;

/// Tron signer for gasless transactions
/// Uses ECDSA secp256k1 signing with SHA256 hashing
pub struct TronSigner {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl TronSigner {
    pub fn new(private_key: String) -> Result<Self> {
        // Remove 0x prefix if present
        let key_str = private_key.trim_start_matches("0x");
        
        // Parse private key as bytes
        let key_bytes = hex::decode(key_str)
            .context("Failed to decode Tron private key hex")?;
        
        // Create secp256k1 context
        let secp = Secp256k1::new();
        
        // Create secret key
        let secret_key = SecretKey::from_slice(&key_bytes)
            .context("Invalid Tron private key")?;
        
        // Derive public key
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// Sign Tron typed data for gasless initiation
    /// Tron uses EIP-712-like signing, returns hex signature
    pub async fn sign_typed_data(&self, typed_data: &Value) -> Result<String> {
        info!("Signing Tron typed data for gasless initiation");
        
        // Extract the message hash from typed_data
        // The Garden API provides typed_data similar to EVM
        let message_str = serde_json::to_string(typed_data)?;
        
        // Hash the typed data with SHA256 (Tron uses SHA256)
        let hash = Sha256::digest(message_str.as_bytes());
        
        // Create secp256k1 context
        let secp = Secp256k1::new();
        
        // Create message from hash
        let message = Message::from_digest_slice(&hash)
            .context("Failed to create message from hash")?;
        
        // Sign the message
        let signature = secp.sign_ecdsa(&message, &self.secret_key);
        
        // Serialize signature to compact format (64 bytes: r + s)
        let sig_bytes = signature.serialize_compact();
        
        // Encode as hex with 0x prefix
        let sig_hex = format!("0x{}", hex::encode(sig_bytes));
        
        info!("Tron signature generated: {}", sig_hex);
        
        Ok(sig_hex)
    }

    /// Sign Tron transaction for non-gasless initiation
    /// Used when gasless is not available
    #[allow(dead_code)]
    pub async fn sign_transaction(&self, tx_data: &Value) -> Result<String> {
        info!("Signing Tron transaction for non-gasless initiation");
        
        // Get raw_data_hex from transaction
        let raw_data_hex = tx_data
            .get("raw_data_hex")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing raw_data_hex in transaction"))?;
        
        // Decode raw data
        let raw_bytes = hex::decode(raw_data_hex.trim_start_matches("0x"))
            .context("Failed to decode raw_data_hex")?;
        
        // Hash the raw data with SHA256 (Tron uses SHA256, not Keccak256)
        let hash = Sha256::digest(&raw_bytes);
        
        // Create secp256k1 context
        let secp = Secp256k1::new();
        
        // Create message from hash
        let message = Message::from_digest_slice(&hash)
            .context("Failed to create message from hash")?;
        
        // Sign the message
        let signature = secp.sign_ecdsa(&message, &self.secret_key);
        
        // Serialize signature to compact format (64 bytes: r + s)
        let sig_bytes = signature.serialize_compact();
        
        // Encode as hex with 0x prefix
        let sig_hex = format!("0x{}", hex::encode(sig_bytes));
        
        info!("Tron signature generated: {}", sig_hex);
        
        Ok(sig_hex)
    }

    /// Send a signed transaction to Tron network (non-gasless fallback)
    #[allow(dead_code)]
    pub async fn send_raw_transaction(
        &self,
        signed_tx: &Value,
        rpc_url: &str,
    ) -> Result<String> {
        info!("Sending Tron transaction via RPC (non-gasless)");
        
        // Create HTTP client
        let client = reqwest::Client::new();
        
        // Tron RPC endpoint for broadcasting
        let broadcast_url = format!("{}/wallet/broadcasttransaction", rpc_url);
        
        // Send transaction
        let response = client
            .post(&broadcast_url)
            .json(signed_tx)
            .send()
            .await
            .context("Failed to send Tron transaction")?;
        
        let status = response.status();
        let body = response.text().await?;
        
        if !status.is_success() {
            return Err(anyhow::anyhow!("Tron RPC error {}: {}", status, body));
        }
        
        // Parse response
        let result: Value = serde_json::from_str(&body)
            .context("Failed to parse Tron response")?;
        
        // Extract transaction ID
        let tx_id = result
            .get("txid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("No txid in Tron response"))?;
        
        info!("Tron transaction broadcasted: {}", tx_id);
        
        Ok(tx_id.to_string())
    }

    /// Get Tron address from public key
    #[allow(dead_code)]
    pub fn get_address(&self) -> Result<String> {
        // Serialize public key (uncompressed, 65 bytes)
        let pubkey_bytes = self.public_key.serialize_uncompressed();
        
        // Take last 64 bytes (skip the 0x04 prefix)
        let pubkey_hash_input = &pubkey_bytes[1..];
        
        // Hash with SHA256 (Tron uses SHA256 for address derivation)
        let hash = Sha256::digest(pubkey_hash_input);
        
        // Take last 20 bytes
        let address_bytes = &hash[12..];
        
        // Add Tron prefix (0x41 for mainnet, 0xa0 for testnet)
        let mut full_address = vec![0x41]; // Mainnet prefix
        full_address.extend_from_slice(address_bytes);
        
        // Encode as base58
        let address = bs58::encode(&full_address).into_string();
        
        Ok(address)
    }
}

/* 
Implementation Notes from gardenjs.md:

1. Tron uses TronWeb for signing
2. Transaction structure from API:
   {
     raw_data: {...},
     raw_data_hex: "0x...",
     txID: "..."
   }

3. Signing process:
   - Use tronweb.trx.sign(transaction)
   - Returns signed transaction object
   - Extract signature from signed transaction

4. Gasless endpoint: PATCH /v2/orders/{orderId}?action=initiate
   Body: { signature: "0x..." }

5. Non-gasless: POST signed transaction to Tron RPC
   - Endpoint: /wallet/broadcasttransaction
   - Body: signed transaction object
   - Returns: { result: true, txid: "..." }

6. Dependencies needed:
   - No official Rust Tron library
   - Options:
     a) Implement custom signing using secp256k1
     b) Use FFI to call TronWeb (Node.js)
     c) Use HTTP API for signing service
   
7. Tron address format:
   - Base58 encoded (starts with T)
   - Example: TQa4rN2Vayesv5vmMAXzP5QA1PnPwD46w6

8. Chain ID: 2494104990 (Shasta testnet)
*/
