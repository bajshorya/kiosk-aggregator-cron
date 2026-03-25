use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
};
use solana_sdk::{
    signature::Keypair,
    transaction::VersionedTransaction,
};
use tracing::info;

pub struct SolanaSigner {
    keypair: Keypair,
}

impl SolanaSigner {
    pub fn new(private_key: &str) -> Result<Self> {
        let keypair = Self::parse_private_key(private_key)?;
        Ok(Self { keypair })
    }

    fn parse_private_key(key: &str) -> Result<Keypair> {
        // Try base58 format first (most common)
        if let Ok(bytes) = bs58::decode(key).into_vec() {
            if bytes.len() == 64 {
                return Keypair::try_from(&bytes[..])
                    .context("Invalid Solana private key bytes");
            }
        }

        // Try JSON array format [1,2,3,...]
        if key.starts_with('[') {
            if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(key) {
                if bytes.len() == 64 {
                    return Keypair::try_from(&bytes[..])
                        .context("Invalid Solana private key from JSON array");
                }
            }
        }

        anyhow::bail!("Invalid Solana private key format. Expected base58 string or JSON array of 64 bytes")
    }

    /// Sign a Solana transaction and return the signed transaction as base64
    /// (for use with Garden's gasless PATCH endpoint)
    pub fn sign_transaction(&self, versioned_tx_base64: &str) -> Result<String> {
        // Decode the base64 transaction from Garden API
        let tx_bytes = STANDARD
            .decode(versioned_tx_base64)
            .context("Failed to decode base64 transaction")?;

        // Deserialize into VersionedTransaction
        let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Failed to deserialize Solana transaction")?;

        info!("Solana transaction decoded, signing with our keypair...");

        // Sign the transaction with our keypair
        let message = tx.message.clone();
        let signed_tx = VersionedTransaction::try_new(message, &[&self.keypair])
            .context("Failed to sign Solana transaction")?;

        info!(
            "Transaction signed, signature: {}",
            signed_tx.signatures[0]
        );

        // Serialize the signed transaction back to base64
        let signed_bytes = bincode::serialize(&signed_tx)
            .context("Failed to serialize signed transaction")?;
        let signed_base64 = STANDARD.encode(&signed_bytes);

        Ok(signed_base64)
    }

    /// Legacy method: Sign and send directly (kept for reference, but gasless is preferred)
    pub async fn sign_and_send(
        &self,
        versioned_tx_base64: &str,
        rpc_url: &str,
    ) -> Result<String> {
        // Decode the base64 transaction from Garden API
        let tx_bytes = STANDARD
            .decode(versioned_tx_base64)
            .context("Failed to decode base64 transaction")?;

        // Deserialize into VersionedTransaction
        let tx: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .context("Failed to deserialize Solana transaction")?;

        info!("Solana transaction decoded, signing with our keypair...");

        // Sign the transaction with our keypair
        // The message is already correct from Garden API, we just add our signature
        let message = tx.message.clone();
        let signed_tx = VersionedTransaction::try_new(message, &[&self.keypair])
            .context("Failed to sign Solana transaction")?;

        info!(
            "Transaction signed, signature: {}",
            signed_tx.signatures[0]
        );

        // Send immediately to avoid blockhash expiration
        let rpc_url = rpc_url.to_string();
        let signature = tokio::task::spawn_blocking(move || -> Result<_, anyhow::Error> {
            let client = RpcClient::new(rpc_url);
            
            // Send with skip_preflight to bypass simulation
            let config = RpcSendTransactionConfig {
                skip_preflight: true,
                preflight_commitment: None,
                encoding: None,
                max_retries: Some(3),
                min_context_slot: None,
            };
            
            client.send_transaction_with_config(&signed_tx, config)
                .map_err(|e| anyhow::anyhow!("RPC error: {}", e))
        })
        .await
        .context("Tokio task failed")??;

        Ok(signature.to_string())
    }
}
