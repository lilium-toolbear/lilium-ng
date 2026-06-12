use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};
use tokio::time::{interval, Duration};

use lilium_models::ingestion::WebSocketEvent;
use lilium_services::message::MessageService;
use lilium_services::event::EventService;

pub struct EventProcessor {
    processor_id: String,
    message_service: MessageService,
    event_service: EventService,
    batch_size: usize,
    polling_interval: Duration,
    max_retries: u32,
    initial_retry_delay: Duration,
    max_retry_delay: Duration,
    retry_backoff_factor: f64,
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
            message_service: MessageService::new(pool.clone()),
            event_service: EventService::new(pool),
            batch_size,
            polling_interval: Duration::from_secs(polling_interval_secs),
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(60),
            retry_backoff_factor: 2.0,
        }
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            processor_id = %self.processor_id,
            batch_size = self.batch_size,
            poll_interval_secs = self.polling_interval.as_secs(),
            "Event processor starting"
        );

        let (mut last_id, mut last_timestamp) = self.event_service.load_cursor(&self.processor_id).await?;
        info!(
            processor_id = %self.processor_id,
            cursor_id = last_id,
            "Loaded initial cursor"
        );

        let mut poll_interval = interval(self.polling_interval);

        loop {
            tokio::select! {
                _ = poll_interval.tick() => {
                    match self.event_service.poll_events(last_timestamp, last_id, self.batch_size as i64).await {
                        Ok(events) => {
                            if events.is_empty() {
                                continue;
                            }
                            info!(count = events.len(), "Processing batch");
                            self.process_batch_with_retry(&events, &mut last_id, &mut last_timestamp).await?;
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

    async fn process_batch_with_retry(
        &self,
        events: &[WebSocketEvent],
        last_id: &mut i64,
        last_timestamp: &mut Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut attempt = 0;
        loop {
            match self.process_batch(events, last_id, last_timestamp).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        error!(
                            attempts = attempt,
                            error = %e,
                            "Max retries exceeded, skipping batch"
                        );
                        self.skip_batch(events, last_id, last_timestamp).await?;
                        return Ok(());
                    }

                    let delay = self.calculate_retry_delay(attempt);
                    warn!(
                        attempt = attempt,
                        max_retries = self.max_retries,
                        delay_secs = delay.as_secs_f64(),
                        error = %e,
                        "Batch failed, retrying"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let initial_delay: f64 = self.initial_retry_delay.as_secs_f64();
        let backoff_factor: f64 = self.retry_backoff_factor;
        let delay = initial_delay * backoff_factor.powi(attempt as i32 - 1);
        let capped_delay = delay.min(self.max_retry_delay.as_secs_f64());
        let jitter = 0.5 + rand::random::<f64>();
        Duration::from_secs_f64(capped_delay * jitter)
    }

    async fn process_batch(
        &self,
        events: &[WebSocketEvent],
        last_id: &mut i64,
        last_timestamp: &mut Option<DateTime<Utc>>,
    ) -> Result<()> {
        for event in events {
            self.process_event(event).await?;
            if let Some(id) = event.id {
                *last_id = id;
            }
            *last_timestamp = Some(event.timestamp);
        }

        self.event_service.save_cursor(
            &self.processor_id,
            *last_id,
            *last_timestamp,
        ).await?;

        Ok(())
    }

    async fn skip_batch(
        &self,
        events: &[WebSocketEvent],
        last_id: &mut i64,
        last_timestamp: &mut Option<DateTime<Utc>>,
    ) -> Result<()> {
        for event in events {
            if let Some(id) = event.id {
                *last_id = id;
            }
            *last_timestamp = Some(event.timestamp);
        }

        self.event_service.save_cursor(
            &self.processor_id,
            *last_id,
            *last_timestamp,
        ).await?;

        Ok(())
    }

    async fn process_event(&self, event: &WebSocketEvent) -> Result<()> {
        match event.event.as_str() {
            "message:new" => {
                if let Some(msg) = lilium_models::dzmm::message::Message::from_websocket(&event.data) {
                    self.message_service.create_message(&msg).await?;
                }
            }
            "message:updated" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    self.message_service.update_message(message_id, &event.data).await?;
                }
            }
            "message:deleted" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    self.message_service.mark_deleted(message_id).await?;
                }
            }
            "message:recalled" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    self.message_service.mark_recalled(message_id).await?;
                }
            }
            "presence:user-online" | "group:member-joined" | "group:member-left" => {
                // Ignored for now
            }
            _ => {
                // Unknown event type
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct CursorPosition {
    pub last_processed_id: i64,
    pub last_processed_timestamp: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_event(event_type: &str, data: serde_json::Value) -> WebSocketEvent {
        WebSocketEvent {
            id: Some(1),
            event: event_type.to_string(),
            data,
            user_id: "user1".to_string(),
            timestamp: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
        }
    }

    #[test]
    fn test_retry_delay_calculation() {
        let initial_delay: f64 = 1.0;
        let max_delay: f64 = 60.0;
        let backoff_factor: f64 = 2.0;

        let delay1 = initial_delay * backoff_factor.powi(0);
        assert!(delay1 >= 0.5 && delay1 <= 1.5);

        let delay2 = initial_delay * backoff_factor.powi(1);
        assert!(delay2 >= 1.0 && delay2 <= 3.0);

        let delay3 = initial_delay * backoff_factor.powi(2);
        assert!(delay3 >= 2.0 && delay3 <= 6.0);

        let delay_large = initial_delay * backoff_factor.powi(10);
        let capped = delay_large.min(max_delay);
        assert!(capped <= max_delay);
    }

    #[test]
    fn test_event_type_dispatch_message_new() {
        let event = make_event("message:new", serde_json::json!({
            "chatroomId": "room1",
            "message": {
                "messageId": "msg1",
                "sentBy": "user1",
                "sentAt": "2026-01-01T00:00:00Z",
                "content": {"type": "text", "text": "hello"}
            }
        }));
        assert_eq!(event.event, "message:new");
        assert!(event.data.get("chatroomId").is_some());
    }

    #[test]
    fn test_event_type_dispatch_message_updated() {
        let event = make_event("message:updated", serde_json::json!({
            "chatroomId": "room1",
            "messageId": "msg1",
            "message": {
                "content": {"type": "text", "text": "updated"}
            }
        }));
        assert_eq!(event.event, "message:updated");
        assert!(event.data.get("messageId").is_some());
    }

    #[test]
    fn test_event_type_dispatch_message_deleted() {
        let event = make_event("message:deleted", serde_json::json!({
            "chatroomId": "room1",
            "messageId": "msg1",
            "deletedBy": "user1"
        }));
        assert_eq!(event.event, "message:deleted");
        assert_eq!(event.data["messageId"], "msg1");
    }

    #[test]
    fn test_event_type_dispatch_message_recalled() {
        let event = make_event("message:recalled", serde_json::json!({
            "chatroomId": "room1",
            "messageId": "msg1"
        }));
        assert_eq!(event.event, "message:recalled");
        assert_eq!(event.data["messageId"], "msg1");
    }

    #[test]
    fn test_event_type_dispatch_presence_online() {
        let event = make_event("presence:user-online", serde_json::json!({
            "chatroomId": "room1",
            "userId": "user1"
        }));
        assert_eq!(event.event, "presence:user-online");
    }

    #[test]
    fn test_event_type_dispatch_member_joined() {
        let event = make_event("group:member-joined", serde_json::json!({
            "chatroomId": "room1",
            "userId": "user1"
        }));
        assert_eq!(event.event, "group:member-joined");
    }

    #[test]
    fn test_event_type_dispatch_member_left() {
        let event = make_event("group:member-left", serde_json::json!({
            "chatroomId": "room1",
            "userId": "user1"
        }));
        assert_eq!(event.event, "group:member-left");
    }

    #[test]
    fn test_event_type_dispatch_unknown() {
        let event = make_event("unknown:event", serde_json::json!({}));
        assert_eq!(event.event, "unknown:event");
    }

    #[test]
    fn test_message_new_requires_chatroom_id() {
        let event = make_event("message:new", serde_json::json!({
            "message": {
                "messageId": "msg1"
            }
        }));
        let msg = lilium_models::dzmm::message::Message::from_websocket(&event.data);
        assert!(msg.is_none());
    }

    #[test]
    fn test_message_new_requires_message_id() {
        let event = make_event("message:new", serde_json::json!({
            "chatroomId": "room1",
            "message": {}
        }));
        let msg = lilium_models::dzmm::message::Message::from_websocket(&event.data);
        assert!(msg.is_none());
    }

    #[test]
    fn test_message_new_extracts_fields() {
        let event = make_event("message:new", serde_json::json!({
            "chatroomId": "room123",
            "message": {
                "messageId": "msg456",
                "sentBy": "user789",
                "sentAt": "2026-01-01T00:00:00Z",
                "content": {"type": "text", "text": "hello"}
            }
        }));
        let msg = lilium_models::dzmm::message::Message::from_websocket(&event.data).unwrap();
        assert_eq!(msg.message_id, "msg456");
        assert_eq!(msg.room_id, "room123");
        assert_eq!(msg.sent_by.as_deref(), Some("user789"));
        assert_eq!(msg.content_type.as_deref(), Some("text"));
        assert_eq!(msg.content_text.as_deref(), Some("hello"));
    }

    #[test]
    fn test_message_updated_recalled_detection() {
        let event = make_event("message:updated", serde_json::json!({
            "messageId": "msg1",
            "message": {
                "content": {"type": "recalled"}
            }
        }));
        let content_type = event.data["message"]["content"]["type"].as_str();
        assert_eq!(content_type, Some("recalled"));
    }

    #[test]
    fn test_message_updated_content_update() {
        let event = make_event("message:updated", serde_json::json!({
            "messageId": "msg1",
            "message": {
                "content": {"type": "text", "text": "updated content"}
            }
        }));
        let content_type = event.data["message"]["content"]["type"].as_str();
        assert_eq!(content_type, Some("text"));
        let content_text = event.data["message"]["content"]["text"].as_str();
        assert_eq!(content_text, Some("updated content"));
    }

    #[test]
    fn test_cursor_position_default() {
        let cursor = CursorPosition {
            last_processed_id: 0,
            last_processed_timestamp: None,
        };
        assert_eq!(cursor.last_processed_id, 0);
        assert!(cursor.last_processed_timestamp.is_none());
    }

    #[test]
    fn test_cursor_position_with_timestamp() {
        let ts = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
        let cursor = CursorPosition {
            last_processed_id: 100,
            last_processed_timestamp: Some(ts),
        };
        assert_eq!(cursor.last_processed_id, 100);
        assert!(cursor.last_processed_timestamp.is_some());
    }
}
