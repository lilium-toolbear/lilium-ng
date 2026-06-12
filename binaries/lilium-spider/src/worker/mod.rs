use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, error};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::ingestion::{EventIngestor, EventWriter, DiskSpillBuffer};
use lilium_models::ingestion::EventEnvelope;

pub struct Worker {
    account_id: String,
    pool: PgPool,
    queue_size: usize,
    batch_size: usize,
    buffer_dir: std::path::PathBuf,
}

impl Worker {
    pub fn new(
        account_id: String,
        pool: PgPool,
        queue_size: usize,
        batch_size: usize,
        buffer_dir: std::path::PathBuf,
    ) -> Self {
        Self { account_id, pool, queue_size, batch_size, buffer_dir }
    }

    pub async fn run(&self) -> Result<()> {
        info!(account = %self.account_id, "Worker starting");

        // Create disk spill buffer
        let buffer_path = self.buffer_dir.join(format!("ws_buffer_{}.jsonl", self.account_id));
        let spill = DiskSpillBuffer::new(buffer_path);

        // Create event ingestor
        let (ingestor, rx) = EventIngestor::new(
            self.account_id.clone(),
            self.queue_size,
            spill,
        );
        let ingestor = Arc::new(ingestor);

        // Create event writer
        let writer = EventWriter::new(ingestor.clone(), self.batch_size);

        // Spawn writer task
        let writer_pool = self.pool.clone();
        let writer_account = self.account_id.clone();
        let writer_task = tokio::spawn(async move {
            Self::run_writer(writer, writer_pool, writer_account).await;
        });

        // Connect to WebSocket and process events
        let ws_account = self.account_id.clone();
        let ws_pool = self.pool.clone();
        let ws_ingestor = ingestor.clone();
        let ws_task = tokio::spawn(async move {
            Self::run_websocket(ws_account, ws_pool, ws_ingestor, rx).await;
        });

        // Wait for either task to complete
        tokio::select! {
            _ = writer_task => {
                info!(account = %self.account_id, "Writer task completed");
            }
            _ = ws_task => {
                info!(account = %self.account_id, "WebSocket task completed");
            }
        }

        info!(account = %self.account_id, "Worker shutting down");
        Ok(())
    }

    async fn run_writer(
        writer: EventWriter,
        _pool: PgPool,
        account_id: String,
    ) {
        info!(account = %account_id, "Event writer starting");

        loop {
            match writer.drain_once().await {
                Ok(0) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                Ok(count) => {
                    info!(account = %account_id, count = count, "Wrote batch to database");
                }
                Err(e) => {
                    error!(account = %account_id, error = %e, "Writer error");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn run_websocket(
        account_id: String,
        pool: PgPool,
        _ingestor: Arc<EventIngestor>,
        mut rx: mpsc::Receiver<EventEnvelope>,
    ) {
        info!(account = %account_id, "WebSocket connection starting");

        // In a real implementation, this would:
        // 1. Load account credentials from database
        // 2. Connect to DZMM.ai WebSocket
        // 3. Handle Socket.IO protocol
        // 4. Process incoming events

        // For now, simulate processing events from the queue
        while let Some(event) = rx.recv().await {
            info!(
                account = %account_id,
                event_type = %event.event_type,
                "Processing event"
            );

            // Process event based on type
            match event.event_type.as_str() {
                "message:new" => {
                    if let Err(e) = Self::process_message_new(&pool, &event).await {
                        error!(account = %account_id, error = %e, "Failed to process message:new");
                    }
                }
                "message:updated" => {
                    if let Err(e) = Self::process_message_updated(&pool, &event).await {
                        error!(account = %account_id, error = %e, "Failed to process message:updated");
                    }
                }
                "message:deleted" => {
                    if let Err(e) = Self::process_message_deleted(&pool, &event).await {
                        error!(account = %account_id, error = %e, "Failed to process message:deleted");
                    }
                }
                "message:recalled" => {
                    if let Err(e) = Self::process_message_recalled(&pool, &event).await {
                        error!(account = %account_id, error = %e, "Failed to process message:recalled");
                    }
                }
                _ => {
                    // Unknown event type, skip
                }
            }
        }
    }

    async fn process_message_new(pool: &PgPool, event: &EventEnvelope) -> Result<()> {
        let message = lilium_models::dzmm::message::Message::from_websocket(&event.payload);
        if let Some(msg) = message {
            lilium_database::queries::messages::create_message_if_missing(pool, &msg).await?;
        }
        Ok(())
    }

    async fn process_message_updated(pool: &PgPool, event: &EventEnvelope) -> Result<()> {
        if let Some(message_id) = event.payload.get("messageId").and_then(|v| v.as_str()) {
            // Check if this is a recall
            if let Some(content) = event.payload.get("message").and_then(|m| m.get("content")) {
                if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
                    lilium_database::queries::messages::mark_recalled(pool, message_id).await?;
                }
            }
        }
        Ok(())
    }

    async fn process_message_deleted(pool: &PgPool, event: &EventEnvelope) -> Result<()> {
        if let Some(message_id) = event.payload.get("messageId").and_then(|v| v.as_str()) {
            lilium_database::queries::messages::mark_deleted(pool, message_id).await?;
        }
        Ok(())
    }

    async fn process_message_recalled(pool: &PgPool, event: &EventEnvelope) -> Result<()> {
        if let Some(message_id) = event.payload.get("messageId").and_then(|v| v.as_str()) {
            lilium_database::queries::messages::mark_recalled(pool, message_id).await?;
        }
        Ok(())
    }
}
