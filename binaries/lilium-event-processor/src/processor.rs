use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use lilium_database::{DbPool, DbSessionContext};
use lilium_models::ingestion::WebSocketEvent;
use lilium_services::event::{EventProcessorOffsetService, WebSocketEventService};
use lilium_services::media::MediaService;
use lilium_services::message::MessageService;
use lilium_services::user::UserService;

pub struct EventProcessor {
    processor_id: String,
    pool: DbPool,
    batch_size: usize,
    polling_interval: Duration,
    max_retries: u32,
    initial_retry_delay: Duration,
    max_retry_delay: Duration,
    retry_backoff_factor: f64,
}

impl EventProcessor {
    pub fn new(
        pool: DbPool,
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

        let mut poll_interval = interval(self.polling_interval);

        loop {
            tokio::select! {
                _ = poll_interval.tick() => {
                    let (events, cursor_id, cursor_timestamp) = self.fetch_batch().await?;
                    if events.is_empty() {
                        continue;
                    }

                    info!(count = events.len(), "Processing batch");
                    self.process_batch_with_retry(&events, cursor_id, cursor_timestamp)
                        .await?;
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down event processor");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn fetch_batch(&self) -> Result<(Vec<WebSocketEvent>, i64, Option<DateTime<Utc>>)> {
        let processor_id = self.processor_id.clone();
        let batch_size = self.batch_size as i64;
        self.pool
            .with_session_context(|session| {
                Box::pin(async move {
                    let mut session = session;
                    Self::fetch_batch_inner(&mut session, processor_id, batch_size).await
                })
            })
            .await
    }

    async fn process_batch_with_retry(
        &self,
        events: &[WebSocketEvent],
        cursor_id: i64,
        cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut attempt = 0;
        loop {
            match self
                .process_batch(events, cursor_id, cursor_timestamp)
                .await
            {
                Ok(()) => return Ok(()),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        error!(
                            attempts = attempt,
                            error = %e,
                            "Max retries exceeded, skipping batch"
                        );
                        self.skip_batch(events, cursor_id, cursor_timestamp).await?;
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
        start_cursor_id: i64,
        start_cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let processor_id = self.processor_id.clone();
        self.pool
            .with_session_context(|session| {
                let events = events.to_vec();
                Box::pin(async move {
                    let mut session = session;
                    Self::process_batch_inner(
                        &mut session,
                        events,
                        processor_id,
                        start_cursor_id,
                        start_cursor_timestamp,
                    )
                    .await
                })
            })
            .await
    }

    async fn skip_batch(
        &self,
        events: &[WebSocketEvent],
        cursor_id: i64,
        cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let (batch_last_id, batch_last_timestamp) = Self::last_cursor_from_events(events);
        let last_id = if batch_last_id == 0 {
            cursor_id
        } else {
            batch_last_id
        };
        let last_timestamp = batch_last_timestamp.or(cursor_timestamp);

        let processor_id = self.processor_id.clone();
        self.pool
            .with_session_context(|session| {
                Box::pin(async move {
                    let mut session = session;
                    Self::skip_batch_inner(&mut session, processor_id, last_id, last_timestamp)
                        .await
                })
            })
            .await
    }

    async fn fetch_batch_inner(
        session: &mut DbSessionContext<'_>,
        processor_id: String,
        batch_size: i64,
    ) -> Result<(Vec<WebSocketEvent>, i64, Option<DateTime<Utc>>)> {
        let (last_id, last_timestamp) = Self::fetch_last_cursor(session, processor_id).await?;
        let events = Self::poll_events(session, last_timestamp, last_id, batch_size).await?;

        Ok((events, last_id, last_timestamp))
    }

    async fn process_batch_inner(
        session: &mut DbSessionContext<'_>,
        events: Vec<WebSocketEvent>,
        processor_id: String,
        start_cursor_id: i64,
        start_cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let (user_fetch_collector, media_message_ids) =
            { Self::collect_updates_from_events(session, &events).await? };

        let (batch_cursor_id, batch_cursor_timestamp) = Self::last_cursor_from_events(&events);
        let last_id = if batch_cursor_id == 0 {
            start_cursor_id
        } else {
            batch_cursor_id
        };
        let last_timestamp = batch_cursor_timestamp.or(start_cursor_timestamp);

        Self::sync_users(session, &user_fetch_collector).await?;
        Self::sync_media(session, &media_message_ids).await?;

        let mut offset_svc =
            EventProcessorOffsetService::new(DbSessionContext::new(session));
        offset_svc
            .update_offset(&processor_id, last_id, last_timestamp, Some(Utc::now()))
            .await?;

        Ok(())
    }

    async fn skip_batch_inner(
        session: &mut DbSessionContext<'_>,
        processor_id: String,
        last_id: i64,
        last_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut offset_svc =
            EventProcessorOffsetService::new(DbSessionContext::new(session));
        offset_svc
            .update_offset(&processor_id, last_id, last_timestamp, Some(Utc::now()))
            .await?;
        Ok(())
    }

    async fn fetch_last_cursor(
        session: &mut DbSessionContext<'_>,
        processor_id: String,
    ) -> Result<(i64, Option<DateTime<Utc>>)> {
        let mut offset_svc =
            EventProcessorOffsetService::new(DbSessionContext::new(session));
        let cursor = offset_svc.get_cursor(&processor_id).await?;
        Ok(cursor
            .map(|c| (c.last_processed_id, c.last_processed_timestamp))
            .unwrap_or((0, None)))
    }

    async fn poll_events(
        session: &mut DbSessionContext<'_>,
        last_timestamp: Option<DateTime<Utc>>,
        last_id: i64,
        batch_size: i64,
    ) -> Result<Vec<WebSocketEvent>> {
        let mut event_svc = WebSocketEventService::new(DbSessionContext::new(session));
        event_svc
            .poll_events(last_timestamp, last_id, batch_size)
            .await
    }

    async fn collect_updates_from_events(
        session: &mut DbSessionContext<'_>,
        events: &[WebSocketEvent],
    ) -> Result<(Vec<(String, String)>, Vec<String>)> {
        let mut message_service = MessageService::new(DbSessionContext::new(session));
        let mut user_fetch_collector = Vec::new();
        let mut media_message_ids = Vec::new();

        for event in events {
            if let Some(msg_id) =
                Self::process_event(&mut message_service, event, &mut user_fetch_collector).await?
            {
                media_message_ids.push(msg_id);
            }
        }

        Ok((user_fetch_collector, media_message_ids))
    }

    async fn sync_users(
        session: &mut DbSessionContext<'_>,
        user_fetch_collector: &[(String, String)],
    ) -> Result<()> {
        if user_fetch_collector.is_empty() {
            return Ok(());
        }

        let mut user_service = UserService::new(DbSessionContext::new(session));
        user_service
            .batch_fetch_and_update(user_fetch_collector)
            .await
            .map(|_| ())
    }

    async fn sync_media(
        session: &mut DbSessionContext<'_>,
        media_message_ids: &[String],
    ) -> Result<()> {
        if media_message_ids.is_empty() {
            return Ok(());
        }

        let mut media_service = MediaService::new(DbSessionContext::new(session));
        media_service
            .download_media_batch(media_message_ids)
            .await
            .map(|_| ())
    }

    fn last_cursor_from_events(events: &[WebSocketEvent]) -> (i64, Option<DateTime<Utc>>) {
        let mut last_id = 0;
        let mut last_timestamp: Option<DateTime<Utc>> = None;

        for event in events {
            if let Some(event_id) = event.id {
                last_id = event_id;
            }
            last_timestamp = Some(event.timestamp);
        }

        (last_id, last_timestamp)
    }

    async fn process_event(
        message_service: &mut MessageService<'_>,
        event: &WebSocketEvent,
        user_fetch_collector: &mut Vec<(String, String)>,
    ) -> Result<Option<String>> {
        match event.event.as_str() {
            "message:new" => {
                if let Some(msg) =
                    lilium_models::dzmm::message::Message::from_websocket(&event.data)
                {
                    user_fetch_collector.push((msg.sent_by.clone(), msg.room_id.clone()));
                    message_service.create_message_if_missing(&msg).await?;

                    if let Some(msg_data) = event.data.get("message") {
                        if let Some(content) = msg_data.get("content") {
                            if let Some(content_type) = content.get("type").and_then(|v| v.as_str())
                            {
                                if matches!(content_type, "image" | "video" | "voice" | "sticker") {
                                    return Ok(Some(msg.message_id.clone()));
                                }
                            }
                        }
                    }
                }
                Ok(None)
            }
            "message:updated" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    message_service
                        .update_message_from_payload(message_id, &event.data)
                        .await?;
                }
                Ok(None)
            }
            "message:deleted" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    message_service.mark_deleted(message_id, None).await?;
                }
                Ok(None)
            }
            "message:recalled" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    message_service.mark_recalled(message_id).await?;
                }
                Ok(None)
            }
            "presence:user-online" => {
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str()) {
                    if let Some(room_id) = event.data.get("roomId").and_then(|v| v.as_str()) {
                        user_fetch_collector.push((user_id.to_string(), room_id.to_string()));
                    }
                }
                Ok(None)
            }
            "group:member-joined" | "group:member-left" => Ok(None),
            _ => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct CursorPosition {
    pub last_processed_id: i64,
    pub last_processed_timestamp: Option<DateTime<Utc>>,
}
