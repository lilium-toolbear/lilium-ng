use anyhow::Result;
use lilium_database::queries::messages as msg_queries;
use lilium_models::dzmm::message::Message;

/// Service for processing message events
pub struct MessageService;

impl MessageService {
    pub fn new() -> Self {
        Self
    }

    /// Process a message:new event
    pub async fn create_message(&self, pool: &sqlx::PgPool, message: &Message) -> Result<bool> {
        msg_queries::create_message_if_missing(pool, message).await
    }

    /// Process a message:updated event
    pub async fn update_message(&self, pool: &sqlx::PgPool, message_id: &str, payload: &serde_json::Value) -> Result<()> {
        if let Some(content) = payload.get("message").and_then(|m| m.get("content")) {
            if content.get("type").and_then(|v| v.as_str()) == Some("recalled") {
                msg_queries::mark_recalled(pool, message_id).await?;
            } else if let Some(text) = content.get("text").and_then(|v| v.as_str()) {
                msg_queries::update_content(pool, message_id, text).await?;
            }
        }
        Ok(())
    }

    /// Process a message:deleted event
    pub async fn mark_deleted(&self, pool: &sqlx::PgPool, message_id: &str) -> Result<()> {
        msg_queries::mark_deleted(pool, message_id).await
    }

    /// Process a message:recalled event
    pub async fn mark_recalled(&self, pool: &sqlx::PgPool, message_id: &str) -> Result<()> {
        msg_queries::mark_recalled(pool, message_id).await
    }
}
