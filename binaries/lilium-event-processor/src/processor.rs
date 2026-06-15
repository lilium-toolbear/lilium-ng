use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
#[cfg(not(test))]
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{Duration, Instant, sleep};
use tracing::{error, info, warn};

#[cfg(not(test))]
use lilium_common::LiliumError;
use lilium_database::{Database, DbSession};
use lilium_models::ingestion::WebSocketEvent;
#[cfg(not(test))]
use lilium_services::account_service as account;
use lilium_services::event;
use lilium_services::media::MediaService;
use lilium_services::message;
use lilium_services::{room_member, user};

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
    wakeup: Arc<Notify>,
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
            wakeup: Arc::new(Notify::new()),
        }
    }

    pub fn shutdown_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown)
    }

    pub fn wake(&self) {
        self.wakeup.notify_one();
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            processor_id = %self.processor_id,
            batch_size = self.batch_size,
            poll_interval_secs = self.polling_interval.as_secs(),
            "Event processor starting"
        );

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
                _ = self.wakeup.notified() => {
                    if let Err(error) = self.process_next_batch().await {
                        error!(error = %error, "Event processor wakeup failed");
                    }
                    poll_sleep.as_mut().reset(Instant::now() + self.polling_interval);
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

    async fn process_next_batch(&self) -> Result<()> {
        let (events, cursor_id, cursor_timestamp) = self.fetch_batch().await?;
        if events.is_empty() {
            return Ok(());
        }

        info!(count = events.len(), "Processing batch");
        self.process_batch_with_retry(&events, cursor_id, cursor_timestamp)
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
    ) -> Result<()> {
        let mut attempt = 0;
        loop {
            match self
                .process_batch(events, cursor_id, cursor_timestamp)
                .await
            {
                Ok(()) => {
                    return Ok(());
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        error!(
                            attempts = attempt,
                            error = %e,
                            "Max retries exceeded, falling back to per-event processing"
                        );
                        self.process_batch_individually(events, cursor_id, cursor_timestamp)
                            .await?;
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

    async fn process_batch_individually(
        &self,
        events: &[WebSocketEvent],
        cursor_id: i64,
        cursor_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        for event in events {
            let event_slice = std::slice::from_ref(event);
            let event_cursor_id = event.id.unwrap_or(cursor_id);
            let event_cursor_timestamp = Some(event.timestamp).or(cursor_timestamp);

            if let Err(error) = self
                .process_batch(event_slice, event_cursor_id, event_cursor_timestamp)
                .await
            {
                error!(
                    event_id = event_cursor_id,
                    error = %error,
                    "Skipping poison event during per-event fallback"
                );
                #[cfg(test)]
                eprintln!(
                    "process_batch_individually: event {event_cursor_id} failed, entering skip_batch"
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

        Ok(())
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
        session: &mut DbSession,
        processor_id: String,
        batch_size: i64,
    ) -> Result<(Vec<WebSocketEvent>, i64, Option<DateTime<Utc>>)> {
        let (last_id, last_timestamp) = Self::fetch_last_cursor(session, processor_id).await?;
        let events = Self::poll_events(session, last_timestamp, last_id, batch_size).await?;

        Ok((events, last_id, last_timestamp))
    }

    async fn process_batch_inner(
        session: &mut DbSession,
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
        let last_processed_id = i32::try_from(last_id)
            .map_err(|_| anyhow!("event id exceeds event_processor_offsets range"))?;

        Self::sync_users(session, &user_fetch_collector).await?;
        Self::sync_media(session, &media_message_ids).await?;

        event::update_offset(
            session,
            &processor_id,
            last_processed_id,
            last_timestamp,
            Some(Utc::now()),
        )
        .await?;

        Ok(())
    }

    async fn skip_batch_inner(
        session: &mut DbSession,
        processor_id: String,
        last_id: i64,
        last_timestamp: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let last_processed_id = i32::try_from(last_id)
            .map_err(|_| anyhow!("event id exceeds event_processor_offsets range"))?;
        event::update_offset(
            session,
            &processor_id,
            last_processed_id,
            last_timestamp,
            Some(Utc::now()),
        )
        .await?;
        Ok(())
    }

    async fn fetch_last_cursor(
        session: &mut DbSession,
        processor_id: String,
    ) -> Result<(i64, Option<DateTime<Utc>>)> {
        let cursor = event::get_cursor(session, &processor_id).await?;
        Ok(cursor
            .map(|c| (i64::from(c.last_processed_id), c.last_processed_timestamp))
            .unwrap_or((0, None)))
    }

    async fn poll_events(
        session: &mut DbSession,
        last_timestamp: Option<DateTime<Utc>>,
        last_id: i64,
        batch_size: i64,
    ) -> Result<Vec<WebSocketEvent>> {
        event::poll_events(session, last_timestamp, last_id, batch_size)
            .await
            .map_err(Into::into)
    }

    async fn collect_updates_from_events(
        session: &mut DbSession,
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
        session: &mut DbSession,
        user_fetch_collector: &[(String, String, String)],
    ) -> Result<()> {
        if user_fetch_collector.is_empty() {
            return Ok(());
        }

        #[cfg(test)]
        {
            let user_room_pairs: Vec<(String, String)> = user_fetch_collector
                .iter()
                .map(|(_, user_id, room_id)| (user_id.clone(), room_id.clone()))
                .collect();
            return user::batch_fetch_and_update(session, &user_room_pairs)
                .await
                .map(|_| ());
        }

        #[cfg(not(test))]
        {
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

                let auth = account::create_auth_client(&account)?;
                let (new_count, updated_count) =
                    user::batch_fetch_and_update_with_auth(session, &auth, &user_room_pairs, 1)
                        .await?;
                total_new += new_count;
                total_updated += updated_count;
            }

            info!(
                new = total_new,
                updated = total_updated,
                accounts = account_count,
                "Batch fetched users via auth clients"
            );

            Ok(())
        }
    }

    async fn sync_media(
        session: &mut DbSession,
        media_message_ids: &[String],
    ) -> Result<()> {
        if media_message_ids.is_empty() {
            return Ok(());
        }

        let media_service = MediaService::new();
        media_service
            .download_media_batch(session, media_message_ids)
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
        session: &mut DbSession,
        event: &WebSocketEvent,
        user_fetch_collector: &mut Vec<(String, String, String)>,
    ) -> Result<Option<String>> {
        #[cfg(test)]
        if event.event == "__test:fail" {
            anyhow::bail!("test-only event processor failure");
        }

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

                    if msg.content_type == "system" {
                        if let Some(content_text) = msg.content_text.as_deref() {
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
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str()) {
                    if let Some(room_id) = event.data.get("roomId").and_then(|v| v.as_str()) {
                        user_fetch_collector.push((
                            event.user_id.clone(),
                            user_id.to_string(),
                            room_id.to_string(),
                        ));
                    }
                }
                Ok(None)
            }
            "group:member-joined" => {
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str()) {
                    if let Some(room_id) = event.data.get("chatroomId").and_then(|v| v.as_str()) {
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
                }
                Ok(None)
            }
            "group:member-left" => {
                if let Some(user_id) = event.data.get("userId").and_then(|v| v.as_str()) {
                    if let Some(room_id) = event.data.get("chatroomId").and_then(|v| v.as_str()) {
                        let _ = room_member::mark_member_left(
                            session,
                            room_id,
                            user_id,
                            Some(event.timestamp),
                        )
                        .await?;
                    }
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
    use lilium_test_fixtures::{FixtureProfile, TestDb, with_db_session};
    use sqlx::query_as;

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
            id: Some(id),
            event: event.to_string(),
            data,
            user_id: user_id.to_string(),
            timestamp: utc(timestamp),
        }
    }

    async fn room_member_row(
        session: &mut DbSession,
        room_id: &str,
        user_id: &str,
    ) -> Result<Option<RoomMember>> {
        let row = query_as::<_, RoomMember>(
            r#"SELECT room_id, user_id, role, joined_at, left_at, raw_data, created_at, updated_at
               FROM room_members
               WHERE room_id = $1 AND user_id = $2"#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_optional(session.as_mut())
        .await?;
        Ok(row)
    }

    #[tokio::test]
    async fn message_new_system_join_updates_room_members() {
        with_db_session(FixtureProfile::Shared, |session| {
            Box::pin(async move {
                let mut session = session;
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
                let result =
                    EventProcessor::process_event(&mut session, &event, &mut user_fetch_collector)
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

                let mut member_session = session;
                let member = room_member_row(&mut member_session, "room_1", "user_joined")
                    .await?
                    .expect("member row");
                assert_eq!(member.room_id, "room_1");
                assert_eq!(member.user_id, "user_joined");
                assert_eq!(member.joined_at, Some(utc("2026-06-02T12:00:00Z")));
                assert!(member.left_at.is_none());

                Ok(())
            })
        })
        .await
        .expect("message_new_system_join_updates_room_members");
    }

    #[tokio::test]
    async fn group_member_left_marks_room_member_left() {
        with_db_session(FixtureProfile::Shared, |session| {
            Box::pin(async move {
                let mut session = session;
                room_member::upsert_member_simple(
                    &mut session,
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
                EventProcessor::process_event(&mut session, &event, &mut user_fetch_collector)
                    .await
                    .expect("process event");

                assert!(user_fetch_collector.is_empty());

                let member = room_member_row(&mut session, "room_1", "user_left")
                    .await?
                    .expect("member row");
                assert_eq!(member.left_at, Some(utc("2026-06-02T12:30:00Z")));

                Ok(())
            })
        })
        .await
        .expect("group_member_left_marks_room_member_left");
    }

    #[tokio::test]
    async fn message_deleted_uses_deleted_by() {
        with_db_session(FixtureProfile::Shared, |session| {
            Box::pin(async move {
                let mut session = session;
                let sent_at = utc("2026-06-01T00:00:00Z");
                let message = DzmmMessage {
                    message_id: "msg_deleted".to_string(),
                    room_id: "room_1".to_string(),
                    sent_at,
                    sent_by: "user_deleted".to_string(),
                    content_type: "text".to_string(),
                    content_text: Some("hello".to_string()),
                    content_tsv: None,
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
                message::create_message(&mut session, &message).await?;

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
                EventProcessor::process_event(&mut session, &event, &mut user_fetch_collector)
                    .await
                    .expect("process event");

                let updated = message::get_by_id_at(&mut session, "msg_deleted", sent_at, false)
                    .await?
                    .expect("message exists");

                assert!(updated.is_deleted);
                assert_eq!(updated.deleted_by.as_deref(), Some("user_admin"));

                Ok(())
            })
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
                "__test:fail",
                "account_1",
                "2026-06-03T00:00:00Z",
                serde_json::json!({}),
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
}
