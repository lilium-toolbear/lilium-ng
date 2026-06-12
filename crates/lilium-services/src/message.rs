use anyhow::Result;
use sqlx::PgPool;

use lilium_models::dzmm::message::Message;

/// Service for processing message events
pub struct MessageService {
    pool: PgPool,
}

impl MessageService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Process a message:new event
    pub async fn create_message(&self, message: &Message) -> Result<bool> {
        let result = sqlx::query(
            r#"INSERT INTO messages (message_id, room_id, sent_by, content_text, content_type,
               sent_at, is_deleted, is_recalled, is_edited, history, raw_data, source)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT DO NOTHING"#,
        )
        .bind(&message.message_id)
        .bind(&message.room_id)
        .bind(&message.sent_by)
        .bind(&message.content_text)
        .bind(&message.content_type)
        .bind(message.sent_at)
        .bind(message.is_deleted)
        .bind(message.is_recalled)
        .bind(message.is_edited)
        .bind(&message.history)
        .bind(&message.raw_data)
        .bind(&message.source)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Process a message:updated event
    pub async fn update_message(&self, message_id: &str, payload: &serde_json::Value) -> Result<()> {
        if let Some(content) = payload.get("message").and_then(|m| m.get("content")) {
            if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
                self.mark_recalled(message_id).await?;
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
        Ok(())
    }

    /// Process a message:deleted event
    pub async fn mark_deleted(&self, message_id: &str) -> Result<()> {
        sqlx::query("UPDATE messages SET is_deleted = true WHERE message_id = $1")
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Process a message:recalled event
    pub async fn mark_recalled(&self, message_id: &str) -> Result<()> {
        sqlx::query("UPDATE messages SET is_recalled = true WHERE message_id = $1")
            .bind(message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
