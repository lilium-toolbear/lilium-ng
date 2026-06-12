use async_trait::async_trait;
use anyhow::Result;

use crate::event::{EventEnvelope, EventType};

/// Abstraction for how events are processed
#[async_trait]
pub trait EventProcessor: Send + Sync {
    /// Process a single event
    async fn process(&self, event: &EventEnvelope) -> Result<()>;
    
    /// Process a batch of events
    async fn process_batch(&self, events: &[EventEnvelope]) -> Result<usize>;
    
    /// Get processor name for logging
    fn name(&self) -> &str;
}

/// Message event processor
pub struct MessageProcessor {
    pool: sqlx::PgPool,
}

impl MessageProcessor {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    async fn process_message_new(&self, event: &EventEnvelope) -> Result<()> {
        if let Some(msg) = lilium_models::dzmm::message::Message::from_websocket(&event.payload) {
            sqlx::query(
                r#"INSERT INTO messages (message_id, room_id, sent_by, content_text, content_type,
                   sent_at, is_deleted, is_recalled, is_edited, history, raw_data, source)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(&msg.message_id)
            .bind(&msg.room_id)
            .bind(&msg.sent_by)
            .bind(&msg.content_text)
            .bind(&msg.content_type)
            .bind(msg.sent_at)
            .bind(msg.is_deleted)
            .bind(msg.is_recalled)
            .bind(msg.is_edited)
            .bind(&msg.history)
            .bind(&msg.raw_data)
            .bind(&msg.source)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn process_message_updated(&self, event: &EventEnvelope) -> Result<()> {
        if let Some(message_id) = event.payload.get("messageId").and_then(|v| v.as_str()) {
            if let Some(content) = event.payload.get("message").and_then(|m| m.get("content")) {
                if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
                    sqlx::query("UPDATE messages SET is_recalled = true WHERE message_id = $1")
                        .bind(message_id)
                        .execute(&self.pool)
                        .await?;
                } else if let Some(text) = content.get("text").and_then(|v| v.as_str()) {
                    sqlx::query(
                        r#"UPDATE messages SET
                           content_text = $1,
                           is_edited = true,
                           history = COALESCE(history, '[]'::jsonb) || jsonb_build_object(
                               'content', content_text,
                               'edited_at', NOW()
                           )
                           WHERE message_id = $2"#,
                    )
                    .bind(text)
                    .bind(message_id)
                    .execute(&self.pool)
                    .await?;
                }
            }
        }
        Ok(())
    }

    async fn process_message_deleted(&self, event: &EventEnvelope) -> Result<()> {
        if let Some(message_id) = event.payload.get("messageId").and_then(|v| v.as_str()) {
            sqlx::query("UPDATE messages SET is_deleted = true WHERE message_id = $1")
                .bind(message_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn process_message_recalled(&self, event: &EventEnvelope) -> Result<()> {
        if let Some(message_id) = event.payload.get("messageId").and_then(|v| v.as_str()) {
            sqlx::query("UPDATE messages SET is_recalled = true WHERE message_id = $1")
                .bind(message_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl EventProcessor for MessageProcessor {
    async fn process(&self, event: &EventEnvelope) -> Result<()> {
        let event_type = EventType::from_str(&event.event_type);
        match event_type {
            EventType::MessageNew => self.process_message_new(event).await,
            EventType::MessageUpdated => self.process_message_updated(event).await,
            EventType::MessageDeleted => self.process_message_deleted(event).await,
            EventType::MessageRecalled => self.process_message_recalled(event).await,
            _ => Ok(()),
        }
    }

    async fn process_batch(&self, events: &[EventEnvelope]) -> Result<usize> {
        let mut count = 0;
        for event in events {
            if let Err(e) = self.process(event).await {
                tracing::warn!(error = %e, "Failed to process event");
            } else {
                count += 1;
            }
        }
        Ok(count)
    }

    fn name(&self) -> &str {
        "message"
    }
}

/// Composite processor that routes events to appropriate handlers
pub struct CompositeProcessor {
    processors: Vec<Box<dyn EventProcessor>>,
}

impl CompositeProcessor {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn EventProcessor>) {
        self.processors.push(processor);
    }
}

#[async_trait]
impl EventProcessor for CompositeProcessor {
    async fn process(&self, event: &EventEnvelope) -> Result<()> {
        for processor in &self.processors {
            processor.process(event).await?;
        }
        Ok(())
    }

    async fn process_batch(&self, events: &[EventEnvelope]) -> Result<usize> {
        let mut count = 0;
        for event in events {
            if self.process(event).await.is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    fn name(&self) -> &str {
        "composite"
    }
}
