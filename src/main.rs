mod api;
mod chains;
mod config;
mod db;
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
                .unwrap_or_else(|_| EnvFilter::new("garden_swap_tester=info,warn")),
        )
        .with_target(false)
        .init();

    // Load config
    let config = AppConfig::from_env().map_err(|e| {
        error!("Failed to load config: {}", e);
        e
    })?;

    info!("Garden Swap Tester starting up");
    info!("API base URL: {}", config.garden.api_base_url);
    info!("Scheduler cron: {}", config.scheduler.cron);
    info!("Swap timeout: {}s", config.scheduler.swap_timeout_secs);
    info!("Database: {}", config.database_url);

    // Parse command from first CLI arg
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "scheduler".to_string());

    // Set up shared resources
    let api = Arc::new(GardenApiClient::new(config.garden.clone())?);
    let db = Arc::new(Database::connect(&config.database_url)?);

    match mode.as_str() {
        // Run once immediately (useful for CI / one-shot testing)
        "run-once" => {
            info!("Mode: run-once — executing all swaps immediately");
            let runner = SwapRunner::new(api, db, config);
            let summary = runner.run_all().await;

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
