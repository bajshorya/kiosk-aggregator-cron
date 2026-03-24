use serde::{Deserialize, Serialize};

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
