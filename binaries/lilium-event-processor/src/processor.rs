use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, Instant, sleep};
use tracing::{Instrument, error, info, warn};

use lilium_common::LiliumError;
use lilium_database::{Database, NotificationConnection, NotificationDatabaseConfig};
use lilium_models::ingestion::WebSocketEvent;
use lilium_services::account;
use lilium_services::event;
use lilium_services::media::MediaService;
use lilium_services::message;
use lilium_services::{room_member, user};
use sea_orm::ConnectionTrait;

// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/event_processor.py
const WEBSOCKET_EVENT_INSERTED_CHANNEL: &str = "websocket_event_inserted";

#[derive(Debug, Default)]
struct BatchSideEffects {
    media_message_ids: Vec<String>,
    avatar_downloads: Vec<user::AvatarDownload>,
}

pub struct EventProcessor {
    processor_id: String,
    database: Database,
    batch_size: usize,
    polling_interval: Duration,
    max_retries: u32,
    initial_retry_delay: Duration,
    max_retry_delay: Duration,
    retry_backoff_factor: f64,
    shutdown: Arc<Notify>,
    notification_config: Option<NotificationDatabaseConfig>,
}

impl EventProcessor {
    pub fn new(
        database: Database,
        processor_id: String,
        batch_size: usize,
        polling_interval_secs: u64,
    ) -> Self {
        Self {
            processor_id,
            database,
            batch_size,
            polling_interval: Duration::from_secs(polling_interval_secs),
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(60),
            retry_backoff_factor: 2.0,
            shutdown: Arc::new(Notify::new()),
            notification_config: None,
        }
    }

    pub fn with_notification_config(mut self, config: NotificationDatabaseConfig) -> Self {
        self.notification_config = Some(config);
        self
    }

    pub fn shutdown_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown)
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            processor_id = %self.processor_id,
            batch_size = self.batch_size,
            poll_interval_secs = self.polling_interval.as_secs(),
            "Event processor starting"
        );

        let mut notification = self.connect_notification_listener().await?;

        if let Err(error) = self.process_next_batch().await {
            error!(error = %error, "Initial event processor poll failed");
        }

        let mut poll_sleep = Box::pin(sleep(self.polling_interval));

        loop {
            tokio::select! {
                _ = self.shutdown.notified() => {
                    info!("Shutting down event processor");
                    break;
                }
                notification = Self::recv_notification(&mut notification) => {
                    match notification {
                        Ok(Some(payload)) => {
                            info!(
                                channel = WEBSOCKET_EVENT_INSERTED_CHANNEL,
                                payload = %payload,
                                "Event processor woke from PostgreSQL notification"
                            );
                            if let Err(error) = self.process_next_batch().await {
                                error!(error = %error, "Event processor notification poll failed");
                            }
                            poll_sleep.as_mut().reset(Instant::now() + self.polling_interval);
                        }
                        Ok(None) => {}
                        Err(error) => {
                            error!(error = %error, "Event processor notification listener failed");
                            break;
                        }
                    }
                }
                _ = &mut poll_sleep => {
                    if let Err(error) = self.process_next_batch().await {
                        error!(error = %error, "Event processor poll failed");
                    }
                    poll_sleep.as_mut().reset(Instant::now() + self.polling_interval);
                }
            }
        }

        Ok(())
    }

    async fn connect_notification_listener(&self) -> Result<Option<NotificationConnection>> {
        let Some(config) = self.notification_config.clone() else {
            return Ok(None);
        };

        let mut connection = NotificationConnection::connect(config).await?;
        connection.listen(WEBSOCKET_EVENT_INSERTED_CHANNEL).await?;
        info!(
            channel = WEBSOCKET_EVENT_INSERTED_CHANNEL,
            "Event processor notification listener attached"
        );
        Ok(Some(connection))
    }

    async fn recv_notification(
        notification: &mut Option<NotificationConnection>,
    ) -> Result<Option<String>> {
        match notification {
            Some(connection) => connection.recv_payload().await.map(Some),
            None => std::future::pending().await,
        }
    }

    async fn process_next_batch(&self) -> Result<()> {
        let batch_span = tracing::info_span!(
            "lilium-event-processor.batch",
            sentry.name = "event_processor batch",
            sentry.op = "queue.process",
            processor.id = %self.processor_id,
            configured.batch_size = self.batch_size,
        );

        async {
            let (events, cursor_id, cursor_timestamp) = self.fetch_batch().await?;
            if events.is_empty() {
                return Ok(());
            }

            info!(count = events.len(), "Processing batch");
            let side_effects = self
                .process_batch_with_retry(&events, cursor_id, cursor_timestamp)
                .await?;
            self.spawn_media_download(side_effects.media_message_ids);
            self.spawn_avatar_download(side_effects.avatar_downloads);
            Ok(())
        }
        .instrument(batch_span)
        .await
    }

    async fn fetch_batch(&self) -> Result<(Vec<WebSocketEvent>, i64, Option<DateTime<Utc>>)> {
        let processor_id = self.processor_id.clone();
        let batch_size = self.batch_size as i64;
        lilium_database::transaction!(self.database, |session| {
            Self::fetch_batch_inner(session, processor_id, batch_size).await
        })
        .await
    }

    async fn process_batch_with_retry(
        &self,
        events: &[WebSocketEvent],
        cursor_id: i64,
        cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<BatchSideEffects> {
        let mut attempt = 0;
        loop {
            match self
                .process_batch(events, cursor_id, cursor_timestamp)
                .await
            {
                Ok(side_effects) => {
                    return Ok(side_effects);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        error!(
                            attempts = attempt,
                            error = %e,
                            "Max retries exceeded, falling back to per-event processing"
                        );
                        return self
                            .process_batch_individually(events, cursor_id, cursor_timestamp)
                            .await;
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

    async fn process_batch_individually(
        &self,
        events: &[WebSocketEvent],
        _cursor_id: i64,
        cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<BatchSideEffects> {
        let mut side_effects = BatchSideEffects::default();

        for event in events {
            let event_slice = std::slice::from_ref(event);
            let event_cursor_id = event.id;
            let event_cursor_timestamp = Some(event.timestamp).or(cursor_timestamp);

            match self
                .process_single_event_fallback(event, event_cursor_id, event_cursor_timestamp)
                .await
            {
                Ok(event_side_effects) => {
                    side_effects
                        .media_message_ids
                        .extend(event_side_effects.media_message_ids);
                    side_effects
                        .avatar_downloads
                        .extend(event_side_effects.avatar_downloads);
                }
                Err(error) => {
                    error!(
                        event_id = event_cursor_id,
                        error = %error,
                        "Skipping poison event during per-event fallback"
                    );
                    if let Err(skip_error) = self
                        .skip_batch(event_slice, event_cursor_id, event_cursor_timestamp)
                        .await
                    {
                        error!(
                            event_id = event_cursor_id,
                            error = %skip_error,
                            "Failed to advance cursor past poison event"
                        );
                    }
                }
            }
        }

        Ok(side_effects)
    }

    async fn process_single_event_fallback(
        &self,
        event: &WebSocketEvent,
        _cursor_id: i64,
        cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<BatchSideEffects> {
        let processor_id = self.processor_id.clone();
        let event = event.clone();
        let event_id = event.id;
        let (user_fetch_collector, media_message_ids) =
            lilium_database::transaction!(self.database, |session| {
                let event = event.clone();
                let processor_id = processor_id.clone();
                async move {
                    let mut user_fetch_collector = Vec::new();
                    let mut media_message_ids = Vec::new();
                    if let Some(msg_id) =
                        Self::process_event(session, &event, &mut user_fetch_collector).await?
                    {
                        media_message_ids.push(msg_id);
                    }

                    event::update_offset(
                        session,
                        &processor_id,
                        event_id,
                        Some(event.timestamp).or(cursor_timestamp),
                        Some(Utc::now()),
                    )
                    .await?;

                    Ok::<_, anyhow::Error>((user_fetch_collector, media_message_ids))
                }
                .await
            })
            .await?;

        let avatar_downloads = if user_fetch_collector.is_empty() {
            Vec::new()
        } else {
            match lilium_database::transaction!(self.database, |session| {
                let user_fetch_collector = user_fetch_collector.clone();
                Self::sync_users(session, &user_fetch_collector).await
            })
            .await
            {
                Ok(downloads) => downloads,
                Err(error) => {
                    warn!(
                        event_id = event_id,
                        error = %error,
                        "User sync failed after single-event fallback commit"
                    );
                    Vec::new()
                }
            }
        };

        Ok(BatchSideEffects {
            media_message_ids,
            avatar_downloads,
        })
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
    ) -> Result<BatchSideEffects> {
        let processor_id = self.processor_id.clone();
        let events = events.to_vec();
        lilium_database::transaction!(self.database, |session| {
            Self::process_batch_inner(
                session,
                events,
                processor_id,
                start_cursor_id,
                start_cursor_timestamp,
            )
            .await
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
        lilium_database::transaction!(self.database, |session| {
            Self::skip_batch_inner(session, processor_id, last_id, last_timestamp).await
        })
        .await
    }

    async fn fetch_batch_inner(
        session: &impl ConnectionTrait,
        processor_id: String,
        batch_size: i64,
    ) -> Result<(Vec<WebSocketEvent>, i64, Option<DateTime<Utc>>)> {
        let (last_id, last_timestamp) = Self::fetch_last_cursor(session, processor_id).await?;
        let events = Self::poll_events(session, last_timestamp, last_id, batch_size).await?;

        Ok((events, last_id, last_timestamp))
    }

    async fn process_batch_inner(
        session: &impl ConnectionTrait,
        events: Vec<WebSocketEvent>,
        processor_id: String,
        start_cursor_id: i64,
        start_cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<BatchSideEffects> {
        let (user_fetch_collector, media_message_ids) =
            { Self::collect_updates_from_events(session, &events).await? };

        let (batch_cursor_id, batch_cursor_timestamp) = Self::last_cursor_from_events(&events);
        let last_id = if batch_cursor_id == 0 {
            start_cursor_id
        } else {
            batch_cursor_id
        };
        let last_timestamp = batch_cursor_timestamp.or(start_cursor_timestamp);
        let avatar_downloads = Self::sync_users(session, &user_fetch_collector).await?;

        event::update_offset(
            session,
            &processor_id,
            last_id,
            last_timestamp,
            Some(Utc::now()),
        )
        .await?;

        Ok(BatchSideEffects {
            media_message_ids,
            avatar_downloads,
        })
    }

    async fn skip_batch_inner(
        session: &impl ConnectionTrait,
        processor_id: String,
        last_id: i64,
        last_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        event::update_offset(
            session,
            &processor_id,
            last_id,
            last_timestamp,
            Some(Utc::now()),
        )
        .await?;
        Ok(())
    }

    async fn fetch_last_cursor(
        session: &impl ConnectionTrait,
        processor_id: String,
    ) -> Result<(i64, Option<DateTime<Utc>>)> {
        let cursor = event::get_cursor(session, &processor_id).await?;
        Ok(cursor
            .map(|c| (c.last_processed_id, c.last_processed_timestamp))
            .unwrap_or((0, None)))
    }

    async fn poll_events(
        session: &impl ConnectionTrait,
        last_timestamp: Option<DateTime<Utc>>,
        last_id: i64,
        batch_size: i64,
    ) -> Result<Vec<WebSocketEvent>> {
        event::poll_events(session, last_timestamp, last_id, batch_size)
            .await
            .map_err(Into::into)
    }

    async fn collect_updates_from_events(
        session: &impl ConnectionTrait,
        events: &[WebSocketEvent],
    ) -> Result<(Vec<(String, String, String)>, Vec<String>)> {
        let mut user_fetch_collector = Vec::new();
        let mut media_message_ids = Vec::new();

        for event in events {
            if let Some(msg_id) =
                Self::process_event(session, event, &mut user_fetch_collector).await?
            {
                media_message_ids.push(msg_id);
            }
        }

        Ok((user_fetch_collector, media_message_ids))
    }

    async fn sync_users(
        session: &impl ConnectionTrait,
        user_fetch_collector: &[(String, String, String)],
    ) -> Result<Vec<user::AvatarDownload>> {
        if user_fetch_collector.is_empty() {
            return Ok(Vec::new());
        }

        let mut grouped: HashMap<String, Vec<(String, String)>> = HashMap::new();
        for (source_account_user_id, user_id, room_id) in user_fetch_collector {
            grouped
                .entry(source_account_user_id.clone())
                .or_default()
                .push((user_id.clone(), room_id.clone()));
        }

        let mut total_new = 0;
        let mut total_updated = 0;
        let account_count = grouped.len();
        let mut avatar_downloads = Vec::new();

        for (source_account_user_id, user_room_pairs) in grouped {
            let account = { account::get_account(session, &source_account_user_id).await? }
                .ok_or_else(|| {
                    LiliumError::service(
                        "ACCOUNT_SYNC_ACCOUNT_NOT_FOUND",
                        format!(
                            "Account '{}' not found for user sync",
                            source_account_user_id
                        ),
                    )
                })?;

            let auth = account::create_auth_client(account)?;
            let result =
                user::batch_fetch_and_update_with_auth(session, &auth, &user_room_pairs, 1).await?;
            total_new += result.new_count;
            total_updated += result.updated_count;
            avatar_downloads.extend(result.avatar_downloads);
        }

        info!(
            new = total_new,
            updated = total_updated,
            accounts = account_count,
            "Batch fetched users via auth clients"
        );

        Ok(avatar_downloads)
    }

    fn spawn_media_download(&self, media_message_ids: Vec<String>) {
        if media_message_ids.is_empty() {
            return;
        }

        let database = self.database.clone();
        tokio::spawn(async move {
            let media_count = media_message_ids.len();
            let downloads = match lilium_database::transaction!(database, |session| {
                let media_message_ids = media_message_ids.clone();
                lilium_services::media::collect_message_media_downloads(session, &media_message_ids)
                    .await
            })
            .await
            {
                Ok(downloads) => downloads,
                Err(error) => {
                    warn!(
                        count = media_count,
                        error = %error,
                        "Background media download lookup failed"
                    );
                    return;
                }
            };

            let media_service = MediaService::new();
            let (updates, failure_count) =
                match media_service.download_media_batch(&downloads).await {
                    Ok(result) => result,
                    Err(error) => {
                        warn!(
                            count = media_count,
                            error = %error,
                            "Background media download batch failed"
                        );
                        return;
                    }
                };

            if !updates.is_empty() {
                let result: Result<()> = lilium_database::transaction!(database, |session| {
                    let updates = updates.clone();
                    lilium_services::media::persist_message_media_files(session, &updates)
                        .await
                        .map(|_| ())
                })
                .await;

                if let Err(error) = result {
                    warn!(
                        count = media_count,
                        error = %error,
                        "Background media file persistence failed"
                    );
                }
            }

            if failure_count > 0 {
                warn!(
                    count = media_count,
                    failures = failure_count,
                    "Background media download batch completed with failures"
                );
            }
        });
    }

    fn spawn_avatar_download(&self, avatar_downloads: Vec<user::AvatarDownload>) {
        if avatar_downloads.is_empty() {
            return;
        }

        let database = self.database.clone();
        tokio::spawn(async move {
            let avatar_count = avatar_downloads.len();
            let media_service = MediaService::new();
            let (updates, failure_count) =
                match media_service.download_user_avatars(&avatar_downloads).await {
                    Ok(result) => result,
                    Err(error) => {
                        warn!(
                            count = avatar_count,
                            error = %error,
                            "Background avatar download batch failed"
                        );
                        return;
                    }
                };

            if !updates.is_empty() {
                let result: Result<()> = lilium_database::transaction!(database, |session| {
                    let updates = updates.clone();
                    lilium_services::media::persist_user_avatar_files(session, &updates)
                        .await
                        .map(|_| ())
                })
                .await;

                if let Err(error) = result {
                    warn!(
                        count = avatar_count,
                        error = %error,
                        "Background avatar file persistence failed"
                    );
                }
            }

            if failure_count > 0 {
                warn!(
                    count = avatar_count,
                    failures = failure_count,
                    "Background avatar download batch completed with failures"
                );
            }
        });
    }

    fn last_cursor_from_events(events: &[WebSocketEvent]) -> (i64, Option<DateTime<Utc>>) {
        let mut last_id = 0;
        let mut last_timestamp: Option<DateTime<Utc>> = None;

        for event in events {
            last_id = event.id;
            last_timestamp = Some(event.timestamp);
        }

        (last_id, last_timestamp)
    }

    async fn process_event(
        session: &impl ConnectionTrait,
        event: &WebSocketEvent,
        user_fetch_collector: &mut Vec<(String, String, String)>,
    ) -> Result<Option<String>> {
        match event.event.as_str() {
            "message:new" => {
                if let Some(msg) =
                    lilium_models::dzmm::message::Message::from_websocket(&event.data)
                {
                    let created = message::create_message_if_missing(session, &msg).await?;

                    if !created {
                        return Ok(None);
                    }

                    if !msg.sent_by.is_empty() {
                        user_fetch_collector.push((
                            event.user_id.clone(),
                            msg.sent_by.clone(),
                            msg.room_id.clone(),
                        ));
                    }

                    if msg.content_type == "system"
                        && let Some(content_text) = msg.content_text.as_deref()
                    {
                        if content_text.contains("加入了群聊") && !msg.sent_by.is_empty() {
                            room_member::upsert_member_simple(
                                session,
                                &msg.room_id,
                                &msg.sent_by,
                                "member",
                                Some(msg.sent_at),
                            )
                            .await?;
                        } else if content_text.contains("离开了群聊") && !msg.sent_by.is_empty()
                        {
                            let _ = room_member::mark_member_left(
                                session,
                                &msg.room_id,
                                &msg.sent_by,
                                Some(msg.sent_at),
                            )
                            .await?;
                        }
                    }

                    if matches!(
                        msg.content_type.as_str(),
                        "image" | "video" | "voice" | "sticker"
                    ) {
                        return Ok(Some(msg.message_id.clone()));
                    }

                    if msg.content_type == "system" {
                        return Ok(None);
                    }

                    if let Some(msg_data) = event.data.get("message")
                        && let Some(content) = msg_data.get("content")
                        && let Some(content_type) = content.get("type").and_then(|v| v.as_str())
                        && matches!(content_type, "image" | "video" | "voice" | "sticker")
                    {
                        return Ok(Some(msg.message_id.clone()));
                    }
                }
                Ok(None)
            }
            "message:updated" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    message::update_message_from_payload(session, message_id, &event.data).await?;
                }
                Ok(None)
            }
            "message:deleted" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    let deleted_by = event.data.get("deletedBy").and_then(|v| v.as_str());
                    message::mark_deleted(session, message_id, deleted_by).await?;
                }
                Ok(None)
            }
            "message:recalled" => {
                if let Some(message_id) = event.data.get("messageId").and_then(|v| v.as_str()) {
                    message::mark_recalled(session, message_id).await?;
                }
                Ok(None)
            }
            "presence:user-online" => {
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str())
                    && let Some(room_id) = event.data.get("chatroomId").and_then(|v| v.as_str())
                {
                    user_fetch_collector.push((
                        event.user_id.clone(),
                        user_id.to_string(),
                        room_id.to_string(),
                    ));
                }
                Ok(None)
            }
            "group:member-joined" => {
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str())
                    && let Some(room_id) = event.data.get("chatroomId").and_then(|v| v.as_str())
                {
                    room_member::upsert_member_simple(
                        session,
                        room_id,
                        user_id,
                        "member",
                        Some(event.timestamp),
                    )
                    .await?;
                    user_fetch_collector.push((
                        event.user_id.clone(),
                        user_id.to_string(),
                        room_id.to_string(),
                    ));
                }
                Ok(None)
            }
            "group:member-left" => {
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str())
                    && let Some(room_id) = event.data.get("chatroomId").and_then(|v| v.as_str())
                {
                    let _ = room_member::mark_member_left(
                        session,
                        room_id,
                        user_id,
                        Some(event.timestamp),
                    )
                    .await?;
                }
                Ok(None)
            }
            "presence:user-offline"
            | "message:user-left"
            | "message:joined"
            | "message:online-status"
            | "match:limit"
            | "connected"
            | "disconnected" => Ok(None),
            _ => {
                warn!(event_type = %event.event, "Unknown event type");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lilium_models::dzmm::message::Message as DzmmMessage;
    use lilium_models::dzmm::room_member::RoomMember;
    use lilium_services::event;
    use lilium_services::message;
    use lilium_test_fixtures::{FixtureProfile, TestDb};

    fn utc(ts: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(ts)
            .expect("valid rfc3339")
            .with_timezone(&Utc)
    }

    fn websocket_event(
        id: i64,
        event: &str,
        user_id: &str,
        timestamp: &str,
        data: serde_json::Value,
    ) -> WebSocketEvent {
        WebSocketEvent {
            id,
            event: event.to_string(),
            data,
            user_id: user_id.to_string(),
            timestamp: utc(timestamp),
        }
    }

    async fn room_member_row(
        session: &impl ConnectionTrait,
        room_id: &str,
        user_id: &str,
    ) -> Result<Option<RoomMember>> {
        lilium_services::room_member::get_member_info(session, room_id, user_id)
            .await
            .map_err(Into::into)
    }

    #[tokio::test]
    async fn message_new_system_join_updates_room_members() {
        let test_db = TestDb::acquire(FixtureProfile::Shared)
            .await
            .expect("init shared db");

        lilium_database::transaction!(test_db.database(), |session| {
            let event = websocket_event(
                11,
                "message:new",
                "account_1",
                "2026-06-02T12:00:00Z",
                serde_json::json!({
                    "chatroomId": "room_1",
                    "message": {
                        "message_id": "msg_join",
                        "chatroom_id": "room_1",
                        "sent_by": "user_joined",
                        "sent_at": "2026-06-02T12:00:00Z",
                        "content": {
                            "type": "system",
                            "text": "Alice 加入了群聊"
                        }
                    }
                }),
            );

            let mut user_fetch_collector = Vec::new();
            let result = EventProcessor::process_event(session, &event, &mut user_fetch_collector)
                .await
                .expect("process event");

            assert!(result.is_none());
            assert_eq!(
                user_fetch_collector,
                vec![(
                    "account_1".to_string(),
                    "user_joined".to_string(),
                    "room_1".to_string()
                )]
            );

            let member = room_member_row(session, "room_1", "user_joined")
                .await?
                .expect("member row");
            assert_eq!(member.room_id, "room_1");
            assert_eq!(member.user_id, "user_joined");
            assert_eq!(member.joined_at, Some(utc("2026-06-02T12:00:00Z")));
            assert!(member.left_at.is_none());

            Ok(())
        })
        .await
        .expect("message_new_system_join_updates_room_members");
    }

    #[tokio::test]
    async fn group_member_left_marks_room_member_left() {
        let test_db = TestDb::acquire(FixtureProfile::Shared)
            .await
            .expect("init shared db");

        lilium_database::transaction!(test_db.database(), |session| {
            room_member::upsert_member_simple(
                session,
                "room_1",
                "user_left",
                "member",
                Some(utc("2026-06-01T00:00:00Z")),
            )
            .await?;

            let event = websocket_event(
                12,
                "group:member-left",
                "account_1",
                "2026-06-02T12:30:00Z",
                serde_json::json!({
                    "chatroomId": "room_1",
                    "userId": "user_left"
                }),
            );

            let mut user_fetch_collector = Vec::new();
            EventProcessor::process_event(session, &event, &mut user_fetch_collector)
                .await
                .expect("process event");

            assert!(user_fetch_collector.is_empty());

            let member = room_member_row(session, "room_1", "user_left")
                .await?
                .expect("member row");
            assert_eq!(member.left_at, Some(utc("2026-06-02T12:30:00Z")));

            Ok(())
        })
        .await
        .expect("group_member_left_marks_room_member_left");
    }

    #[tokio::test]
    async fn presence_user_online_collects_chatroom_user() {
        let test_db = TestDb::acquire(FixtureProfile::Shared)
            .await
            .expect("init shared db");

        lilium_database::transaction!(test_db.database(), |session| {
            let event = websocket_event(
                14,
                "presence:user-online",
                "account_1",
                "2026-06-02T12:45:00Z",
                serde_json::json!({
                    "chatroomId": "room_1",
                    "userId": "user_online"
                }),
            );

            let mut user_fetch_collector = Vec::new();
            EventProcessor::process_event(session, &event, &mut user_fetch_collector)
                .await
                .expect("process event");

            assert_eq!(
                user_fetch_collector,
                vec![(
                    "account_1".to_string(),
                    "user_online".to_string(),
                    "room_1".to_string()
                )]
            );

            Ok(())
        })
        .await
        .expect("presence_user_online_collects_chatroom_user");
    }

    #[tokio::test]
    async fn message_deleted_uses_deleted_by() {
        let test_db = TestDb::acquire(FixtureProfile::Shared)
            .await
            .expect("init shared db");

        lilium_database::transaction!(test_db.database(), |session| {
            let sent_at = utc("2026-06-01T00:00:00Z");
            let message = DzmmMessage {
                message_id: "msg_deleted".to_string(),
                room_id: "room_1".to_string(),
                sent_at,
                sent_by: "user_deleted".to_string(),
                content_type: "text".to_string(),
                content_text: Some("hello".to_string()),
                attachment_url: None,
                attachment_file: None,
                sticker_id: None,
                alt_text: None,
                metadata: None,
                raw_data: serde_json::json!({"message_id": "msg_deleted"}),
                source: "spider".to_string(),
                created_at: sent_at,
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
            message::create_message(session, &message).await?;

            let event = websocket_event(
                13,
                "message:deleted",
                "account_1",
                "2026-06-02T13:00:00Z",
                serde_json::json!({
                    "chatroomId": "room_1",
                    "messageId": "msg_deleted",
                    "deletedBy": "user_admin"
                }),
            );

            let mut user_fetch_collector = Vec::new();
            EventProcessor::process_event(session, &event, &mut user_fetch_collector)
                .await
                .expect("process event");

            let updated = message::get_by_id_at(session, "msg_deleted", sent_at, false)
                .await?
                .expect("message exists");

            assert!(updated.is_deleted);
            assert_eq!(updated.deleted_by.as_deref(), Some("user_admin"));

            Ok(())
        })
        .await
        .expect("message_deleted_uses_deleted_by");
    }

    #[tokio::test]
    async fn batch_retry_falls_back_to_per_event_processing() {
        let test_db = TestDb::acquire(FixtureProfile::Event)
            .await
            .expect("init event db");

        let mut processor = EventProcessor::new(
            test_db.database().clone(),
            "test_processor".to_string(),
            10,
            60,
        );
        processor.max_retries = 0;

        let events = vec![
            websocket_event(
                21,
                "presence:user-online",
                "account_1",
                "2026-06-03T00:00:00Z",
                serde_json::json!({
                    "chatroomId": "room_1",
                    "userId": "user_sync_missing_account"
                }),
            ),
            websocket_event(
                22,
                "connected",
                "account_1",
                "2026-06-03T00:01:00Z",
                serde_json::json!({}),
            ),
        ];

        processor
            .process_batch_with_retry(&events, 0, None)
            .await
            .expect("fallback processing");

        let offset = test_db
            .database()
            .transaction(|session| {
                Box::pin(async move {
                    event::get_offset(session, "test_processor")
                        .await
                        .map_err(Into::into)
                })
            })
            .await
            .expect("fetch offset");
        assert_eq!(offset, 22);
    }

    #[tokio::test]
    async fn per_event_fallback_keeps_message_when_user_sync_fails() {
        let test_db = TestDb::acquire(FixtureProfile::Shared)
            .await
            .expect("init shared db");

        let mut processor = EventProcessor::new(
            test_db.database().clone(),
            "test_processor".to_string(),
            10,
            60,
        );
        processor.max_retries = 0;

        let event = websocket_event(
            31,
            "message:new",
            "account_1",
            "2026-06-03T00:00:00Z",
            serde_json::json!({
                "chatroomId": "room_1",
                "message": {
                    "message_id": "msg_user_sync_fail",
                    "chatroom_id": "room_1",
                    "sent_by": "user_sync_missing_account",
                    "sent_at": "2026-06-03T00:00:00Z",
                    "content": {
                        "type": "text",
                        "text": "message survives user sync failure"
                    }
                }
            }),
        );

        processor
            .process_batch_with_retry(&[event], 0, None)
            .await
            .expect("fallback processing");

        test_db
            .database()
            .transaction(|session| {
                Box::pin(async move {
                    let message = message::get_by_id(session, "msg_user_sync_fail", false)
                        .await?
                        .expect("message should be committed before user sync skip");
                    assert_eq!(message.sent_by, "user_sync_missing_account");

                    let offset = event::get_offset(session, "test_processor").await?;
                    assert_eq!(offset, 31);
                    Ok::<_, anyhow::Error>(())
                })
            })
            .await
            .expect("verify committed message and offset");
    }
}
