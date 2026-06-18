// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_runtime.py spider/ws_worker.py
use anyhow::{Context, Result};
use lilium_api_client::http::{CookieRefreshCallback, DzmmApi, DzmmApiAuth};
use lilium_api_client::websocket::{SocketCommandError, SocketCommandExecutor, WsClient};
use lilium_database::{
    Database, DedicatedDatabaseConfig, DedicatedDbConnection, NotificationConnection,
    NotificationDatabaseConfig,
};
use lilium_models::dzmm::outgoing_command::{self as outgoing_commands, status};
use lilium_services::{account, outgoing_command, websocket_connection};
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::UnixListener;
use tokio::sync::Notify;
use tracing::{error, info, instrument, warn};

use crate::commands::ws_client::control::{
    self, ControlAction, ControlCommand, ControlResponse, read_message, write_message,
};
use crate::commands::ws_client::ingestion::{DiskSpillBuffer, EventIngestor, EventWriter};

pub struct Worker {
    account_id: String,
    database: Database,
    runtime: WorkerRuntimeConfig,
}

pub struct WorkerRuntimeConfig {
    pub notification_config: NotificationDatabaseConfig,
    pub lock_config: DedicatedDatabaseConfig,
    pub queue_size: usize,
    pub batch_size: usize,
    pub buffer_dir: std::path::PathBuf,
    pub runtime_dir: std::path::PathBuf,
    pub websocket_url: String,
    pub reconnect_delay_ms: u64,
}

struct WebsocketRunContext {
    account_id: String,
    database: Database,
    websocket_url: String,
    reconnect_delay_ms: u64,
    ingestor: Arc<EventIngestor>,
    stop_event: Arc<AtomicBool>,
    shutdown: Arc<Notify>,
    reconnect_notify: Arc<Notify>,
    socket_executor: SocketCommandExecutor,
}

struct WorkerControlContext {
    account_id: String,
    ingestor: Arc<EventIngestor>,
    writer: Arc<EventWriter>,
    socket_executor: SocketCommandExecutor,
    reconnect_notify: Arc<Notify>,
    stop_event: Arc<AtomicBool>,
    shutdown: Arc<Notify>,
}

impl Worker {
    pub fn new(account_id: String, database: Database, runtime: WorkerRuntimeConfig) -> Self {
        Self {
            account_id,
            database,
            runtime,
        }
    }
}

impl Worker {
    pub async fn run(&self, shutdown: Arc<Notify>) -> Result<()> {
        info!(account = %self.account_id, "Worker starting");

        // Create disk spill buffer
        let buffer_path = self
            .runtime
            .buffer_dir
            .join(format!("ws_buffer_{}.jsonl", self.account_id));
        let spill = DiskSpillBuffer::new(buffer_path);

        // Create event ingestor
        let (ingestor, rx) =
            EventIngestor::new(self.account_id.clone(), self.runtime.queue_size, spill);
        let ingestor = Arc::new(ingestor);

        // Create event writer
        let writer = Arc::new(EventWriter::new(
            self.database.clone(),
            ingestor.clone(),
            rx,
            self.runtime.batch_size,
        ));

        let stop_event = Arc::new(AtomicBool::new(false));
        let writer_stop = stop_event.clone();
        let writer_for_run = writer.clone();
        let writer_fut = async move { writer_for_run.run(&writer_stop).await };

        let reconnect_notify = Arc::new(Notify::new());
        let socket_executor = SocketCommandExecutor::new();
        let mut lock_connection = DedicatedDbConnection::connect(self.runtime.lock_config.clone())
            .await
            .context("connect websocket advisory-lock session")?;
        let lock_id = websocket_connection::acquire_dedicated_connection_lock(
            &mut lock_connection,
            &self.account_id,
        )
        .await
        .context("acquire websocket advisory lock")?;
        let lock_connection = Arc::new(tokio::sync::Mutex::new(lock_connection));
        let ws_fut = Self::run_websocket(WebsocketRunContext {
            account_id: self.account_id.clone(),
            database: self.database.clone(),
            websocket_url: self.runtime.websocket_url.clone(),
            reconnect_delay_ms: self.runtime.reconnect_delay_ms,
            ingestor: ingestor.clone(),
            stop_event: stop_event.clone(),
            shutdown: shutdown.clone(),
            reconnect_notify: reconnect_notify.clone(),
            socket_executor: socket_executor.clone(),
        });

        let command_fut = Self::run_outgoing_command_listener(
            self.account_id.clone(),
            self.database.clone(),
            self.runtime.notification_config.clone(),
            socket_executor.clone(),
            reconnect_notify.clone(),
            stop_event.clone(),
            shutdown.clone(),
        );

        let control_socket =
            control::worker_socket_path(&self.runtime.runtime_dir, &self.account_id);
        let (control_listener, control_socket_identity) = match control::bind_unix_control_socket(
            &control_socket,
        )
        .await
        {
            Ok(bound) => bound,
            Err(error) => {
                let mut connection = lock_connection.lock().await;
                if let Err(release_error) = websocket_connection::release_dedicated_connection_lock(
                    &mut connection,
                    lock_id,
                )
                .await
                {
                    warn!(
                        account = %self.account_id,
                        lock_id,
                        error = %release_error,
                        "Failed to release websocket advisory lock after control socket bind failure"
                    );
                }
                return Err(error).context("bind worker control socket");
            }
        };
        let control_fut = Self::run_control_server(
            control_listener,
            WorkerControlContext {
                account_id: self.account_id.clone(),
                ingestor: ingestor.clone(),
                writer: writer.clone(),
                socket_executor: socket_executor.clone(),
                reconnect_notify: reconnect_notify.clone(),
                stop_event: stop_event.clone(),
                shutdown: shutdown.clone(),
            },
        );

        let heartbeat_fut = Self::run_heartbeat_loop(
            self.account_id.clone(),
            lock_id,
            lock_connection.clone(),
            socket_executor.clone(),
            stop_event.clone(),
            shutdown.clone(),
        );

        tokio::select! {
            _ = shutdown.notified() => {
                info!(account = %self.account_id, "Shutdown requested");
                ingestor.stop_accepting();
                stop_event.store(true, Ordering::Relaxed);
            }
            ws_result = ws_fut => {
                ingestor.stop_accepting();
                stop_event.store(true, Ordering::Relaxed);
                match ws_result {
                    Ok(()) => info!(account = %self.account_id, "WebSocket task completed"),
                    Err(e) => {
                        let error_chain = format_error_chain(&e);
                        error!(
                            account = %self.account_id,
                            error = %e,
                            error_chain = %error_chain,
                            "WebSocket task failed"
                        );
                    }
                }
            }
            _ = writer_fut => {
                ingestor.stop_accepting();
                stop_event.store(true, Ordering::Relaxed);
                info!(account = %self.account_id, "Writer task completed");
            }
            command_result = command_fut => {
                ingestor.stop_accepting();
                stop_event.store(true, Ordering::Relaxed);
                match command_result {
                    Ok(()) => info!(account = %self.account_id, "Outgoing command listener completed"),
                    Err(e) => error!(account = %self.account_id, error = %e, "Outgoing command listener failed"),
                }
            }
            control_result = control_fut => {
                ingestor.stop_accepting();
                stop_event.store(true, Ordering::Relaxed);
                match control_result {
                    Ok(()) => info!(account = %self.account_id, "Worker control server completed"),
                    Err(e) => error!(account = %self.account_id, error = %e, "Worker control server failed"),
                }
            }
            heartbeat_result = heartbeat_fut => {
                ingestor.stop_accepting();
                stop_event.store(true, Ordering::Relaxed);
                match heartbeat_result {
                    Ok(()) => info!(account = %self.account_id, "WebSocket heartbeat loop completed"),
                    Err(e) => error!(account = %self.account_id, error = %e, "WebSocket heartbeat loop failed"),
                }
            }
        }

        stop_event.store(true, Ordering::Relaxed);
        control::unlink_bound_unix_socket(&control_socket, control_socket_identity);
        let mut connection = lock_connection.lock().await;
        if let Err(error) =
            websocket_connection::release_dedicated_connection_lock(&mut connection, lock_id).await
        {
            warn!(
                account = %self.account_id,
                lock_id,
                error = %error,
                "Failed to release websocket advisory lock"
            );
        }
        info!(account = %self.account_id, "Worker shutting down");
        Ok(())
    }

    async fn build_auth_client(database: &Database, account_id: &str) -> Result<DzmmApi> {
        let account_id = account_id.to_string();
        let lookup_account_id = account_id.clone();
        let account = lilium_database::transaction!(database, |session| {
            let account = account::get_account(session, &lookup_account_id).await?;
            Ok(account)
        })
        .await?
        .with_context(|| format!("Account '{}' not found", account_id))?;

        let database_for_callback = database.clone();
        let account_id_for_callback = account_id.clone();
        let on_cookies_refreshed: CookieRefreshCallback = Arc::new(move |cookies| {
            let database = database_for_callback.clone();
            let account_id = account_id_for_callback.clone();
            Box::pin(async move {
                let update_account_id = account_id.clone();
                let result = lilium_database::transaction!(database, |session| {
                    account::update_cookies(session, &update_account_id, &cookies).await?;
                    Ok(())
                })
                .await;

                match result {
                    Ok(()) => {
                        info!(account = %account_id, "Persisted refreshed DZMM cookies");
                    }
                    Err(error) => {
                        warn!(
                            account = %account_id,
                            error = %error,
                            "Failed to persist refreshed DZMM cookies"
                        );
                    }
                }
            })
        });

        DzmmApi::new(DzmmApiAuth {
            email: account.email.map(Cow::Owned),
            password: account.password.map(Cow::Owned),
            signin_code: account.signin_code.map(Cow::Owned),
            signin_code_image: account.signin_code_image,
            signin_code_image_mime: account.signin_code_image_mime.map(Cow::Owned),
            cookies: account.cookies.map(Cow::Owned),
            user_id: Some(Cow::Owned(account.user_id)),
            auto_refresh: true,
            on_cookies_refreshed: Some(on_cookies_refreshed),
        })
        .context("Failed to build DZMM auth client")
    }

    #[instrument(level = "debug" skip(context))]
    async fn run_websocket(context: WebsocketRunContext) -> Result<()> {
        let WebsocketRunContext {
            account_id,
            database,
            websocket_url,
            reconnect_delay_ms,
            ingestor,
            stop_event,
            shutdown,
            reconnect_notify,
            socket_executor,
        } = context;

        info!(account = %account_id, "WebSocket connection starting");

        loop {
            if stop_event.load(Ordering::Relaxed) {
                break;
            }

            let result = async {
                let auth = Self::build_auth_client(&database, &account_id).await?;
                auth.authenticate()
                    .await
                    .context("DZMM authentication failed")?;
                let cookie_header = auth.get_cookie_string().await;

                let mut client = WsClient::new(
                    account_id.clone(),
                    websocket_url.clone(),
                    Some(cookie_header),
                );
                let ingest = ingestor.clone();
                let ws_stop = stop_event.clone();
                let ws_shutdown = shutdown.clone();
                client
                    .run(
                        move |event| {
                            let ingest = ingest.clone();
                            Box::pin(async move {
                                let _accepted = ingest.accept_event(event).await;
                            })
                        },
                        ws_stop,
                        ws_shutdown,
                        reconnect_notify.clone(),
                        Some(socket_executor.clone()),
                    )
                    .await
            }
            .await;

            if stop_event.load(Ordering::Relaxed) {
                break;
            }

            match result {
                Ok(()) => {
                    warn!(
                        account = %account_id,
                        "WebSocket connection ended, reconnecting after delay"
                    );
                }
                Err(e) => {
                    let error_chain = format_error_chain(&e);
                    warn!(
                        account = %account_id,
                        error = %e,
                        error_chain = %error_chain,
                        "WebSocket error, reconnecting"
                    );
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(reconnect_delay_ms)).await;
        }

        info!(account = %account_id, "WebSocket loop stopped");
        Ok(())
    }

    #[instrument(level = "debug" skip_all)]
    async fn run_outgoing_command_listener(
        account_id: String,
        database: Database,
        notification_config: NotificationDatabaseConfig,
        socket_executor: SocketCommandExecutor,
        reconnect_notify: Arc<Notify>,
        stop_event: Arc<AtomicBool>,
        shutdown: Arc<Notify>,
    ) -> Result<()> {
        // Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_runtime.py
        const OUTGOING_COMMAND_INSERTED_CHANNEL: &str = "outgoing_command_inserted";
        const POLLING_INTERVAL: Duration = Duration::from_secs(30);

        let mut listener = NotificationConnection::connect(notification_config)
            .await
            .context("connect outgoing command notification listener")?;
        listener
            .listen(OUTGOING_COMMAND_INSERTED_CHANNEL)
            .await
            .context("listen for outgoing command notifications")?;

        Self::process_pending_commands(&account_id, &database, &socket_executor, &reconnect_notify)
            .await?;

        let mut polling = tokio::time::interval(POLLING_INTERVAL);
        loop {
            if stop_event.load(Ordering::Relaxed) {
                break;
            }

            tokio::select! {
                _ = shutdown.notified() => {
                    break;
                }
                payload = listener.recv_payload() => {
                    let payload = payload.context("receive outgoing command notification")?;
                    tracing::debug!(
                        account = %account_id,
                        channel = OUTGOING_COMMAND_INSERTED_CHANNEL,
                        payload = %payload,
                        "received outgoing command notification"
                    );
                    Self::process_pending_commands(
                        &account_id,
                        &database,
                        &socket_executor,
                        &reconnect_notify,
                    )
                    .await?;
                }
                _ = polling.tick() => {
                    Self::process_pending_commands(
                        &account_id,
                        &database,
                        &socket_executor,
                        &reconnect_notify,
                    )
                    .await?;
                }
                _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            }
        }

        Ok(())
    }

    async fn run_control_server(
        listener: UnixListener,
        context: WorkerControlContext,
    ) -> Result<()> {
        // Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_runtime.py
        loop {
            tokio::select! {
                _ = context.shutdown.notified() => {
                    break;
                }
                accept = listener.accept() => {
                    match accept {
                        Ok((mut socket, _addr)) => {
                            let response = match read_message(&mut socket).await {
                                Ok(raw) => match ControlCommand::from_json(&raw) {
                                    Ok(command) => Self::handle_control_command(&context, command).await,
                                    Err(e) => ControlResponse::error(e),
                                },
                                Err(e) => ControlResponse::error(format!("failed to read control command: {e}")),
                            };
                            let _ = write_message(&mut socket, &response.to_json()).await;
                        }
                        Err(e) => {
                            return Err(e).context("worker control socket accept failed");
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn run_heartbeat_loop(
        account_id: String,
        lock_id: i64,
        lock_connection: Arc<tokio::sync::Mutex<DedicatedDbConnection>>,
        socket_executor: SocketCommandExecutor,
        stop_event: Arc<AtomicBool>,
        shutdown: Arc<Notify>,
    ) -> Result<()> {
        // Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_runtime.py
        let mut heartbeat = tokio::time::interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                _ = shutdown.notified() => break,
                _ = heartbeat.tick() => {
                    if stop_event.load(Ordering::Relaxed) {
                        break;
                    }
                    if !socket_executor.is_connected().await {
                        continue;
                    }

                    let timestamp = chrono::Utc::now().timestamp_millis();
                    match socket_executor
                        .execute("heartbeat", serde_json::json!({ "timestamp": timestamp }), false)
                        .await
                    {
                        Ok(_) => {
                            let mut connection = lock_connection.lock().await;
                            websocket_connection::ensure_dedicated_connection_lock(
                                &mut connection,
                                &account_id,
                                Some(lock_id),
                            )
                            .await
                            .context("ensure websocket advisory lock during heartbeat")?;
                            websocket_connection::update_dedicated_heartbeat(
                                &mut connection,
                                lock_id,
                            )
                            .await
                            .context("update websocket heartbeat")?;
                        }
                        Err(SocketCommandError::NotConnected) => {}
                        Err(error) => {
                            return Err(anyhow::anyhow!(error))
                                .context("emit websocket heartbeat");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_control_command(
        context: &WorkerControlContext,
        command: ControlCommand,
    ) -> ControlResponse {
        match command.action {
            ControlAction::Status => {
                let ingestor_metrics = context.ingestor.metrics();
                let writer_metrics = context.writer.metrics();
                ControlResponse::success("ok").with_data(serde_json::json!({
                    "account_user_id": context.account_id,
                    "queue_depth": ingestor_metrics.queue_depth,
                    "inserted_count": writer_metrics.inserted_count,
                    "accepted_count": ingestor_metrics.accepted_count,
                    "spilled_count": ingestor_metrics.spilled_count,
                }))
            }
            ControlAction::Reconnect => {
                let previous_generation = context
                    .socket_executor
                    .connected_generation()
                    .await
                    .unwrap_or(0);
                context.reconnect_notify.notify_waiters();
                let success = context
                    .socket_executor
                    .wait_for_connection_after(previous_generation, Duration::from_secs(30))
                    .await;
                let message = if success {
                    "reconnected"
                } else {
                    "reconnect failed"
                };
                ControlResponse {
                    ok: success,
                    message: message.to_string(),
                    data: Some(serde_json::json!({ "account_user_id": context.account_id })),
                }
            }
            ControlAction::Stop => {
                context.ingestor.stop_accepting();
                context.stop_event.store(true, Ordering::Relaxed);
                context.shutdown.notify_waiters();
                ControlResponse::success("stopping")
                    .with_data(serde_json::json!({ "account_user_id": context.account_id }))
            }
            action => ControlResponse::error(format!("unsupported worker action: {action:?}")),
        }
    }

    async fn process_pending_commands(
        account_id: &str,
        database: &Database,
        socket_executor: &SocketCommandExecutor,
        reconnect_notify: &Arc<Notify>,
    ) -> Result<()> {
        let account_id_owned = account_id.to_owned();
        let db = database.clone();
        let commands = lilium_database::transaction!(db, |session| {
            let commands =
                outgoing_command::get_pending_commands(session, &account_id_owned, 100).await?;
            Ok(commands)
        })
        .await?;

        for command in commands {
            Self::execute_outgoing_command(database, socket_executor, reconnect_notify, command)
                .await?;
        }

        Ok(())
    }

    async fn execute_outgoing_command(
        database: &Database,
        socket_executor: &SocketCommandExecutor,
        reconnect_notify: &Arc<Notify>,
        command: outgoing_commands::Model,
    ) -> Result<bool> {
        if command.status != status::PENDING {
            return Ok(false);
        }

        if command.event == "system:reconnect" {
            return Self::execute_reconnect_command(
                database,
                socket_executor,
                reconnect_notify,
                command,
            )
            .await;
        }

        if !socket_executor.is_connected().await {
            warn!(
                command_id = command.id,
                account = %command.account_user_id,
                event = %command.event,
                "Socket.IO client is not connected; leaving outgoing command pending"
            );
            return Ok(false);
        }

        let command_id = command.id;
        let db = database.clone();
        lilium_database::transaction!(db, |session| {
            outgoing_command::mark_processing(session, command_id).await?;
            Ok(())
        })
        .await?;

        match socket_executor
            .execute(&command.event, command.data.clone(), command.require_ack)
            .await
        {
            Ok(ack_response) => {
                if let Some(error_message) = ack_failure_message(ack_response.as_ref()) {
                    let db = database.clone();
                    lilium_database::transaction!(db, |session| {
                        outgoing_command::retry_or_fail(session, command_id, &error_message)
                            .await?;
                        Ok(())
                    })
                    .await?;
                    Ok(false)
                } else {
                    let db = database.clone();
                    lilium_database::transaction!(db, |session| {
                        outgoing_command::mark_success(session, command_id, ack_response).await?;
                        Ok(())
                    })
                    .await?;
                    Ok(true)
                }
            }
            Err(SocketCommandError::AckTimeout) => {
                let db = database.clone();
                lilium_database::transaction!(db, |session| {
                    outgoing_command::mark_timeout(session, command_id).await?;
                    Ok(())
                })
                .await?;
                Ok(false)
            }
            Err(error) => {
                let error_message = error.to_string();
                let db = database.clone();
                lilium_database::transaction!(db, |session| {
                    outgoing_command::retry_or_fail(session, command_id, &error_message).await?;
                    Ok(())
                })
                .await?;
                Ok(false)
            }
        }
    }

    async fn execute_reconnect_command(
        database: &Database,
        socket_executor: &SocketCommandExecutor,
        reconnect_notify: &Arc<Notify>,
        command: outgoing_commands::Model,
    ) -> Result<bool> {
        let command_id = command.id;
        let previous_generation = socket_executor.connected_generation().await.unwrap_or(0);

        let db = database.clone();
        lilium_database::transaction!(db, |session| {
            outgoing_command::mark_processing(session, command_id).await?;
            Ok(())
        })
        .await?;

        reconnect_notify.notify_waiters();

        let reconnected = socket_executor
            .wait_for_connection_after(previous_generation, Duration::from_secs(30))
            .await;
        let ack = if reconnected {
            serde_json::json!({"status": "reconnected"})
        } else {
            serde_json::json!({"status": "failed"})
        };

        let db = database.clone();
        lilium_database::transaction!(db, |session| {
            outgoing_command::mark_success(session, command_id, Some(ack)).await?;
            Ok(())
        })
        .await?;

        Ok(reconnected)
    }
}

fn format_error_chain(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" | caused by: ")
}

fn ack_failure_message(ack_response: Option<&serde_json::Value>) -> Option<String> {
    let response = ack_response?.as_object()?;
    let success = response.get("success")?.as_bool()?;
    if success {
        return None;
    }

    Some(match response.get("error") {
        Some(serde_json::Value::String(error)) => {
            format!("ACK returned success=false: {error}")
        }
        Some(error) => format!("ACK returned success=false: {error}"),
        None => format!(
            "ACK returned success=false: {}",
            serde_json::Value::Object(response.clone())
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn error_chain_includes_root_cause() {
        let error = std::fs::read_to_string("/definitely/not/a/lilium/file")
            .context("outer websocket failure")
            .expect_err("missing file should fail");

        let chain = format_error_chain(&error);

        assert!(chain.contains("outer websocket failure"));
        assert!(chain.contains("No such file") || chain.contains("os error"));
    }

    #[test]
    fn ack_success_false_with_error_is_retryable_failure() {
        let ack = serde_json::json!({
            "success": false,
            "error": "rate limit"
        });

        assert_eq!(
            ack_failure_message(Some(&ack)),
            Some("ACK returned success=false: rate limit".to_string())
        );
    }

    #[test]
    fn ack_success_true_is_not_failure() {
        let ack = serde_json::json!({
            "success": true,
            "message_id": "message-1"
        });

        assert_eq!(ack_failure_message(Some(&ack)), None);
    }
}
