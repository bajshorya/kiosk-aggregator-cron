use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use tokio::task::JoinSet;
use tokio::time::sleep;
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

    /// Test a single swap pair by asset names
    pub async fn test_single_swap(&self, from_asset: &str, to_asset: &str) -> Result<SwapRecord, anyhow::Error> {
        use crate::chains::all_swap_pairs;
        
        let pairs = all_swap_pairs(&self.config.wallets);
        let pair = pairs
            .iter()
            .find(|p| p.source.asset == from_asset && p.destination.asset == to_asset)
            .ok_or_else(|| anyhow::anyhow!(
                "Swap pair not found: {} -> {}\nUse 'list-swaps' to see available pairs",
                from_asset,
                to_asset
            ))?;

        let run_id = uuid::Uuid::new_v4().to_string();
        info!("Testing single swap: {}", pair.label());
        
        let record = self.run_single_swap(&run_id, pair).await;
        
        // Save to database
        let summary = crate::models::RunSummary {
            run_id: run_id.clone(),
            total_swaps: 1,
            completed: if record.status == SwapStatus::Completed { 1 } else { 0 },
            failed: if record.status == SwapStatus::Failed { 1 } else { 0 },
            timed_out: if record.status == SwapStatus::TimedOut { 1 } else { 0 },
            pending: if record.status == SwapStatus::Pending { 1 } else { 0 },
            started_at: record.started_at,
            results: vec![record.clone()],
        };
        
        let _ = self.db.insert_run_summary(&summary);
        
        Ok(record)
    }

    /// Run all swap pairs CONCURRENTLY - all swaps start at once
    pub async fn run_all(&self) -> RunSummary {
        let run_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();
        let pairs = all_swap_pairs(&self.config.wallets);
        let total = pairs.len();

        info!(run_id = %run_id, "=== Starting CONCURRENT swap test run ({} pairs) ===", total);
        info!("All swaps will start simultaneously!");

        // Spawn all swaps as concurrent tasks
        let mut set: JoinSet<SwapRecord> = JoinSet::new();
        
        for pair in pairs {
            let api = Arc::clone(&self.api);
            let db = Arc::clone(&self.db);
            let config = self.config.clone();
            let run_id_c = run_id.clone();
            
            set.spawn(async move {
                let runner = SwapRunner { api, db, config };
                runner.run_single_swap(&run_id_c, &pair).await
            });
        }

        // Collect results as tasks complete
        let mut records: Vec<SwapRecord> = Vec::with_capacity(total);
        while let Some(res) = set.join_next().await {
            match res {
                Ok(record) => {
                    info!(
                        pair = %record.swap_pair,
                        status = %record.status,
                        duration = ?record.duration_secs,
                        "✅ Task finished"
                    );
                    records.push(record);
                }
                Err(e) => error!("Swap task panicked: {}", e),
            }
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

        // ── Step 3: Poll for completion with DB updates on each poll ──────
        let timeout = self.config.swap_timeout();
        let poll_every = self.config.poll_interval();
        let deadline = Instant::now() + timeout;
        let mut poll_count: u32 = 0;

        loop {
            sleep(poll_every).await;
            poll_count += 1;

            if Instant::now() >= deadline {
                warn!(
                    pair = %pair.label(),
                    order_id = %order_id,
                    polls = poll_count,
                    "⏰ Swap timed out after {}s",
                    timeout.as_secs()
                );
                let now = Utc::now();
                record.status = SwapStatus::TimedOut;
                record.error_message = Some(format!("Timed out after {}s ({} polls)", timeout.as_secs(), poll_count));
                record.completed_at = Some(now);
                record.duration_secs = Some((now - record.started_at).num_seconds());
                let _ = self.db.update_swap_record(&record);
                return record;
            }

            let status_resp = match self.api.get_order_status(&order_id).await {
                Ok(r) => r,
                Err(e) => {
                    warn!(pair = %pair.label(), "Poll #{} error: {}. Retrying...", poll_count, e);
                    continue;
                }
            };

            let src = &status_resp.result.source_swap;
            let dst = &status_resp.result.destination_swap;

            info!(
                pair = %pair.label(),
                poll = poll_count,
                src_init = %src.initiate_tx_hash.as_deref().unwrap_or("—"),
                src_redeem = %src.redeem_tx_hash.as_deref().unwrap_or("—"),
                dst_init = %dst.initiate_tx_hash.as_deref().unwrap_or("—"),
                dst_redeem = %dst.redeem_tx_hash.as_deref().unwrap_or("—"),
                "Poll"
            );

            // Update TX hashes
            let prev_src_init = record.source_initiate_tx.clone();
            record.source_initiate_tx = src.initiate_tx_hash.clone();
            record.source_redeem_tx = src.redeem_tx_hash.clone();
            record.dest_initiate_tx = dst.initiate_tx_hash.clone();
            record.dest_redeem_tx = dst.redeem_tx_hash.clone();

            // Update DB on every poll when TX hashes change
            if record.source_initiate_tx != prev_src_init
                || record.dest_initiate_tx.is_some()
                || record.dest_redeem_tx.is_some()
            {
                if let Err(e) = self.db.update_swap_record(&record) {
                    warn!(pair = %pair.label(), "Failed to persist poll state: {}", e);
                }
            }

            // Terminal state: destination redeemed = success
            if dst.is_redeemed() {
                let now = Utc::now();
                record.status = SwapStatus::Completed;
                record.completed_at = Some(now);
                record.duration_secs = Some((now - record.started_at).num_seconds());
                info!(
                    pair = %pair.label(),
                    order_id = %order_id,
                    polls = poll_count,
                    duration_secs = record.duration_secs.unwrap_or(0),
                    "✅ Swap completed"
                );
                let _ = self.db.update_swap_record(&record);
                return record;
            }

            // Terminal state: source refunded = failed swap
            if src.is_refunded() {
                let now = Utc::now();
                record.status = SwapStatus::Refunded;
                record.error_message = Some("Source swap was refunded".to_string());
                record.completed_at = Some(now);
                record.duration_secs = Some((now - record.started_at).num_seconds());
                warn!(
                    pair = %pair.label(),
                    order_id = %order_id,
                    polls = poll_count,
                    "↩️  Swap refunded"
                );
                let _ = self.db.update_swap_record(&record);
                return record;
            }
        }
    }

    fn fail(&self, mut record: SwapRecord, message: String) -> SwapRecord {
        error!(pair = %record.swap_pair, "❌ {}", message);
        let now = Utc::now();
        record.status = SwapStatus::Failed;
        record.error_message = Some(message);
        record.completed_at = Some(now);
        record.duration_secs = Some((now - record.started_at).num_seconds());
        // Only update if we have an ID (initial insert succeeded)
        if record.id.is_some() {
            let _ = self.db.update_swap_record(&record);
        }
        record
    }
}
