use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn};

use super::bitcoin_signer::BitcoinUTXO;

#[derive(Debug, Clone, Deserialize)]
struct BlockstreamUTXO {
    txid: String,
    vout: u32,
    value: u64,
    status: BlockstreamStatus,
}

#[derive(Debug, Clone, Deserialize)]
struct BlockstreamStatus {
    confirmed: bool,
}

pub struct BitcoinProvider {
    client: Client,
    base_url: String,
}

impl BitcoinProvider {
    pub fn new(base_url: String) -> Self {
        // Create client with 30 second timeout for Bitcoin APIs
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
            
        Self {
            client,
            base_url,
        }
    }

    /// Get UTXOs for an address
    pub async fn get_utxos(&self, address: &str) -> Result<Vec<BitcoinUTXO>> {
        let url = format!("{}/address/{}/utxo", self.base_url, address);
        info!("Fetching UTXOs from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to fetch UTXOs from {}", url))?;

        let status = response.status();
        info!("UTXO API response status: {}", status);

        if !status.is_success() {
            anyhow::bail!("Failed to fetch UTXOs: HTTP {}", status);
        }

        let utxos: Vec<BlockstreamUTXO> = response
            .json()
            .await
            .context("Failed to parse UTXO response")?;

        info!("Received {} UTXOs from API (before filtering)", utxos.len());

        // Convert to our UTXO format - include BOTH confirmed and unconfirmed
        let bitcoin_utxos: Vec<BitcoinUTXO> = utxos
            .into_iter()
            .map(|u| {
                info!(
                    "UTXO: txid={}, vout={}, value={} sats, confirmed={}",
                    u.txid, u.vout, u.value, u.status.confirmed
                );
                BitcoinUTXO {
                    txid: u.txid,
                    vout: u.vout,
                    value: u.value,
                    script_pubkey: String::new(), // Not needed for our use case
                }
            })
            .collect();

        info!("Found {} total UTXOs (confirmed + unconfirmed)", bitcoin_utxos.len());
        
        if bitcoin_utxos.is_empty() {
            warn!("No UTXOs found for address {}. This could mean:", address);
            warn!("  1. The address has no funds");
            warn!("  2. All UTXOs are unconfirmed (wait for confirmations)");
            warn!("  3. The API is experiencing issues");
            warn!("  4. The address format doesn't match the private key");
        }
        
        Ok(bitcoin_utxos)
    }

    /// Get balance for an address
    #[allow(dead_code)]
    pub async fn get_balance(&self, address: &str) -> Result<u64> {
        let url = format!("{}/address/{}", self.base_url, address);
        
        #[derive(Deserialize)]
        struct AddressInfo {
            chain_stats: ChainStats,
        }
        
        #[derive(Deserialize)]
        struct ChainStats {
            funded_txo_sum: u64,
            spent_txo_sum: u64,
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch address info")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch balance: HTTP {}", response.status());
        }

        let info: AddressInfo = response
            .json()
            .await
            .context("Failed to parse address info")?;

        let balance = info.chain_stats.funded_txo_sum - info.chain_stats.spent_txo_sum;
        Ok(balance)
    }

    /// Broadcast a transaction
    pub async fn broadcast(&self, tx_hex: &str) -> Result<String> {
        let url = format!("{}/tx", self.base_url);
        info!("Broadcasting transaction to: {}", url);

        let response = self
            .client
            .post(&url)
            .body(tx_hex.to_string())
            .send()
            .await
            .context("Failed to broadcast transaction")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to broadcast transaction: {}", error_text);
        }

        let txid = response.text().await.context("Failed to get txid")?;
        info!("Transaction broadcasted: {}", txid);
        Ok(txid)
    }

    /// Estimate fee (returns fee in sats/vbyte)
    pub async fn estimate_fee(&self) -> Result<u64> {
        let url = format!("{}/fee-estimates", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch fee estimates")?;

        if !response.status().is_success() {
            warn!("Failed to fetch fee estimates, using default 2 sats/vbyte for testnet");
            return Ok(2);
        }

        let fees: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse fee estimates")?;

        // Get the fee for next 6 blocks (medium priority)
        let fee = fees
            .get("6")
            .and_then(|v| v.as_f64())
            .map(|f| f.ceil() as u64)
            .unwrap_or(2);

        // Cap fee at 20 sats/vbyte for testnet (mainnet fees don't apply)
        let capped_fee = fee.min(20);
        
        if fee > 20 {
            warn!("API returned high fee rate {} sats/vbyte, capping at 20 sats/vbyte for testnet", fee);
        }

        info!("Estimated fee: {} sats/vbyte", capped_fee);
        Ok(capped_fee)
    }

    /// Calculate transaction fee based on inputs and outputs
    /// Typical P2WPKH transaction: ~140 vbytes for 1 input, 2 outputs
    pub fn calculate_fee(&self, num_inputs: usize, num_outputs: usize, fee_rate: u64) -> u64 {
        // P2WPKH transaction size estimation
        // Base: 10.5 vbytes
        // Input: 68 vbytes each
        // Output: 31 vbytes each
        let vbytes = 10.5 + (num_inputs as f64 * 68.0) + (num_outputs as f64 * 31.0);
        (vbytes.ceil() as u64) * fee_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fee_calculation() {
        let provider = BitcoinProvider::new("https://blockstream.info/testnet/api".to_string());
        
        // 1 input, 2 outputs (typical transaction)
        let fee = provider.calculate_fee(1, 2, 10);
        assert!(fee > 0);
        assert!(fee < 2000); // Should be reasonable
    }
}
