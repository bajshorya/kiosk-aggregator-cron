use anyhow::{Context, Result};
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use serde_json::Value;
use std::str::FromStr;
use tracing::info;

pub struct EvmSigner {
    private_key: String,
}

impl EvmSigner {
    pub fn new(private_key: String) -> Self {
        Self { private_key }
    }
    pub async fn execute_initiate_tx(&self, tx_data: &Value, rpc_url: &str) -> Result<String> {
        let provider = Provider::<Http>::try_from(rpc_url).context("Failed to create provider")?;

        let chain_id: u64 = tx_data["chain_id"]
            .as_u64()
            .context("Missing chain_id in tx data")?;

        let wallet = self
            .private_key
            .parse::<LocalWallet>()
            .context("Failed to parse private key")?
            .with_chain_id(chain_id);

        let client = SignerMiddleware::new(provider, wallet);

        let to = tx_data["to"]
            .as_str()
            .context("Missing 'to' field")?
            .parse::<Address>()
            .context("Invalid 'to' address")?;

        let data_hex = tx_data["data"].as_str().context("Missing 'data' field")?;
        let data = ethers::utils::hex::decode(data_hex.trim_start_matches("0x"))
            .context("Failed to decode tx data hex")?;

        let gas_limit_str = tx_data["gas_limit"].as_str().context("Missing gas_limit")?;
        let gas_limit = U256::from_str(gas_limit_str).context("Failed to parse gas_limit")?;

        let value_str = tx_data["value"].as_str().unwrap_or("0x0");
        let value = U256::from_str(value_str).unwrap_or(U256::zero());

        // Use EIP-1559 tx with provider-estimated fees instead of hardcoded gas price
        let tx = Eip1559TransactionRequest::new()
            .to(to)
            .data(data)
            .gas(gas_limit)
            .value(value);

        info!(
            "Broadcasting EVM tx to chain_id={} via {}",
            chain_id, rpc_url
        );

        let pending_tx = client
            .send_transaction(tx, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send transaction: {}", e))?;

        let tx_hash = format!("{:?}", pending_tx.tx_hash());
        info!("EVM tx submitted: {}", tx_hash);

        Ok(tx_hash)
    }
}
