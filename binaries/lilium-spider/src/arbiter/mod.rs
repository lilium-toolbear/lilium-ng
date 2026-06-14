use anyhow::Result;
use lilium_database::DbPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::{Notify, RwLock};
use tracing::{error, info, warn};

#[cfg(test)]
use std::path::PathBuf;

use crate::config::Config;
use crate::control::{
    self, read_message, write_message, ControlAction, ControlCommand, ControlResponse,
};

pub struct Arbiter {
    config: Config,
    pool: DbPool,
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
    shutdown: Arc<tokio::sync::Notify>,
}

struct WorkerHandle {
    shutdown: Arc<Notify>,
}

impl Arbiter {
    pub fn new(config: Config, pool: DbPool) -> Self {
        Self {
            config,
            pool,
            workers: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Arbiter starting");

        let control_socket = control::arbiter_socket_path(&self.config.worker.runtime_dir);
        let (listener, socket_identity) = control::bind_unix_control_socket(&control_socket)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let control_shutdown = self.shutdown.clone();
        let mut control_task = Box::pin({
            let this = self.clone_state();
            async move {
                if let Err(e) = this.run_control_server(listener, control_shutdown).await {
                    error!(error = %e, "Control server exited with error");
                }
            }
        });

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

        tokio::select! {
            _ = self.shutdown.notified() => {
                info!("Shutting down arbiter");
            }
            _ = &mut control_task => {
                warn!("Control server exited; shutting down arbiter");
                self.shutdown.notify_waiters();
            }
        }

        self.stop_all_workers().await;
        control::unlink_bound_unix_socket(&control_socket, socket_identity);
        Ok(())
    }

    async fn load_enabled_accounts(&self) -> Result<Vec<String>> {
        self.pool
            .with_session_boxed(|session| {
                Box::pin(async move {
                    sqlx::query_scalar::<_, String>(
                        "SELECT user_id FROM accounts WHERE is_enabled = true ORDER BY user_id",
                    )
                    .fetch_all(session.as_mut())
                    .await
                    .map_err(Into::into)
                })
            })
            .await
    }

    pub async fn start_worker(&self, account_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        if workers.contains_key(account_id) {
            warn!(account = account_id, "Worker already running");
            return Ok(());
        }

        let account = account_id.to_string();
        let shutdown = Arc::new(Notify::new());
        let worker_shutdown = shutdown.clone();
        let pool = self.pool.clone();
        let websocket_url = self.config.worker.websocket_url.clone();
        let reconnect_delay_ms = self.config.worker.reconnect_delay_ms;
        let queue_size = self.config.worker.queue_size;
        let batch_size = self.config.worker.batch_size;
        let buffer_dir = self.config.worker.buffer_dir.clone();

        #[cfg(test)]
        tokio::spawn(async move {
            let _ = (
                pool,
                queue_size,
                batch_size,
                buffer_dir,
                websocket_url,
                reconnect_delay_ms,
                worker_shutdown,
            );
            info!(account = %account, "Worker starting (test noop)");
        });

        #[cfg(not(test))]
        tokio::spawn(async move {
            info!(account = %account, "Worker starting");
            let worker = super::worker::Worker::new(
                account.clone(),
                pool,
                queue_size,
                batch_size,
                buffer_dir,
                websocket_url,
                reconnect_delay_ms,
            );
            if let Err(e) = worker.run(worker_shutdown).await {
                error!(account = %account, error = %e, "Worker failed");
            } else {
                info!(account = %account, "Worker exited");
            }
        });

        workers.insert(account_id.to_string(), WorkerHandle { shutdown });
        Ok(())
    }

    pub async fn stop_worker(&self, account_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        if let Some(handle) = workers.remove(account_id) {
            handle.shutdown.notify_waiters();
            info!(account = account_id, "Worker stop requested");
        } else {
            warn!(account = account_id, "Worker not running");
        }
        Ok(())
    }

    async fn stop_all_workers(&self) {
        let mut workers = self.workers.write().await;
        for (account_id, handle) in workers.drain() {
            info!(account = %account_id, "Stopping worker");
            handle.shutdown.notify_waiters();
        }
        info!("All workers stopped");
    }

    async fn run_control_server(self, listener: UnixListener, shutdown: Arc<Notify>) -> Result<()> {
        info!("Control server starting");
        loop {
            tokio::select! {
                _ = shutdown.notified() => {
                    info!("Control server shutting down");
                    break;
                }
                accept = listener.accept() => {
                    match accept {
                        Ok((mut socket, _addr)) => {
                            let response = match read_message(&mut socket).await {
                                Ok(raw) => match ControlCommand::from_json(&raw) {
                                    Ok(command) => self.handle_control_command(command).await,
                                    Err(e) => ControlResponse::error(e),
                                },
                                Err(e) => ControlResponse::error(format!("failed to read control command: {e}")),
                            };
                            let _ = write_message(&mut socket, &response.to_json()).await;
                        }
                        Err(e) => {
                            error!(error = %e, "Control socket accept failed");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_control_command(&self, command: ControlCommand) -> ControlResponse {
        match command.action {
            ControlAction::Status => {
                let workers = self.workers.read().await;
                let accounts: Vec<String> = workers.keys().cloned().collect();
                ControlResponse::success("arbiter status").with_data(serde_json::json!({
                    "worker_count": accounts.len(),
                    "worker_ids": accounts,
                }))
            }
            ControlAction::Rescan => match self.rescan_workers().await {
                Ok(result) => ControlResponse::success("rescanned workers").with_data(result),
                Err(e) => ControlResponse::error(format!("{e}")),
            },
            ControlAction::Stop => {
                let Some(account_user_id) = command.account_user_id.as_deref() else {
                    return ControlResponse::error("account_user_id is required for this action");
                };
                match self.stop_worker(account_user_id).await {
                    Ok(()) => ControlResponse::success("worker stopped"),
                    Err(e) => ControlResponse::error(format!("{e}")),
                }
            }
            ControlAction::Start => {
                let Some(account_user_id) = command.account_user_id.as_deref() else {
                    return ControlResponse::error("account_user_id is required for this action");
                };
                match self.start_worker(account_user_id).await {
                    Ok(()) => ControlResponse::success("worker started"),
                    Err(e) => ControlResponse::error(format!("{e}")),
                }
            }
            ControlAction::Reconnect | ControlAction::Reload | ControlAction::Restart => {
                let Some(account_user_id) = command.account_user_id.as_deref() else {
                    return ControlResponse::error("account_user_id is required for this action");
                };
                if let Err(e) = self.stop_worker(account_user_id).await {
                    return ControlResponse::error(format!("{e}"));
                }
                match self.start_worker(account_user_id).await {
                    Ok(()) => ControlResponse::success("worker restarted"),
                    Err(e) => ControlResponse::error(format!("{e}")),
                }
            }
        }
    }

    async fn rescan_workers(&self) -> Result<serde_json::Value> {
        let enabled_accounts = self.load_enabled_accounts().await?;
        let enabled_set: std::collections::HashSet<String> =
            enabled_accounts.iter().cloned().collect();
        let current_accounts: Vec<String> = {
            let workers = self.workers.read().await;
            workers.keys().cloned().collect()
        };

        for account_id in &enabled_accounts {
            let should_start = {
                let workers = self.workers.read().await;
                !workers.contains_key(account_id)
            };
            if should_start {
                self.start_worker(account_id).await?;
            }
        }

        for account_id in current_accounts {
            if !enabled_set.contains(&account_id) {
                self.stop_worker(&account_id).await?;
            }
        }

        Ok(serde_json::json!({
            "enabled_accounts": enabled_accounts,
        }))
    }

    fn clone_state(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
            workers: self.workers.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_arbiter_start_stop() {
        let pool = lilium_test_fixtures::connect_test_db().await;

        let config = Config {
            database: crate::config::DatabaseConfig { pool_size: 1 },
            worker: crate::config::WorkerConfig {
                queue_size: 100,
                batch_size: 10,
                buffer_dir: PathBuf::from("data/event/buffer"),
                runtime_dir: PathBuf::from("runtime/spider"),
                websocket_url: "wss://dzmm.com/ws".to_string(),
                reconnect_delay_ms: 5_000,
            },
        };

        let arbiter = Arbiter::new(config, pool.inner().clone());
        arbiter.start_worker("test_user").await.unwrap();
        assert!(arbiter.workers.read().await.contains_key("test_user"));

        let _ = arbiter.stop_worker("test_user").await;
        assert!(!arbiter.workers.read().await.contains_key("test_user"));
    }
}
