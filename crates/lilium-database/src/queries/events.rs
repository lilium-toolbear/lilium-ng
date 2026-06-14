use anyhow::Result;
use chrono::{DateTime, Utc};
use lilium_models::ingestion::{EventEnvelope, WebSocketEvent};
use sqlx::{Executor, Postgres};

pub async fn insert_events<'e, E>(exec: &mut E, events: &[EventEnvelope]) -> Result<usize>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
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
        .execute(&mut *exec)
        .await?;
        count += 1;
    }
    Ok(count)
}

pub async fn get_events_after_offset<'e, E>(
    exec: &mut E,
    last_timestamp: Option<DateTime<Utc>>,
    last_id: i64,
    limit: i64,
) -> Result<Vec<WebSocketEvent>>
where
    for<'q> &'q mut E: Executor<'q, Database = Postgres>,
{
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
        .fetch_all(&mut *exec)
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
        .fetch_all(&mut *exec)
        .await?
    };
    Ok(events)
}
