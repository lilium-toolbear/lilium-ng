use anyhow::{Context, Result};
use chrono::Utc;
use futures::{future::BoxFuture, StreamExt};
use lilium_models::dzmm::message::Message;
use lilium_models::ingestion::EventEnvelope;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        http::{header::COOKIE, Request},
        Message as WsMessage,
    },
};
use tracing::instrument;
use tracing::{debug, error, info, warn};

pub struct WebSocketEventDecoder;

impl WebSocketEventDecoder {
    pub fn decode_data(data: &serde_json::Value) -> Option<serde_json::Value> {
        match data {
            serde_json::Value::String(s) => serde_json::from_str(s).ok(),
            serde_json::Value::Array(arr) => {
                if arr.len() == 1 {
                    Self::decode_data(&arr[0])
                } else {
                    Some(data.clone())
                }
            }
            serde_json::Value::Object(_) => Some(data.clone()),
            serde_json::Value::Null => None,
            _ => Some(data.clone()),
        }
    }

    pub fn classify_event(event_data: &serde_json::Value) -> (String, bool) {
        let event_type = event_data
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let is_room_message = matches!(
            event_type.as_str(),
            "message" | "message:new" | "message:updated" | "message:deleted" | "message:recalled"
        );

        (event_type, is_room_message)
    }

    pub fn extract_room_id(event_data: &serde_json::Value) -> Option<String> {
        let top = event_data
            .get("roomId")
            .or_else(|| event_data.get("chatroomId"))
            .and_then(|v| v.as_str());
        if let Some(id) = top {
            return Some(id.to_string());
        }

        let data_arr = event_data.get("data")?;
        let first = data_arr.as_array()?.first()?;
        let first_obj = first.as_object()?;

        first_obj
            .get("chatroomId")
            .or_else(|| first_obj.get("roomId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                first_obj
                    .get("message")
                    .and_then(|m| m.as_object())
                    .and_then(|m| m.get("chatroom_id"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
    }

    pub fn extract_structured_message(event_data: &serde_json::Value) -> Option<serde_json::Value> {
        let data_arr = event_data.get("data")?.as_array()?;
        let first = data_arr.first()?;
        first.get("message").cloned()
    }

    pub fn decode_message(event_data: &serde_json::Value) -> Option<Message> {
        let decoded = Self::decode_data(event_data)?;
        Message::from_websocket(&decoded)
    }

    pub fn event_to_message(
        event_data: &serde_json::Value,
        room_id: Option<&str>,
    ) -> Result<Message, String> {
        let extracted = event_data
            .get("roomId")
            .or_else(|| event_data.get("room_id"))
            .and_then(|v| v.as_str());
        let final_room_id = room_id.or(extracted);
        let final_room_id =
            final_room_id.ok_or("room_id must be provided or present in event_data")?;

        let mut data = event_data.clone();
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "chatroomId".to_string(),
                serde_json::Value::String(final_room_id.to_string()),
            );
        }

        Message::from_websocket(&data).ok_or("Failed to parse message from event data".to_string())
    }

    pub fn is_deletion_event(event_type: &str) -> bool {
        event_type == "message:deleted"
    }

    pub fn is_update_event(event_type: &str) -> bool {
        event_type == "message:updated" || event_type == "message:recalled"
    }

    pub fn is_new_message_event(event_type: &str) -> bool {
        event_type == "message" || event_type == "message:new"
    }
}

pub struct WsClient {
    account_id: String,
    url: String,
    cookie_header: Option<String>,
    session_id: Option<String>,
}

impl WsClient {
    #[instrument(fields(account_id = %account_id, has_cookie_header = cookie_header.is_some()))]
    pub fn new(account_id: String, url: String, cookie_header: Option<String>) -> Self {
        Self {
            account_id,
            url,
            cookie_header,
            session_id: None,
        }
    }

    fn build_request(&self) -> Result<Request<()>> {
        let mut builder = Request::builder().uri(&self.url);
        if let Some(cookie_header) = &self.cookie_header {
            builder = builder.header(COOKIE, cookie_header);
        }
        builder
            .body(())
            .context("Failed to build WebSocket request")
    }

    #[instrument(skip(self, on_event, shutdown, shutdown_notify, reconnect_notify), fields(account_id = %self.account_id, url = %self.url))]
    pub async fn run<F>(
        &mut self,
        mut on_event: F,
        shutdown: Arc<AtomicBool>,
        shutdown_notify: Arc<Notify>,
        reconnect_notify: Arc<Notify>,
    ) -> Result<()>
    where
        F: FnMut(EventEnvelope) -> BoxFuture<'static, ()> + Send,
    {
        info!(account = %self.account_id, url = %self.url, "Connecting to WebSocket");

        let request = self.build_request()?;
        let (ws_stream, _) = connect_async(request)
            .await
            .context("Failed to connect to WebSocket")?;

        let (_, mut read) = ws_stream.split();

        info!(account = %self.account_id, "WebSocket connected");

        loop {
            if shutdown.load(Ordering::Relaxed) {
                info!(account = %self.account_id, "Shutdown requested");
                break;
            }

            tokio::select! {
                _ = shutdown_notify.notified() => {
                    info!(account = %self.account_id, "WebSocket shutdown signalled");
                    break;
                }
                _ = reconnect_notify.notified() => {
                    info!(account = %self.account_id, "WebSocket reconnect requested");
                    break;
                }
                msg = read.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => match self.handle_message(&text).await {
                            Ok(Some(event)) => {
                                on_event(event).await;
                            }
                            Ok(None) => {}
                            Err(e) => {
                                warn!(account = %self.account_id, error = %e, "Failed to handle message");
                            }
                        },
                        Some(Ok(WsMessage::Binary(data))) => {
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                match self.handle_message(&text).await {
                                    Ok(Some(event)) => {
                                        on_event(event).await;
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        warn!(account = %self.account_id, error = %e, "Failed to handle binary message");
                                    }
                                }
                            }
                        }
                        Some(Ok(WsMessage::Ping(data))) => {
                            debug!(account = %self.account_id, len = data.len(), "Received ping");
                        }
                        Some(Ok(WsMessage::Pong(_))) => {
                            debug!(account = %self.account_id, "Received pong");
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            info!(account = %self.account_id, "WebSocket closed");
                            break;
                        }
                        Some(Err(e)) => {
                            error!(account = %self.account_id, error = %e, "WebSocket error");
                            break;
                        }
                        None => break,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    #[instrument(skip(self, raw), fields(account_id = %self.account_id, raw_len = raw.len()))]
    async fn handle_message(&mut self, raw: &str) -> Result<Option<EventEnvelope>> {
        let trimmed = raw.trim();

        if trimmed == "2" {
            debug!(account = %self.account_id, "Received ping, sending pong");
            return Ok(None);
        }

        if trimmed == "3" {
            debug!(account = %self.account_id, "Received pong");
            return Ok(None);
        }

        if let Some(json_part) = trimmed.strip_prefix("40") {
            let session_id = if trimmed.len() > 2 {
                if json_part.starts_with('{') {
                    serde_json::from_str::<serde_json::Value>(json_part)
                        .ok()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                } else {
                    let quoted = format!("\"{}\"", json_part);
                    serde_json::from_str::<serde_json::Value>(&quoted)
                        .ok()
                        .and_then(|v| v.as_str().map(|s| s.to_string()))
                }
            } else {
                None
            };

            if let Some(sid) = session_id {
                self.session_id = Some(sid.clone());
                info!(account = %self.account_id, session_id = %sid, "Socket.IO connected");
            } else {
                info!(account = %self.account_id, "Socket.IO connected (no session ID)");
            }
            return Ok(None);
        }

        if trimmed.starts_with("43") {
            debug!(account = %self.account_id, "Socket.IO connect acknowledgement");
            return Ok(None);
        }

        if trimmed.starts_with('4') && trimmed.len() > 1 && trimmed.as_bytes()[1] == b'2' {
            let json_str = &trimmed[2..];
            let parsed: serde_json::Value = serde_json::from_str(json_str)
                .context("Failed to parse Socket.IO event payload")?;

            if let Some(arr) = parsed.as_array() {
                if arr.len() >= 2 {
                    let event_name = arr[0].as_str().unwrap_or("unknown").to_string();
                    let event_data = &arr[1];

                    let decoded = WebSocketEventDecoder::decode_data(event_data)
                        .unwrap_or_else(|| event_data.clone());

                    let (classified_type, is_room_msg) =
                        WebSocketEventDecoder::classify_event(&decoded);

                    if is_room_msg {
                        if let Some(message) = WebSocketEventDecoder::decode_message(&decoded) {
                            return Ok(Some(EventEnvelope {
                                account_user_id: self.account_id.clone(),
                                event_type: classified_type,
                                payload: serde_json::to_value(&message)
                                    .unwrap_or(serde_json::Value::Null),
                                received_at: Utc::now(),
                                source: "socket".to_string(),
                            }));
                        }
                    }

                    return Ok(Some(EventEnvelope {
                        account_user_id: self.account_id.clone(),
                        event_type: event_name,
                        payload: decoded,
                        received_at: Utc::now(),
                        source: "socket".to_string(),
                    }));
                }
            }

            return Ok(None);
        }

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
            let (event_type, _) = WebSocketEventDecoder::classify_event(&val);
            return Ok(Some(EventEnvelope {
                account_user_id: self.account_id.clone(),
                event_type,
                payload: val,
                received_at: Utc::now(),
                source: "socket".to_string(),
            }));
        }

        debug!(account = %self.account_id, raw = %trimmed, "Unrecognized message format");
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_data_string() {
        let data = serde_json::json!("\"hello\"");
        let result = WebSocketEventDecoder::decode_data(&data);
        assert_eq!(result, Some(serde_json::json!("hello")));
    }

    #[test]
    fn test_decode_data_object() {
        let data = serde_json::json!({"key": "value"});
        let result = WebSocketEventDecoder::decode_data(&data);
        assert_eq!(result, Some(data.clone()));
    }

    #[test]
    fn test_decode_data_array_single() {
        let data = serde_json::json!([{"key": "value"}]);
        let result = WebSocketEventDecoder::decode_data(&data);
        assert_eq!(result, Some(serde_json::json!({"key": "value"})));
    }

    #[test]
    fn test_classify_event_room_message() {
        let data = serde_json::json!({"event": "message", "chatroomId": "room1", "message": {}});
        let (event_type, is_room) = WebSocketEventDecoder::classify_event(&data);
        assert!(is_room);
        assert_eq!(event_type, "message");
    }

    #[test]
    fn test_classify_event_new_message() {
        let data = serde_json::json!({"event": "message:new"});
        let (event_type, is_room) = WebSocketEventDecoder::classify_event(&data);
        assert_eq!(event_type, "message:new");
        assert!(is_room);
    }

    #[test]
    fn test_classify_event_updated() {
        let data = serde_json::json!({"event": "message:updated"});
        let (event_type, is_room) = WebSocketEventDecoder::classify_event(&data);
        assert_eq!(event_type, "message:updated");
        assert!(is_room);
    }

    #[test]
    fn test_classify_event_deleted() {
        let data = serde_json::json!({"event": "message:deleted"});
        let (_, is_room) = WebSocketEventDecoder::classify_event(&data);
        assert!(is_room);
    }

    #[test]
    fn test_classify_event_recalled() {
        let data = serde_json::json!({"event": "message:recalled"});
        let (_, is_room) = WebSocketEventDecoder::classify_event(&data);
        assert!(is_room);
    }

    #[test]
    fn test_classify_event_non_room() {
        let data = serde_json::json!({"event": "typing"});
        let (event_type, is_room) = WebSocketEventDecoder::classify_event(&data);
        assert_eq!(event_type, "typing");
        assert!(!is_room);
    }

    #[test]
    fn test_extract_room_id_variants() {
        assert_eq!(
            WebSocketEventDecoder::extract_room_id(&serde_json::json!({"chatroomId": "r1"})),
            Some("r1".into())
        );
        assert_eq!(
            WebSocketEventDecoder::extract_room_id(&serde_json::json!({"roomId": "r2"})),
            Some("r2".into())
        );
        assert_eq!(
            WebSocketEventDecoder::extract_room_id(
                &serde_json::json!({"data": [{"chatroomId": "r4"}]})
            ),
            Some("r4".into())
        );
        assert_eq!(
            WebSocketEventDecoder::extract_room_id(
                &serde_json::json!({"data": [{"roomId": "r5"}]})
            ),
            Some("r5".into())
        );
        assert_eq!(
            WebSocketEventDecoder::extract_room_id(
                &serde_json::json!({"data": [{"message": {"chatroom_id": "r6"}}]})
            ),
            Some("r6".into())
        );
        assert_eq!(
            WebSocketEventDecoder::extract_room_id(&serde_json::json!({})),
            None
        );
    }

    #[test]
    fn test_extract_structured_message() {
        let data = serde_json::json!({"data": [{"message": {"messageId": "m1"}}]});
        let result = WebSocketEventDecoder::extract_structured_message(&data);
        assert_eq!(result, Some(serde_json::json!({"messageId": "m1"})));
    }

    #[test]
    fn test_extract_structured_message_none() {
        let data = serde_json::json!({"data": [{"no_message": true}]});
        let result = WebSocketEventDecoder::extract_structured_message(&data);
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_message() {
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
        let msg = WebSocketEventDecoder::decode_message(&data).unwrap();
        assert_eq!(msg.message_id, "msg456");
        assert_eq!(msg.room_id, "room123");
    }

    #[test]
    fn test_event_to_message_with_room_id() {
        let data = serde_json::json!({
            "message": {
                "messageId": "msg789",
                "sentBy": "user1",
                "sentAt": "2026-01-01T00:00:00Z",
                "content": {
                    "type": "text",
                    "text": "test"
                }
            }
        });
        let msg = WebSocketEventDecoder::event_to_message(&data, Some("room456")).unwrap();
        assert_eq!(msg.message_id, "msg789");
        assert_eq!(msg.room_id, "room456");
    }

    #[test]
    fn test_event_to_message_from_envelope() {
        let data = serde_json::json!({
            "roomId": "room789",
            "message": {
                "messageId": "msg101",
                "sentBy": "user2",
                "sentAt": "2026-01-01T00:00:00Z",
                "content": {
                    "type": "text",
                    "text": "test"
                }
            }
        });
        let msg = WebSocketEventDecoder::event_to_message(&data, None).unwrap();
        assert_eq!(msg.message_id, "msg101");
        assert_eq!(msg.room_id, "room789");
    }

    #[test]
    fn test_event_to_message_no_room_id() {
        let data = serde_json::json!({"message": {"messageId": "x"}});
        let result = WebSocketEventDecoder::event_to_message(&data, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_deletion_event() {
        assert!(WebSocketEventDecoder::is_deletion_event("message:deleted"));
        assert!(!WebSocketEventDecoder::is_deletion_event("message:new"));
    }

    #[test]
    fn test_is_update_event() {
        assert!(WebSocketEventDecoder::is_update_event("message:updated"));
        assert!(WebSocketEventDecoder::is_update_event("message:recalled"));
        assert!(!WebSocketEventDecoder::is_update_event("message:new"));
    }

    #[test]
    fn test_is_new_message_event() {
        assert!(WebSocketEventDecoder::is_new_message_event("message"));
        assert!(WebSocketEventDecoder::is_new_message_event("message:new"));
        assert!(!WebSocketEventDecoder::is_new_message_event(
            "message:updated"
        ));
    }
}
