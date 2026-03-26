pub mod runner;

use std::sync::Arc;

use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;

use crate::api::GardenApiClient;
use crate::config::AppConfig;
use crate::db::Database;
use crate::models::RunSummary;
use runner::SwapRunner;

pub async fn start_scheduler(
    config: AppConfig,
    api: Arc<GardenApiClient>,
    db: Arc<Database>,
) -> Result<()> {
    let cron = config.scheduler.cron.clone();
    info!("Starting scheduler with cron: {}", cron);

    let sched = JobScheduler::new().await?;

    let config_c = config.clone();
    let api_c = Arc::clone(&api);
    let db_c = Arc::clone(&db);

    let job = Job::new_async(cron.as_str(), move |_uuid, _lock| {
        let config = config_c.clone();
        let api = Arc::clone(&api_c);
        let db = Arc::clone(&db_c);
        Box::pin(async move {
            info!("Cron fired — starting swap test run");
            let runner = SwapRunner::new(api, db, config.clone());
            let summary = runner.run_all(config.enable_balance_check).await;
            print_summary(&summary);
        })
    })?;

    sched.add(job).await?;
    sched.start().await?;

    info!("Scheduler running — waiting for next trigger...");
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

fn print_summary(s: &RunSummary) {
    info!("┌──────────────────────────────────────────────────────┐");
    info!("│              SWAP TEST RUN SUMMARY                   │");
    info!("├──────────────────────────────────────────────────────┤");
    info!("│ Run ID  : {:<43} │", &s.run_id[..s.run_id.len().min(43)]);
    info!("│ Total   : {:<43} │", s.total_swaps);
    info!("│ OK      : {:<43} │", s.completed);
    info!("│ Failed  : {:<43} │", s.failed);
    info!("│ Timeout : {:<43} │", s.timed_out);
    info!("│ Pending : {:<43} │", s.pending);
    info!("├────────────────────────────────────┬─────────────────┤");
    info!("│ Swap Pair                          │ Status          │");
    info!("├────────────────────────────────────┼─────────────────┤");
    for r in &s.results {
        let pair = if r.swap_pair.len() > 36 {
            &r.swap_pair[..36]
        } else {
            &r.swap_pair
        };
        info!("│ {:34} │ {:15} │", pair, r.status);
    }
    info!("└────────────────────────────────────┴─────────────────┘");
}
