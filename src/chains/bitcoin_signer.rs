use anyhow::{Context, Result};
use bitcoin::{
    Address, Network, PrivateKey, PublicKey, Transaction, TxIn, TxOut, OutPoint,
    Sequence, Witness, ScriptBuf, Amount, absolute::LockTime,
};
use bitcoin::secp256k1::{Secp256k1, Message};
use bitcoin::sighash::{SighashCache, EcdsaSighashType};
use bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: u32,
    pub value: u64, // in satoshis
    pub script_pubkey: String,
    #[serde(default)]
    pub confirmed: bool, // Track if UTXO is confirmed or in mempool
}

pub struct BitcoinSigner {
    private_key: PrivateKey,
    network: Network,
}

impl BitcoinSigner {
    /// Create a new Bitcoin signer from a WIF private key
    pub fn new(wif: String, network: Network) -> Result<Self> {
        let private_key = PrivateKey::from_wif(&wif)
            .context("Failed to parse Bitcoin WIF private key")?;
        
        Ok(Self {
            private_key,
            network,
        })
    }

    /// Get the Bitcoin address for this wallet
    pub fn get_address(&self) -> Result<String> {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_private_key(&secp, &self.private_key);
        
        // Generate P2WPKH (native segwit) address
        let address = Address::p2wpkh(&public_key, self.network)
            .context("Failed to generate P2WPKH address")?;
        
        Ok(address.to_string())
    }

    /// Get the public key
    #[allow(dead_code)]
    pub fn get_public_key(&self) -> String {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_private_key(&secp, &self.private_key);
        public_key.to_string()
    }

    /// Send Bitcoin to an address
    /// 
    /// This function:
    /// 1. Fetches UTXOs from the provider
    /// 2. Builds a transaction with inputs and outputs
    /// 3. Signs the transaction
    /// 4. Returns the signed transaction hex for broadcasting
    pub async fn send(
        &self,
        to_address: &str,
        amount_sats: u64,
        utxos: Vec<BitcoinUTXO>,
        fee_sats: u64,
    ) -> Result<String> {
        info!("Building Bitcoin transaction: {} sats to {}", amount_sats, to_address);
        
        // Parse recipient address
        let recipient_addr = to_address.parse::<Address<_>>()
            .context("Invalid recipient address")?
            .require_network(self.network)
            .context("Address network mismatch")?;

        // Calculate total input value
        let total_input: u64 = utxos.iter().map(|u| u.value).sum();
        
        if total_input < amount_sats + fee_sats {
            anyhow::bail!(
                "Insufficient funds: have {} sats, need {} sats (amount: {}, fee: {})",
                total_input,
                amount_sats + fee_sats,
                amount_sats,
                fee_sats
            );
        }

        // Build transaction
        let mut tx = Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: LockTime::ZERO,
            input: Vec::new(),
            output: Vec::new(),
        };

        // Add inputs from UTXOs
        for utxo in &utxos {
            // Decode hex and reverse bytes (Bitcoin uses little-endian)
            let mut txid_bytes = hex::decode(&utxo.txid)
                .context(format!("Invalid UTXO txid hex: {}", utxo.txid))?;
            
            if txid_bytes.len() != 32 {
                anyhow::bail!("Invalid txid length: expected 32 bytes, got {}", txid_bytes.len());
            }
            
            // Reverse bytes for Bitcoin's little-endian format
            txid_bytes.reverse();
            
            let txid = bitcoin::Txid::from_slice(&txid_bytes)
                .context(format!("Invalid UTXO txid: {}", utxo.txid))?;

            info!("Adding input: txid={}, vout={}, value={}", utxo.txid, utxo.vout, utxo.value);

            tx.input.push(TxIn {
                previous_output: OutPoint {
                    txid,
                    vout: utxo.vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            });
        }

        // Add output to recipient
        tx.output.push(TxOut {
            value: Amount::from_sat(amount_sats),
            script_pubkey: recipient_addr.script_pubkey(),
        });

        // Add change output if necessary
        let change = total_input - amount_sats - fee_sats;
        if change > 546 { // Dust limit
            let change_addr = self.get_address()?
                .parse::<Address<_>>()
                .context("Invalid change address")?
                .require_network(self.network)
                .context("Change address network mismatch")?;

            tx.output.push(TxOut {
                value: Amount::from_sat(change),
                script_pubkey: change_addr.script_pubkey(),
            });
        }

        // Sign all inputs
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_private_key(&secp, &self.private_key);
        
        // Compute all signatures first
        let mut signatures = Vec::new();
        
        for (index, utxo) in utxos.iter().enumerate() {
            info!("Computing signature for input {}/{}: txid={}, vout={}", index + 1, utxos.len(), utxo.txid, utxo.vout);
            
            // For P2WPKH, we need the script_pubkey of the output being spent
            // This is a P2PKH script with the pubkey hash
            let script_code = Address::p2wpkh(&public_key, self.network)
                .context("Failed to create P2WPKH address")?
                .script_pubkey();
            
            // Create a sighash cache for this transaction
            let mut sighash_cache = SighashCache::new(&tx);
            
            // Compute sighash for P2WPKH input
            let sighash = sighash_cache
                .p2wpkh_signature_hash(
                    index,
                    &script_code,
                    Amount::from_sat(utxo.value),
                    EcdsaSighashType::All,
                )
                .context(format!("Failed to compute sighash for input {} (txid: {}, vout: {}, value: {})", 
                    index, utxo.txid, utxo.vout, utxo.value))?;

            let message = Message::from_digest(sighash.to_byte_array());
            let signature = secp.sign_ecdsa(&message, &self.private_key.inner);
            
            signatures.push(signature);
            info!("Signature computed for input {}", index + 1);
        }
        
        // Now add witnesses to transaction
        for (index, signature) in signatures.iter().enumerate() {
            let mut witness = Witness::new();
            let sig = bitcoin::ecdsa::Signature {
                sig: *signature,
                hash_ty: EcdsaSighashType::All,
            };
            witness.push(sig.to_vec());
            witness.push(public_key.to_bytes());

            tx.input[index].witness = witness;
            info!("Input {} signed successfully", index + 1);
        }

        // Serialize transaction to hex
        let tx_hex = bitcoin::consensus::encode::serialize_hex(&tx);
        info!("Bitcoin transaction built: {} bytes", tx_hex.len() / 2);
        
        Ok(tx_hex)
    }

    /// Sign a message (for verification purposes)
    #[allow(dead_code)]
    pub fn sign_message(&self, message: &[u8]) -> Result<String> {
        let secp = Secp256k1::new();
        let msg = Message::from_digest_slice(message)
            .context("Invalid message for signing")?;
        
        let signature = secp.sign_ecdsa(&msg, &self.private_key.inner);
        Ok(hex::encode(signature.serialize_compact()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_generation() {
        // Test with a known private key (testnet)
        let wif = "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy";
        let signer = BitcoinSigner::new(wif.to_string(), Network::Testnet).unwrap();
        
        let address = signer.get_address().unwrap();
        assert!(address.starts_with("tb1")); // Testnet bech32 address
    }

    #[test]
    fn test_public_key() {
        let wif = "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy";
        let signer = BitcoinSigner::new(wif.to_string(), Network::Testnet).unwrap();
        
        let pubkey = signer.get_public_key();
        assert_eq!(pubkey.len(), 66); // Compressed public key hex
    }
}
