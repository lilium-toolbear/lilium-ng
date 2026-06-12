use sqlx::PgPool;
use chrono::{DateTime, Utc};
use lilium_models::ingestion::{WebSocketEvent, EventEnvelope};
use anyhow::Result;

pub async fn insert_events(pool: &PgPool, events: &[EventEnvelope]) -> Result<usize> {
    let mut count = 0;
    for event in events {
        sqlx::query(
            r#"INSERT INTO websocket_events (event, data, user_id, timestamp)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&event.event_type)
        .bind(&event.payload)
        .bind(&event.account_user_id)
        .bind(event.received_at)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

pub async fn get_events_after_offset(
    pool: &PgPool,
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
        .fetch_all(pool)
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
        .fetch_all(pool)
        .await?
    };
    Ok(events)
}
