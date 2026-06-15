use crate::config::dzmm_local_address_from_env;
use anyhow::{Context, Result};
use chrono::Utc;
use futures::{FutureExt, future::BoxFuture};
use lilium_models::dzmm::message::Message;
use lilium_models::ingestion::EventEnvelope;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;
use tracing::instrument;
use tracing::{error, info, warn};
use url::Url;

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
}

impl WsClient {
    #[instrument(fields(account_id = %account_id, has_cookie_header = cookie_header.is_some()))]
    pub fn new(account_id: String, url: String, cookie_header: Option<String>) -> Self {
        Self {
            account_id,
            url,
            cookie_header,
        }
    }

    fn build_socketio_client<F>(
        &self,
        on_event: Arc<tokio::sync::Mutex<F>>,
        disconnect_notify: Arc<Notify>,
        disconnect_state: Arc<AtomicBool>,
    ) -> Result<rust_socketio::asynchronous::ClientBuilder>
    where
        F: FnMut(EventEnvelope) -> BoxFuture<'static, ()> + Send + 'static,
    {
        let mut builder = rust_socketio::asynchronous::ClientBuilder::new(self.url.clone())
            .transport_type(rust_socketio::TransportType::Websocket)
            .reconnect(false);

        if let Some(local_address) = dzmm_local_address_from_env()? {
            builder = builder.local_address(local_address);
        }

        let cookie_header = self.cookie_header.clone().unwrap_or_default();
        let mut origin_url = Url::parse(&self.url).context("Invalid websocket URL")?;
        origin_url.set_path("");
        origin_url.set_query(None);
        origin_url.set_fragment(None);
        let origin = origin_url.to_string().trim_end_matches('/').to_string();
        let referer = format!("{origin}/chat");

        builder = builder
            .opening_header("Cookie", cookie_header)
            .opening_header("Origin", origin)
            .opening_header("Referer", referer)
            .opening_header(
                "User-Agent",
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0",
            )
            .opening_header("Accept-Encoding", "gzip, deflate, br")
            .opening_header("Accept-Language", "en-US,en;q=0.9")
            .opening_header("Cache-Control", "no-cache")
            .opening_header("Pragma", "no-cache");

        let account_id = self.account_id.clone();
        let on_event_connect = on_event.clone();
        builder = builder.on("connect", move |_, _| {
            let account_id = account_id.clone();
            let on_event = on_event_connect.clone();
            async move {
                info!(account = %account_id, "Connected to DZMM WebSocket");
                emit_envelope(
                    on_event,
                    EventEnvelope {
                        account_user_id: account_id,
                        event_type: "sio:connect".to_string(),
                        payload: serde_json::json!({}),
                        received_at: Utc::now(),
                        source: "socket".to_string(),
                    },
                )
                .await;
            }
            .boxed()
        });

        let account_id = self.account_id.clone();
        let on_event_disconnect = on_event.clone();
        let disconnect_notify_disconnect = disconnect_notify.clone();
        let disconnect_state_disconnect = disconnect_state.clone();
        builder = builder.on("disconnect", move |_, _| {
            let account_id = account_id.clone();
            let on_event = on_event_disconnect.clone();
            let disconnect_notify = disconnect_notify_disconnect.clone();
            let disconnect_state = disconnect_state_disconnect.clone();
            async move {
                warn!(account = %account_id, "Disconnected from DZMM WebSocket");
                disconnect_state.store(true, Ordering::Relaxed);
                emit_envelope(
                    on_event,
                    EventEnvelope {
                        account_user_id: account_id,
                        event_type: "sio:disconnect".to_string(),
                        payload: serde_json::json!({}),
                        received_at: Utc::now(),
                        source: "socket".to_string(),
                    },
                )
                .await;
                disconnect_notify.notify_waiters();
            }
            .boxed()
        });

        let account_id = self.account_id.clone();
        let on_event_error = on_event.clone();
        let disconnect_notify_error = disconnect_notify.clone();
        let disconnect_state_error = disconnect_state.clone();
        builder = builder.on("error", move |payload, _| {
            let account_id = account_id.clone();
            let on_event = on_event_error.clone();
            let disconnect_notify = disconnect_notify_error.clone();
            let disconnect_state = disconnect_state_error.clone();
            async move {
                let payload_value = socketio_payload_to_value(payload);
                error!(account = %account_id, error = %payload_value, "Socket.IO connection error");
                disconnect_state.store(true, Ordering::Relaxed);
                emit_envelope(
                    on_event,
                    EventEnvelope {
                        account_user_id: account_id,
                        event_type: "sio:connect_error".to_string(),
                        payload: serde_json::json!({ "error": payload_value }),
                        received_at: Utc::now(),
                        source: "socket".to_string(),
                    },
                )
                .await;
                disconnect_notify.notify_waiters();
            }
            .boxed()
        });

        let account_id = self.account_id.clone();
        builder = builder.on_any(move |event, payload, _| {
            let account_id = account_id.clone();
            let on_event = on_event.clone();
            async move {
                let event_name: String = event.into();
                let payload_value = socketio_payload_to_value(payload);
                let (classified_type, is_room_message) =
                    WebSocketEventDecoder::classify_event(&payload_value);

                let (event_type, payload) = if is_room_message {
                    if let Some(message) = WebSocketEventDecoder::decode_message(&payload_value) {
                        (
                            classified_type,
                            serde_json::to_value(message).unwrap_or(serde_json::Value::Null),
                        )
                    } else {
                        (event_name, payload_value)
                    }
                } else {
                    (event_name, payload_value)
                };

                emit_envelope(
                    on_event,
                    EventEnvelope {
                        account_user_id: account_id,
                        event_type,
                        payload,
                        received_at: Utc::now(),
                        source: "socket".to_string(),
                    },
                )
                .await;
            }
            .boxed()
        });

        Ok(builder)
    }

    #[instrument(skip(self, on_event, shutdown, shutdown_notify, reconnect_notify), fields(account_id = %self.account_id, url = %self.url))]
    pub async fn run<F>(
        &mut self,
        on_event: F,
        shutdown: Arc<AtomicBool>,
        shutdown_notify: Arc<Notify>,
        reconnect_notify: Arc<Notify>,
    ) -> Result<()>
    where
        F: FnMut(EventEnvelope) -> BoxFuture<'static, ()> + Send + 'static,
    {
        info!(account = %self.account_id, url = %self.url, "Connecting to WebSocket");

        if shutdown.load(Ordering::Relaxed) {
            info!(account = %self.account_id, "Shutdown requested before connect");
            return Ok(());
        }

        let on_event = Arc::new(tokio::sync::Mutex::new(on_event));
        let disconnect_notify = Arc::new(Notify::new());
        let disconnect_state = Arc::new(AtomicBool::new(false));
        let builder = self.build_socketio_client(
            on_event,
            disconnect_notify.clone(),
            disconnect_state.clone(),
        )?;

        let connect_result =
            tokio::time::timeout(std::time::Duration::from_secs(10), builder.connect())
                .await
                .context("Timed out while connecting to Socket.IO")?
                .context("Failed to connect to Socket.IO")?;

        let socket = connect_result;
        info!(account = %self.account_id, "WebSocket connected");

        if disconnect_state.load(Ordering::Relaxed) {
            info!(account = %self.account_id, "WebSocket disconnected during connect");
            let _ = socket.disconnect().await;
            return Ok(());
        }

        tokio::select! {
            _ = shutdown_notify.notified() => {
                info!(account = %self.account_id, "WebSocket shutdown signalled");
            }
            _ = reconnect_notify.notified() => {
                info!(account = %self.account_id, "WebSocket reconnect requested");
            }
            _ = disconnect_notify.notified() => {
                info!(account = %self.account_id, "WebSocket disconnected");
            }
        }

        let _ = socket.disconnect().await;
        Ok(())
    }
}

async fn emit_envelope<F>(on_event: Arc<tokio::sync::Mutex<F>>, event: EventEnvelope)
where
    F: FnMut(EventEnvelope) -> BoxFuture<'static, ()> + Send + 'static,
{
    let fut = {
        let mut handler = on_event.lock().await;
        (handler)(event)
    };

    fut.await;
}

fn socketio_payload_to_value(payload: rust_socketio::Payload) -> serde_json::Value {
    match payload {
        rust_socketio::Payload::Text(values) => match values.as_slice() {
            [single] => decode_value(single),
            [] => serde_json::Value::Null,
            _ => serde_json::Value::Array(
                values
                    .into_iter()
                    .map(|value| decode_value(&value))
                    .collect(),
            ),
        },
        rust_socketio::Payload::Binary(bytes) => match String::from_utf8(bytes.to_vec()) {
            Ok(text) => serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text)),
            Err(_) => serde_json::Value::Array(
                bytes
                    .into_iter()
                    .map(|byte| serde_json::Value::from(byte))
                    .collect(),
            ),
        },
        #[allow(deprecated)]
        rust_socketio::Payload::String(text) => {
            serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text))
        }
    }
}

fn decode_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            serde_json::from_str(s).unwrap_or_else(|_| serde_json::Value::String(s.clone()))
        }
        serde_json::Value::Array(arr) if arr.len() == 1 => decode_value(&arr[0]),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(decode_value).collect())
        }
        serde_json::Value::Object(_) => value.clone(),
        serde_json::Value::Null => serde_json::Value::Null,
        _ => value.clone(),
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
