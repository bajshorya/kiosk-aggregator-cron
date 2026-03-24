use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::GardenApiClient;
use crate::chains::{SwapPair, all_swap_pairs, requires_manual_deposit};
use crate::config::AppConfig;
use crate::db::Database;
use crate::models::{OrderAsset, RunSummary, SubmitOrderRequest, SwapRecord, SwapStatus};

pub struct SwapRunner {
    api: Arc<GardenApiClient>,
    db: Arc<Database>,
    config: AppConfig,
}

impl SwapRunner {
    pub fn new(api: Arc<GardenApiClient>, db: Arc<Database>, config: AppConfig) -> Self {
        Self { api, db, config }
    }

    pub async fn run_all(&self) -> RunSummary {
        let run_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();
        let pairs = all_swap_pairs(&self.config.wallets);
        let total = pairs.len();

        info!(run_id = %run_id, "=== Starting swap test run ({} pairs) ===", total);

        let mut records: Vec<SwapRecord> = Vec::with_capacity(total);

        for pair in &pairs {
            let record = self.run_single_swap(&run_id, pair).await;
            records.push(record);
            thread::sleep(Duration::from_millis(500));
        }

        let completed = records
            .iter()
            .filter(|r| r.status == SwapStatus::Completed)
            .count();
        let failed = records
            .iter()
            .filter(|r| r.status == SwapStatus::Failed)
            .count();
        let timed_out = records
            .iter()
            .filter(|r| r.status == SwapStatus::TimedOut)
            .count();
        let pending = records
            .iter()
            .filter(|r| r.status == SwapStatus::Pending)
            .count();

        info!(
            run_id = %run_id,
            "=== Run complete: total={} ok={} fail={} timeout={} pending={} ===",
            total, completed, failed, timed_out, pending
        );

        let summary = RunSummary {
            run_id,
            total_swaps: total,
            completed,
            failed,
            timed_out,
            pending,
            started_at,
            results: records,
        };

        if let Err(e) = self.db.insert_run_summary(&summary) {
            error!("Failed to save run summary: {}", e);
        }

        summary
    }

    async fn run_single_swap(&self, run_id: &str, pair: &SwapPair) -> SwapRecord {
        let mut record = SwapRecord::new(
            run_id,
            &pair.source.asset,
            &pair.destination.asset,
            &pair.source.default_from_amount,
        );

        info!(pair = %pair.label(), "Starting swap");

        // Persist initial record
        match self.db.insert_swap_record(&record) {
            Ok(id) => record.id = Some(id),
            Err(e) => error!("Failed to insert initial record: {}", e),
        }

        // ── Step 1: Get quote ──────────────────────────────────────────────
        let quote = match self
            .api
            .get_quote(
                &pair.source.asset,
                &pair.destination.asset,
                &pair.source.default_from_amount,
            )
            .await
        {
            Ok(q) => q,
            Err(e) => return self.fail(record, format!("Quote failed: {}", e)),
        };

        info!(
            pair = %pair.label(),
            src = %quote.source.amount,
            dst = %quote.destination.amount,
            "Quote received"
        );

        // ── Step 2: Submit order ───────────────────────────────────────────
        let req = SubmitOrderRequest {
            source: OrderAsset {
                asset: quote.source.asset.clone(),
                owner: pair.source.owner.clone(),
                amount: quote.source.amount.clone(),
            },
            destination: OrderAsset {
                asset: quote.destination.asset.clone(),
                owner: pair.destination.owner.clone(),
                amount: quote.destination.amount.clone(),
            },
        };

        let order_resp = match self.api.submit_order(&req).await {
            Ok(r) => r,
            Err(e) => return self.fail(record, format!("Submit order failed: {}", e)),
        };

        let order_id = order_resp.result.order_id.clone();
        record.order_id = Some(order_id.clone());
        record.deposit_address = order_resp.result.to.clone();
        record.status = SwapStatus::Pending;

        info!(pair = %pair.label(), order_id = %order_id, "Order submitted");

        // Log UTXO deposit address prominently
        if requires_manual_deposit(&pair.source.asset) {
            if let Some(ref addr) = order_resp.result.to {
                info!(
                    pair = %pair.label(),
                    order_id = %order_id,
                    "[DEPOSIT NEEDED] Send {} {} to {}",
                    pair.source.default_from_amount,
                    pair.source.asset,
                    addr
                );
            }
        }

        // Log EVM transaction data
        if let Some(ref tx) = order_resp.result.initiate_transaction {
            info!(pair = %pair.label(), "EVM initiate_transaction: {}",
                serde_json::to_string(tx).unwrap_or_default());
        }
        if let Some(ref vtx) = order_resp.result.versioned_tx {
            info!(
                pair = %pair.label(),
                "Solana versioned_tx ({} chars)",
                vtx.len()
            );
        }

        if let Err(e) = self.db.update_swap_record(&record) {
            error!("Failed to update record to Pending: {}", e);
        }

        // ── Step 3: Poll for completion ────────────────────────────────────
        let timeout = self.config.swap_timeout();
        let poll_every = self.config.poll_interval();
        let deadline = Instant::now() + timeout;

        loop {
            thread::sleep(poll_every);

            if Instant::now() >= deadline {
                warn!(pair = %pair.label(), order_id = %order_id, "Swap timed out");
                let now = Utc::now();
                record.status = SwapStatus::TimedOut;
                record.error_message = Some(format!("Timed out after {}s", timeout.as_secs()));
                record.completed_at = Some(now);
                record.duration_secs = Some((now - record.started_at).num_seconds());
                let _ = self.db.update_swap_record(&record);
                return record;
            }

            let status_resp = match self.api.get_order_status(&order_id).await {
                Ok(r) => r,
                Err(e) => {
                    warn!(pair = %pair.label(), "Poll error: {}. Retrying...", e);
                    continue;
                }
            };

            let src = &status_resp.result.source_swap;
            let dst = &status_resp.result.destination_swap;

            info!(
                pair = %pair.label(),
                src_init = %src.initiate_tx_hash.as_deref().unwrap_or(""),
                dst_redeem = %dst.redeem_tx_hash.as_deref().unwrap_or(""),
                "Poll"
            );

            record.source_initiate_tx = src.initiate_tx_hash.clone();
            record.source_redeem_tx = src.redeem_tx_hash.clone();
            record.dest_initiate_tx = dst.initiate_tx_hash.clone();
            record.dest_redeem_tx = dst.redeem_tx_hash.clone();

            if dst.is_redeemed() {
                let now = Utc::now();
                record.status = SwapStatus::Completed;
                record.completed_at = Some(now);
                record.duration_secs = Some((now - record.started_at).num_seconds());
                info!(pair = %pair.label(), order_id = %order_id, "[OK] Swap completed");
                let _ = self.db.update_swap_record(&record);
                return record;
            }

            if src.is_refunded() {
                let now = Utc::now();
                record.status = SwapStatus::Refunded;
                record.error_message = Some("Source swap was refunded".to_string());
                record.completed_at = Some(now);
                record.duration_secs = Some((now - record.started_at).num_seconds());
                warn!(pair = %pair.label(), order_id = %order_id, "[REFUND] Swap refunded");
                let _ = self.db.update_swap_record(&record);
                return record;
            }
        }
    }

    fn fail(&self, mut record: SwapRecord, message: String) -> SwapRecord {
        error!(pair = %record.swap_pair, "{}", message);
        let now = Utc::now();
        record.status = SwapStatus::Failed;
        record.error_message = Some(message);
        record.completed_at = Some(now);
        record.duration_secs = Some((now - record.started_at).num_seconds());
        let _ = self.db.update_swap_record(&record);
        record
    }
}
