use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
            "Failed" => SwapStatus::Failed,
            "TimedOut" => SwapStatus::TimedOut,
            _ => SwapStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRecord {
    pub id: Option<i64>,
    pub run_id: String,
    pub swap_pair: String, // e.g. "bitcoin_testnet:btc -> base_sepolia:wbtc"
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
