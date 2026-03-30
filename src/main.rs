mod api;
mod chains;
mod config;
mod db;
mod discord;
mod models;
mod scheduler;

use std::sync::Arc;

use anyhow::Result;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use api::GardenApiClient;
use config::AppConfig;
use db::Database;
use scheduler::{runner::SwapRunner, start_scheduler};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    // Load config
    let config = AppConfig::from_env().map_err(|e| {
        error!("Failed to load config: {}", e);
        e
    })?;

    info!("Garden Swap Tester starting up");
    info!("Network mode: {:?}", config.network_mode);
    info!("API base URL: {}", config.garden.api_base_url);
    info!("Scheduler cron: {}", config.scheduler.cron);
    info!("Swap timeout: {}s", config.scheduler.swap_timeout_secs);
    info!("Database: {}", config.database_url);
    info!("Balance checking: {}", if config.enable_balance_check { "ENABLED" } else { "DISABLED" });

    // Parse command from first CLI arg
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("scheduler");

    // Set up shared resources
    let api = GardenApiClient::new(config.garden.clone())?;
    let api = Arc::new(api);
    let db = Arc::new(Database::connect(&config.database_url)?);

    match mode {
        // Run once immediately (useful for CI / one-shot testing)
        "run-once" => {
            info!("Mode: run-once — executing all swaps concurrently");
            let runner = SwapRunner::new(api, db, config.clone());
            let summary = runner.run_all(config.enable_balance_check).await;

            println!("\n═══ Final Run Summary ═══");
            println!("Run ID   : {}", summary.run_id);
            println!("Total    : {}", summary.total_swaps);
            println!("Completed: {}", summary.completed);
            println!("Failed   : {}", summary.failed);
            println!("Timed Out: {}", summary.timed_out);
            println!("Pending  : {}", summary.pending);
            println!();

            for r in &summary.results {
                let marker = match r.status {
                    models::SwapStatus::Completed => "✅",
                    models::SwapStatus::Failed => "❌",
                    models::SwapStatus::TimedOut => "⏰",
                    models::SwapStatus::Refunded => "↩️ ",
                    _ => "⏳",
                };
                println!(
                    "{} {:55} | {:12} | {}",
                    marker,
                    r.swap_pair,
                    r.status,
                    r.error_message.as_deref().unwrap_or("")
                );
            }

            // Exit with non-zero if any failures
            if summary.failed > 0 || summary.timed_out > 0 {
                std::process::exit(1);
            }
        }

        // Test a single swap pair
        "test-swap" => {
            let swap_spec = args.get(2).ok_or_else(|| {
                anyhow::anyhow!("Usage: cargo run --release -- test-swap <from_asset> <to_asset> [amount]\nExample: cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc\nExample: cargo run --release -- test-swap solana_testnet:usdc arbitrum_sepolia:usdc 200000000")
            })?;
            let to_asset = args.get(3).ok_or_else(|| {
                anyhow::anyhow!("Usage: cargo run --release -- test-swap <from_asset> <to_asset> [amount]\nExample: cargo run --release -- test-swap ethereum_sepolia:wbtc base_sepolia:wbtc\nExample: cargo run --release -- test-swap solana_testnet:usdc arbitrum_sepolia:usdc 200000000")
            })?;
            
            // Optional custom amount (in smallest units)
            let custom_amount = args.get(4).map(|s| s.clone());

            if let Some(ref amt) = custom_amount {
                info!("Mode: test-swap — testing single swap {} -> {} with custom amount {}", swap_spec, to_asset, amt);
            } else {
                info!("Mode: test-swap — testing single swap {} -> {}", swap_spec, to_asset);
            }
            
            let runner = SwapRunner::new(api, db, config);
            let result = runner.test_single_swap_with_amount(swap_spec, to_asset, custom_amount).await?;

            println!("\n═══ Swap Test Result ═══");
            println!("Pair      : {}", result.swap_pair);
            println!("Status    : {}", result.status);
            println!("Order ID  : {}", result.order_id.as_deref().unwrap_or("N/A"));
            println!("Duration  : {}s", result.duration_secs.unwrap_or(0));
            
            if let Some(ref addr) = result.deposit_address {
                println!("Deposit   : {}", addr);
            }
            
            if let Some(ref src_init) = result.source_initiate_tx {
                println!("Src Init  : {}", src_init);
            }
            if let Some(ref dst_redeem) = result.dest_redeem_tx {
                println!("Dst Redeem: {}", dst_redeem);
            }
            
            if let Some(ref err) = result.error_message {
                println!("Error     : {}", err);
            }
            
            println!();

            // Exit with non-zero if failed
            if result.status == models::SwapStatus::Failed 
                || result.status == models::SwapStatus::TimedOut {
                std::process::exit(1);
            }
        }

        // Show history from DB
        "history" => {
            info!("Mode: history — showing recent runs");
            let runs = db.get_recent_runs(10)?;
            if runs.is_empty() {
                println!("No runs found in database.");
            } else {
                for run in runs {
                    println!(
                        "[{}] {} | total={} ✅={} ❌={} ⏰={}",
                        run.started_at.format("%Y-%m-%d %H:%M UTC"),
                        run.run_id,
                        run.total_swaps,
                        run.completed,
                        run.failed,
                        run.timed_out,
                    );
                }
            }
        }

        // List available swap pairs
        "list-swaps" => {
            info!("Mode: list-swaps — showing all available swap pairs");
            let pairs = chains::all_swap_pairs(&config.wallets);
            
            println!("\n═══ Available Swap Pairs ({}) ═══\n", pairs.len());
            for (idx, pair) in pairs.iter().enumerate() {
                let manual = if chains::requires_manual_deposit(&pair.source.asset) {
                    " ⚠️  DEPOSIT"
                } else {
                    ""
                };
                println!(
                    "{:2}. {} -> {}{}",
                    idx + 1,
                    pair.source.asset,
                    pair.destination.asset,
                    manual
                );
            }
            println!();
        }

        // Start Discord bot
        "discord-bot" => {
            info!("Mode: discord-bot — starting Discord bot");
            
            // Load Discord token from environment
            let token = std::env::var("DISCORD_TOKEN")
                .map_err(|_| anyhow::anyhow!("DISCORD_TOKEN environment variable not set"))?;
            
            println!("Starting Discord bot...");
            println!("Press Ctrl+C to stop.\n");
            
            discord::start_discord_bot(token).await?;
        }

        // Default: start the cron scheduler
        _ => {
            info!("Mode: scheduler — starting cron loop");
            println!("Garden Swap Tester running in scheduler mode.");
            println!("Cron: {}", config.scheduler.cron);
            println!("Press Ctrl+C to stop.\n");

            start_scheduler(config, api, db).await?;
        }
    }

    Ok(())
}
