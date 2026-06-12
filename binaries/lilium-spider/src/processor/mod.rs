use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};
use tokio::time::{interval, Duration};

use lilium_models::ingestion::{WebSocketEvent, EventProcessorOffset};

pub struct EventProcessor {
    processor_id: String,
    pool: PgPool,
    batch_size: usize,
    polling_interval: Duration,
    max_retries: u32,
    initial_retry_delay: Duration,
    max_retry_delay: Duration,
}

impl EventProcessor {
    pub fn new(
        pool: PgPool,
        processor_id: String,
        batch_size: usize,
        polling_interval_secs: u64,
    ) -> Self {
        Self {
            processor_id,
            pool,
            batch_size,
            polling_interval: Duration::from_secs(polling_interval_secs),
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(60),
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            processor_id = %self.processor_id,
            batch_size = self.batch_size,
            poll_interval_secs = self.polling_interval.as_secs(),
            "Event processor starting"
        );

        // Load initial cursor
        let mut cursor = self.load_cursor().await?;
        info!(
            processor_id = %self.processor_id,
            cursor_id = cursor.last_processed_id,
            "Loaded initial cursor"
        );

        // Main processing loop with polling fallback
        let mut poll_interval = interval(self.polling_interval);

        loop {
            tokio::select! {
                _ = poll_interval.tick() => {
                    // Poll for new events
                    match self.poll_new_events(&cursor).await {
                        Ok(events) => {
                            if events.is_empty() {
                                continue;
                            }
                            info!(count = events.len(), "Processing batch");
                            self.process_batch(&events, &mut cursor).await?;
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to poll events");
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down event processor");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn load_cursor(&self) -> Result<CursorPosition> {
        let offset = sqlx::query_as::<_, EventProcessorOffset>(
            "SELECT * FROM event_processor_offsets WHERE processor_id = $1"
        )
        .bind(&self.processor_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(match offset {
            Some(o) => CursorPosition {
                last_processed_id: o.last_processed_id,
                last_processed_timestamp: o.last_processed_timestamp,
            },
            None => CursorPosition {
                last_processed_id: 0,
                last_processed_timestamp: None,
            },
        })
    }

    async fn save_cursor(&self, cursor: &CursorPosition) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO event_processor_offsets (processor_id, last_processed_id, last_processed_timestamp, updated_at)
               VALUES ($1, $2, $3, NOW())
               ON CONFLICT (processor_id) DO UPDATE SET
                   last_processed_id = $2,
                   last_processed_timestamp = $3,
                   updated_at = NOW()"#,
        )
        .bind(&self.processor_id)
        .bind(cursor.last_processed_id)
        .bind(cursor.last_processed_timestamp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn poll_new_events(&self, cursor: &CursorPosition) -> Result<Vec<WebSocketEvent>> {
        let events = if let Some(ts) = cursor.last_processed_timestamp {
            sqlx::query_as::<_, WebSocketEvent>(
                r#"SELECT id, event, data, user_id, timestamp
                   FROM websocket_events
                   WHERE (timestamp > $1) OR (timestamp = $1 AND id > $2)
                   ORDER BY timestamp ASC, id ASC
                   LIMIT $3"#,
            )
            .bind(ts)
            .bind(cursor.last_processed_id)
            .bind(self.batch_size as i64)
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
            .bind(cursor.last_processed_id)
            .bind(self.batch_size as i64)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(events)
    }

    async fn process_batch(
        &self,
        events: &[WebSocketEvent],
        cursor: &mut CursorPosition,
    ) -> Result<()> {
        let mut last_id = cursor.last_processed_id;
        let mut last_timestamp = cursor.last_processed_timestamp;

        for event in events {
            match self.process_event(event).await {
                Ok(()) => {
                    if let Some(id) = event.id {
                        last_id = id;
                    }
                    last_timestamp = Some(event.timestamp);
                }
                Err(e) => {
                    warn!(
                        event_id = event.id,
                        error = %e,
                        "Failed to processing event"
                    );
                    // Skip this event and continue
                    if let Some(id) = event.id {
                        last_id = id;
                    }
                    last_timestamp = Some(event.timestamp);
                }
            }
        }

        // Update cursor
        cursor.last_processed_id = last_id;
        cursor.last_processed_timestamp = last_timestamp;
        self.save_cursor(cursor).await?;

        Ok(())
    }

    async fn process_event(&self, event: &WebSocketEvent) -> Result<()> {
        match event.event.as_str() {
            "message:new" => {
                if let Some(msg) = lilium_models::dzmm::message::Message::from_websocket(&event.data) {
                    lilium_database::queries::messages::create_message_if_missing(&self.pool, &msg).await?;
                }
            }
            "message:updated" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    if let Some(content) = event.data.get("message").and_then(|m| m.get("content")) {
                        if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
                            lilium_database::queries::messages::mark_recalled(&self.pool, message_id).await?;
                        }
                    }
                }
            }
            "message:deleted" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    lilium_database::queries::messages::mark_deleted(&self.pool, message_id).await?;
                }
            }
            "message:recalled" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    lilium_database::queries::messages::mark_recalled(&self.pool, message_id).await?;
                }
            }
            "presence:user-online" | "group:member-joined" | "group:member-left" => {
                // These events are ignored for now
            }
            _ => {
                // Unknown event type
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct CursorPosition {
    last_processed_id: i64,
    last_processed_timestamp: Option<DateTime<Utc>>,
}
