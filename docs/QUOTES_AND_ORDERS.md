# Understanding Quotes and Orders in Garden Finance

## Overview

In Garden Finance's atomic swap system, **Quotes** and **Orders** are two distinct steps in the swap process. Think of it like booking a flight:

- **Quote** = Checking flight prices and availability
- **Order** = Actually booking the flight

---

## What is a Quote?

A **Quote** is a price inquiry that tells you:
- How much you'll receive for your swap
- What the exchange rate is
- How long the swap will take
- What fees you'll pay

### Quote Request

**API Endpoint:**
```
GET /v2/quote?from={source_asset}&to={dest_asset}&from_amount={amount}
```

**Example Request:**
```
GET /v2/quote?from=bitcoin_testnet:btc&to=solana_testnet:usdc&from_amount=50000
```

This asks: "If I swap 50,000 satoshis of Bitcoin, how much Solana USDC will I get?"

### Quote Response

**Data Structure:**
```rust
pub struct Quote {
    pub source: QuoteAsset,        // What you're sending
    pub destination: QuoteAsset,   // What you'll receive
    pub solver_id: String,         // Which solver will execute
    pub estimated_time: u64,       // Estimated completion time (seconds)
    pub slippage: u64,            // Price slippage tolerance
    pub fee: u64,                 // Fee percentage
    pub fixed_fee: String,        // Fixed fee amount
}

pub struct QuoteAsset {
    pub asset: String,    // e.g., "bitcoin_testnet:btc"
    pub amount: String,   // e.g., "50000" (in smallest unit)
    pub display: String,  // Human-readable: "0.0005 BTC"
    pub value: String,    // USD value: "$25.00"
}
```

**Example Response:**
```json
{
  "status": "Ok",
  "result": [
    {
      "source": {
        "asset": "bitcoin_testnet:btc",
        "amount": "50000",
        "display": "0.0005 BTC",
        "value": "$25.00"
      },
      "destination": {
        "asset": "solana_testnet:usdc",
        "amount": "3378005",
        "display": "3.378 USDC",
        "value": "$3.38"
      },
      "solver_id": "garden-solver-1",
      "estimated_time": 600,
      "slippage": 50,
      "fee": 30,
      "fixed_fee": "0"
    }
  ]
}
```

### What the Quote Tells You

From the example above:
- **You send**: 50,000 satoshis (0.0005 BTC)
- **You receive**: 3,378,005 USDC (3.378 USDC)
- **Estimated time**: 600 seconds (10 minutes)
- **Fee**: 30 basis points (0.3%)
- **Solver**: garden-solver-1 will execute the swap

### Important Notes About Quotes

1. **Non-Binding**: A quote is just an estimate, not a guarantee
2. **Expires Quickly**: Quotes are only valid for a short time (usually 30-60 seconds)
3. **No Commitment**: Getting a quote doesn't lock in the price or reserve funds
4. **Multiple Quotes**: API may return multiple quotes from different solvers
5. **Rate Limiting**: Too many quote requests = 429 errors (as you saw!)

---

## What is an Order?

An **Order** is the actual swap execution. When you submit an order, you're committing to the swap and Garden creates the atomic swap contract.

### Order Request

**API Endpoint:**
```
POST /v2/orders
```

**Data Structure:**
```rust
pub struct SubmitOrderRequest {
    pub source: OrderAsset,
    pub destination: OrderAsset,
}

pub struct OrderAsset {
    pub asset: String,   // e.g., "bitcoin_testnet:btc"
    pub owner: String,   // Your wallet address
    pub amount: String,  // Amount from the quote
}
```

**Example Request:**
```json
{
  "source": {
    "asset": "bitcoin_testnet:btc",
    "owner": "tb1qypyl87pagnmcted8d8gnmxxps7dyqstllhhe9z",
    "amount": "50000"
  },
  "destination": {
    "asset": "solana_testnet:usdc",
    "owner": "GdnVDMCHJGNidTU7sL8RkRsHHpvDJu1PDfzEPoLvQDU",
    "amount": "3378005"
  }
}
```

### Order Response

**Data Structure:**
```rust
pub struct SubmitOrderResult {
    pub order_id: String,                          // Unique order identifier
    pub to: Option<String>,                        // Deposit address (for Bitcoin/manual)
    pub amount: Option<String>,                    // Amount to deposit
    pub initiate_transaction: Option<Value>,       // Pre-built transaction (EVM)
    pub approval_transaction: Option<Value>,       // ERC20 approval (if needed)
    pub typed_data: Option<Value>,                 // EIP-712 data (for gasless)
    pub versioned_tx: Option<String>,              // Solana transaction
    pub versioned_tx_gasless: Option<String>,      // Solana gasless transaction
    pub ptb_bytes: Option<Value>,                  // Sui transaction
}
```

**Example Response (Bitcoin):**
```json
{
  "status": "Ok",
  "result": {
    "order_id": "bba25a7d50c3cc4d313e2e7b2d937e36d3f055d2f7d1b31ed91c6d92c67b1883",
    "to": "tb1px5u9ryqjwffyjsql2p8xg77dyfta8s2sqygvwgwvj7rpne04nlxs3f2tw5",
    "amount": "50000",
    "initiate_transaction": null,
    "approval_transaction": null,
    "typed_data": null,
    "versioned_tx": null,
    "versioned_tx_gasless": null,
    "ptb_bytes": null
  }
}
```

**Example Response (EVM with Gasless):**
```json
{
  "status": "Ok",
  "result": {
    "order_id": "878f980338bef1590dbff100d08d0b0c87b2dedc8de0a439d4677a3f40bcc626",
    "to": null,
    "amount": null,
    "initiate_transaction": {
      "chain_id": 5115,
      "data": "0x97ffc7ae...",
      "gas_limit": "0x493e0",
      "to": "0x730be401ef981d199a0560c87dfddafd3ec1c493",
      "value": "0x0"
    },
    "approval_transaction": {
      "chain_id": 5115,
      "data": "0x095ea7b3...",
      "gas_limit": "0x186a0",
      "to": "0x5fd0f2b5970d8cbc803736b8fef84a24b4ab39b3",
      "value": "0x0"
    },
    "typed_data": {
      "domain": {...},
      "message": {...},
      "primaryType": "Initiate",
      "types": {...}
    },
    "versioned_tx": null,
    "versioned_tx_gasless": null,
    "ptb_bytes": null
  }
}
```

### What the Order Response Contains

The response varies by chain type:

#### Bitcoin/Manual Deposit Chains
- `order_id`: Track the swap
- `to`: Deposit address where you send funds
- `amount`: Exact amount to send

#### EVM Chains (Ethereum, Base, Arbitrum, etc.)
- `order_id`: Track the swap
- `initiate_transaction`: Pre-built transaction to sign and broadcast
- `approval_transaction`: ERC20 approval (if swapping tokens)
- `typed_data`: EIP-712 signature data (for gasless swaps)

#### Solana
- `order_id`: Track the swap
- `versioned_tx`: Transaction to sign and broadcast
- `versioned_tx_gasless`: Gasless transaction (if available)

#### Starknet/Tron/Sui
- `order_id`: Track the swap
- `typed_data`: Signature data for gasless
- `ptb_bytes`: Sui programmable transaction blocks

---

## The Complete Flow: Quote → Order → Execution

### Step 1: Get Quote
```
You: "How much USDC for 50,000 sats?"
Garden: "You'll get ~3.378 USDC, takes ~10 min, 0.3% fee"
```

### Step 2: Submit Order
```
You: "OK, I want to do that swap"
Garden: "Order created! ID: bba25a7d... Send 50,000 sats to tb1px5u9..."
```

### Step 3: Initiate Swap
```
You: [Sign and broadcast transaction]
Garden: "Received! Waiting for confirmations..."
```

### Step 4: Poll Status
```
You: "What's the status?"
Garden: "Source initiated ✓, waiting for destination..."
```

### Step 5: Complete
```
You: "What's the status?"
Garden: "Completed! Destination redeemed ✓"
```

---

## Quote vs Order: Key Differences

| Aspect | Quote | Order |
|--------|-------|-------|
| **Purpose** | Price inquiry | Actual swap execution |
| **Commitment** | None | Binding commitment |
| **Cost** | Free | Requires funds |
| **Expiration** | 30-60 seconds | Until timeout (15 min) |
| **Reversible** | N/A (no action taken) | Only via refund |
| **Rate Limiting** | High (many requests) | Low (few requests) |
| **Response** | Exchange rate info | Transaction data |

---

## Real Example from Your Logs

### Quote Request
```
INFO GET quote bitcoin_testnet:btc -> bnbchain_testnet:wbtc (50000)
```

### Quote Response
```
INFO Quote received pair=bitcoin_testnet:btc -> bnbchain_testnet:wbtc 
     src=50000 dst=49850
```
Translation: "50,000 sats → 49,850 wBTC units"

### Order Submission
```
INFO POST order bitcoin_testnet:btc -> bnbchain_testnet:wbtc
```

### Order Response
```
INFO Order submitted 
     pair=bitcoin_testnet:btc -> bnbchain_testnet:wbtc 
     order_id=bba25a7d50c3cc4d313e2e7b2d937e36d3f055d2f7d1b31ed91c6d92c67b1883

INFO [DEPOSIT NEEDED] Send 50000 bitcoin_testnet:btc to 
     tb1px5u9ryqjwffyjsql2p8xg77dyfta8s2sqygvwgwvj7rpne04nlxs3f2tw5
```

### Automatic Initiation (Your System)
```
INFO Bitcoin private key found, initiating automatic transaction
INFO Bitcoin swap: sending 50000 sats to tb1px5u9ryq...
INFO Bitcoin transaction broadcasted: b4a7157529628501...
INFO Source initiation sent tx=b4a7157529628501...
```

---

## Why Two Steps?

### Separation of Concerns

1. **Quote = Discovery**
   - Check if swap is possible
   - Compare rates from multiple solvers
   - Estimate costs and timing
   - No commitment required

2. **Order = Execution**
   - Lock in the swap
   - Create atomic swap contract
   - Generate transaction data
   - Commit funds

### Benefits

1. **User Experience**: Users can shop around before committing
2. **Price Discovery**: See real-time rates without executing
3. **Error Prevention**: Validate swap before spending gas
4. **Rate Limiting**: Separate limits for quotes vs orders
5. **Flexibility**: Can get multiple quotes, submit one order

---

## Order Lifecycle

Once an order is submitted, it goes through these states:

```
1. Created      → Order submitted, waiting for initiation
2. Initiated    → Source chain transaction sent
3. Pending      → Waiting for confirmations
4. Executing    → Destination chain processing
5. Completed    → Destination redeemed ✓
   OR
5. Refunded     → Source refunded (timeout/failure)
```

### Polling Order Status

After submitting an order, you poll for status updates:

**API Endpoint:**
```
GET /v2/orders/{order_id}
```

**Response:**
```json
{
  "status": "Ok",
  "result": {
    "order_id": "bba25a7d...",
    "source_swap": {
      "initiate_tx_hash": "b4a7157529628501...",
      "redeem_tx_hash": null,
      "refund_tx_hash": null
    },
    "destination_swap": {
      "initiate_tx_hash": "0x7de9f46c...",
      "redeem_tx_hash": "0x8af3b21d...",
      "refund_tx_hash": null
    }
  }
}
```

### Status Indicators

**Source Swap:**
- `initiate_tx_hash`: Your deposit transaction
- `redeem_tx_hash`: Garden redeems your deposit (gets the secret)
- `refund_tx_hash`: Refund if swap fails

**Destination Swap:**
- `initiate_tx_hash`: Garden deposits to destination
- `redeem_tx_hash`: You redeem destination (swap complete!)
- `refund_tx_hash`: Garden refunds if you don't redeem

**Completion Check:**
```rust
// Swap is complete when destination is redeemed
if destination_swap.redeem_tx_hash.is_some() {
    println!("✅ Swap completed!");
}

// Swap failed if source is refunded
if source_swap.refund_tx_hash.is_some() {
    println!("❌ Swap refunded");
}
```

---

## Rate Limiting Issues

### The Problem You Encountered

```
ERROR ❌ Quote failed: Quote API 429 Too Many Requests - Rate limit exceeded
```

**What Happened:**
- Your system tried to get quotes for 51 swaps simultaneously
- Garden API has rate limits (e.g., 10 requests/second)
- 51 concurrent requests exceeded the limit
- Many swaps failed with 429 errors

### Solution Strategies

1. **Batch Quotes**: Request quotes in smaller batches
2. **Add Delays**: Wait between quote requests
3. **Retry Logic**: Retry failed quotes with exponential backoff
4. **Cache Quotes**: Reuse recent quotes if still valid
5. **Reduce Concurrency**: Limit concurrent quote requests

---

## Summary

### Quote
- **What**: Price inquiry
- **When**: Before committing to swap
- **Purpose**: Check rates, fees, timing
- **Commitment**: None
- **Cost**: Free (but rate limited)

### Order
- **What**: Swap execution
- **When**: After accepting quote
- **Purpose**: Execute the actual swap
- **Commitment**: Binding (funds locked)
- **Cost**: Requires funds + fees

### Analogy
Think of it like buying a concert ticket:
- **Quote** = Checking ticket prices on Ticketmaster
- **Order** = Actually purchasing the ticket
- **Initiation** = Paying for the ticket
- **Completion** = Receiving the ticket

The quote tells you what you'll get, the order makes it happen!
