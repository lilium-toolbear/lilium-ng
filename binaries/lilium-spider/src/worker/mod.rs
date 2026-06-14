use anyhow::Result;
use lilium_api_client::websocket::WsClient;
use lilium_database::DbPool;
use lilium_services::account_service::AccountService;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{error, info, warn};

use crate::ingestion::{DiskSpillBuffer, EventIngestor, EventWriter};

pub struct Worker {
    account_id: String,
    pool: DbPool,
    queue_size: usize,
    batch_size: usize,
    buffer_dir: std::path::PathBuf,
    websocket_url: String,
    reconnect_delay_ms: u64,
}

impl Worker {
    pub fn new(
        account_id: String,
        pool: DbPool,
        queue_size: usize,
        batch_size: usize,
        buffer_dir: std::path::PathBuf,
        websocket_url: String,
        reconnect_delay_ms: u64,
    ) -> Self {
        Self {
            account_id,
            pool,
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
        let writer = EventWriter::new(self.pool.clone(), ingestor.clone(), rx, self.batch_size);

        let stop_event = Arc::new(AtomicBool::new(false));
        let writer_stop = stop_event.clone();
        let writer_fut = writer.run(&writer_stop);

        let ws_account = self.account_id.clone();
        let ws_pool = self.pool.clone();
        let ws_ingestor = ingestor.clone();
        let ws_url = self.websocket_url.clone();
        let ws_shutdown = shutdown.clone();
        let ws_stop = stop_event.clone();
        let ws_delay = self.reconnect_delay_ms;
        let ws_fut = Self::run_websocket(
            ws_account,
            ws_pool,
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
                    Err(e) => error!(account = %self.account_id, error = %e, "WebSocket task failed"),
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

    async fn load_cookie_header(pool: &DbPool, account_id: &str) -> Result<Option<String>> {
        pool.with_session_context(|session| {
            let account_id = account_id.to_string();
            Box::pin(async move {
                let mut svc = AccountService::new(session);
                let account = svc.get_account(&account_id).await?;
                Ok(account.and_then(|a| a.cookies))
            })
        })
        .await
    }

    async fn run_websocket(
        account_id: String,
        pool: DbPool,
        websocket_url: String,
        reconnect_delay_ms: u64,
        ingestor: Arc<EventIngestor>,
        stop_event: Arc<AtomicBool>,
        shutdown: Arc<Notify>,
    ) -> Result<()> {
        info!(account = %account_id, "WebSocket connection starting");
        let cookie_header = Self::load_cookie_header(&pool, &account_id).await?;

        loop {
            if stop_event.load(Ordering::Relaxed) {
                break;
            }

            let mut client = WsClient::new(
                account_id.clone(),
                websocket_url.clone(),
                cookie_header.clone(),
            );
            let ingest = ingestor.clone();
            let ws_stop = stop_event.clone();
            let ws_shutdown = shutdown.clone();
            let reconnect_notify = Arc::new(Notify::new());

            let result = client
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
                    warn!(account = %account_id, error = %e, "WebSocket error, reconnecting");
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(reconnect_delay_ms)).await;
        }

        info!(account = %account_id, "WebSocket loop stopped");
        Ok(())
    }
}
