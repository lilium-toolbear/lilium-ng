use anyhow::Result;
use lilium_database::queries::messages as msg_queries;
use lilium_models::dzmm::message::Message;
use crate::room_member::RoomMemberService;

/// Service for processing message events
pub struct MessageService;

impl MessageService {
    pub fn new() -> Self {
        Self
    }

    /// Process a message:new event
    /// Returns Some(message_id) if the message has media content that needs downloading
    pub async fn create_message(
        &self,
        pool: &sqlx::PgPool,
        message: &Message,
        data: &serde_json::Value,
    ) -> Result<Option<String>> {
        msg_queries::create_message_if_missing(pool, message).await?;

        // Detect membership system messages and update room member state
        if message.content_type.as_deref() == Some("system") {
            if let Some(text) = &message.content_text {
                if text.contains("加入了群聊") {
                    if let Some(sent_by) = &message.sent_by {
                        let room_member_svc = RoomMemberService::new(pool.clone());
                        room_member_svc.upsert_member(
                            &message.room_id,
                            sent_by,
                            "member",
                            message.sent_at,
                        ).await?;
                    }
                } else if text.contains("离开了群聊") {
                    if let Some(sent_by) = &message.sent_by {
                        let room_member_svc = RoomMemberService::new(pool.clone());
                        room_member_svc.mark_member_left(
                            &message.room_id,
                            sent_by,
                            message.sent_at,
                        ).await?;
                    }
                }
            }
        }

        // Check if message has media content
        if let Some(msg_data) = data.get("message") {
            if let Some(content) = msg_data.get("content") {
                if let Some(content_type) = content.get("type").and_then(|v| v.as_str()) {
                    if matches!(content_type, "image" | "video" | "voice" | "sticker") {
                        return Ok(Some(message.message_id.clone()));
                    }
                }
            }
        }

        Ok(None)
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
