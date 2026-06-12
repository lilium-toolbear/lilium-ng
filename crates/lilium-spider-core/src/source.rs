use async_trait::async_trait;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::event::EventEnvelope;

/// Abstraction for where events come from
#[async_trait]
pub trait EventSource: Send + Sync {
    /// Start producing events
    async fn run(&self, sender: mpsc::Sender<EventEnvelope>) -> Result<()>;
    
    /// Stop producing events
    async fn stop(&self) -> Result<()>;
    
    /// Get source name for logging
    fn name(&self) -> &str;
}

/// WebSocket event source
pub struct WebSocketSource {
    account_id: String,
    // WebSocket connection would go here
}

impl WebSocketSource {
    pub fn new(account_id: String) -> Self {
        Self { account_id }
    }
}

#[async_trait]
impl EventSource for WebSocketSource {
    async fn run(&self, _sender: mpsc::Sender<EventEnvelope>) -> Result<()> {
        // TODO: Implement WebSocket connection
        // 1. Connect to DZMM.ai
        // 2. Handle Socket.IO protocol
        // 3. Send events to sender
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.account_id
    }
}

/// Disk replay event source
pub struct DiskReplaySource {
    path: std::path::PathBuf,
}

impl DiskReplaySource {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl EventSource for DiskReplaySource {
    async fn run(&self, sender: mpsc::Sender<EventEnvelope>) -> Result<()> {
        use crate::event::SpillRecord;
        use tokio::io::{AsyncBufReadExt, BufReader};
        use tokio::fs::File;

        if !self.path.exists() {
            return Ok(());
        }

        let file = File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let record: SpillRecord = serde_json::from_str(&line)?;
            if record.schema_version != 2 {
                return Err(anyhow::anyhow!("Legacy spill buffer format"));
            }
            if let Some(env) = record.to_event_envelope() {
                let _ = sender.send(env).await;
            }
        }

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "disk_replay"
    }
}
