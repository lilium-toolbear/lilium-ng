use anyhow::Result;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use lilium_models::ingestion::{WebSocketEvent, EventProcessorOffset};

/// Service for processing events
pub struct EventService {
    pool: PgPool,
}

impl EventService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Load processor cursor from database
    pub async fn load_cursor(&self, processor_id: &str) -> Result<(i64, Option<DateTime<Utc>>)> {
        let offset = sqlx::query_as::<_, EventProcessorOffset>(
            "SELECT * FROM event_processor_offsets WHERE processor_id = $1"
        )
        .bind(processor_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(match offset {
            Some(o) => (o.last_processed_id, o.last_processed_timestamp),
            None => (0, None),
        })
    }

    /// Save processor cursor to database
    pub async fn save_cursor(
        &self,
        processor_id: &str,
        last_processed_id: i64,
        last_processed_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO event_processor_offsets (processor_id, last_processed_id, last_processed_timestamp, updated_at)
               VALUES ($1, $2, $3, NOW())
               ON CONFLICT (processor_id) DO UPDATE SET
                   last_processed_id = $2,
                   last_processed_timestamp = $3,
                   updated_at = NOW()"#,
        )
        .bind(processor_id)
        .bind(last_processed_id)
        .bind(last_processed_timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Poll for new events after cursor
    pub async fn poll_events(
        &self,
        last_timestamp: Option<DateTime<Utc>>,
        last_id: i64,
        limit: i64,
    ) -> Result<Vec<WebSocketEvent>> {
        let events = if let Some(ts) = last_timestamp {
            sqlx::query_as::<_, WebSocketEvent>(
                r#"SELECT id, event, data, user_id, timestamp
                   FROM websocket_events
                   WHERE (timestamp > $1) OR (timestamp = $1 AND id > $2)
                   ORDER BY timestamp ASC, id ASC
                   LIMIT $3"#,
            )
            .bind(ts)
            .bind(last_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, WebSocketEvent>(
                r#"SELECT id, event, data, user_id, timestamp
                   FROM websocket_events
                   WHERE id > $1
                   ORDER BY id ASC
                   LIMIT $2"#,
            )
            .bind(last_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(events)
    }
}
