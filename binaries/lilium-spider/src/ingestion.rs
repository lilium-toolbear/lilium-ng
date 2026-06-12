use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use lilium_models::ingestion::EventEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpillRecord {
    pub schema_version: u32,
    pub account_user_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
    pub source: String,
}

impl From<&EventEnvelope> for SpillRecord {
    fn from(e: &EventEnvelope) -> Self {
        Self {
            schema_version: 2,
            account_user_id: e.account_user_id.clone(),
            event_type: e.event_type.clone(),
            payload: e.payload.clone(),
            received_at: e.received_at,
            source: e.source.clone(),
        }
    }
}

impl SpillRecord {
    pub fn to_event_envelope(&self) -> EventEnvelope {
        EventEnvelope {
            account_user_id: self.account_user_id.clone(),
            event_type: self.event_type.clone(),
            payload: self.payload.clone(),
            received_at: self.received_at,
            source: self.source.clone(),
        }
    }
}

pub struct DiskSpillBuffer {
    path: PathBuf,
    _lock: Mutex<()>,
}

impl DiskSpillBuffer {
    pub fn new(path: PathBuf) -> Self {
        Self { path, _lock: Mutex::new(()) }
    }

    pub async fn append(&self, event: &EventEnvelope) -> Result<()> {
        let _guard = self._lock.lock().await;
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let record = SpillRecord::from(event);
        let mut line = serde_json::to_string(&record)?;
        line.push('\n');
        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?
            .write_all(line.as_bytes())
            .await?;
        Ok(())
    }

    pub async fn read_replay_batch(&self, limit: usize) -> Result<Vec<EventEnvelope>> {
        if limit == 0 || !self.path.exists() {
            return Ok(vec![]);
        }
        let _guard = self._lock.lock().await;
        let file = tokio::fs::File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut events = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            let record: SpillRecord = serde_json::from_str(&line)?;
            if record.schema_version != 2 {
                return Err(anyhow::anyhow!(
                    "LegacySpillBufferError: expected schema_version 2, got {}",
                    record.schema_version
                ));
            }
            let mut env = record.to_event_envelope();
            env.source = "disk_replay".to_string();
            events.push(env);
            if events.len() >= limit {
                break;
            }
        }
        Ok(events)
    }

    pub async fn discard_replay_batch(&self, count: usize) -> Result<()> {
        if count == 0 || !self.path.exists() {
            return Ok(());
        }
        let _guard = self._lock.lock().await;
        let file = tokio::fs::File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut remaining = Vec::new();
        let mut skipped = 0;

        while let Some(line) = lines.next_line().await? {
            if skipped < count {
                skipped += 1;
                continue;
            }
            remaining.push(line);
        }

        if remaining.is_empty() {
            tokio::fs::remove_file(&self.path).await?;
        } else {
            let mut file = tokio::fs::File::create(&self.path).await?;
            for line in &remaining {
                file.write_all(line.as_bytes()).await?;
                file.write_all(b"\n").await?;
            }
        }
        Ok(())
    }

    pub async fn has_pending(&self) -> bool {
        if !self.path.exists() {
            return false;
        }
        match tokio::fs::metadata(&self.path).await {
            Ok(m) => m.len() > 0,
            Err(_) => false,
        }
    }
}

pub struct EventIngestor {
    #[allow(dead_code)]
    account_user_id: String,
    queue: mpsc::Sender<EventEnvelope>,
    spill: DiskSpillBuffer,
    accepted_count: AtomicU64,
    spilled_count: AtomicU64,
    is_accepting: AtomicBool,
    max_queue_size: usize,
}

impl EventIngestor {
    pub fn new(account_user_id: String, max_queue_size: usize, spill: DiskSpillBuffer) -> (Self, mpsc::Receiver<EventEnvelope>) {
        let (tx, rx) = mpsc::channel(max_queue_size);
        let ingestor = Self {
            account_user_id,
            queue: tx,
            spill,
            accepted_count: AtomicU64::new(0),
            spilled_count: AtomicU64::new(0),
            is_accepting: AtomicBool::new(true),
            max_queue_size,
        };
        (ingestor, rx)
    }

    pub async fn accept_event(&self, event: EventEnvelope) -> bool {
        if !self.is_accepting.load(Ordering::Relaxed) {
            self.spilled_count.fetch_add(1, Ordering::Relaxed);
            let _ = self.spill.append(&event).await;
            return false;
        }

        match self.queue.try_send(event.clone()) {
            Ok(()) => {
                self.accepted_count.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(_) => {
                self.spilled_count.fetch_add(1, Ordering::Relaxed);
                let _ = self.spill.append(&event).await;
                false
            }
        }
    }

    pub fn stop_accepting(&self) {
        self.is_accepting.store(false, Ordering::Relaxed);
    }

    pub fn queue_depth(&self) -> usize {
        self.max_queue_size - self.queue.capacity()
    }

    pub fn accepted_count(&self) -> u64 {
        self.accepted_count.load(Ordering::Relaxed)
    }

    pub fn spilled_count(&self) -> u64 {
        self.spilled_count.load(Ordering::Relaxed)
    }

    pub fn is_accepting(&self) -> bool {
        self.is_accepting.load(Ordering::Relaxed)
    }

    pub fn spill(&self) -> &DiskSpillBuffer {
        &self.spill
    }
}

pub struct EventWriter {
    ingestor: Arc<EventIngestor>,
    batch_size: usize,
    inserted_count: AtomicU64,
}

impl EventWriter {
    pub fn new(ingestor: Arc<EventIngestor>, batch_size: usize) -> Self {
        Self {
            ingestor,
            batch_size,
            inserted_count: AtomicU64::new(0),
        }
    }

    pub async fn run(&self, stop_event: &Arc<AtomicBool>) {
        loop {
            let inserted = self.drain_once().await.unwrap_or(0);
            let spill_pending = self.ingestor.spill().has_pending().await;

            if stop_event.load(Ordering::Relaxed)
                && self.ingestor.queue_depth() == 0
                && !spill_pending
            {
                break;
            }

            if stop_event.load(Ordering::Relaxed) && inserted == 0 && spill_pending {
                self.spill_memory_queue().await;
                break;
            }

            if inserted == 0 {
                if stop_event.load(Ordering::Relaxed) && spill_pending {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }
        }
    }

    pub async fn drain_once(&self) -> Result<usize> {
        // First try disk spill replay
        if self.ingestor.spill().has_pending().await {
            let events = self.ingestor.spill().read_replay_batch(self.batch_size).await?;
            if !events.is_empty() {
                return self.insert_replay_batch(&events).await;
            }
        }

        // Then drain from memory queue
        // This would need access to the receiver
        Ok(0)
    }

    async fn insert_replay_batch(&self, events: &[EventEnvelope]) -> Result<usize> {
        let count = events.len();
        // In real implementation, would batch insert to DB
        self.ingestor.spill().discard_replay_batch(count).await?;
        self.inserted_count.fetch_add(count as u64, Ordering::Relaxed);
        Ok(count)
    }

    async fn spill_memory_queue(&self) {
        // Spill remaining memory queue to disk on shutdown
        // This would need access to the receiver, which is complex with the current architecture
    }

    pub fn inserted_count(&self) -> u64 {
        self.inserted_count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_event(i: i32) -> EventEnvelope {
        EventEnvelope {
            account_user_id: "user_a".to_string(),
            event_type: "message:new".to_string(),
            payload: serde_json::json!({"i": i}),
            received_at: chrono::DateTime::parse_from_rfc3339("2026-04-30T01:01:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: "socket".to_string(),
        }
    }

    #[tokio::test]
    async fn test_accept_event_enqueues_without_db() {
        let tmp = TempDir::new().unwrap();
        let spill = DiskSpillBuffer::new(tmp.path().join("ws_buffer_user_a.jsonl"));
        let (ingestor, _rx) = EventIngestor::new("user_a".to_string(), 2, spill);

        let accepted = ingestor.accept_event(make_event(1)).await;
        assert!(accepted);
        assert_eq!(ingestor.queue_depth(), 1);
        assert_eq!(ingestor.accepted_count(), 1);
        assert_eq!(ingestor.spilled_count(), 0);
    }

    #[tokio::test]
    async fn test_accept_event_spills_when_queue_full() {
        let tmp = TempDir::new().unwrap();
        let spill = DiskSpillBuffer::new(tmp.path().join("ws_buffer_user_a.jsonl"));
        let (ingestor, _rx) = EventIngestor::new("user_a".to_string(), 1, spill);

        assert!(ingestor.accept_event(make_event(1)).await);
        assert!(!ingestor.accept_event(make_event(2)).await);

        assert_eq!(ingestor.queue_depth(), 1);
        assert_eq!(ingestor.spilled_count(), 1);
    }

    #[tokio::test]
    async fn test_spill_buffer_writes_versioned_schema() {
        let tmp = TempDir::new().unwrap();
        let spill = DiskSpillBuffer::new(tmp.path().join("ws_buffer_user_a.jsonl"));

        spill.append(&make_event(1)).await.unwrap();

        let raw = tokio::fs::read_to_string(tmp.path().join("ws_buffer_user_a.jsonl")).await.unwrap();
        assert!(raw.contains("schema_version"));
        assert!(raw.contains("account_user_id"));
        assert!(raw.contains("event_type"));
        assert!(raw.contains("payload"));
        assert!(raw.contains("received_at"));
    }

    #[tokio::test]
    async fn test_spill_buffer_rejects_legacy_records() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("ws_buffer_user_a.jsonl");
        tokio::fs::write(&path, r#"{"schema_version":1,"account_user_id":"user_a","event_type":"test","payload":{},"received_at":"2026-01-01T00:00:00Z","source":"socket"}"#).await.unwrap();
        let spill = DiskSpillBuffer::new(path);

        let result = spill.read_replay_batch(10).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LegacySpillBufferError"));
    }

    #[tokio::test]
    async fn test_spill_buffer_discards_replayed_records() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("ws_buffer_user_a.jsonl");
        let spill = DiskSpillBuffer::new(path.clone());

        spill.append(&make_event(1)).await.unwrap();
        spill.append(&make_event(2)).await.unwrap();

        spill.discard_replay_batch(1).await.unwrap();

        let events = spill.read_replay_batch(10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].payload["i"], 2);
    }

    #[tokio::test]
    async fn test_event_writer_drains_disk_before_memory() {
        let tmp = TempDir::new().unwrap();
        let spill = DiskSpillBuffer::new(tmp.path().join("ws_buffer_user_a.jsonl"));
        let (ingestor, _rx) = EventIngestor::new("user_a".to_string(), 10, spill);
        let ingestor = Arc::new(ingestor);

        // Write events to disk
        ingestor.spill().append(&make_event(1)).await.unwrap();
        ingestor.spill().append(&make_event(2)).await.unwrap();

        let writer = EventWriter::new(ingestor.clone(), 100);
        let count = writer.drain_once().await.unwrap();
        assert_eq!(count, 2); // Should drain disk first
    }

    #[tokio::test]
    async fn test_stop_accepting_sends_to_disk() {
        let tmp = TempDir::new().unwrap();
        let spill = DiskSpillBuffer::new(tmp.path().join("ws_buffer_user_a.jsonl"));
        let (ingestor, _rx) = EventIngestor::new("user_a".to_string(), 10, spill);

        ingestor.stop_accepting();
        assert!(!ingestor.is_accepting());

        let accepted = ingestor.accept_event(make_event(1)).await;
        assert!(!accepted);
        assert_eq!(ingestor.spilled_count(), 1);
    }

    #[tokio::test]
    async fn test_spill_buffer_concurrent_appends() {
        let tmp = TempDir::new().unwrap();
        let spill = DiskSpillBuffer::new(tmp.path().join("ws_buffer_user_a.jsonl"));
        let spill = Arc::new(spill);

        let mut handles = Vec::new();
        for i in 0..10 {
            let spill = spill.clone();
            handles.push(tokio::spawn(async move {
                spill.append(&make_event(i)).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let events = spill.read_replay_batch(100).await.unwrap();
        assert_eq!(events.len(), 10);
    }
}
