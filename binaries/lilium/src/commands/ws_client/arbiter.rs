// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_arbiter.py
use anyhow::Result;
use lilium_database::Database;
use lilium_services::account;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::UnixListener;
use tokio::process::Child;
use tokio::sync::{Notify, RwLock};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::commands::ws_client::control::{
    self, ControlAction, ControlCommand, ControlResponse, read_message, write_message,
};
use crate::config::Config;

pub struct Arbiter {
    config: Config,
    database: Database,
    workers: Arc<RwLock<HashMap<String, WorkerHandle>>>,
    worker_spawner: Arc<dyn WorkerSpawner>,
    shutdown: Arc<tokio::sync::Notify>,
}

struct WorkerHandle {
    child: Child,
    restart_count: u32,
    last_restart: Instant,
}

#[derive(Clone)]
struct WorkerSpec {
    account: String,
    database: Database,
    notification_config: lilium_database::NotificationDatabaseConfig,
    lock_config: lilium_database::DedicatedDatabaseConfig,
    queue_size: usize,
    batch_size: usize,
    buffer_dir: PathBuf,
    runtime_dir: PathBuf,
    websocket_url: String,
    reconnect_delay_ms: u64,
}

fn backoff_delay(restart_count: u32) -> Duration {
    let base: u64 = 100;
    let max: u64 = 30_000;
    let millis = base.saturating_mul(2u64.saturating_pow(restart_count));
    Duration::from_millis(std::cmp::min(millis, max))
}

trait WorkerSpawner: Send + Sync {
    fn spawn_worker(&self, spec: WorkerSpec) -> WorkerHandle;
}

struct ProcessWorkerSpawner;

impl WorkerSpawner for ProcessWorkerSpawner {
    fn spawn_worker(&self, spec: WorkerSpec) -> WorkerHandle {
        let account = spec.account;
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command
            .arg("ws-client")
            .arg("worker")
            .arg("--account")
            .arg(&account)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let child = command.spawn().unwrap_or_else(|e| {
            panic!("failed to spawn worker process for account {account}: {e}")
        });

        WorkerHandle {
            child,
            restart_count: 0,
            last_restart: Instant::now(),
        }
    }
}

impl Arbiter {
    pub fn new(config: Config, database: Database) -> Self {
        Self::with_worker_spawner(config, database, Arc::new(ProcessWorkerSpawner))
    }

    fn with_worker_spawner(
        config: Config,
        database: Database,
        worker_spawner: Arc<dyn WorkerSpawner>,
    ) -> Self {
        Self {
            config,
            database,
            workers: Arc::new(RwLock::new(HashMap::new())),
            worker_spawner,
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Arbiter starting");

        let control_socket = control::arbiter_socket_path(&self.config.spider.runtime_dir);
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

        // Spawn the restart watcher: polls every 1s for crashed workers,
        // restarts them with exponential backoff.
        let restart_shutdown = self.shutdown.clone();
        let restart_workers = self.workers.clone();
        let restart_spawner = self.worker_spawner.clone();
        let restart_config = self.config.clone();
        let restart_db = self.database.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = restart_shutdown.notified() => break,
                    _ = sleep(Duration::from_secs(1)) => {
                        let mut workers = restart_workers.write().await;
                        let mut to_restart = Vec::new();
                        for (account, handle) in workers.iter_mut() {
                            match handle.child.try_wait() {
                                Ok(Some(status)) if !status.success() => {
                                    warn!(account = %account, status = ?status, "worker exited; will restart");
                                    to_restart.push((account.clone(), false));
                                }
                                Ok(Some(status)) if status.success() => {
                                    info!(account = %account, "worker exited cleanly; removing");
                                    to_restart.push((account.clone(), true));
                                }
                                _ => {}
                            }
                        }
                        for (account, clean_exit) in to_restart {
                            if clean_exit {
                                workers.remove(&account);
                                continue;
                            }
                            if let Some(mut handle) = workers.remove(&account) {
                                let _ = handle.child.start_kill();
                                let delay = backoff_delay(handle.restart_count);
                                let new_handle = restart_spawner.spawn_worker(WorkerSpec {
                                    account: account.clone(),
                                    database: restart_db.clone(),
                                    notification_config: restart_config.notification.clone().into(),
                                    lock_config: restart_config.database.clone().into(),
                                    queue_size: restart_config.spider.queue_size,
                                    batch_size: restart_config.spider.batch_size,
                                    buffer_dir: restart_config.spider.buffer_dir.clone(),
                                    runtime_dir: restart_config.spider.runtime_dir.clone(),
                                    websocket_url: restart_config.spider.websocket_url.clone(),
                                    reconnect_delay_ms: restart_config.spider.reconnect_delay_ms,
                                });
                                workers.insert(account, WorkerHandle {
                                    child: new_handle.child,
                                    restart_count: handle.restart_count + 1,
                                    last_restart: Instant::now(),
                                });
                                sleep(delay).await;
                            }
                        }
                    }
                }
            }
        });

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
        lilium_database::transaction!(self.database, |session| {
            let accounts = account::list_accounts(session, true).await?;
            Ok(accounts
                .into_iter()
                .map(|account| account.user_id)
                .collect())
        })
        .await
    }

    fn worker_spec(&self, account: String) -> WorkerSpec {
        WorkerSpec {
            account,
            database: self.database.clone(),
            notification_config: self.config.notification.clone().into(),
            lock_config: self.config.database.clone().into(),
            queue_size: self.config.spider.queue_size,
            batch_size: self.config.spider.batch_size,
            buffer_dir: self.config.spider.buffer_dir.clone(),
            runtime_dir: self.config.spider.runtime_dir.clone(),
            websocket_url: self.config.spider.websocket_url.clone(),
            reconnect_delay_ms: self.config.spider.reconnect_delay_ms,
        }
    }

    pub async fn start_worker(&self, account_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        if workers.contains_key(account_id) {
            warn!(account = account_id, "Worker already running");
            return Ok(());
        }

        let handle = self
            .worker_spawner
            .spawn_worker(self.worker_spec(account_id.to_string()));

        workers.insert(account_id.to_string(), handle);
        Ok(())
    }

    const WORKER_STOP_TIMEOUT_SECS: u64 = 10;

    pub async fn stop_worker(&self, account_id: &str) -> Result<()> {
        let mut workers = self.workers.write().await;
        let Some(mut handle) = workers.remove(account_id) else {
            warn!(account = account_id, "Worker not running");
            return Ok(());
        };

        let socket_path = control::worker_socket_path(&self.config.spider.runtime_dir, account_id);

        let graceful = async {
            let command = serde_json::json!({
                "action": "stop",
                "account_user_id": account_id,
            })
            .to_string();
            match tokio::net::UnixStream::connect(&socket_path).await {
                Ok(mut stream) => {
                    let _ = control::write_message(&mut stream, &command).await;
                    let _ = tokio::time::timeout(
                        Duration::from_secs(Self::WORKER_STOP_TIMEOUT_SECS),
                        control::read_message(&mut stream),
                    )
                    .await;
                }
                Err(e) => {
                    warn!(account = account_id, error = %e, "worker control socket unavailable");
                }
            }
        };

        graceful.await;

        match tokio::time::timeout(
            Duration::from_secs(Self::WORKER_STOP_TIMEOUT_SECS),
            handle.child.wait(),
        )
        .await
        {
            Ok(Ok(_)) => {
                info!(account = account_id, "worker stopped gracefully");
            }
            _ => {
                warn!(
                    account = account_id,
                    "worker did not stop gracefully; sending SIGTERM"
                );
                let _ = handle.child.start_kill();
                match tokio::time::timeout(Duration::from_secs(5), handle.child.wait()).await {
                    Ok(Ok(_)) => info!(account = account_id, "worker killed"),
                    _ => warn!(account = account_id, "worker may be a zombie"),
                }
            }
        }

        Ok(())
    }

    async fn stop_all_workers(&self) {
        let workers: HashMap<String, WorkerHandle> = {
            let mut w = self.workers.write().await;
            std::mem::take(&mut *w)
        };

        for (account_id, mut handle) in workers {
            let socket_path =
                control::worker_socket_path(&self.config.spider.runtime_dir, &account_id);

            let graceful = async {
                let command = serde_json::json!({
                    "action": "stop",
                    "account_user_id": account_id,
                })
                .to_string();
                match tokio::net::UnixStream::connect(&socket_path).await {
                    Ok(mut stream) => {
                        let _ = control::write_message(&mut stream, &command).await;
                        let _ = tokio::time::timeout(
                            Duration::from_secs(Self::WORKER_STOP_TIMEOUT_SECS),
                            control::read_message(&mut stream),
                        )
                        .await;
                    }
                    Err(e) => {
                        warn!(account = account_id, error = %e, "worker control socket unavailable");
                    }
                }
            };

            graceful.await;

            match tokio::time::timeout(
                Duration::from_secs(Self::WORKER_STOP_TIMEOUT_SECS),
                handle.child.wait(),
            )
            .await
            {
                Ok(Ok(_)) => {
                    info!(account = %account_id, "worker stopped gracefully");
                }
                _ => {
                    warn!(account = %account_id, "worker did not stop gracefully; sending SIGTERM");
                    let _ = handle.child.start_kill();
                    match tokio::time::timeout(Duration::from_secs(5), handle.child.wait()).await {
                        Ok(Ok(_)) => info!(account = %account_id, "worker killed"),
                        _ => warn!(account = %account_id, "worker may be a zombie"),
                    }
                }
            }
        }
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
            database: self.database.clone(),
            workers: self.workers.clone(),
            worker_spawner: self.worker_spawner.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mockall::mock! {
        WorkerSpawner {}

        impl WorkerSpawner for WorkerSpawner {
            fn spawn_worker(&self, spec: WorkerSpec) -> WorkerHandle;
        }
    }

    #[tokio::test]
    async fn test_arbiter_start_stop() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Empty)
                .await
                .expect("acquire test db");

        let config = Config {
            database: crate::config::DatabaseConfig {
                url: "postgres://localhost/lilium_test".to_string(),
                max_connections: 1,
            },
            notification: crate::config::NotificationConfig {
                url: "postgres://localhost/lilium_test_notify".to_string(),
            },
            spider: crate::config::SpiderConfig {
                queue_size: 100,
                batch_size: 10,
                buffer_dir: PathBuf::from("data/event/buffer"),
                runtime_dir: PathBuf::from("runtime/spider"),
                websocket_url: lilium_api_client::config::DZMM_SOCKETIO_URL.to_string(),
                reconnect_delay_ms: 5_000,
            },
            processor: crate::config::ProcessorConfig {
                polling_interval_secs: 5,
                batch_size: 100,
            },
            cli: crate::config::CliConfig {
                data_path: "./data".to_string(),
            },
        };

        let mut worker_spawner = MockWorkerSpawner::new();
        worker_spawner
            .expect_spawn_worker()
            .withf(|spec| spec.account == "test_user")
            .times(1)
            .returning(|_| WorkerHandle {
                child: tokio::process::Command::new("true")
                    .spawn()
                    .expect("spawn true"),
                restart_count: 0,
                last_restart: Instant::now(),
            });

        let arbiter = Arbiter::with_worker_spawner(
            config,
            test_db.database().clone(),
            Arc::new(worker_spawner),
        );
        arbiter.start_worker("test_user").await.unwrap();
        assert!(arbiter.workers.read().await.contains_key("test_user"));

        let _ = arbiter.stop_worker("test_user").await;
        assert!(!arbiter.workers.read().await.contains_key("test_user"));
    }
}
