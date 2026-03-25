use anyhow::{Context, Result};
use serde_json::Value;
use starknet::core::crypto::{ecdsa_sign, pedersen_hash};
use starknet::core::types::FieldElement;
use tracing::info;

/// Starknet signer for gasless transactions
/// Uses ECDSA signing with Pedersen hash for typed data
#[allow(dead_code)]
pub struct StarknetSigner {
    private_key: FieldElement,
}

#[allow(dead_code)]
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
    /// Returns signature as array of hex strings [r, s]
    pub async fn sign_typed_data(&self, typed_data: &Value) -> Result<Vec<String>> {
        info!("Signing Starknet typed data for gasless initiation");
        
        // Build message hash for Starknet
        let message_hash = self.compute_message_hash(typed_data)?;
        
        info!("Message hash: {:?}", message_hash);
        
        // Sign the message hash
        let signature = ecdsa_sign(&self.private_key, &message_hash)
            .context("Failed to sign Starknet typed data")?;
        
        // Format signature as array of strings (decimal format, not hex)
        // According to formatStarknetSignature in gardenjs.md
        let sig_array = vec![
            signature.r.to_string(),
            signature.s.to_string(),
        ];
        
        info!("Starknet signature generated: {:?}", sig_array);
        
        Ok(sig_array)
    }

    /// Compute message hash for Starknet typed data
    fn compute_message_hash(&self, typed_data: &Value) -> Result<FieldElement> {
        // For Starknet, we need to hash the typed data according to EIP-712-like structure
        // This is a simplified version - in production, use proper Starknet typed data hashing
        
        let message = typed_data.get("message")
            .ok_or_else(|| anyhow::anyhow!("Missing message"))?;
        
        // Extract fields from message
        let redeemer = message.get("redeemer")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing redeemer"))?;
        let amount = message.get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing amount"))?;
        let timelock = message.get("timelock")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing timelock"))?;
        
        // Parse as FieldElements
        let redeemer_fe = FieldElement::from_hex_be(redeemer.trim_start_matches("0x"))?;
        let amount_fe = FieldElement::from_hex_be(amount.trim_start_matches("0x"))?;
        let timelock_fe = FieldElement::from_hex_be(timelock.trim_start_matches("0x"))?;
        
        // Compute Pedersen hash
        let hash1 = pedersen_hash(&redeemer_fe, &amount_fe);
        let hash2 = pedersen_hash(&hash1, &timelock_fe);
        
        Ok(hash2)
    }

    /// Send a transaction to Starknet (non-gasless fallback)
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

4. Signature format: Array of field elements [r, s]
   - Must be formatted using formatStarknetSignature()
   - Converts signature object to array format

5. Gasless endpoint: PATCH /v2/orders/{orderId}?action=initiate
   Body: { signature: ["0x...", "0x..."] }

6. Dependencies needed:
   - starknet-rs for signing
   - starknet-crypto for cryptographic operations
   - Add to Cargo.toml:
     starknet = "0.9"
     starknet-crypto = "0.6"
*/
