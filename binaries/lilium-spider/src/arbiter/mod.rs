use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, warn, error};

use crate::config::Config;

pub struct Arbiter {
    #[allow(dead_code)]
    config: Config,
    pool: PgPool,
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
    shutdown: Arc<tokio::sync::Notify>,
}

struct WorkerHandle {
    _tx: tokio::sync::mpsc::Sender<()>,
}

impl Arbiter {
    pub fn new(config: Config, pool: PgPool) -> Self {
        Self {
            config,
            pool,
            workers: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Arbiter starting");

        let accounts = self.load_enabled_accounts().await?;
        info!(count = accounts.len(), "Loaded enabled accounts");

        for account_id in &accounts {
            self.start_worker(account_id).await?;
        }

        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            shutdown.notify_waiters();
        });

        self.shutdown.notified().await;
        info!("Shutting down arbiter");
        self.stop_all_workers().await;
        Ok(())
    }

    async fn load_enabled_accounts(&self) -> Result<Vec<String>> {
        let ids = sqlx::query_scalar::<_, String>(
            "SELECT user_id FROM accounts WHERE is_enabled = true ORDER BY user_id",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(ids)
    }

    pub async fn start_worker(&self, account_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        if workers.contains_key(account_id) {
            warn!(account = account_id, "Worker already running");
            return Ok(());
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let pool = self.pool.clone();
        let account = account_id.to_string();

        tokio::spawn(async move {
            info!(account = %account, "Worker starting");
            let worker = super::worker::Worker::new(account.clone(), pool);
            tokio::select! {
                result = worker.run() => {
                    if let Err(e) = result {
                        error!(account = %account, error = %e, "Worker failed");
                    }
                }
                _ = rx.recv() => {
                    info!(account = %account, "Worker shutting down");
                }
            }
        });

        workers.insert(account_id.to_string(), WorkerHandle { _tx: tx });
        Ok(())
    }

    pub async fn stop_worker(&self, account_id: &str) {
        let mut workers = self.workers.write().await;
        workers.remove(account_id);
        info!(account = account_id, "Worker stopped");
    }

    async fn stop_all_workers(&self) {
        let mut workers = self.workers.write().await;
        workers.clear();
        info!("All workers stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_arbiter_start_stop() {
        let config = Config {
            database: crate::config::DatabaseConfig {
                url: "postgres://localhost/test".to_string(),
                pool_size: 1,
            },
            worker: crate::config::WorkerConfig {
                queue_size: 100,
                batch_size: 10,
            },
            processor: crate::config::ProcessorConfig {
                polling_interval_secs: 1,
                batch_size: 10,
            },
        };
        let pool = PgPool::connect_lazy("postgres://localhost/test").unwrap();
        let arbiter = Arbiter::new(config, pool);

        arbiter.start_worker("test_user").await.unwrap();
        assert!(arbiter.workers.read().await.contains_key("test_user"));

        arbiter.stop_worker("test_user").await;
        assert!(!arbiter.workers.read().await.contains_key("test_user"));
    }
}
