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
        
        // Check balance first
        let balance = client.get_balance(self.wallet.address(), None).await
            .context("Failed to get wallet balance")?;
        info!("Wallet balance: {} wei ({} ETH)", balance, ethers::utils::format_units(balance, "ether").unwrap_or_default());
        
        // Send transaction
        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to send transaction: {}. This usually means insufficient gas (need testnet ETH) or network issues.", e)
            })?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());
        info!("Transaction sent: {}", tx_hash);

        Ok(tx_hash)
    }
}
