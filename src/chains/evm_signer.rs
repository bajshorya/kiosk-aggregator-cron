use anyhow::{Context, Result};
use ethers::prelude::*;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::transaction::eip712::TypedData;
use serde_json::Value;
use tracing::info;

pub struct EvmSigner {
    wallet: LocalWallet,
}

impl EvmSigner {
    pub fn new(private_key: String) -> Result<Self> {
        let wallet = private_key
            .parse::<LocalWallet>()
            .context("Failed to parse EVM private key")?;
        Ok(Self { wallet })
    }

    /// Sign EIP-712 typed_data and return hex signature for gasless initiation
    pub async fn sign_typed_data(&self, typed_data: &Value) -> Result<String> {
        info!("Signing EIP-712 typed_data for gasless initiation");
        
        // Log the typed_data structure for debugging
        info!("Typed data to sign: {}", serde_json::to_string_pretty(typed_data).unwrap_or_default());
        
        // Parse the typed_data JSON into ethers TypedData struct
        let typed: TypedData = serde_json::from_value(typed_data.clone())
            .context("Failed to parse typed_data as EIP-712")?;

        // Sign the typed data
        let signature = self
            .wallet
            .sign_typed_data(&typed)
            .await
            .context("Failed to sign EIP-712 typed_data")?;

        // Format as hex string with 0x prefix
        let sig_hex = format!("0x{}", signature);
        info!("EIP-712 signature generated: {}", sig_hex);
        
        Ok(sig_hex)
    }

    /// Send a raw transaction to the network (non-gasless fallback)
    pub async fn send_transaction(
        &self,
        tx_data: &Value,
        rpc_url: &str,
    ) -> Result<String> {
        info!("Sending EVM transaction via RPC (non-gasless)");
        
        // Parse transaction data
        let to: Address = tx_data["to"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'to' field"))?
            .parse()
            .context("Invalid 'to' address")?;
        
        let data: Bytes = tx_data["data"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'data' field"))?
            .parse()
            .context("Invalid 'data' hex")?;
        
        let value: U256 = tx_data["value"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'value' field"))?
            .parse()
            .context("Invalid 'value' hex")?;
        
        let gas_limit: U256 = tx_data["gas_limit"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'gas_limit' field"))?
            .parse()
            .context("Invalid 'gas_limit' hex")?;
        
        let chain_id: u64 = tx_data["chain_id"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'chain_id'"))?;

        // Connect to RPC
        let provider = Provider::<Http>::try_from(rpc_url)
            .context("Failed to connect to RPC")?;
        let client = SignerMiddleware::new(provider, self.wallet.clone().with_chain_id(chain_id));

        // Build transaction
        let tx = TransactionRequest::new()
            .to(to)
            .data(data)
            .value(value)
            .gas(gas_limit);

        info!("Broadcasting transaction to chain_id={}", chain_id);
        
        // Check balance first (with timeout to avoid blocking)
        match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.get_balance(self.wallet.address(), None)
        ).await {
            Ok(Ok(balance)) => {
                info!("Wallet balance: {} wei ({} ETH)", balance, ethers::utils::format_units(balance, "ether").unwrap_or_default());
            }
            Ok(Err(e)) => {
                info!("Could not check balance ({}), proceeding with transaction anyway", e);
            }
            Err(_) => {
                info!("Balance check timed out after 10s, proceeding with transaction anyway");
            }
        }
        
        // Send transaction
        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to send transaction: {}. This usually means insufficient gas (need testnet ETH) or network issues.", e)
            })?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());
        info!("Transaction sent: {}", tx_hash);
        
        // Verify transaction is in mempool
        info!("Verifying transaction is in mempool...");
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get_transaction(pending_tx.tx_hash())
        ).await {
            Ok(Ok(Some(tx))) => {
                info!("✅ Transaction found in mempool");
                info!("From: {:?}, To: {:?}, Value: {}", tx.from, tx.to, tx.value);
                info!("Nonce: {}, Gas: {}", tx.nonce, tx.gas);
            }
            Ok(Ok(None)) => {
                info!("⚠️  WARNING: Transaction NOT found in mempool! May have been dropped.");
            }
            Ok(Err(e)) => {
                info!("⚠️  Error checking mempool: {}", e);
            }
            Err(_) => {
                info!("⏱️  Mempool check timed out");
            }
        }
        
        // Wait for transaction to be mined (with timeout)
        info!("Waiting for transaction to be mined (30s timeout)...");
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            pending_tx
        ).await {
            Ok(Ok(Some(receipt))) => {
                info!("✅ Transaction mined in block {}", receipt.block_number.unwrap_or_default());
                info!("Gas used: {}", receipt.gas_used.unwrap_or_default());
                info!("Status: {}", if receipt.status == Some(1.into()) { "SUCCESS" } else { "FAILED" });
            }
            Ok(Ok(None)) => {
                info!("⚠️  Transaction sent but receipt not available yet");
            }
            Ok(Err(e)) => {
                info!("⚠️  Error waiting for receipt: {}", e);
            }
            Err(_) => {
                info!("⏱️  Transaction confirmation timed out after 30s (may still be pending)");
            }
        }

        Ok(tx_hash)
    }
}
