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
