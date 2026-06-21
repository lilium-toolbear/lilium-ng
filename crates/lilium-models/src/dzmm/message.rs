use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Python parity source: dzmm_archive@227bc1179 models/dzmm/message.py
// Parity decisions:
// - Python exposes `content_tsv` (TSVECTOR), but Rust intentionally keeps PostgreSQL
//   search vectors out of the table row model. The DB column still exists and is
//   only referenced from search predicates.
// - Python's `message_metadata` maps to DB column `metadata`; Rust field is named `metadata`.
// - Python `add_to_history()` also sets `updated_at = utc_now()`; Rust does not
//   (no `updated_at` mutation in the Rust helper).
// - `message_id`/`room_id`/`reference_message_id`/`sent_by`/`deleted_by` are all
//   UUID after the message-id chain migration (Python c4e5f6a7b8c9).
// - Python `from_api()` parses `UUID(str(data["message_id"]))`, `UUID(str(room_id))`,
//   `UUID(str(reference["id"]))` and raises on a non-UUID id. Rust's boundary
//   parsers return `Option` and skip the message on parse failure (`.ok()?`)
//   rather than raising. Post-migration DZMM only emits UUID-shaped ids, so this
//   divergence is not exercised in practice.
pub type Message = Model;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: Uuid,
    pub room_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub sent_at: DateTime<Utc>,
    pub sent_by: Uuid,
    pub content_type: String,
    pub content_text: Option<String>,
    pub attachment_url: Option<String>,
    pub attachment_file: Option<String>,
    pub sticker_id: Option<String>,
    pub alt_text: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_data: serde_json::Value,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub is_recalled: bool,
    pub is_edited: bool,
    pub history: Option<serde_json::Value>,
    pub reference_message_id: Option<Uuid>,
    pub reference_data: Option<serde_json::Value>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Message {
    pub fn from_websocket(data: &serde_json::Value) -> Option<Self> {
        let event_data = if data.get("data").is_some() {
            data.get("data")?.as_array()?.first()?
        } else {
            data
        };

        let message_data = event_data.get("message")?;
        let chatroom_id = event_data
            .get("chatroomId")
            .or_else(|| message_data.get("chatroom_id"))?
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())?;

        let message_id = message_data
            .get("message_id")
            .or_else(|| message_data.get("messageId"))?
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let sent_by = message_data
            .get("sent_by")
            .or_else(|| message_data.get("sentBy"))
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let sent_at = message_data
            .get("sent_at")
            .or_else(|| message_data.get("sentAt"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let content = message_data.get("content")?;
        let content_type = content
            .get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "text".to_string());
        let content_text = content
            .get("text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let attachment_url = content
            .get("url")
            .or_else(|| content.get("videoUrl"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let alt_text = content
            .get("alt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let sticker_id = content
            .get("stickerId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let reference_data = content.get("reference").filter(|v| v.is_object()).cloned();
        let reference_message_id = reference_data
            .as_ref()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        let mut metadata = message_data
            .get("metadata")
            .filter(|v| v.is_object())
            .cloned();
        if content_type == "video" {
            let mut video = serde_json::Map::new();
            for key in ["width", "height", "status", "blurhash", "duration"] {
                if let Some(value) = content.get(key).filter(|v| !v.is_null()) {
                    video.insert(key.to_string(), value.clone());
                }
            }
            if let Some(value) = content.get("thumbnailUrl").filter(|v| !v.is_null()) {
                video.insert("thumbnail_url".to_string(), value.clone());
            }
            if !video.is_empty() {
                let mut map = metadata
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();
                map.insert("video".to_string(), serde_json::Value::Object(video));
                metadata = Some(serde_json::Value::Object(map));
            }
        }

        let now = Utc::now();
        Some(Self {
            message_id,
            room_id: chatroom_id,
            sent_at,
            sent_by,
            content_type,
            content_text,
            attachment_url,
            attachment_file: None,
            sticker_id,
            alt_text,
            metadata,
            raw_data: event_data.clone(),
            source: "spider".to_string(),
            created_at: now,
            updated_at: None,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
            is_recalled: false,
            is_edited: false,
            history: None,
            reference_message_id,
            reference_data,
        })
    }

    /// Create a Message from a REST API history response dict. Mirrors Python
    /// `Message.from_api`. The `data` dict has top-level `message_id`,
    /// `sent_by`, `sent_at`/`sentAt`, `content`, and optional `metadata`/
    /// `attachment_file`.
    pub fn from_api(data: &serde_json::Value, room_id: Uuid) -> Option<Self> {
        let message_id = data
            .get("message_id")?
            .as_str()
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let sent_by = data
            .get("sent_by")
            .or_else(|| data.get("sentBy"))
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())?;
        let sent_at = data
            .get("sent_at")
            .or_else(|| data.get("sentAt"))
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let content = data.get("content").unwrap_or(&serde_json::Value::Null);
        let content_type = content
            .get("type")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| "text".to_string());
        let content_text = content
            .get("text")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let attachment_url = content
            .get("url")
            .or_else(|| content.get("videoUrl"))
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let alt_text = content
            .get("alt")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let sticker_id = content
            .get("stickerId")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        let reference_data = content.get("reference").filter(|v| v.is_object()).cloned();
        let reference_message_id = reference_data
            .as_ref()
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        let mut metadata = data.get("metadata").filter(|v| v.is_object()).cloned();
        if content_type == "video" {
            let mut video = serde_json::Map::new();
            for key in ["width", "height", "status", "blurhash", "duration"] {
                if let Some(value) = content.get(key).filter(|v| !v.is_null()) {
                    video.insert(key.to_string(), value.clone());
                }
            }
            if let Some(value) = content.get("thumbnailUrl").filter(|v| !v.is_null()) {
                video.insert("thumbnail_url".to_string(), value.clone());
            }
            if !video.is_empty() {
                let mut map = metadata
                    .and_then(|v| v.as_object().cloned())
                    .unwrap_or_default();
                map.insert("video".to_string(), serde_json::Value::Object(video));
                metadata = Some(serde_json::Value::Object(map));
            }
        }

        let now = Utc::now();
        Some(Self {
            message_id,
            room_id,
            sent_at,
            sent_by,
            content_type,
            content_text,
            attachment_url,
            attachment_file: data
                .get("attachment_file")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            sticker_id,
            alt_text,
            metadata,
            raw_data: data.clone(),
            source: "api".to_string(),
            created_at: now,
            updated_at: None,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
            is_recalled: false,
            is_edited: false,
            history: None,
            reference_message_id,
            reference_data,
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

    // Post-migration DZMM user ids are UUID-shaped; fixtures use this constant.
    const TEST_USER_UUID: &str = "00000000-0000-4000-8000-000000000001";
    fn test_user() -> Uuid {
        Uuid::parse_str(TEST_USER_UUID).unwrap()
    }

    // Deterministic name -> UUID for room/message/reference ids in tests.
    fn test_uuid(name: &str) -> Uuid {
        Uuid::new_v5(&Uuid::NAMESPACE_OID, name.as_bytes())
    }

    fn sample_wrapped_text_event(i: u32) -> serde_json::Value {
        serde_json::json!({
            "event": "message:new",
            "data": [{
                "chatroomId": test_uuid(&format!("text-room-{i}")).to_string(),
                "message": {
                    "message_id": test_uuid(&format!("text-msg-{i}")).to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "text",
                        "text": format!("message #{i}"),
                    },
                },
            }],
        })
    }

    fn sample_wrapped_image_event(i: u32) -> serde_json::Value {
        serde_json::json!({
            "event": "message:new",
            "data": [{
                "chatroomId": test_uuid(&format!("image-room-{i}")).to_string(),
                "message": {
                    "message_id": test_uuid(&format!("image-msg-{i}")).to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "image",
                        "url": format!("https://example.com/image-{i}.png"),
                        "alt": format!("image alt {i}"),
                    },
                },
            }],
        })
    }

    fn sample_wrapped_sticker_event(i: u32) -> serde_json::Value {
        serde_json::json!({
            "event": "message:new",
            "data": [{
                "chatroomId": test_uuid(&format!("sticker-room-{i}")).to_string(),
                "message": {
                    "message_id": test_uuid(&format!("sticker-msg-{i}")).to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "sticker",
                        "stickerId": format!("sticker-{i}"),
                        "url": format!("https://example.com/sticker-{i}.png"),
                    },
                },
            }],
        })
    }

    fn sample_wrapped_text_with_reference_event(i: u32) -> serde_json::Value {
        serde_json::json!({
            "event": "message:new",
            "data": [{
                "chatroomId": test_uuid(&format!("reference-room-{i}")).to_string(),
                "message": {
                    "message_id": test_uuid(&format!("reference-msg-{i}")).to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "text",
                        "text": format!("message with reference #{i}"),
                        "reference": {
                            "id": test_uuid(&format!("ref-{i}")).to_string(),
                            "sentBy": "user_ref",
                            "content": {
                                "type": "text",
                                "text": "original",
                            },
                        }
                    }
                },
            }],
        })
    }

    #[test]
    fn test_from_websocket() {
        let data = serde_json::json!({
                "chatroomId": test_uuid("room123").to_string(),
            "message": {
                "messageId": test_uuid("msg456").to_string(),
                "sentBy": TEST_USER_UUID,
                "sentAt": "2026-01-01T00:00:00Z",
                "content": {
                    "type": "text",
                    "text": "hello"
                }
            }
        });
        let msg = Message::from_websocket(&data).unwrap();
        assert_eq!(msg.message_id, test_uuid("msg456"));
        assert_eq!(msg.room_id, test_uuid("room123"));
        assert_eq!(msg.sent_by, test_user());
        assert_eq!(msg.content_type, "text");
        assert_eq!(msg.content_text.as_deref(), Some("hello"));
        assert_eq!(msg.source, "spider");
    }

    #[test]
    fn test_from_websocket_wrapped_event_uses_python_fixture_shape() {
        let data = serde_json::json!({
            "event": "message:new",
            "data": [{
                "chatroomId": test_uuid("room_wrapped").to_string(),
                "message": {
                    "message_id": test_uuid("msg_wrapped").to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-04-08T16:58:28.961Z",
                    "content": {
                        "type": "text",
                        "text": "wrapped hello"
                    }
                }
            }]
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.message_id, test_uuid("msg_wrapped"));
        assert_eq!(msg.room_id, test_uuid("room_wrapped"));
        assert_eq!(msg.sent_by, test_user());
        assert_eq!(msg.content_type, "text");
        assert_eq!(msg.content_text.as_deref(), Some("wrapped hello"));
        assert_eq!(
            msg.raw_data["chatroomId"],
            test_uuid("room_wrapped").to_string()
        );
    }

    #[test]
    fn test_from_websocket_system_message() {
        let data = serde_json::json!({
            "event": "message:new",
            "data": [{
                "chatroomId": test_uuid("room_system").to_string(),
                "message": {
                    "message_id": test_uuid("msg_system").to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "system",
                        "text": "游客 加入了群聊"
                    }
                }
            }]
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.content_type, "system");
        assert_eq!(msg.content_text.as_deref(), Some("游客 加入了群聊"));
        assert_eq!(msg.sent_by, test_user());
    }

    #[test]
    fn test_from_websocket_recalled_message() {
        let data = serde_json::json!({
            "event": "message:updated",
            "data": [{
                "chatroomId": test_uuid("room_recalled").to_string(),
                "message": {
                    "message_id": test_uuid("msg_recalled").to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "recalled"
                    }
                }
            }]
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.content_type, "recalled");
        assert!(msg.content_text.is_none());
    }

    #[test]
    fn test_from_websocket_edited_message() {
        let data = serde_json::json!({
            "event": "message:updated",
            "data": [{
                "chatroomId": test_uuid("room_edited").to_string(),
                "message": {
                    "message_id": test_uuid("msg_edited").to_string(),
                    "sent_by": TEST_USER_UUID,
                    "sent_at": "2026-01-01T00:00:00Z",
                    "content": {
                        "type": "text",
                        "text": "edited text",
                        "edited": true
                    }
                }
            }]
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.content_type, "text");
        assert_eq!(msg.content_text.as_deref(), Some("edited text"));
    }

    #[test]
    fn test_from_websocket_invalid_event_missing_data_returns_none() {
        let invalid_event = serde_json::json!({
            "event": "message:new"
        });

        assert!(Message::from_websocket(&invalid_event).is_none());
    }

    #[test]
    fn test_from_websocket_invalid_event_data_not_array_returns_none() {
        let invalid_event = serde_json::json!({
            "event": "message:new",
            "data": {"chatroomId": "some-room", "message": {"message_id": "1"}}
        });

        assert!(Message::from_websocket(&invalid_event).is_none());
    }

    #[test]
    fn test_from_websocket_invalid_event_missing_message_returns_none() {
        let invalid_event = serde_json::json!({
            "event": "message:new",
            "data": [{"chatroomId": "some-room"}]
        });

        assert!(Message::from_websocket(&invalid_event).is_none());
    }

    #[test]
    fn test_from_websocket_image_extracts_attachment_url_and_alt_text() {
        let data = serde_json::json!({
                "chatroomId": test_uuid("room_image").to_string(),
            "message": {
                "message_id": test_uuid("msg_image").to_string(),
                "sent_by": TEST_USER_UUID,
                "sent_at": "2026-04-08T16:58:28.961Z",
                "content": {
                    "type": "image",
                    "url": "https://example.com/image.png",
                    "alt": "image alt"
                }
            }
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.content_type, "image");
        assert_eq!(
            msg.attachment_url.as_deref(),
            Some("https://example.com/image.png")
        );
        assert_eq!(msg.alt_text.as_deref(), Some("image alt"));
    }

    #[test]
    fn test_from_websocket_video_extracts_url_and_metadata() {
        let data = serde_json::json!({
            "chatroomId": test_uuid("room_video").to_string(),
            "message": {
                "message_id": test_uuid("msg_video").to_string(),
                "chatroom_id": test_uuid("room_video").to_string(),
                "sent_by": TEST_USER_UUID,
                "sent_at": "2026-04-08T16:58:28.961Z",
                "metadata": null,
                "content": {
                    "type": "video",
                    "width": 406,
                    "height": 720,
                    "status": "ready",
                    "blurhash": "LKJHHwx]57In~WxG-pNGogslxZjF",
                    "duration": 51.5,
                    "videoUrl": "https://example.com/video.mp4",
                    "thumbnailUrl": "https://example.com/video-thumbnail.jpg"
                }
            }
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.content_type, "video");
        assert_eq!(
            msg.attachment_url.as_deref(),
            Some("https://example.com/video.mp4")
        );
        assert_eq!(
            msg.metadata.as_ref().unwrap()["video"]["duration"],
            serde_json::json!(51.5)
        );
        assert_eq!(
            msg.metadata.as_ref().unwrap()["video"]["thumbnail_url"],
            serde_json::json!("https://example.com/video-thumbnail.jpg")
        );
    }

    #[test]
    fn test_from_websocket_sticker_extracts_sticker_id_and_url() {
        let data = serde_json::json!({
            "chatroomId": test_uuid("room_sticker").to_string(),
            "message": {
                "message_id": test_uuid("msg_sticker").to_string(),
                "sent_by": TEST_USER_UUID,
                "sent_at": "2026-04-08T16:58:28.961Z",
                "content": {
                    "type": "sticker",
                    "stickerId": "sticker_123",
                    "url": "https://example.com/sticker.webp"
                }
            }
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.content_type, "sticker");
        assert_eq!(msg.sticker_id.as_deref(), Some("sticker_123"));
        assert_eq!(
            msg.attachment_url.as_deref(),
            Some("https://example.com/sticker.webp")
        );
    }

    #[test]
    fn test_from_websocket_text_extracts_reference() {
        let data = serde_json::json!({
                "chatroomId": test_uuid("room_reference").to_string(),
                "message": {
                "message_id": test_uuid("msg_reference").to_string(),
                "sent_by": TEST_USER_UUID,
                "sent_at": "2026-04-08T16:58:28.961Z",
                "content": {
                    "type": "text",
                    "text": "reply",
                    "reference": {
                        "id": test_uuid("msg_original").to_string(),
                        "content": {
                            "type": "text",
                            "text": "original"
                        }
                    }
                }
            }
        });

        let msg = Message::from_websocket(&data).unwrap();

        assert_eq!(msg.reference_message_id, Some(test_uuid("msg_original")));
        assert_eq!(
            msg.reference_data.as_ref().unwrap()["id"],
            test_uuid("msg_original").to_string()
        );
        assert_eq!(
            msg.reference_data.as_ref().unwrap()["content"]["text"],
            "original"
        );
    }

    #[test]
    fn test_from_websocket_all_text_samples() {
        for i in 0..10u32 {
            let msg = Message::from_websocket(&sample_wrapped_text_event(i)).unwrap();

            assert_eq!(msg.content_type, "text");
            assert_eq!(msg.content_text, Some(format!("message #{i}")));
            assert_eq!(msg.room_id, test_uuid(&format!("text-room-{i}")));
            assert_eq!(msg.message_id, test_uuid(&format!("text-msg-{i}")));
        }
    }

    #[test]
    fn test_from_websocket_all_image_samples() {
        for i in 0..10u32 {
            let msg = Message::from_websocket(&sample_wrapped_image_event(i)).unwrap();

            assert_eq!(msg.content_type, "image");
            assert_eq!(
                msg.attachment_url,
                Some(format!("https://example.com/image-{i}.png"))
            );
            assert_eq!(msg.room_id, test_uuid(&format!("image-room-{i}")));
            assert_eq!(msg.message_id, test_uuid(&format!("image-msg-{i}")));
        }
    }

    #[test]
    fn test_from_websocket_all_sticker_samples() {
        for i in 0..10u32 {
            let msg = Message::from_websocket(&sample_wrapped_sticker_event(i)).unwrap();

            assert_eq!(msg.content_type, "sticker");
            assert_eq!(msg.sticker_id, Some(format!("sticker-{i}")));
            assert_eq!(
                msg.attachment_url,
                Some(format!("https://example.com/sticker-{i}.png"))
            );
            assert_eq!(msg.room_id, test_uuid(&format!("sticker-room-{i}")));
            assert_eq!(msg.message_id, test_uuid(&format!("sticker-msg-{i}")));
        }
    }

    #[test]
    fn test_from_websocket_all_reference_samples() {
        for i in 0..10u32 {
            let msg =
                Message::from_websocket(&sample_wrapped_text_with_reference_event(i)).unwrap();
            let expected_ref = test_uuid(&format!("ref-{i}"));

            assert_eq!(msg.content_type, "text");
            assert_eq!(msg.reference_message_id, Some(expected_ref));
            assert_eq!(
                msg.reference_data.as_ref().unwrap()["id"].as_str(),
                Some(expected_ref.to_string().as_str())
            );
            assert_eq!(
                msg.reference_data.as_ref().unwrap()["content"]["text"],
                "original"
            );
            assert_eq!(msg.room_id, test_uuid(&format!("reference-room-{i}")));
            assert_eq!(msg.message_id, test_uuid(&format!("reference-msg-{i}")));
        }
    }

    #[test]
    fn test_add_to_history() {
        let now = Utc::now();
        let mut msg = Message {
            message_id: test_uuid("m1"),
            room_id: test_uuid("r1"),
            sent_at: now,
            sent_by: test_user(),
            content_type: "text".into(),
            content_text: Some("new".into()),
            attachment_url: None,
            attachment_file: None,
            sticker_id: None,
            alt_text: None,
            metadata: None,
            raw_data: serde_json::json!({}),
            source: "spider".into(),
            created_at: now,
            updated_at: None,
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
            is_recalled: false,
            is_edited: false,
            history: None,
            reference_message_id: None,
            reference_data: None,
        };
        msg.add_to_history("old".into());
        assert!(msg.is_edited);
        let history = msg.history.unwrap();
        assert_eq!(history.as_array().unwrap().len(), 1);
    }

    #[test]
    fn from_api_parses_rest_history_dict() {
        let data = serde_json::json!({
            "message_id": test_uuid("msg-1").to_string(),
            "sent_by": TEST_USER_UUID,
            "sent_at": "2024-11-16T04:27:00.123Z",
            "content": {"type": "image", "url": "https://x/img.png", "alt": "hi"},
            "metadata": {"k": "v"},
        });
        let m = Message::from_api(&data, test_uuid("room-1")).expect("parse");
        assert_eq!(m.message_id, test_uuid("msg-1"));
        assert_eq!(m.room_id, test_uuid("room-1"));
        assert_eq!(m.sent_by, test_user());
        assert_eq!(m.content_type, "image");
        assert_eq!(m.attachment_url.as_deref(), Some("https://x/img.png"));
        assert_eq!(m.alt_text.as_deref(), Some("hi"));
        assert_eq!(m.source, "api");
        assert_eq!(m.metadata.unwrap().get("k").unwrap().as_str().unwrap(), "v");
    }

    #[test]
    fn from_api_returns_none_without_message_id() {
        let data = serde_json::json!({"sent_by": TEST_USER_UUID});
        assert!(Message::from_api(&data, test_uuid("r")).is_none());
    }
}
