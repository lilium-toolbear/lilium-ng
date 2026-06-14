use anyhow::Result;
use chrono::{DateTime, Utc};
use lilium_database::DbSessionContext;

use lilium_models::ingestion::{EventProcessorOffset, WebSocketEvent};

pub struct WebSocketEventInsert {
    pub user_id: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

pub struct WebSocketEventService<'a> {
    session: DbSessionContext<'a>,
}

impl<'a> WebSocketEventService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    pub async fn insert_event(
        &mut self,
        user_id: &str,
        event: &str,
        data: serde_json::Value,
        timestamp: DateTime<Utc>,
    ) -> Result<WebSocketEvent> {
        let record = sqlx::query_as::<_, WebSocketEvent>(
            r#"INSERT INTO websocket_events (user_id, event, data, timestamp)
               VALUES ($1, $2, $3, $4)
               RETURNING id, user_id, event, data, timestamp"#,
        )
        .bind(user_id)
        .bind(event)
        .bind(data)
        .bind(timestamp)
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(record)
    }

    pub async fn insert_events(&mut self, events: &[WebSocketEventInsert]) -> Result<i64> {
        if events.is_empty() {
            return Ok(0);
        }
        let mut query =
            String::from("INSERT INTO websocket_events (user_id, event, data, timestamp) VALUES ");
        for (i, _) in events.iter().enumerate() {
            if i > 0 {
                query.push(',');
            }
            let offset = i * 4;
            query.push_str(&format!(
                " (${}, ${}, ${}, ${})",
                offset + 1,
                offset + 2,
                offset + 3,
                offset + 4,
            ));
        }

        let mut q = sqlx::query(&query);
        for e in events {
            q = q
                .bind(&e.user_id)
                .bind(&e.event)
                .bind(&e.data)
                .bind(e.timestamp);
        }
        let result = q.execute(self.session.as_mut()).await?;
        Ok(result.rows_affected() as i64)
    }

    pub async fn get_pending_events(
        &mut self,
        limit: i64,
        user_id: Option<&str>,
        event_type: Option<&str>,
    ) -> Result<Vec<WebSocketEvent>> {
        let mut query = String::from(
            "SELECT id, user_id, event, data, timestamp FROM websocket_events WHERE 1=1",
        );
        let mut param_idx = 1;
        if user_id.is_some() {
            query.push_str(&format!(" AND user_id = ${}", param_idx));
            param_idx += 1;
        }
        if event_type.is_some() {
            query.push_str(&format!(" AND event = ${}", param_idx));
            param_idx += 1;
        }
        query.push_str(&format!(
            " ORDER BY timestamp ASC, id ASC LIMIT ${}",
            param_idx
        ));

        let mut q = sqlx::query_as::<_, WebSocketEvent>(&query);
        if let Some(uid) = user_id {
            q = q.bind(uid);
        }
        if let Some(et) = event_type {
            q = q.bind(et);
        }
        q = q.bind(limit);
        let events = q.fetch_all(self.session.as_mut()).await?;
        Ok(events)
    }

    pub async fn get_events_after_offset(
        &mut self,
        last_processed_id: i64,
        last_processed_timestamp: Option<DateTime<Utc>>,
        limit: i64,
        user_id: Option<&str>,
        event_type: Option<&str>,
    ) -> Result<Vec<WebSocketEvent>> {
        let mut conditions = Vec::new();
        let mut param_idx = 1;

        if last_processed_timestamp.is_some() {
            conditions.push(format!(
                "(timestamp > ${} OR (timestamp = ${} AND id > ${}))",
                param_idx,
                param_idx,
                param_idx + 1,
            ));
            param_idx += 2;
        } else {
            conditions.push(format!("id > ${}", param_idx));
            param_idx += 1;
        }

        if user_id.is_some() {
            conditions.push(format!("user_id = ${}", param_idx));
            param_idx += 1;
        }
        if event_type.is_some() {
            conditions.push(format!("event = ${}", param_idx));
            param_idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let order_clause = if last_processed_timestamp.is_some() {
            " ORDER BY timestamp ASC, id ASC"
        } else {
            " ORDER BY id ASC"
        };

        let query = format!(
            "SELECT id, user_id, event, data, timestamp FROM websocket_events{}{} LIMIT ${}",
            where_clause, order_clause, param_idx,
        );

        let mut q = sqlx::query_as::<_, WebSocketEvent>(&query);
        if let Some(ts) = last_processed_timestamp {
            q = q.bind(ts).bind(last_processed_id);
        } else {
            q = q.bind(last_processed_id);
        }
        if let Some(uid) = user_id {
            q = q.bind(uid);
        }
        if let Some(et) = event_type {
            q = q.bind(et);
        }
        q = q.bind(limit);
        let events = q.fetch_all(self.session.as_mut()).await?;
        Ok(events)
    }

    pub async fn delete_event(&mut self, event_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM websocket_events WHERE id = $1")
            .bind(event_id)
            .execute(self.session.as_mut())
            .await?;
        Ok(())
    }

    pub async fn get_queue_depth(&mut self) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM websocket_events")
            .fetch_one(self.session.as_mut())
            .await?;
        Ok(count)
    }

    pub async fn get_oldest_event_age(&mut self) -> Result<Option<std::time::Duration>> {
        let oldest = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            "SELECT timestamp FROM websocket_events ORDER BY timestamp ASC, id ASC LIMIT 1",
        )
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(oldest.and_then(|ts| Utc::now().signed_duration_since(ts).to_std().ok()))
    }

    pub async fn get_max_event_id(&mut self) -> Result<Option<i64>> {
        let max_id = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(id) FROM websocket_events")
            .fetch_one(self.session.as_mut())
            .await?;
        Ok(max_id)
    }

    pub async fn get_latest_event_cursor(&mut self) -> Result<(Option<DateTime<Utc>>, i64)> {
        let row = sqlx::query_as::<_, (Option<DateTime<Utc>>, i64)>(
            "SELECT timestamp, id FROM websocket_events ORDER BY timestamp DESC, id DESC LIMIT 1",
        )
        .fetch_optional(self.session.as_mut())
        .await?;
        Ok(row.unwrap_or((None, 0)))
    }

    pub async fn get_latest_event(
        &mut self,
        user_id: Option<&str>,
        event_type: Option<&str>,
    ) -> Result<Option<WebSocketEvent>> {
        let mut query = String::from(
            "SELECT id, user_id, event, data, timestamp FROM websocket_events WHERE 1=1",
        );
        let mut param_idx = 1;
        if user_id.is_some() {
            query.push_str(&format!(" AND user_id = ${}", param_idx));
            param_idx += 1;
        }
        if event_type.is_some() {
            query.push_str(&format!(" AND event = ${}", param_idx));
            param_idx += 1;
        }
        query.push_str(&format!(
            " ORDER BY timestamp DESC, id DESC LIMIT ${}",
            param_idx
        ));

        let mut q = sqlx::query_as::<_, WebSocketEvent>(&query);
        if let Some(uid) = user_id {
            q = q.bind(uid);
        }
        if let Some(et) = event_type {
            q = q.bind(et);
        }
        q = q.bind(1i64);
        let event = q.fetch_optional(self.session.as_mut()).await?;
        Ok(event)
    }

    pub async fn get_latest_timestamp_for_id(
        &mut self,
        event_id: i64,
    ) -> Result<Option<DateTime<Utc>>> {
        let ts = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            "SELECT MAX(timestamp) FROM websocket_events WHERE id = $1",
        )
        .bind(event_id)
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(ts)
    }

    pub async fn poll_events(
        &mut self,
        last_timestamp: Option<DateTime<Utc>>,
        last_id: i64,
        limit: i64,
    ) -> Result<Vec<WebSocketEvent>> {
        let events = if let Some(ts) = last_timestamp {
            sqlx::query_as::<_, WebSocketEvent>(
                r#"SELECT id, event, data, user_id, timestamp
                   FROM websocket_events
                   WHERE (timestamp > $1) OR (timestamp = $1 AND id > $2)
                   ORDER BY timestamp ASC, id ASC
                   LIMIT $3"#,
            )
            .bind(ts)
            .bind(last_id)
            .bind(limit)
            .fetch_all(self.session.as_mut())
            .await?
        } else {
            sqlx::query_as::<_, WebSocketEvent>(
                r#"SELECT id, event, data, user_id, timestamp
                   FROM websocket_events
                   WHERE id > $1
                   ORDER BY id ASC
                   LIMIT $2"#,
            )
            .bind(last_id)
            .bind(limit)
            .fetch_all(self.session.as_mut())
            .await?
        };
        Ok(events)
    }
}

pub struct EventProcessorOffsetService<'a> {
    session: DbSessionContext<'a>,
}

impl<'a> EventProcessorOffsetService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    pub async fn get_cursor(&mut self, processor_id: &str) -> Result<Option<EventProcessorOffset>> {
        let offset = sqlx::query_as::<_, EventProcessorOffset>(
            "SELECT * FROM event_processor_offsets WHERE processor_id = $1",
        )
        .bind(processor_id)
        .fetch_optional(self.session.as_mut())
        .await?;
        Ok(offset)
    }

    pub async fn get_offset(&mut self, processor_id: &str) -> Result<i64> {
        let offset = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT last_processed_id FROM event_processor_offsets WHERE processor_id = $1",
        )
        .bind(processor_id)
        .fetch_optional(self.session.as_mut())
        .await?;
        Ok(offset.flatten().unwrap_or(0))
    }

    pub async fn update_offset(
        &mut self,
        processor_id: &str,
        last_processed_id: i64,
        last_processed_timestamp: Option<DateTime<Utc>>,
        last_processed_at: Option<DateTime<Utc>>,
    ) -> Result<EventProcessorOffset> {
        let record = sqlx::query_as::<_, EventProcessorOffset>(
            r#"INSERT INTO event_processor_offsets
                   (processor_id, last_processed_id, last_processed_timestamp, last_processed_at, updated_at)
               VALUES ($1, $2, $3, $4, NOW())
               ON CONFLICT (processor_id) DO UPDATE SET
                   last_processed_id = $2,
                   last_processed_timestamp = $3,
                   last_processed_at = $4,
                   updated_at = NOW()
               RETURNING *"#,
        )
        .bind(processor_id)
        .bind(last_processed_id)
        .bind(last_processed_timestamp)
        .bind(last_processed_at)
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(record)
    }

    pub async fn delete_offset(&mut self, processor_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM event_processor_offsets WHERE processor_id = $1")
            .bind(processor_id)
            .execute(self.session.as_mut())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    fn unique_event_id() -> String {
        format!(
            "event_{}_{}",
            Utc::now().timestamp_micros(),
            std::process::id()
        )
    }

    #[tokio::test]
    async fn websocket_event_service_struct_can_be_created() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Event,
            |session| {
                Box::pin(async move {
                    let _svc = WebSocketEventService::new(session);
                    Ok(())
                })
            },
        )
        .await
        .expect("websocket event service create");
    }

    #[tokio::test]
    async fn websocket_event_service_roundtrip() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Event,
            |session| {
                Box::pin(async move {
                    let mut svc = WebSocketEventService::new(session);
                    let now = Utc::now();
                    let user_id = unique_event_id();
                    svc.insert_event(&user_id, "test", json!({"hello": "world"}), now)
                        .await
                        .expect("insert event");
                    let events = svc
                        .get_pending_events(10, Some(&user_id), Some("test"))
                        .await
                        .expect("pending events");
                    assert!(!events.is_empty());
                    assert!(events.iter().any(|e| e.user_id == user_id));
                    Ok(())
                })
            },
        )
        .await
        .expect("event roundtrip");
    }

    #[tokio::test]
    async fn event_processor_offset_service_roundtrip() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Event,
            |session| {
                Box::pin(async move {
                    let mut svc = EventProcessorOffsetService::new(session);
                    let processor_id = unique_event_id();

                    let offset = svc.get_offset(&processor_id).await.expect("initial offset");
                    assert_eq!(offset, 0);

                    let updated = svc
                        .update_offset(&processor_id, 42, Some(Utc::now()), Some(Utc::now()))
                        .await
                        .expect("update offset");
                    assert_eq!(updated.processor_id, processor_id);
                    assert_eq!(updated.last_processed_id, 42);

                    let cursor = svc
                        .get_cursor(&processor_id)
                        .await
                        .expect("get cursor")
                        .expect("cursor exists");
                    assert_eq!(cursor.last_processed_id, 42);

                    svc.delete_offset(&processor_id)
                        .await
                        .expect("delete offset");
                    Ok(())
                })
            },
        )
        .await
        .expect("offset roundtrip");
    }
}
