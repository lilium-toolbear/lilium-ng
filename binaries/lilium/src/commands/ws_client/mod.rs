pub mod arbiter;
pub mod control;
pub mod ingestion;
pub mod worker;

use anyhow::Result;
use lilium_database::Database;
use std::sync::Arc;

use crate::config::Config;

pub async fn run(config: Config, db: Database) -> Result<()> {
    let arbiter = arbiter::Arbiter::new(config, db);
    arbiter.run().await
}

/// Run a single account worker directly (used by the `ws-client --account` subcommand).
///
/// Creates a `Worker` with the given account ID and runs it with a fresh
/// shutdown signal. This is the entry point spawned by the arbiter process
/// when it forks a worker for an individual account.
pub async fn run_worker(account: String, config: Config, db: Database) -> Result<()> {
    let worker = worker::Worker::new(account, db, build_worker_runtime(&config));
    let shutdown = Arc::new(tokio::sync::Notify::new());
    worker.run(shutdown).await
}

pub fn build_worker_runtime(config: &Config) -> worker::WorkerRuntimeConfig {
    worker::WorkerRuntimeConfig {
        notification_config: config.notification.clone().into(),
        lock_config: config.database.clone().into(),
        queue_size: config.spider.queue_size,
        batch_size: config.spider.batch_size,
        buffer_dir: config.spider.buffer_dir.clone(),
        runtime_dir: config.spider.runtime_dir.clone(),
        websocket_url: config.spider.websocket_url.clone(),
        reconnect_delay_ms: config.spider.reconnect_delay_ms,
    }
}
