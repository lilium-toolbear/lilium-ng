use crate::Result;
use chrono::{DateTime, Utc};
use lilium_common::LiliumError;
use lilium_models::ingestion::{
    event_processor_offset as event_processor_offsets, websocket_event as websocket_events,
};
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use tracing::instrument;

type EventProcessorOffset = event_processor_offsets::Model;
type WebSocketEvent = websocket_events::Model;

#[derive(Debug, Clone)]
pub struct WebSocketEventInsert {
    pub user_id: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[instrument(level = "debug" skip(db, data), fields(user_id = %user_id, event = %event, timestamp = %timestamp))]
pub async fn insert_event(
    db: &impl ConnectionTrait,
    user_id: &str,
    event: &str,
    data: serde_json::Value,
    timestamp: DateTime<Utc>,
) -> Result<WebSocketEvent> {
    let record = websocket_events::Entity::insert(websocket_events::ActiveModel {
        id: Default::default(),
        timestamp: Set(timestamp),
        user_id: Set(user_id.to_owned()),
        event: Set(event.to_owned()),
        data: Set(data),
    })
    .exec_with_returning(db)
    .await?;
    Ok(record)
}

#[instrument(level = "debug" skip(db, events), fields(event_count = events.len()))]
pub async fn insert_events(
    db: &impl ConnectionTrait,
    events: &[WebSocketEventInsert],
) -> Result<i64> {
    if events.is_empty() {
        return Ok(0);
    }

    let inserted = websocket_events::Entity::insert_many(events.iter().cloned().map(|e| {
        websocket_events::ActiveModel {
            id: Default::default(),
            timestamp: Set(e.timestamp),
            user_id: Set(e.user_id),
            event: Set(e.event),
            data: Set(e.data),
        }
    }))
    .exec_with_returning_many(db)
    .await?;

    Ok(inserted.len() as i64)
}

#[instrument(level = "debug" skip(db), fields(limit, user_id = ?user_id, event_type = ?event_type))]
pub async fn get_pending_events(
    db: &impl ConnectionTrait,
    limit: i64,
    user_id: Option<&str>,
    event_type: Option<&str>,
) -> Result<Vec<WebSocketEvent>> {
    let mut query = websocket_events::Entity::find();

    if let Some(uid) = user_id {
        query = query.filter(websocket_events::Column::UserId.eq(uid));
    }
    if let Some(et) = event_type {
        query = query.filter(websocket_events::Column::Event.eq(et));
    }

    Ok(query
        .order_by_asc(websocket_events::Column::Timestamp)
        .order_by_asc(websocket_events::Column::Id)
        .limit(limit as u64)
        .all(db)
        .await?
        .into_iter()
        .collect())
}

#[instrument(level = "debug" skip(db), fields(last_processed_id, last_processed_timestamp = ?last_processed_timestamp, limit, user_id = ?user_id, event_type = ?event_type))]
pub async fn get_events_after_offset(
    db: &impl ConnectionTrait,
    last_processed_id: i64,
    last_processed_timestamp: Option<DateTime<Utc>>,
    limit: i64,
    user_id: Option<&str>,
    event_type: Option<&str>,
) -> Result<Vec<WebSocketEvent>> {
    let mut query = websocket_events::Entity::find();

    if let Some(uid) = user_id {
        query = query.filter(websocket_events::Column::UserId.eq(uid));
    }
    if let Some(et) = event_type {
        query = query.filter(websocket_events::Column::Event.eq(et));
    }

    query = if let Some(ts) = last_processed_timestamp {
        query
            .filter(
                Condition::any()
                    .add(Expr::col(websocket_events::Column::Timestamp).gt(ts))
                    .add(
                        Condition::all()
                            .add(Expr::col(websocket_events::Column::Timestamp).eq(ts))
                            .add(Expr::col(websocket_events::Column::Id).gt(last_processed_id)),
                    ),
            )
            .order_by_asc(websocket_events::Column::Timestamp)
            .order_by_asc(websocket_events::Column::Id)
    } else {
        query
            .filter(websocket_events::Column::Id.gt(last_processed_id))
            .order_by_asc(websocket_events::Column::Id)
    };

    Ok(query
        .limit(limit as u64)
        .all(db)
        .await?
        .into_iter()
        .collect())
}

#[instrument(level = "debug" skip(db), fields(event_id))]
pub async fn delete_event(db: &impl ConnectionTrait, event_id: i64) -> Result<()> {
    websocket_events::Entity::delete_many()
        .filter(websocket_events::Column::Id.eq(event_id))
        .exec(db)
        .await?;
    Ok(())
}

#[instrument(level = "debug" skip(db))]
pub async fn get_queue_depth(db: &impl ConnectionTrait) -> Result<i64> {
    let count = websocket_events::Entity::find()
        .select_only()
        .column_as(websocket_events::Column::Id.count(), "count")
        .into_tuple::<(i64,)>()
        .one(db)
        .await?
        .map(|(count,)| count)
        .unwrap_or(0);
    Ok(count)
}

#[instrument(level = "debug" skip(db))]
pub async fn get_oldest_event_age(
    db: &impl ConnectionTrait,
) -> Result<Option<std::time::Duration>> {
    let oldest = websocket_events::Entity::find()
        .select_only()
        .column(websocket_events::Column::Timestamp)
        .order_by_asc(websocket_events::Column::Timestamp)
        .order_by_asc(websocket_events::Column::Id)
        .into_tuple::<(DateTime<Utc>,)>()
        .one(db)
        .await?
        .map(|(timestamp,)| timestamp);
    Ok(oldest.and_then(|ts| Utc::now().signed_duration_since(ts).to_std().ok()))
}

#[instrument(level = "debug" skip(db))]
pub async fn get_max_event_id(db: &impl ConnectionTrait) -> Result<Option<i64>> {
    let max_id = websocket_events::Entity::find()
        .select_only()
        .column_as(websocket_events::Column::Id.max(), "max_id")
        .into_tuple::<(Option<i64>,)>()
        .one(db)
        .await?
        .and_then(|(max_id,)| max_id);
    Ok(max_id)
}

#[instrument(level = "debug" skip(db))]
pub async fn get_latest_event_cursor(
    db: &impl ConnectionTrait,
) -> Result<(Option<DateTime<Utc>>, i64)> {
    let row = websocket_events::Entity::find()
        .select_only()
        .column(websocket_events::Column::Timestamp)
        .column(websocket_events::Column::Id)
        .order_by_desc(websocket_events::Column::Timestamp)
        .order_by_desc(websocket_events::Column::Id)
        .into_tuple::<(DateTime<Utc>, i64)>()
        .one(db)
        .await?;
    Ok(row
        .map(|(timestamp, id)| (Some(timestamp), id))
        .unwrap_or((None, 0)))
}

#[instrument(level = "debug" skip(db), fields(user_id = ?user_id, event_type = ?event_type))]
pub async fn get_latest_event(
    db: &impl ConnectionTrait,
    user_id: Option<&str>,
    event_type: Option<&str>,
) -> Result<Option<WebSocketEvent>> {
    let mut query = websocket_events::Entity::find();

    if let Some(uid) = user_id {
        query = query.filter(websocket_events::Column::UserId.eq(uid));
    }
    if let Some(et) = event_type {
        query = query.filter(websocket_events::Column::Event.eq(et));
    }

    Ok(query
        .order_by_desc(websocket_events::Column::Timestamp)
        .order_by_desc(websocket_events::Column::Id)
        .limit(1)
        .all(db)
        .await?
        .into_iter()
        .next())
}

#[instrument(level = "debug" skip(db), fields(event_id))]
pub async fn get_latest_timestamp_for_id(
    db: &impl ConnectionTrait,
    event_id: i64,
) -> Result<Option<DateTime<Utc>>> {
    let ts = websocket_events::Entity::find()
        .select_only()
        .column(websocket_events::Column::Timestamp)
        .filter(websocket_events::Column::Id.eq(event_id))
        .order_by_desc(websocket_events::Column::Timestamp)
        .into_tuple::<(DateTime<Utc>,)>()
        .one(db)
        .await?
        .map(|(timestamp,)| timestamp);
    Ok(ts)
}

#[instrument(level = "debug" skip(db), fields(last_timestamp = ?last_timestamp, last_id, limit))]
pub async fn poll_events(
    db: &impl ConnectionTrait,
    last_timestamp: Option<DateTime<Utc>>,
    last_id: i64,
    limit: i64,
) -> Result<Vec<WebSocketEvent>> {
    if last_timestamp.is_some() {
        get_events_after_offset(db, last_id, last_timestamp, limit, None, None).await
    } else {
        get_events_after_offset(db, last_id, None, limit, None, None).await
    }
}

#[instrument(level = "debug" skip(db), fields(processor_id = %processor_id))]
pub async fn get_cursor(
    db: &impl ConnectionTrait,
    processor_id: &str,
) -> Result<Option<EventProcessorOffset>> {
    let offset = event_processor_offsets::Entity::find_by_id(processor_id.to_owned())
        .one(db)
        .await?;
    Ok(offset)
}

#[instrument(level = "debug" skip(db), fields(processor_id = %processor_id))]
pub async fn get_offset(db: &impl ConnectionTrait, processor_id: &str) -> Result<i64> {
    let offset = event_processor_offsets::Entity::find_by_id(processor_id.to_owned())
        .one(db)
        .await?
        .map(|model| model.last_processed_id)
        .unwrap_or(0);
    Ok(offset)
}

#[instrument(level = "debug" skip(db), fields(processor_id = %processor_id, last_processed_id, has_last_processed_timestamp = last_processed_timestamp.is_some(), has_last_processed_at = last_processed_at.is_some()))]
pub async fn update_offset(
    db: &impl ConnectionTrait,
    processor_id: &str,
    last_processed_id: i64,
    last_processed_timestamp: Option<DateTime<Utc>>,
    last_processed_at: Option<DateTime<Utc>>,
) -> Result<EventProcessorOffset> {
    let now = Utc::now();
    event_processor_offsets::Entity::insert(event_processor_offsets::ActiveModel {
        processor_id: Set(processor_id.to_owned()),
        last_processed_id: Set(last_processed_id),
        last_processed_timestamp: Set(last_processed_timestamp),
        last_processed_at: Set(last_processed_at),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::column(event_processor_offsets::Column::ProcessorId)
            .update_columns([
                event_processor_offsets::Column::LastProcessedId,
                event_processor_offsets::Column::LastProcessedTimestamp,
                event_processor_offsets::Column::LastProcessedAt,
                event_processor_offsets::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(db)
    .await?;
    let record = event_processor_offsets::Entity::find_by_id(processor_id.to_owned())
        .one(db)
        .await?
        .ok_or_else(|| {
            LiliumError::service(
                "EVENT_PROCESSOR_OFFSET_UPSERT_RETURNED_NO_ROW",
                "upsert offset must return one row",
            )
        })?;
    Ok(record)
}

#[instrument(level = "debug" skip(db), fields(processor_id = %processor_id))]
pub async fn delete_offset(db: &impl ConnectionTrait, processor_id: &str) -> Result<()> {
    event_processor_offsets::Entity::delete_by_id(processor_id.to_owned())
        .exec(db)
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
        let db = test_db.database().orm();

        let now = Utc::now();
        let user_id = unique_event_id();
        insert_event(db, &user_id, "test", json!({"hello": "world"}), now)
            .await
            .expect("insert event");
        let events = get_pending_events(db, 10, Some(&user_id), Some("test"))
            .await
            .expect("pending events");
        assert!(!events.is_empty());
        assert!(events.iter().any(|e| e.user_id == user_id));
    }

    #[tokio::test]
    async fn event_processor_offset_service_roundtrip() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Event)
                .await
                .expect("init event db");
        let db = test_db.database().orm();

        let processor_id = unique_event_id();

        let offset = get_offset(db, &processor_id).await.expect("initial offset");
        assert_eq!(offset, 0);

        let updated = update_offset(db, &processor_id, 42, Some(Utc::now()), Some(Utc::now()))
            .await
            .expect("update offset");
        assert_eq!(updated.processor_id, processor_id);
        assert_eq!(updated.last_processed_id, 42);

        let cursor = get_cursor(db, &processor_id)
            .await
            .expect("get cursor")
            .expect("cursor exists");
        assert_eq!(cursor.last_processed_id, 42);

        delete_offset(db, &processor_id)
            .await
            .expect("delete offset");
    }
}
