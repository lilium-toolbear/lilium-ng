use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info};

use crate::ingestion::{DiskSpillBuffer, EventIngestor, EventWriter};

pub struct Worker {
    account_id: String,
    queue_size: usize,
    batch_size: usize,
    buffer_dir: std::path::PathBuf,
}

impl Worker {
    pub fn new(
        account_id: String,
        queue_size: usize,
        batch_size: usize,
        buffer_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            account_id,
            queue_size,
            batch_size,
            buffer_dir,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(account = %self.account_id, "Worker starting");

        // Create disk spill buffer
        let buffer_path = self
            .buffer_dir
            .join(format!("ws_buffer_{}.jsonl", self.account_id));
        let spill = DiskSpillBuffer::new(buffer_path);

        // Create event ingestor
        let (ingestor, _rx) = EventIngestor::new(self.account_id.clone(), self.queue_size, spill);
        let ingestor = Arc::new(ingestor);

        // Create event writer
        let writer = EventWriter::new(ingestor.clone(), self.batch_size);

        // Spawn writer task
        let writer_account = self.account_id.clone();
        let writer_task = tokio::spawn(async move {
            Self::run_writer(writer, writer_account).await;
        });

        // Connect to WebSocket and enqueue events (NOT process them)
        let ws_account = self.account_id.clone();
        let ws_ingestor = ingestor.clone();
        let ws_task = tokio::spawn(async move {
            Self::run_websocket(ws_account, ws_ingestor).await;
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

    async fn run_writer(writer: EventWriter, account_id: String) {
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

    async fn run_websocket(account_id: String, _ingestor: Arc<EventIngestor>) {
        info!(account = %account_id, "WebSocket connection starting");

        // In a real implementation, this would:
        // 1. Load account credentials from database
        // 2. Connect to DZMM.ai WebSocket
        // 3. Handle Socket.IO protocol
        // 4. Receive events and enqueue them via ingestor.accept_event()

        // For now, simulate receiving events
        // In production, this would be a real WebSocket connection
        // that calls ingestor.accept_event() for each received event

        info!(account = %account_id, "WebSocket connection established (simulated)");
    }
}
