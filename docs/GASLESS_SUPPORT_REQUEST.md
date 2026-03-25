# Garden Finance Gasless Support Request

## Issue Summary
Gasless initiation has been enabled for app ID `79702d04cb63391922f2e1471afe4743b0e3ba71f260e3c2117aa36e7fb74a9c`, but the API is not returning `versioned_tx_gasless` for Solana swaps, and gasless initiation attempts are failing.

## Environment
- **Environment**: Testnet
- **App ID**: `79702d04cb63391922f2e1471afe4743b0e3ba71f260e3c2117aa36e7fb74a9c`
- **Chains**: EVM (Ethereum, Base, Arbitrum) and Solana

## Current Behavior

### Solana
**API Response from Create Order**:
```json
{
  "versioned_tx": "base64...",           // ✅ Present
  "versioned_tx_gasless": false,         // ❌ Not provided
  "typed_data": false,
  "initiate_transaction": false
}
```

**Gasless Initiation Attempt**:
```
PATCH /v2/orders/{order_id}?action=initiate
Headers: {"garden-app-id": "..."}
Body: {
  "order_id": "8fcfdac8216e0397a552be037eb34f49fcf27158e40b127e6e2fad04f21be1df",
  "serialized_tx": "AQbuncolxA0I9ZNFgIN26lvOkddZ7djlYSPMzslaOzgwYxF5Jskl2+m24BtcD92GZO5T9Qxw2BrtdXtEiRwSiwcBAAIESfnT2pGkaS63TOXnrjECw6vYv8h9rK09W7PAtBXqmKJ8stjAwSzNk7l+zIxeZ6vleCBNgEtpIi1rYZupOyEYHgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAF7eCjKbE8GtRE5pgJU+B/hJJyOKxIv6/MsqBMiufHo5L7Uaxpp4LMITf1ftlnJgIVFM1y3AkC1eSBQ6mvSlgcAEDAwEAAlgFP3txmUuUDgDh9QUAAAAAwEsDAAAAAAA64o7QHg9PzXO+FaTYOSeFGUCFN1VInZgwPm6mAb8bjSnc11x0Xm7J4bgWM4Hy/hhDrZoh+WvGblQLPj7s680P"
}

Response: 400 Bad Request
{"status":"Error","error":"Missing signature"}
```

### EVM
**API Response from Create Order**:
```json
{
  "typed_data": {...},                   // ✅ Present
  "initiate_transaction": {...},
  "versioned_tx": false,
  "versioned_tx_gasless": false
}
```

**Gasless Initiation Attempt**:
```
PATCH /v2/orders/{order_id}?action=initiate
Headers: {"garden-app-id": "..."}
Body: {
  "signature": "0x9ef82829940559a5bc4dbec06a3e571d7aebf9ce4d8d7e5aa8d1f6941ee24988091df6dfca077b30377e8caab2694e9af10de93df0590650e78a7278d094d6861c"
}

Response: 400 Bad Request
{"status":"Error","error":"Invalid swap request"}
```

## Expected Behavior

According to the [documentation](https://docs.garden.finance/api-reference/endpoint/orders/patch):

### Solana
- Create order should return `versioned_tx_gasless`
- Gasless initiation should accept `{"order_id": "...", "serialized_tx": "..."}`
- Should return `200 OK` with successful initiation

### EVM
- Create order returns `typed_data` ✅
- Gasless initiation should accept `{"signature": "0x..."}`
- Should return `200 OK` with successful initiation

## Implementation Details

Our implementation follows the documentation exactly:

**EVM (EIP-712 Signing)**:
```rust
// Sign typed_data with ethers-rs
let signature = wallet.sign_typed_data(&typed_data).await?;
// Submit: {"signature": "0x..."}
```

**Solana (Transaction Signing)**:
```rust
// Deserialize versioned_tx, sign with keypair, serialize back to base64
let signed_tx = sign_transaction(versioned_tx)?;
// Submit: {"order_id": "...", "serialized_tx": "base64..."}
```

## Questions

1. **Is gasless fully enabled for our app ID on testnet?**
   - We enabled it but API responses suggest otherwise

2. **Why is `versioned_tx_gasless` not being returned for Solana?**
   - Only `versioned_tx` is provided
   - Documentation says `versioned_tx_gasless` should be present

3. **Are there additional requirements or configuration needed?**
   - Special headers?
   - Different endpoint?
   - Whitelist specific wallet addresses?

4. **Is gasless available on testnet for Solana?**
   - Maybe it's mainnet-only?

## Test Orders

Recent test orders that failed gasless initiation:

**Solana → Ethereum**:
- Order ID: `8fcfdac8216e0397a552be037eb34f49fcf27158e40b127e6e2fad04f21be1df`
- Status: Created but not initiated
- Error: "Missing signature"

**Ethereum → Base**:
- Order ID: `e3a3626752bbbbfb3d06210fa7351b3d07be9566c3d04f260a4d9a961f76e340`
- Status: Created but not initiated  
- Error: "Invalid swap request"

## Request

Please help us:
1. Verify gasless is properly enabled for our app ID
2. Confirm if Solana gasless is available on testnet
3. Provide guidance on why we're getting these errors
4. Share any additional configuration or requirements needed

## Contact

- Discord: [Your Discord Handle]
- Email: [Your Email]
- GitHub: [Your GitHub]

Thank you for your help!
