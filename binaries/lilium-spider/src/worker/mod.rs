#![allow(dead_code)]

use anyhow::{Context, Result};
use lilium_api_client::http::{CookieRefreshCallback, DzmmApi};
use lilium_api_client::websocket::WsClient;
use lilium_database::Database;
use lilium_services::account;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;
use tracing::{error, info, warn};

use crate::ingestion::{DiskSpillBuffer, EventIngestor, EventWriter};

pub struct Worker {
    account_id: String,
    database: Database,
    queue_size: usize,
    batch_size: usize,
    buffer_dir: std::path::PathBuf,
    websocket_url: String,
    reconnect_delay_ms: u64,
}

impl Worker {
    pub fn new(
        account_id: String,
        database: Database,
        queue_size: usize,
        batch_size: usize,
        buffer_dir: std::path::PathBuf,
        websocket_url: String,
        reconnect_delay_ms: u64,
    ) -> Self {
        Self {
            account_id,
            database,
            queue_size,
            batch_size,
            buffer_dir,
            websocket_url,
            reconnect_delay_ms,
        }
    }

    pub async fn run(&self, shutdown: Arc<Notify>) -> Result<()> {
        info!(account = %self.account_id, "Worker starting");

        // Create disk spill buffer
        let buffer_path = self
            .buffer_dir
            .join(format!("ws_buffer_{}.jsonl", self.account_id));
        let spill = DiskSpillBuffer::new(buffer_path);

        // Create event ingestor
        let (ingestor, rx) = EventIngestor::new(self.account_id.clone(), self.queue_size, spill);
        let ingestor = Arc::new(ingestor);

        // Create event writer
        let writer = EventWriter::new(self.database.clone(), ingestor.clone(), rx, self.batch_size);

        let stop_event = Arc::new(AtomicBool::new(false));
        let writer_stop = stop_event.clone();
        let writer_fut = writer.run(&writer_stop);

        let ws_account = self.account_id.clone();
        let ws_database = self.database.clone();
        let ws_ingestor = ingestor.clone();
        let ws_url = self.websocket_url.clone();
        let ws_shutdown = shutdown.clone();
        let ws_stop = stop_event.clone();
        let ws_delay = self.reconnect_delay_ms;
        let ws_fut = Self::run_websocket(
            ws_account,
            ws_database,
            ws_url,
            ws_delay,
            ws_ingestor,
            ws_stop,
            ws_shutdown,
        );

        tokio::select! {
            _ = shutdown.notified() => {
                info!(account = %self.account_id, "Shutdown requested");
                stop_event.store(true, Ordering::Relaxed);
            }
            ws_result = ws_fut => {
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
                stop_event.store(true, Ordering::Relaxed);
                info!(account = %self.account_id, "Writer task completed");
            }
        }

        stop_event.store(true, Ordering::Relaxed);
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

        DzmmApi::new(
            account.email,
            account.password,
            account.signin_code,
            account.signin_code_image,
            account.signin_code_image_mime,
            account.cookies,
            Some(account.user_id),
            true,
            Some(on_cookies_refreshed),
        )
        .context("Failed to build DZMM auth client")
    }

    async fn run_websocket(
        account_id: String,
        database: Database,
        websocket_url: String,
        reconnect_delay_ms: u64,
        ingestor: Arc<EventIngestor>,
        stop_event: Arc<AtomicBool>,
        shutdown: Arc<Notify>,
    ) -> Result<()> {
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
                let reconnect_notify = Arc::new(Notify::new());

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
                        reconnect_notify,
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
}

fn format_error_chain(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" | caused by: ")
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
}
