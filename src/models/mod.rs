// All types consolidated in this file to avoid duplicate definitions
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── Quote ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuoteAsset {
    pub asset: String,
    pub amount: String,
    pub display: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Quote {
    pub source: QuoteAsset,
    pub destination: QuoteAsset,
    pub solver_id: String,
    pub estimated_time: u64,
    pub slippage: u64,
    pub fee: u64,
    pub fixed_fee: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuoteResponse {
    pub status: String,
    pub result: Vec<Quote>,
}

// ─── Order ───────────────────────────────────────────────────────────────────

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
    pub to: Option<String>,
    pub amount: Option<String>,
    pub initiate_transaction: Option<serde_json::Value>,
    pub approval_transaction: Option<serde_json::Value>,
    pub typed_data: Option<serde_json::Value>,
    pub versioned_tx: Option<String>,
    pub versioned_tx_gasless: Option<String>,
    pub ptb_bytes: Option<serde_json::Value>,
    pub transaction: Option<String>,
    pub transaction_gasless: Option<String>,
    pub tron_transaction: Option<serde_json::Value>,
    pub starknet_typed_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitOrderResponse {
    pub status: String,
    pub result: SubmitOrderResult,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SwapDetails {
    pub swap_id: Option<String>,
    pub chain: Option<String>,
    pub asset: Option<String>,
    pub amount: Option<String>,
    pub secret_hash: Option<String>,
    pub secret: Option<String>,
    pub initiate_tx_hash: Option<String>,
    pub redeem_tx_hash: Option<String>,
    pub refund_tx_hash: Option<String>,
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
    pub order_id: String,
    pub source_swap: SwapDetails,
    pub destination_swap: SwapDetails,
    pub created_at: Option<String>,
    pub solver_id: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderStatusResponse {
    pub status: String,
    pub result: OrderStatus,
}

// ─── Swap Record / Summary ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwapStatus {
    Initiated,
    Pending,
    Completed,
    Refunded,
    Failed,
    TimedOut,
}

impl std::fmt::Display for SwapStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapStatus::Initiated => write!(f, "Initiated"),
            SwapStatus::Pending => write!(f, "Pending"),
            SwapStatus::Completed => write!(f, "Completed"),
            SwapStatus::Refunded => write!(f, "Refunded"),
            SwapStatus::Failed => write!(f, "Failed"),
            SwapStatus::TimedOut => write!(f, "TimedOut"),
        }
    }
}

impl From<String> for SwapStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Initiated" => SwapStatus::Initiated,
            "Pending" => SwapStatus::Pending,
            "Completed" => SwapStatus::Completed,
            "Refunded" => SwapStatus::Refunded,
            "TimedOut" => SwapStatus::TimedOut,
            _ => SwapStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRecord {
    pub id: Option<i64>,
    pub run_id: String,
    pub swap_pair: String,
    pub from_chain: String,
    pub to_chain: String,
    pub from_asset: String,
    pub to_asset: String,
    pub from_amount: String,
    pub order_id: Option<String>,
    pub deposit_address: Option<String>,
    pub status: SwapStatus,
    pub error_message: Option<String>,
    pub source_initiate_tx: Option<String>,
    pub source_redeem_tx: Option<String>,
    pub dest_initiate_tx: Option<String>,
    pub dest_redeem_tx: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<i64>,
}

impl SwapRecord {
    pub fn new(run_id: &str, from_asset: &str, to_asset: &str, from_amount: &str) -> Self {
        let from_chain = from_asset.split(':').next().unwrap_or("").to_string();
        let to_chain = to_asset.split(':').next().unwrap_or("").to_string();
        SwapRecord {
            id: None,
            run_id: run_id.to_string(),
            swap_pair: format!("{} -> {}", from_asset, to_asset),
            from_chain,
            to_chain,
            from_asset: from_asset.to_string(),
            to_asset: to_asset.to_string(),
            from_amount: from_amount.to_string(),
            order_id: None,
            deposit_address: None,
            status: SwapStatus::Initiated,
            error_message: None,
            source_initiate_tx: None,
            source_redeem_tx: None,
            dest_initiate_tx: None,
            dest_redeem_tx: None,
            started_at: Utc::now(),
            completed_at: None,
            duration_secs: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RunSummary {
    pub run_id: String,
    pub total_swaps: usize,
    pub completed: usize,
    pub failed: usize,
    pub timed_out: usize,
    pub pending: usize,
    pub started_at: DateTime<Utc>,
    pub results: Vec<SwapRecord>,
}
