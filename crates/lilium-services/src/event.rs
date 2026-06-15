use crate::Result;
use chrono::{DateTime, Utc};
use lilium_database::DbSession;

use lilium_models::ingestion::{EventProcessorOffset, WebSocketEvent};
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct WebSocketEventInsert {
    pub user_id: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[instrument(skip(session, data), fields(user_id = %user_id, event = %event, timestamp = %timestamp))]
pub async fn insert_event(
    session: &mut DbSession,
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
    .fetch_one(session.as_mut())
    .await?;
    Ok(record)
}

#[instrument(skip(session, events), fields(event_count = events.len()))]
pub async fn insert_events(
    session: &mut DbSession,
    events: &[WebSocketEventInsert],
) -> Result<i64> {
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
    let result = q.execute(session.as_mut()).await?;
    Ok(result.rows_affected() as i64)
}

#[instrument(skip(session), fields(limit, user_id = ?user_id, event_type = ?event_type))]
pub async fn get_pending_events(
    session: &mut DbSession,
    limit: i64,
    user_id: Option<&str>,
    event_type: Option<&str>,
) -> Result<Vec<WebSocketEvent>> {
    let mut query =
        String::from("SELECT id, user_id, event, data, timestamp FROM websocket_events WHERE 1=1");
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
    let events = q.fetch_all(session.as_mut()).await?;
    Ok(events)
}

#[instrument(skip(session), fields(last_processed_id, last_processed_timestamp = ?last_processed_timestamp, limit, user_id = ?user_id, event_type = ?event_type))]
pub async fn get_events_after_offset(
    session: &mut DbSession,
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
    let events = q.fetch_all(session.as_mut()).await?;
    Ok(events)
}

#[instrument(skip(session), fields(event_id))]
pub async fn delete_event(session: &mut DbSession, event_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM websocket_events WHERE id = $1")
        .bind(event_id)
        .execute(session.as_mut())
        .await?;
    Ok(())
}

#[instrument(skip(session))]
pub async fn get_queue_depth(session: &mut DbSession) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM websocket_events")
        .fetch_one(session.as_mut())
        .await?;
    Ok(count)
}

#[instrument(skip(session))]
pub async fn get_oldest_event_age(session: &mut DbSession) -> Result<Option<std::time::Duration>> {
    let oldest = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT timestamp FROM websocket_events ORDER BY timestamp ASC, id ASC LIMIT 1",
    )
    .fetch_one(session.as_mut())
    .await?;
    Ok(oldest.and_then(|ts| Utc::now().signed_duration_since(ts).to_std().ok()))
}

#[instrument(skip(session))]
pub async fn get_max_event_id(session: &mut DbSession) -> Result<Option<i64>> {
    let max_id = sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(id) FROM websocket_events")
        .fetch_one(session.as_mut())
        .await?;
    Ok(max_id)
}

#[instrument(skip(session))]
pub async fn get_latest_event_cursor(
    session: &mut DbSession,
) -> Result<(Option<DateTime<Utc>>, i64)> {
    let row = sqlx::query_as::<_, (Option<DateTime<Utc>>, i64)>(
        "SELECT timestamp, id FROM websocket_events ORDER BY timestamp DESC, id DESC LIMIT 1",
    )
    .fetch_optional(session.as_mut())
    .await?;
    Ok(row.unwrap_or((None, 0)))
}

#[instrument(skip(session), fields(user_id = ?user_id, event_type = ?event_type))]
pub async fn get_latest_event(
    session: &mut DbSession,
    user_id: Option<&str>,
    event_type: Option<&str>,
) -> Result<Option<WebSocketEvent>> {
    let mut query =
        String::from("SELECT id, user_id, event, data, timestamp FROM websocket_events WHERE 1=1");
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
    let event = q.fetch_optional(session.as_mut()).await?;
    Ok(event)
}

#[instrument(skip(session), fields(event_id))]
pub async fn get_latest_timestamp_for_id(
    session: &mut DbSession,
    event_id: i64,
) -> Result<Option<DateTime<Utc>>> {
    let ts = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
        "SELECT MAX(timestamp) FROM websocket_events WHERE id = $1",
    )
    .bind(event_id)
    .fetch_one(session.as_mut())
    .await?;
    Ok(ts)
}

#[instrument(skip(session), fields(last_timestamp = ?last_timestamp, last_id, limit))]
pub async fn poll_events(
    session: &mut DbSession,
    last_timestamp: Option<DateTime<Utc>>,
    last_id: i64,
    limit: i64,
) -> Result<Vec<WebSocketEvent>> {
    if last_timestamp.is_some() {
        get_events_after_offset(session, last_id, last_timestamp, limit, None, None).await
    } else {
        get_events_after_offset(session, last_id, None, limit, None, None).await
    }
}

#[instrument(skip(session), fields(processor_id = %processor_id))]
pub async fn get_cursor(
    session: &mut DbSession,
    processor_id: &str,
) -> Result<Option<EventProcessorOffset>> {
    let offset = sqlx::query_as::<_, EventProcessorOffset>(
        "SELECT * FROM event_processor_offsets WHERE processor_id = $1",
    )
    .bind(processor_id)
    .fetch_optional(session.as_mut())
    .await?;
    Ok(offset)
}

#[instrument(skip(session), fields(processor_id = %processor_id))]
pub async fn get_offset(session: &mut DbSession, processor_id: &str) -> Result<i64> {
    let offset = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT last_processed_id FROM event_processor_offsets WHERE processor_id = $1",
    )
    .bind(processor_id)
    .fetch_optional(session.as_mut())
    .await?;
    Ok(offset.flatten().unwrap_or(0))
}

#[instrument(skip(session), fields(processor_id = %processor_id, last_processed_id, has_last_processed_timestamp = last_processed_timestamp.is_some(), has_last_processed_at = last_processed_at.is_some()))]
pub async fn update_offset(
    session: &mut DbSession,
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
    .fetch_one(session.as_mut())
    .await?;
    Ok(record)
}

#[instrument(skip(session), fields(processor_id = %processor_id))]
pub async fn delete_offset(session: &mut DbSession, processor_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM event_processor_offsets WHERE processor_id = $1")
        .bind(processor_id)
        .execute(session.as_mut())
        .await?;
    Ok(())
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
    async fn websocket_event_service_roundtrip() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Event)
                .await
                .expect("init event db");

        lilium_database::transaction!(test_db.database(), |session| {
            let now = Utc::now();
            let user_id = unique_event_id();
            insert_event(session, &user_id, "test", json!({"hello": "world"}), now)
                .await
                .expect("insert event");
            let events = get_pending_events(session, 10, Some(&user_id), Some("test"))
                .await
                .expect("pending events");
            assert!(!events.is_empty());
            assert!(events.iter().any(|e| e.user_id == user_id));
            Ok(())
        })
        .await
        .expect("event roundtrip");
    }

    #[tokio::test]
    async fn event_processor_offset_service_roundtrip() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Event)
                .await
                .expect("init event db");

        lilium_database::transaction!(test_db.database(), |session| {
            let processor_id = unique_event_id();

            let offset = get_offset(session, &processor_id)
                .await
                .expect("initial offset");
            assert_eq!(offset, 0);

            let updated = update_offset(
                session,
                &processor_id,
                42,
                Some(Utc::now()),
                Some(Utc::now()),
            )
            .await
            .expect("update offset");
            assert_eq!(updated.processor_id, processor_id);
            assert_eq!(updated.last_processed_id, 42);

            let cursor = get_cursor(session, &processor_id)
                .await
                .expect("get cursor")
                .expect("cursor exists");
            assert_eq!(cursor.last_processed_id, 42);

            delete_offset(session, &processor_id)
                .await
                .expect("delete offset");
            Ok(())
        })
        .await
        .expect("offset roundtrip");
    }
}
