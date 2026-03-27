use anyhow::{Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use starknet::core::crypto::ecdsa_sign;
use starknet::core::types::FieldElement;
use starknet_crypto::get_public_key;
use tracing::info;

/// Starknet signer for gasless transactions
/// Uses ECDSA signing with Starknet typed data (EIP-712-like)
pub struct StarknetSigner {
    private_key: FieldElement,
}

impl StarknetSigner {
    pub fn new(private_key: String) -> Result<Self> {
        // Remove 0x prefix if present
        let key_str = private_key.trim_start_matches("0x");
        
        // Parse private key as FieldElement
        let private_key = FieldElement::from_hex_be(key_str)
            .context("Failed to parse Starknet private key")?;
        
        Ok(Self { private_key })
    }

    /// Sign Starknet typed data for gasless initiation
    /// Returns signature as comma-separated string "r,s" in decimal format
    pub async fn sign_typed_data(&self, typed_data: &Value) -> Result<String> {
        info!("Signing Starknet typed data for gasless initiation");
        
        // The typed_data from Garden API should already contain the message hash
        // We need to extract it and sign it
        let message_hash = self.extract_message_hash(typed_data)?;
        
        info!("Message hash: {:#x}", message_hash);
        
        // Sign the message hash
        let signature = ecdsa_sign(&self.private_key, &message_hash)
            .context("Failed to sign Starknet typed data")?;
        
        // Format signature as comma-separated decimal strings: "r,s"
        // This matches the formatStarknetSignature function from gardenjs.md
        let sig_str = format!("{},{}", signature.r, signature.s);
        
        info!("Starknet signature generated: {}", sig_str);
        
        Ok(sig_str)
    }

    /// Extract message hash from typed data
    /// The Garden API provides the typed_data structure that needs to be hashed
    fn extract_message_hash(&self, typed_data: &Value) -> Result<FieldElement> {
        // For Starknet, the typed_data contains domain, types, primaryType, and message
        // We need to compute the message hash according to Starknet's EIP-712-like standard
        
        // For now, we'll use a simplified approach
        // In production, you'd want to use starknet-rs's typed data hashing
        
        let message = typed_data.get("message")
            .ok_or_else(|| anyhow::anyhow!("Missing message in typed_data"))?;
        
        // Convert the message to a string and hash it
        let message_str = serde_json::to_string(message)?;
        
        // Use a simple hash for now - in production, use proper Starknet typed data hashing
        // This is a placeholder that should be replaced with proper implementation
        let hash_bytes = Sha256::digest(message_str.as_bytes());
        let hash_hex = hex::encode(hash_bytes);
        
        FieldElement::from_hex_be(&hash_hex)
            .context("Failed to create message hash")
    }

    /// Get Starknet address from private key
    #[allow(dead_code)]
    pub fn get_address(&self) -> Result<String> {
        let public_key = get_public_key(&self.private_key);
        Ok(format!("{:#x}", public_key))
    }

    /// Send a transaction to Starknet (non-gasless fallback)
    #[allow(dead_code)]
    pub async fn send_transaction(
        &self,
        _tx_data: &Value,
        _rpc_url: &str,
    ) -> Result<String> {
        info!("Sending Starknet transaction via RPC (non-gasless)");
        
        // TODO: Implement Starknet transaction broadcasting
        // This requires starknet-providers and proper transaction construction
        Err(anyhow::anyhow!(
            "Starknet non-gasless transaction broadcasting not yet implemented. \
            Use gasless mode or implement RPC broadcasting."
        ))
    }
}

/* 
Implementation Notes from gardenjs.md:

1. Starknet uses typed data signing similar to EIP-712
2. Domain structure:
   {
     name: 'HTLC',
     version: '2',
     chainId: '0x534e5f5345504f4c4941', // Starknet Sepolia
     revision: 'ACTIVE'
   }

3. Message structure for Initiate:
   {
     redeemer: ContractAddress,
     amount: u256,
     timelock: u128,
     secretHash: u128[],
     verifyingContract: ContractAddress
   }

4. Signature format from formatStarknetSignature():
   - Input: { r: FieldElement, s: FieldElement }
   - Output: "r,s" as comma-separated decimal strings
   - Example: "123456789,987654321"

5. Gasless endpoint: PATCH /v2/orders/{orderId}?action=initiate
   Body: { signature: "r,s" }

6. Dependencies:
   - starknet = "0.10"
   - starknet-crypto = "0.6"
   - sha2 = "0.10" (for hashing)
   - hex = "0.4"
*/
