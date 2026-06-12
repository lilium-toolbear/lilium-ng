use async_trait::async_trait;
use anyhow::Result;

use crate::event::EventEnvelope;

/// Abstraction for where events go
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Store an event
    async fn store(&self, event: &EventEnvelope) -> Result<()>;
    
    /// Store multiple events in a batch
    async fn store_batch(&self, events: &[EventEnvelope]) -> Result<usize>;
    
    /// Get sink name for logging
    fn name(&self) -> &str;
}

/// Database event sink
pub struct DatabaseSink {
    pool: sqlx::PgPool,
}

impl DatabaseSink {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EventSink for DatabaseSink {
    async fn store(&self, event: &EventEnvelope) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO websocket_events (event, data, user_id, timestamp)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(&event.account_user_id)
        .bind(event.received_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn store_batch(&self, events: &[EventEnvelope]) -> Result<usize> {
        let mut tx = self.pool.begin().await?;
        let mut count = 0;

        for event in events {
            sqlx::query(
                r#"INSERT INTO websocket_events (event, data, user_id, timestamp)
                   VALUES ($1, $2, $3, $4)"#,
            )
            .bind(&event.event_type)
            .bind(&event.payload)
            .bind(&event.account_user_id)
            .bind(event.received_at)
            .execute(&mut *tx)
            .await?;
            count += 1;
        }

        tx.commit().await?;
        Ok(count)
    }

    fn name(&self) -> &str {
        "database"
    }
}

/// Disk buffer event sink
pub struct DiskBufferSink {
    path: std::path::PathBuf,
    _lock: tokio::sync::Mutex<()>,
}

impl DiskBufferSink {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self { path, _lock: tokio::sync::Mutex::new(()) }
    }
}

#[async_trait]
impl EventSink for DiskBufferSink {
    async fn store(&self, event: &EventEnvelope) -> Result<()> {
        use crate::event::SpillRecord;
        use tokio::io::AsyncWriteExt;

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

    async fn store_batch(&self, events: &[EventEnvelope]) -> Result<usize> {
        let mut count = 0;
        for event in events {
            self.store(event).await?;
            count += 1;
        }
        Ok(count)
    }

    fn name(&self) -> &str {
        "disk_buffer"
    }
}
