use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub message_id: String,
    pub room_id: String,
    pub sent_by: Option<String>,
    pub content_text: Option<String>,
    pub content_type: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub is_recalled: bool,
    pub is_edited: bool,
    pub history: Option<serde_json::Value>,
    pub raw_data: serde_json::Value,
    pub source: String,
}

impl Message {
    pub fn from_websocket(data: &serde_json::Value) -> Option<Self> {
        let chatroom_id = data.get("chatroomId")?.as_str()?;
        let message_data = data.get("message")?;

        let message_id = message_data.get("messageId")?.as_str()?.to_string();
        let sent_by = message_data
            .get("sentBy")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let sent_at = message_data
            .get("sentAt")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let content = message_data.get("content")?;
        let content_type = content.get("type").and_then(|v| v.as_str()).map(|s| s.to_string());
        let content_text = content
            .get("text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Some(Self {
            message_id,
            room_id: chatroom_id.to_string(),
            sent_by,
            content_text,
            content_type,
            sent_at,
            is_deleted: false,
            is_recalled: false,
            is_edited: false,
            history: None,
            raw_data: data.clone(),
            source: "spider".to_string(),
        })
    }

    pub fn add_to_history(&mut self, old_content: String) {
        let history = self.history.get_or_insert(serde_json::Value::Array(vec![]));
        if let Some(arr) = history.as_array_mut() {
            arr.push(serde_json::json!({
                "content": old_content,
                "edited_at": Utc::now().to_rfc3339()
            }));
        }
        self.is_edited = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_websocket() {
        let data = serde_json::json!({
            "chatroomId": "room123",
            "message": {
                "messageId": "msg456",
                "sentBy": "user789",
                "sentAt": "2026-01-01T00:00:00Z",
                "content": {
                    "type": "text",
                    "text": "hello"
                }
            }
        });
        let msg = Message::from_websocket(&data).unwrap();
        assert_eq!(msg.message_id, "msg456");
        assert_eq!(msg.room_id, "room123");
        assert_eq!(msg.sent_by.as_deref(), Some("user789"));
        assert_eq!(msg.content_type.as_deref(), Some("text"));
        assert_eq!(msg.content_text.as_deref(), Some("hello"));
    }

    #[test]
    fn test_add_to_history() {
        let mut msg = Message {
            message_id: "m1".into(),
            room_id: "r1".into(),
            sent_by: None,
            content_text: Some("new".into()),
            content_type: Some("text".into()),
            sent_at: None,
            is_deleted: false,
            is_recalled: false,
            is_edited: false,
            history: None,
            raw_data: serde_json::json!({}),
            source: "spider".into(),
        };
        msg.add_to_history("old".into());
        assert!(msg.is_edited);
        let history = msg.history.unwrap();
        assert_eq!(history.as_array().unwrap().len(), 1);
    }
}
