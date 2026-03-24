use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::sync::Mutex;
use tracing::info;

use crate::models::{RunSummary, SwapRecord};

pub struct Database {
    conn: Mutex<Connection>,
}

// rusqlite::Connection is Send, so Database can be safely shared across threads
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    pub fn connect(path: &str) -> Result<Self> {
        info!("Opening database: {}", path);
        let conn = Connection::open(path).context("Failed to open SQLite database")?;
        
        // Enable WAL mode for better concurrent access
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;"
        ).context("Failed to set WAL mode")?;
        
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS swap_records (
                id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id              TEXT NOT NULL,
                swap_pair           TEXT NOT NULL,
                from_chain          TEXT NOT NULL,
                to_chain            TEXT NOT NULL,
                from_asset          TEXT NOT NULL,
                to_asset            TEXT NOT NULL,
                from_amount         TEXT NOT NULL,
                order_id            TEXT,
                deposit_address     TEXT,
                status              TEXT NOT NULL DEFAULT 'Initiated',
                error_message       TEXT,
                source_initiate_tx  TEXT,
                source_redeem_tx    TEXT,
                dest_initiate_tx    TEXT,
                dest_redeem_tx      TEXT,
                started_at          TEXT NOT NULL,
                completed_at        TEXT,
                duration_secs       INTEGER
            );

            CREATE TABLE IF NOT EXISTS run_summaries (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id      TEXT NOT NULL UNIQUE,
                total_swaps INTEGER NOT NULL,
                completed   INTEGER NOT NULL DEFAULT 0,
                failed      INTEGER NOT NULL DEFAULT 0,
                timed_out   INTEGER NOT NULL DEFAULT 0,
                pending     INTEGER NOT NULL DEFAULT 0,
                started_at  TEXT NOT NULL
            );
            "#,
        )
        .context("Migration failed")?;
        info!("Database migration complete");
        Ok(())
    }

    pub fn insert_swap_record(&self, record: &SwapRecord) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO swap_records
               (run_id, swap_pair, from_chain, to_chain, from_asset, to_asset, from_amount,
                order_id, deposit_address, status, error_message,
                source_initiate_tx, source_redeem_tx, dest_initiate_tx, dest_redeem_tx,
                started_at, completed_at, duration_secs)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)"#,
            params![
                record.run_id,
                record.swap_pair,
                record.from_chain,
                record.to_chain,
                record.from_asset,
                record.to_asset,
                record.from_amount,
                record.order_id,
                record.deposit_address,
                record.status.to_string(),
                record.error_message,
                record.source_initiate_tx,
                record.source_redeem_tx,
                record.dest_initiate_tx,
                record.dest_redeem_tx,
                record.started_at.to_rfc3339(),
                record.completed_at.map(|t: DateTime<Utc>| t.to_rfc3339()),
                record.duration_secs,
            ],
        )
        .context("Insert swap record failed")?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_swap_record(&self, record: &SwapRecord) -> Result<()> {
        // Guard against None id
        let id = match record.id {
            Some(id) => id,
            None => anyhow::bail!("Cannot update a record without an id"),
        };
        
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"UPDATE swap_records SET
               order_id=?1, deposit_address=?2, status=?3, error_message=?4,
               source_initiate_tx=?5, source_redeem_tx=?6,
               dest_initiate_tx=?7, dest_redeem_tx=?8,
               completed_at=?9, duration_secs=?10
               WHERE id=?11"#,
            params![
                record.order_id,
                record.deposit_address,
                record.status.to_string(),
                record.error_message,
                record.source_initiate_tx,
                record.source_redeem_tx,
                record.dest_initiate_tx,
                record.dest_redeem_tx,
                record.completed_at.map(|t: DateTime<Utc>| t.to_rfc3339()),
                record.duration_secs,
                id,
            ],
        )
        .context("Update swap record failed")?;
        Ok(())
    }

    pub fn insert_run_summary(&self, summary: &RunSummary) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT OR REPLACE INTO run_summaries
               (run_id, total_swaps, completed, failed, timed_out, pending, started_at)
               VALUES (?1,?2,?3,?4,?5,?6,?7)"#,
            params![
                summary.run_id,
                summary.total_swaps as i64,
                summary.completed as i64,
                summary.failed as i64,
                summary.timed_out as i64,
                summary.pending as i64,
                summary.started_at.to_rfc3339(),
            ],
        )
        .context("Insert run summary failed")?;
        Ok(())
    }

    pub fn get_recent_runs(&self, limit: i64) -> Result<Vec<RunSummary>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT run_id, total_swaps, completed, failed, timed_out, pending, started_at
             FROM run_summaries ORDER BY started_at DESC LIMIT ?1",
            )
            .context("Prepare get_recent_runs failed")?;

        let rows = stmt
            .query_map(params![limit], |row| {
                let started_at_str: String = row.get(6)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    started_at_str,
                ))
            })
            .context("Query recent runs failed")?;

        let mut summaries = Vec::new();
        for row in rows {
            let (run_id, total, completed, failed, timed_out, pending, started_at_str) =
                row.context("Row read failed")?;
            summaries.push(RunSummary {
                run_id,
                total_swaps: total as usize,
                completed: completed as usize,
                failed: failed as usize,
                timed_out: timed_out as usize,
                pending: pending as usize,
                started_at: started_at_str
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                results: vec![],
            });
        }
        Ok(summaries)
    }
}
