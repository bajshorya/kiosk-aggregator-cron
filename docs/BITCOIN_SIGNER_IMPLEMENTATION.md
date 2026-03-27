# Bitcoin Signer Implementation

## Overview

Implemented a Bitcoin transaction signer based on Garden.js Bitcoin wallet patterns. The signer supports P2WPKH (native SegWit) transactions for Bitcoin testnet and mainnet.

## Implementation Details

### Module: `src/chains/bitcoin_signer.rs`

Based on Garden.js patterns from:
- `packages/core/src/lib/bitcoin/wallet/wallet.ts`
- `packages/core/src/lib/bitcoin/wallet/abstractWallet.ts`

### Key Features

1. **WIF Private Key Support**
   - Initialize from WIF (Wallet Import Format) private key
   - Supports both testnet and mainnet networks

2. **P2WPKH Address Generation**
   - Native SegWit (bech32) addresses
   - Testnet: `tb1...`
   - Mainnet: `bc1...`

3. **Transaction Building**
   - UTXO-based transaction construction
   - Automatic change output calculation
   - Dust limit handling (546 sats minimum)
   - RBF (Replace-By-Fee) enabled

4. **Transaction Signing**
   - SegWit v0 signature generation
   - Proper witness construction
   - ECDSA signatures with SIGHASH_ALL

5. **Fee Management**
   - Custom fee specification
   - Insufficient funds detection
   - Change output optimization

## Usage Example

```rust
use bitcoin::Network;
use crate::chains::bitcoin_signer::{BitcoinSigner, BitcoinUTXO};

// Initialize signer
let wif = "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy";
let signer = BitcoinSigner::new(wif.to_string(), Network::Testnet)?;

// Get address
let address = signer.get_address()?;
println!("Bitcoin address: {}", address);

// Build and sign transaction
let utxos = vec![
    BitcoinUTXO {
        txid: "abc123...".to_string(),
        vout: 0,
        value: 100000, // 100,000 sats
        script_pubkey: "...".to_string(),
    }
];

let tx_hex = signer.send(
    "tb1qrecipient...",
    50000,  // 50,000 sats
    utxos,
    1000,   // 1,000 sats fee
).await?;

// Broadcast tx_hex to Bitcoin network
```

## Garden.js Patterns Implemented

### 1. Transaction Structure (from wallet.ts)
```typescript
// Garden.js pattern
const psbt = new Psbt({ network });
psbt.addInput({ hash, index, nonWitnessUtxo });
psbt.addOutput({ address, value });
psbt.signAllInputs(signer).finalizeAllInputs();
```

**Rust Implementation:**
```rust
let mut tx = Transaction {
    version: Version::TWO,
    lock_time: LockTime::ZERO,
    input: Vec::new(),
    output: Vec::new(),
};
tx.input.push(TxIn { ... });
tx.output.push(TxOut { ... });
// Sign each input with witness
```

### 2. P2WPKH Address Generation (from abstractWallet.ts)
```typescript
// Garden.js pattern
const { address } = payments.p2wpkh({
    pubkey: this.signer.publicKey,
    network: this.network,
});
```

**Rust Implementation:**
```rust
let address = Address::p2wpkh(&public_key, self.network)?;
```

### 3. SegWit Signing (from wallet.ts spend method)
```typescript
// Garden.js pattern
const signatureHash = tx.hashForWitnessV0(
    index, script, utxos[index].value, hashType
);
const signature = signer.sign(signatureHash);
witness.push(signature);
witness.push(publicKey);
```

**Rust Implementation:**
```rust
let sighash = sighash_cache.p2wpkh_signature_hash(
    index, &script_code, Amount::from_sat(utxo.value), EcdsaSighashType::All
)?;
let signature = secp.sign_ecdsa(&message, &private_key.inner);
witness.push(sig.to_vec());
witness.push(public_key.to_bytes());
```

### 4. Change Output Handling (from wallet.ts _send method)
```typescript
// Garden.js pattern
const change = totalUTXOValue - amt - fee;
if (change > 0) {
    psbt.addOutput({ address: fromAddress, value: change });
}
```

**Rust Implementation:**
```rust
let change = total_input - amount_sats - fee_sats;
if change > 546 { // Dust limit
    tx.output.push(TxOut {
        value: Amount::from_sat(change),
        script_pubkey: change_addr.script_pubkey(),
    });
}
```

## Dependencies

Added to `Cargo.toml`:
```toml
bitcoin = { version = "0.31", features = ["serde", "rand"] }
```

## Testing

Unit tests included:
- Address generation verification
- Public key format validation
- Network compatibility checks

Run tests:
```bash
cargo test bitcoin_signer
```

## Integration with Garden API

The signer can be integrated with Garden API's Bitcoin swap flow:

1. **Get deposit address** from Garden API
2. **Fetch UTXOs** from Bitcoin provider
3. **Build transaction** using `send()` method
4. **Broadcast** signed transaction hex to Bitcoin network
5. **Submit transaction hash** to Garden API

## Limitations & Future Work

### Current Implementation
- ✅ P2WPKH (native SegWit) support
- ✅ Transaction building and signing
- ✅ Change output handling
- ✅ Fee management

### Not Yet Implemented
- ❌ P2SH (legacy) addresses
- ❌ P2TR (Taproot) addresses
- ❌ Multi-signature wallets
- ❌ HTLC (Hash Time Locked Contract) support
- ❌ RBF transaction replacement
- ❌ CPFP (Child Pays For Parent)

### For HTLC Support (Required for Atomic Swaps)
Reference Garden.js implementation:
- `packages/core/src/lib/bitcoin/htlcScript.ts` - HTLC script generation
- `packages/core/src/lib/bitcoin/wallet/abstractWallet.ts` - BitcoinHTLC class
- Requires P2WSH script support
- Requires witness script construction for redeem/refund

## Security Considerations

1. **Private Key Storage**: WIF keys should be stored securely (env vars, key management systems)
2. **Network Validation**: Always verify address network matches signer network
3. **Fee Validation**: Check fee doesn't exceed amount
4. **Dust Limits**: Enforce 546 sat minimum for change outputs
5. **UTXO Selection**: Ensure sufficient funds before building transaction

## References

- Garden.js Bitcoin Wallet: `packages/core/src/lib/bitcoin/wallet/`
- Bitcoin BIP 84 (P2WPKH): https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki
- Rust Bitcoin Library: https://docs.rs/bitcoin/latest/bitcoin/
- SegWit Transactions: https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki

## Conclusion

The Bitcoin signer successfully implements core Garden.js patterns for P2WPKH transactions. It provides a foundation for Bitcoin swap integration and can be extended to support HTLC contracts for atomic swaps.
