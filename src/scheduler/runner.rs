use crate::chains::evm_signer::EvmSigner;
use crate::chains::solana_signer::SolanaSigner;
use crate::chains::starknet_signer::StarknetSigner;
use crate::chains::tron_signer::TronSigner;
use anyhow::Context;
use std::sync::Arc;
use tokio::sync::Mutex; // ← not std::sync::Mutex

use std::time::Instant;

use chrono::Utc;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::GardenApiClient;
use crate::chains::{all_swap_pairs, requires_manual_deposit, SwapPair};
use crate::config::AppConfig;
use crate::db::Database;
use crate::models::{OrderAsset, RunSummary, SubmitOrderRequest, SwapRecord, SwapStatus};

pub struct SwapRunner {
    api: Arc<GardenApiClient>,
    db: Arc<Database>,
    config: AppConfig,
    evm_lock: Arc<Mutex<()>>,
}

impl SwapRunner {
    pub fn new(api: Arc<GardenApiClient>, db: Arc<Database>, config: AppConfig) -> Self {
        Self {
            api,
            db,
            config,
            evm_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Test a single swap pair by asset names
    #[allow(dead_code)]
    pub async fn test_single_swap(
        &self,
        from_asset: &str,
        to_asset: &str,
    ) -> Result<SwapRecord, anyhow::Error> {
        self.test_single_swap_with_amount(from_asset, to_asset, None).await
    }

    /// Test a single swap pair by asset names with custom amount
    pub async fn test_single_swap_with_amount(
        &self,
        from_asset: &str,
        to_asset: &str,
        custom_amount: Option<String>,
    ) -> Result<SwapRecord, anyhow::Error> {
        use crate::chains::all_swap_pairs;

        let pairs = all_swap_pairs(&self.config.wallets);
        let mut pair = pairs
            .iter()
            .find(|p| p.source.asset == from_asset && p.destination.asset == to_asset)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Swap pair not found: {} -> {}\nUse 'list-swaps' to see available pairs",
                    from_asset,
                    to_asset
                )
            })?
            .clone();

        // Override amount if provided
        if let Some(amount) = custom_amount {
            pair.source.default_from_amount = amount;
        }

        let run_id = uuid::Uuid::new_v4().to_string();
        info!("Testing single swap: {}", pair.label());

        let record = self.run_single_swap(&run_id, &pair).await;

        // Save to database
        let summary = crate::models::RunSummary {
            run_id: run_id.clone(),
            total_swaps: 1,
            completed: if record.status == SwapStatus::Completed {
                1
            } else {
                0
            },
            failed: if record.status == SwapStatus::Failed {
                1
            } else {
                0
            },
            timed_out: if record.status == SwapStatus::TimedOut {
                1
            } else {
                0
            },
            pending: if record.status == SwapStatus::Pending {
                1
            } else {
                0
            },
            started_at: record.started_at,
            results: vec![record.clone()],
        };

        let _ = self.db.insert_run_summary(&summary);

        Ok(record)
    }

    /// Run all swap pairs CONCURRENTLY - all swaps start at once
    /// Run all swap pairs in batches, filtering by available balance
    pub async fn run_all(&self, check_balance: bool) -> RunSummary {
        let run_id = Uuid::new_v4().to_string();
        let started_at = Utc::now();
        let all_pairs = all_swap_pairs(&self.config.wallets);
        
        info!(run_id = %run_id, "=== Starting swap test run ({} pairs) ===", all_pairs.len());

        // Filter pairs based on available balance (only if check_balance is enabled)
        let executable_pairs = if check_balance {
            info!("Checking balances for EVM chains only (fast check)...");
            let mut filtered_pairs = Vec::new();
            
            for pair in &all_pairs {
                let chain = pair.source.asset.split(':').next().unwrap_or("");
                
                // Only check balance for EVM chains (Arbitrum, Base, Ethereum)
                // Skip balance check for other chains to avoid delays
                if !chain.contains("arbitrum") && !chain.contains("base") && !chain.contains("ethereum") {
                    filtered_pairs.push(pair.clone());
                    continue;
                }

                let rpc_url = match self.rpc_url_for_chain(chain) {
                    Ok(url) => url,
                    Err(_) => {
                        info!("Skipping {} (no RPC configured)", pair.label());
                        continue;
                    }
                };

                match crate::chains::balance_checker::check_balance(
                    chain,
                    &pair.source.asset,
                    &pair.source.owner,
                    &pair.source.default_from_amount,
                    &rpc_url,
                )
                .await
                {
                    Ok(true) => {
                        filtered_pairs.push(pair.clone());
                    }
                    Ok(false) => {
                        info!("⏭️  Skipping {} (insufficient balance)", pair.label());
                    }
                    Err(e) => {
                        warn!("Balance check error for {}: {}, will attempt anyway", pair.label(), e);
                        filtered_pairs.push(pair.clone());
                    }
                }
            }

            info!(
                "✅ Balance check complete: {}/{} pairs will be attempted",
                filtered_pairs.len(),
                all_pairs.len()
            );
            filtered_pairs
        } else {
            info!("Balance checking disabled - attempting all {} pairs", all_pairs.len());
            all_pairs.clone()
        };

        // Separate Bitcoin swaps from other swaps
        let (bitcoin_pairs, other_pairs): (Vec<_>, Vec<_>) = executable_pairs
            .into_iter()
            .partition(|pair| {
                let chain = pair.source.asset.split(':').next().unwrap_or("");
                chain.starts_with("bitcoin_")
            });

        let total = bitcoin_pairs.len() + other_pairs.len();
        
        // Batch configuration
        const BATCH_SIZE: usize = 10;
        const BATCH_DELAY_SECS: u64 = 2;
        
        if !bitcoin_pairs.is_empty() {
            info!(
                "⚠️  Bitcoin UTXO conflict prevention: {} Bitcoin swaps will run SEQUENTIALLY (5s delay between each)",
                bitcoin_pairs.len()
            );
        }
        
        let batch_count = (other_pairs.len() + BATCH_SIZE - 1) / BATCH_SIZE;
        info!(
            "📦 Batch processing: {} non-Bitcoin swaps in {} batches of {} ({}s delay between batches)",
            other_pairs.len(),
            batch_count,
            BATCH_SIZE,
            BATCH_DELAY_SECS
        );
        
        let mut set: JoinSet<SwapRecord> = JoinSet::new();

        // Process non-Bitcoin swaps in batches
        for (batch_idx, batch) in other_pairs.chunks(BATCH_SIZE).enumerate() {
            if batch_idx > 0 {
                info!(
                    "⏱️  Waiting {}s before batch {}/{}...",
                    BATCH_DELAY_SECS,
                    batch_idx + 1,
                    batch_count
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(BATCH_DELAY_SECS)).await;
            }
            
            info!(
                "🚀 Starting batch {}/{}: {} swaps",
                batch_idx + 1,
                batch_count,
                batch.len()
            );
            
            // Spawn all swaps in this batch concurrently
            for pair in batch {
                let api = Arc::clone(&self.api);
                let db = Arc::clone(&self.db);
                let config = self.config.clone();
                let run_id_c = run_id.clone();
                let pair_c = pair.clone();

                let evm_lock = Arc::clone(&self.evm_lock);
                set.spawn(async move {
                    let runner = SwapRunner {
                        api,
                        db,
                        config,
                        evm_lock,
                    };
                    runner.run_single_swap(&run_id_c, &pair_c).await
                });
            }
        }

        // Spawn Bitcoin swaps SEQUENTIALLY with delays
        let bitcoin_count = bitcoin_pairs.len();
        if !bitcoin_pairs.is_empty() {
            let api = Arc::clone(&self.api);
            let db = Arc::clone(&self.db);
            let config = self.config.clone();
            let run_id_c = run_id.clone();
            
            // Spawn a single task that runs all Bitcoin swaps sequentially
            tokio::spawn(async move {
                for (idx, pair) in bitcoin_pairs.iter().enumerate() {
                    if idx > 0 {
                        // Wait 5 seconds between Bitcoin swaps to allow:
                        // 1. Previous transaction to broadcast
                        // 2. Change UTXO to appear in mempool
                        // 3. Next swap to fetch and use the new UTXO
                        info!("⏱️  Waiting 5 seconds before next Bitcoin swap (UTXO reuse)...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                    
                    let runner = SwapRunner {
                        api: Arc::clone(&api),
                        db: Arc::clone(&db),
                        config: config.clone(),
                        evm_lock: Arc::new(Mutex::new(())),
                    };
                    
                    info!("🔶 Bitcoin swap {}/{}: {}", idx + 1, bitcoin_pairs.len(), pair.label());
                    let _record = runner.run_single_swap(&run_id_c, pair).await;
                    // Records are saved to DB by run_single_swap, we don't need to collect them here
                }
            });
        }

        // Collect all results as they complete
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

        // Wait a bit for Bitcoin swaps to complete and save to DB
        if bitcoin_count > 0 {
            info!("⏱️  Waiting for Bitcoin swaps to complete...");
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
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
        
        // Debug: Log the full order response to see what fields are available
        info!(
            pair = %pair.label(),
            "Order response fields: versioned_tx={}, versioned_tx_gasless={}, initiate_transaction={}, typed_data={}",
            order_resp.result.versioned_tx.is_some(),
            order_resp.result.versioned_tx_gasless.is_some(),
            order_resp.result.initiate_transaction.is_some(),
            order_resp.result.typed_data.is_some()
        );

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
        let init_result = self
            .dispatch_initiation(&pair.source.asset, &order_resp.result)
            .await;
        match init_result {
            Ok(hash) => {
                info!(pair = %pair.label(), tx = %hash, "Source initiation sent");
                record.source_initiate_tx = Some(hash);
            }
            Err(e) => {
                if !requires_manual_deposit(&pair.source.asset) {
                    return self.fail(record, format!("Initiation failed: {}", e));
                }
                // manual deposit chains: log and continue polling
                warn!(pair = %pair.label(), "Awaiting manual deposit: {}", e);
            }
        }

        // Log EVM transaction data (keep for debugging)
        if let Some(ref tx) = order_resp.result.initiate_transaction {
            info!(pair = %pair.label(), "EVM initiate_transaction: {}",
                serde_json::to_string(tx).unwrap_or_default());
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
                record.error_message = Some(format!(
                    "Timed out after {}s ({} polls)",
                    timeout.as_secs(),
                    poll_count
                ));
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
    async fn dispatch_initiation(
        &self,
        source_asset: &str,
        order_result: &crate::models::SubmitOrderResult,
    ) -> Result<String, anyhow::Error> {
        let chain = source_asset.split(':').next().unwrap_or("");

        match chain {
            c if c.starts_with("ethereum_")
                || c.starts_with("base_")
                || c.starts_with("arbitrum_")
                || c.starts_with("alpen_")
                || c.starts_with("bnbchain_")
                || c.starts_with("citrea_")
                || c.starts_with("monad_")
                || c.starts_with("xrpl_") =>
            {
                let signer = EvmSigner::new(self.config.wallets.evm_private_key.clone())?;
                
                // Check if approval transaction is needed (for ERC20 tokens)
                if let Some(approval_tx) = &order_result.approval_transaction {
                    info!("Approval transaction required for ERC20 token");
                    let rpc_url = self.rpc_url_for_chain(chain)?;
                    
                    info!("Executing approval transaction via RPC");
                    let approval_hash = signer.send_transaction(approval_tx, &rpc_url).await?;
                    info!("Approval transaction sent: {}", approval_hash);
                    
                    // Wait a bit for approval to be mined
                    info!("Waiting 10s for approval transaction to be mined...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
                
                // Check if gasless is available (typed_data present)
                if let Some(typed_data) = &order_result.typed_data {
                    // Try EIP-712 gasless flow first
                    info!("Attempting EIP-712 gasless initiation for {}", source_asset);
                    
                    // Sign typed data for gasless initiation
                    let signature = signer.sign_typed_data(typed_data).await?;

                    let order_id = &order_result.order_id;
                    info!("Submitting EIP-712 signature for order {}", order_id);
                    
                    // Try gasless, but fall back to direct transaction if it fails
                    match self.api.initiate_swap_gasless_evm(order_id, &signature).await {
                        Ok(_) => {
                            info!("EVM gasless initiation submitted successfully");
                            return Ok(format!("gasless-evm-{}", order_id));
                        }
                        Err(e) => {
                            warn!(
                                "Gasless initiation failed: {}. Falling back to direct transaction broadcast.",
                                e
                            );
                            
                            // Fall through to direct transaction broadcast
                            if let Some(tx_data) = &order_result.initiate_transaction {
                                let rpc_url = self.rpc_url_for_chain(chain)?;
                                info!("Broadcasting EVM transaction via RPC (fallback)");
                                let tx_hash = signer.send_transaction(tx_data, &rpc_url).await?;
                                info!("EVM transaction broadcasted: {}", tx_hash);
                                return Ok(tx_hash);
                            } else {
                                return Err(anyhow::anyhow!(
                                    "Gasless failed and no initiate_transaction available for fallback"
                                ));
                            }
                        }
                    }
                } else if let Some(tx_data) = &order_result.initiate_transaction {
                    // No gasless available, use traditional transaction broadcasting
                    warn!(
                        asset = %source_asset,
                        "Gasless not available (typed_data=null). Using traditional transaction broadcasting."
                    );
                    
                    let rpc_url = self.rpc_url_for_chain(chain)?;
                    
                    info!("Broadcasting EVM transaction via RPC");
                    let tx_hash = signer.send_transaction(tx_data, &rpc_url).await?;
                    
                    info!("EVM transaction broadcasted: {}", tx_hash);
                    Ok(tx_hash)
                } else {
                    Err(anyhow::anyhow!(
                        "No typed_data or initiate_transaction in EVM order response"
                    ))
                }
            }
            c if c.starts_with("tron_") => {
                // Check if Tron private key is configured
                let private_key = self
                    .config
                    .wallets
                    .tron_private_key
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "TRON_PRIVATE_KEY not set in .env. \
                            Please add your Tron private key to enable automatic signing."
                        )
                    })?;

                let signer = TronSigner::new(private_key.clone())?;
                
                // Check if approval transaction is needed
                if let Some(_approval_tx) = &order_result.approval_transaction {
                    info!("Approval transaction required for Tron token");
                    let _rpc_url = self.rpc_url_for_chain(chain)?;
                    
                    // For Tron, we need to sign and broadcast the approval
                    // This is a simplified version - in production, use proper Tron transaction handling
                    warn!("Tron approval transaction handling not fully implemented");
                }
                
                // Check if gasless is available (typed_data present)
                if let Some(typed_data) = &order_result.typed_data {
                    info!("Attempting gasless initiation for {}", source_asset);
                    
                    // Sign typed data for gasless initiation
                    let signature = signer.sign_typed_data(typed_data).await?;

                    let order_id = &order_result.order_id;
                    info!("Submitting Tron signature for order {}", order_id);
                    
                    // Try gasless
                    match self.api.initiate_swap_gasless_tron(order_id, &signature).await {
                        Ok(_) => {
                            info!("Tron gasless initiation submitted successfully");
                            return Ok(format!("gasless-tron-{}", order_id));
                        }
                        Err(e) => {
                            warn!(
                                "Gasless initiation failed: {}. Tron non-gasless not yet implemented.",
                                e
                            );
                            return Err(anyhow::anyhow!("Tron gasless failed and non-gasless not implemented"));
                        }
                    }
                } else {
                    warn!(
                        asset = %source_asset,
                        "Gasless not available (typed_data=null). Tron non-gasless not yet implemented."
                    );
                    Err(anyhow::anyhow!("Tron non-gasless transaction broadcasting not yet implemented"))
                }
            }

            c if c.starts_with("starknet_") => {
                // Check if Starknet private key is configured
                let private_key = self
                    .config
                    .wallets
                    .starknet_private_key
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "STARKNET_PRIVATE_KEY not set in .env. \
                            Please add your Starknet private key to enable automatic signing."
                        )
                    })?;

                let signer = StarknetSigner::new(private_key.clone())?;
                
                // Check if approval transaction is needed
                if let Some(_approval_tx) = &order_result.approval_transaction {
                    info!("Approval transaction required for Starknet token");
                    // Starknet approval handling would go here
                    warn!("Starknet approval transaction handling not fully implemented");
                }
                
                // Check if gasless is available (typed_data present)
                if let Some(typed_data) = &order_result.typed_data {
                    info!("Attempting gasless initiation for {}", source_asset);
                    
                    // Sign typed data for gasless initiation
                    let signature = signer.sign_typed_data(typed_data).await?;

                    let order_id = &order_result.order_id;
                    info!("Submitting Starknet signature for order {}", order_id);
                    
                    // Try gasless
                    match self.api.initiate_swap_gasless_starknet(order_id, &signature).await {
                        Ok(_) => {
                            info!("Starknet gasless initiation submitted successfully");
                            return Ok(format!("gasless-starknet-{}", order_id));
                        }
                        Err(e) => {
                            warn!(
                                "Gasless initiation failed: {}. Starknet non-gasless not yet implemented.",
                                e
                            );
                            return Err(anyhow::anyhow!("Starknet gasless failed and non-gasless not implemented"));
                        }
                    }
                } else {
                    warn!(
                        asset = %source_asset,
                        "Gasless not available (typed_data=null). Starknet non-gasless not yet implemented."
                    );
                    Err(anyhow::anyhow!("Starknet non-gasless transaction broadcasting not yet implemented"))
                }
            }

            c if c.starts_with("solana_") => {
                // Check if private key is configured
                info!(
                    asset = %source_asset,
                    "Checking Solana private key: is_some={}",
                    self.config.wallets.solana_private_key.is_some()
                );
                
                let private_key = self
                    .config
                    .wallets
                    .solana_private_key
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "SOLANA_PRIVATE_KEY not set in .env. \
                            Please add your Solana private key to enable automatic signing."
                        )
                    })?;

                let signer = SolanaSigner::new(private_key)
                    .context("Failed to create Solana signer")?;

                // Check if gasless is available (versioned_tx_gasless present)
                if let Some(versioned_tx_gasless) = &order_result.versioned_tx_gasless {
                    info!(
                        asset = %source_asset,
                        "Using versioned_tx_gasless ({} chars) - GASLESS MODE",
                        versioned_tx_gasless.len()
                    );

                    // Sign the gasless transaction
                    info!(asset = %source_asset, "Signing Solana gasless transaction...");
                    let signed_tx_base64 = signer
                        .sign_transaction(versioned_tx_gasless)
                        .map_err(|e| {
                            error!(asset = %source_asset, error = %e, "Solana signing failed");
                            anyhow::anyhow!("Failed to sign Solana transaction: {}", e)
                        })?;

                    info!(asset = %source_asset, "Signed transaction length: {} chars", signed_tx_base64.len());

                    // Submit via gasless /initiate endpoint (not PATCH)
                    let order_id = &order_result.order_id;
                    info!(asset = %source_asset, order_id = %order_id, "Submitting via gasless /initiate endpoint...");
                    
                    let tx_hash = self.api
                        .initiate_swap_gasless_solana(order_id, &signed_tx_base64)
                        .await
                        .map_err(|e| {
                            error!(asset = %source_asset, error = %e, "Gasless initiation failed");
                            anyhow::anyhow!("Failed to initiate swap via gasless endpoint: {}", e)
                        })?;

                    info!(asset = %source_asset, order_id = %order_id, tx_hash = %tx_hash, "Gasless initiation submitted successfully");
                    Ok(tx_hash)
                } else if let Some(versioned_tx) = &order_result.versioned_tx {
                    // Non-gasless: broadcast directly to Solana RPC
                    warn!(
                        asset = %source_asset,
                        "Gasless not enabled (versioned_tx_gasless=null). Broadcasting to Solana RPC directly."
                    );
                    
                    info!(
                        asset = %source_asset,
                        "Using versioned_tx ({} chars) - NON-GASLESS MODE",
                        versioned_tx.len()
                    );

                    let rpc_url = self.config.rpc_urls.solana_testnet.clone();
                    info!(asset = %source_asset, "Broadcasting to Solana RPC: {}", rpc_url);
                    
                    // Use sign_and_send to broadcast directly
                    let tx_hash = signer
                        .sign_and_send(versioned_tx, &rpc_url)
                        .await
                        .map_err(|e| {
                            error!(asset = %source_asset, error = %e, "Solana broadcast failed");
                            anyhow::anyhow!("Failed to broadcast Solana transaction: {}", e)
                        })?;

                    info!(asset = %source_asset, "Solana transaction broadcasted: {}", tx_hash);
                    Ok(tx_hash)
                } else {
                    Err(anyhow::anyhow!("No versioned_tx or versioned_tx_gasless in Solana response"))
                }
            }

            c if c.starts_with("bitcoin_") => {
                // Check if Bitcoin private key is configured
                let private_key_wif = self
                    .config
                    .wallets
                    .bitcoin_testnet_private_key
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "BITCOIN_TESTNET_PRIVATE_KEY not set in .env. \
                            Bitcoin swaps require manual deposit or set your WIF private key."
                        )
                    })?;

                info!(asset = %source_asset, "Bitcoin private key found, initiating automatic transaction");

                // Get deposit address from order
                let deposit_address = order_result
                    .to
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No deposit address in order response"))?;

                // Parse amount from order response
                let amount_sats: u64 = order_result
                    .amount
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No amount in order response"))?
                    .parse()
                    .context("Failed to parse Bitcoin amount")?;

                info!(
                    "Bitcoin swap: sending {} sats to {}",
                    amount_sats, deposit_address
                );

                // Initialize Bitcoin signer and provider
                use crate::chains::bitcoin_signer::BitcoinSigner;
                use crate::chains::bitcoin_provider::BitcoinProvider;
                use bitcoin::Network;

                let network = if c.contains("testnet") {
                    Network::Testnet
                } else {
                    Network::Bitcoin
                };

                let signer = BitcoinSigner::new(private_key_wif.clone(), network)
                    .context("Failed to create Bitcoin signer")?;

                let provider = BitcoinProvider::new(self.config.rpc_urls.bitcoin_testnet.clone());

                // Get wallet address
                let wallet_address = signer.get_address()?;
                info!("Bitcoin wallet address: {}", wallet_address);

                // Fetch UTXOs
                let all_utxos = provider
                    .get_utxos(&wallet_address)
                    .await
                    .context("Failed to fetch Bitcoin UTXOs")?;

                if all_utxos.is_empty() {
                    anyhow::bail!("No UTXOs available for Bitcoin address {}", wallet_address);
                }

                let confirmed_count = all_utxos.iter().filter(|u| u.confirmed).count();
                let unconfirmed_count = all_utxos.len() - confirmed_count;
                let confirmed_total: u64 = all_utxos.iter().filter(|u| u.confirmed).map(|u| u.value).sum();
                let unconfirmed_total: u64 = all_utxos.iter().filter(|u| !u.confirmed).map(|u| u.value).sum();
                let total_available: u64 = all_utxos.iter().map(|u| u.value).sum();
                
                info!(
                    "Found {} UTXOs with total {} sats ({} confirmed: {} sats, {} unconfirmed: {} sats)",
                    all_utxos.len(),
                    total_available,
                    confirmed_count,
                    confirmed_total,
                    unconfirmed_count,
                    unconfirmed_total
                );

                // Estimate fee
                let fee_rate = provider.estimate_fee().await.unwrap_or(2); // sats/vbyte
                
                // Select UTXOs - use only what we need (greedy algorithm)
                // IMPORTANT: Include both confirmed AND unconfirmed UTXOs to enable continuous testing
                let mut selected_utxos = Vec::new();
                let mut selected_total: u64 = 0;
                
                // Sort UTXOs by value (largest first) for efficiency
                let mut sorted_utxos = all_utxos.clone();
                sorted_utxos.sort_by(|a, b| b.value.cmp(&a.value));
                
                // Estimate fee for transaction (start with 2 outputs: recipient + change)
                let mut estimated_fee = provider.calculate_fee(1, 2, fee_rate);
                
                for utxo in sorted_utxos {
                    selected_utxos.push(utxo.clone());
                    selected_total += utxo.value;
                    
                    // Recalculate fee with current number of inputs
                    estimated_fee = provider.calculate_fee(selected_utxos.len(), 2, fee_rate);
                    
                    // Check if we have enough
                    if selected_total >= amount_sats + estimated_fee {
                        break;
                    }
                }

                // Log selected UTXOs with confirmation status
                let selected_confirmed = selected_utxos.iter().filter(|u| u.confirmed).count();
                let selected_unconfirmed = selected_utxos.len() - selected_confirmed;
                info!(
                    "Selected {} UTXOs with total {} sats (need {} sats including fee)",
                    selected_utxos.len(),
                    selected_total,
                    amount_sats + estimated_fee
                );
                info!(
                    "  └─ {} confirmed, {} unconfirmed (mempool) UTXOs",
                    selected_confirmed,
                    selected_unconfirmed
                );
                info!("Estimated fee: {} sats ({} sats/vbyte)", estimated_fee, fee_rate);
                
                // Log each selected UTXO
                for (idx, utxo) in selected_utxos.iter().enumerate() {
                    info!(
                        "  UTXO #{}: {}...{} vout={} value={} sats [{}]",
                        idx + 1,
                        &utxo.txid[..8],
                        &utxo.txid[utxo.txid.len()-8..],
                        utxo.vout,
                        utxo.value,
                        if utxo.confirmed { "CONFIRMED" } else { "MEMPOOL" }
                    );
                }

                if selected_total < amount_sats + estimated_fee {
                    anyhow::bail!(
                        "Insufficient Bitcoin balance: have {} sats, need {} sats (amount: {}, fee: {})",
                        selected_total,
                        amount_sats + estimated_fee,
                        amount_sats,
                        estimated_fee
                    );
                }

                // Build and sign transaction
                info!("Building Bitcoin transaction with {} UTXOs, fee: {} sats", selected_utxos.len(), estimated_fee);
                let tx_hex = match signer
                    .send(deposit_address, amount_sats, selected_utxos, estimated_fee)
                    .await
                {
                    Ok(hex) => hex,
                    Err(e) => {
                        error!("Failed to build Bitcoin transaction: {}", e);
                        anyhow::bail!("Failed to build Bitcoin transaction: {}", e);
                    }
                };

                // Broadcast transaction
                let txid = provider
                    .broadcast(&tx_hex)
                    .await
                    .context("Failed to broadcast Bitcoin transaction")?;

                info!("Bitcoin transaction broadcasted: {}", txid);
                Ok(txid)
            }

            _ => order_result
                .to
                .clone()
                .ok_or_else(|| anyhow::anyhow!("No deposit address returned")),
        }
    }

    fn rpc_url_for_chain(&self, chain: &str) -> Result<String, anyhow::Error> {
        match chain {
            "ethereum_sepolia" => Ok(self.config.rpc_urls.ethereum_sepolia.clone()),
            "base_sepolia" => Ok(self.config.rpc_urls.base_sepolia.clone()),
            "arbitrum_sepolia" => Ok(self.config.rpc_urls.arbitrum_sepolia.clone()),
            "alpen_testnet" => Ok(self.config.rpc_urls.alpen_testnet.clone()),
            "bnbchain_testnet" => Ok(self.config.rpc_urls.bnbchain_testnet.clone()),
            "citrea_testnet" => Ok(self.config.rpc_urls.citrea_testnet.clone()),
            "monad_testnet" => Ok(self.config.rpc_urls.monad_testnet.clone()),
            "xrpl_testnet" => Ok(self.config.rpc_urls.xrpl_testnet.clone()),
            _ => Err(anyhow::anyhow!(
                "No RPC URL configured for chain: {}",
                chain
            )),
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
