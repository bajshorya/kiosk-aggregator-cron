use serde::{Deserialize, Serialize};

// --- Submit Order ---

#[derive(Debug, Serialize, Clone)]
pub struct OrderAsset {
    pub asset: String,
    pub owner: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
pub struct SubmitOrderRequest {
    pub source: OrderAsset,
    pub destination: OrderAsset,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubmitOrderResult {
    pub order_id: String,
    // Bitcoin / Litecoin style: deposit address
    pub to: Option<String>,
    pub amount: Option<String>,
    // EVM / Starknet style: transactions
    pub initiate_transaction: Option<serde_json::Value>,
    pub approval_transaction: Option<serde_json::Value>,
    pub typed_data: Option<serde_json::Value>,
    // Solana style
    pub versioned_tx: Option<String>,
    pub versioned_tx_gasless: Option<String>,
    // Sui style
    pub ptb_bytes: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitOrderResponse {
    pub status: String,
    pub result: SubmitOrderResult,
}

// --- Order Status ---

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SwapDetails {
    pub swap_id: Option<String>,
    pub chain: Option<String>,
    pub asset: Option<String>,
    pub initiator: Option<String>,
    pub redeemer: Option<String>,
    pub timelock: Option<serde_json::Value>,
    pub filled_amount: Option<String>,
    pub asset_price: Option<serde_json::Value>,
    pub amount: Option<String>,
    pub secret_hash: Option<String>,
    pub secret: Option<String>,
    pub initiate_tx_hash: Option<String>,
    pub redeem_tx_hash: Option<String>,
    pub refund_tx_hash: Option<String>,
    pub initiate_block_number: Option<serde_json::Value>,
    pub redeem_block_number: Option<serde_json::Value>,
    pub refund_block_number: Option<serde_json::Value>,
    pub required_confirmations: Option<u64>,
    pub current_confirmations: Option<u64>,
    pub initiate_timestamp: Option<String>,
    pub redeem_timestamp: Option<String>,
    pub refund_timestamp: Option<String>,
}

impl SwapDetails {
    pub fn is_redeemed(&self) -> bool {
        self.redeem_tx_hash
            .as_ref()
            .map(|h| !h.is_empty())
            .unwrap_or(false)
    }

    pub fn is_refunded(&self) -> bool {
        self.refund_tx_hash
            .as_ref()
            .map(|h| !h.is_empty())
            .unwrap_or(false)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderStatus {
    pub created_at: Option<String>,
    pub order_id: String,
    pub source_swap: SwapDetails,
    pub destination_swap: SwapDetails,
    pub nonce: Option<serde_json::Value>,
    pub solver_id: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderStatusResponse {
    pub status: String,
    pub result: OrderStatus,
}
