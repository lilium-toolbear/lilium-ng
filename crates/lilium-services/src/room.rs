// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 services/room_service.py
// Ports the RoomService methods used by the sync/history CLIs: get_by_id,
// get_all_rooms (RoomFilters), upsert_room_from_dict, mark_inactive_rooms,
// update_backfill_progress, mark_history_complete. Member/stats methods are
// covered by the room_member service and are not duplicated here.
use crate::Result;
use chrono::{DateTime, Utc};
use lilium_models::dzmm::room::{self as rooms, Model as Room};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use tracing::instrument;
use uuid::Uuid;

/// Filters for [`get_all_rooms`]. Mirrors Python `services/types.py::RoomFilters`.
#[derive(Debug, Clone, Default)]
pub struct RoomFilters {
    pub chat_type: Option<String>,
    pub is_active: Option<bool>,
    pub is_public: Option<bool>,
    pub search_query: Option<String>,
    pub has_messages: Option<bool>,
    pub account_id: Option<Uuid>,
}

/// Parsed room fields extracted from a DZMM API chat dict (`Room.from_api`).
struct ParsedRoom {
    room_id: Uuid,
    title: String,
    chat_type: Option<String>,
    avatar_url: Option<String>,
    member_count: Option<i32>,
    tags: Option<Vec<String>>,
    is_public: Option<bool>,
    creator_id: Option<Uuid>,
    last_message_at: Option<DateTime<Utc>>,
    dissolved_at: Option<DateTime<Utc>>,
}

#[instrument(level = "debug", skip(db), fields(room_id = %room_id))]
pub async fn get_by_id<C>(db: &C, room_id: Uuid) -> Result<Option<Room>>
where
    C: ConnectionTrait,
{
    let room = rooms::Entity::find_by_id(room_id).one(db).await?;
    Ok(room)
}

#[instrument(level = "debug", skip(db), fields(has_filters = filters.is_some()))]
pub async fn get_all_rooms<C>(db: &C, filters: Option<&RoomFilters>) -> Result<Vec<Room>>
where
    C: ConnectionTrait,
{
    let mut query = rooms::Entity::find();
    if let Some(f) = filters {
        if let Some(ref v) = f.chat_type {
            query = query.filter(rooms::Column::ChatType.eq(v.clone()));
        }
        if let Some(v) = f.is_active {
            query = query.filter(rooms::Column::IsActive.eq(v));
        }
        if let Some(v) = f.is_public {
            query = query.filter(rooms::Column::IsPublic.eq(v));
        }
        if let Some(ref q) = f.search_query {
            query = query.filter(rooms::Column::Title.contains(q));
        }
        if Some(true) == f.has_messages {
            query = query.filter(rooms::Column::MessageCount.gt(0));
        }
        // account_id (PG array containment) is applied in Rust below; sea_query's
        // custom-expression placeholder handling does not reliably bind array
        // params, and room counts are modest.
    }
    let account_id_filter = filters.and_then(|f| f.account_id);
    let rooms: Vec<Room> = query
        .order_by_desc(rooms::Column::MessageCount)
        .all(db)
        .await?
        .into_iter()
        .filter(|room| match &account_id_filter {
            Some(account_id) => room.account_ids.contains(account_id),
            None => true,
        })
        .collect();
    Ok(rooms)
}

/// Insert or update a room from an API chat dict (`chat["data"]`).
/// Returns `true` if a new room was created, `false` if an existing one was
/// updated. Mirrors Python `RoomService.upsert_room_from_dict`.
#[instrument(level = "debug", skip(db, data), fields(account_user_id = _account_user_id.is_some()))]
pub async fn upsert_room_from_dict<C>(
    db: &C,
    data: &serde_json::Value,
    _account_user_id: Option<Uuid>,
) -> Result<bool>
where
    C: ConnectionTrait,
{
    let parsed = parse_room_fields(data).ok_or_else(|| {
        lilium_common::LiliumError::domain_service_with_code(
            "ROOM_INVALID_API_DATA",
            "Room data must contain either 'id' or 'chatroomId'".to_string(),
        )
    })?;

    let now = Utc::now();
    if let Some(existing) = get_by_id(db, parsed.room_id).await? {
        // Update existing room (update_from_api semantics: always re-activate).
        let mut active: rooms::ActiveModel = existing.into();
        active.title = Set(parsed.title);
        if parsed.chat_type.is_some() {
            active.chat_type = Set(parsed.chat_type);
        }
        if parsed.avatar_url.is_some() {
            active.avatar_url = Set(parsed.avatar_url);
        }
        if parsed.member_count.is_some() {
            active.member_count = Set(parsed.member_count);
        }
        if data.get("isPublic").is_some() {
            active.is_public = Set(parsed.is_public);
        }
        if parsed.last_message_at.is_some() {
            active.last_message_at = Set(parsed.last_message_at);
        }
        if parsed.dissolved_at.is_some() {
            active.dissolved_at = Set(parsed.dissolved_at);
        }
        if parsed.tags.is_some() {
            active.tags = Set(parsed.tags);
        }
        active.is_active = Set(true);
        active.raw_data = Set(Some(data.clone()));
        active.updated_at = Set(now);

        // Append account_user_id if not already present.
        if let Some(account_id) = _account_user_id {
            let current = active.account_ids.clone().unwrap(); // Unchanged value
            let mut ids: Vec<Uuid> = current.clone();
            if !ids.contains(&account_id) {
                ids.push(account_id);
                active.account_ids = Set(ids);
            }
        }
        active.update(db).await?;
        Ok(false)
    } else {
        let mut account_ids: Vec<Uuid> = Vec::new();
        if let Some(account_id) = _account_user_id {
            account_ids.push(account_id);
        }
        let model = rooms::ActiveModel {
            room_id: Set(parsed.room_id),
            title: Set(parsed.title),
            chat_type: Set(parsed.chat_type),
            avatar_url: Set(parsed.avatar_url),
            member_count: Set(parsed.member_count),
            tags: Set(parsed.tags),
            is_public: Set(parsed.is_public),
            creator_id: Set(parsed.creator_id),
            account_ids: Set(account_ids),
            last_message_at: Set(parsed.last_message_at),
            first_message_at: Set(None),
            backfill_until: Set(None),
            history_complete: Set(false),
            message_count: Set(0),
            deleted_count: Set(0),
            recalled_count: Set(0),
            edited_count: Set(0),
            image_count: Set(0),
            is_active: Set(true),
            dissolved_at: Set(parsed.dissolved_at),
            raw_data: Set(Some(data.clone())),
            created_at: Set(now),
            updated_at: Set(now),
        };
        model.insert(db).await?;
        Ok(true)
    }
}

/// Mark rooms not in `active_room_ids` inactive for the given account. A room
/// is only deactivated when `account_ids` becomes empty after removing the
/// syncing account. Mirrors Python `RoomService.mark_inactive_rooms`.
#[instrument(level = "debug", skip(db, active_room_ids), fields(active_count = active_room_ids.len(), account_user_id = _account_user_id.is_some()))]
pub async fn mark_inactive_rooms<C>(
    db: &C,
    active_room_ids: &[Uuid],
    _account_user_id: Option<Uuid>,
) -> Result<i64>
where
    C: ConnectionTrait,
{
    let mut query = rooms::Entity::find().filter(rooms::Column::IsActive.eq(true));
    if !active_room_ids.is_empty() {
        query = query.filter(rooms::Column::RoomId.is_not_in(active_room_ids.iter().copied()));
    }

    // When an account is syncing, only consider rooms that contain this account
    // (so it can be removed) or have no accounts at all (zombie rooms). The PG
    // array-containment condition is applied in Rust for reliable binding.
    let candidates: Vec<Room> = query.all(db).await?;
    let now = Utc::now();
    let mut inactive_count: i64 = 0;
    for room in candidates {
        if let Some(account_id) = _account_user_id {
            let concerns_account = room.account_ids.contains(&account_id);
            let is_zombie = room.account_ids.is_empty();
            if !concerns_account && !is_zombie {
                continue;
            }
        }
        let mut ids = room.account_ids.clone();
        if let Some(account_id) = _account_user_id {
            ids.retain(|id| *id != account_id);
        }
        let mut active: rooms::ActiveModel = room.into();
        active.account_ids = Set(ids.clone());
        active.updated_at = Set(now);
        if ids.is_empty() {
            active.is_active = Set(false);
            inactive_count += 1;
        }
        active.update(db).await?;
    }
    Ok(inactive_count)
}

#[instrument(level = "debug", skip(db), fields(room_id = %room_id))]
pub async fn update_backfill_progress<C>(db: &C, room_id: Uuid, until: DateTime<Utc>) -> Result<()>
where
    C: ConnectionTrait,
{
    if let Some(room) = get_by_id(db, room_id).await? {
        let mut active: rooms::ActiveModel = room.into();
        active.backfill_until = Set(Some(until));
        active.updated_at = Set(Utc::now());
        active.update(db).await?;
    }
    Ok(())
}

#[instrument(level = "debug", skip(db), fields(room_id = %room_id))]
pub async fn mark_history_complete<C>(db: &C, room_id: Uuid) -> Result<()>
where
    C: ConnectionTrait,
{
    if let Some(room) = get_by_id(db, room_id).await? {
        let mut active: rooms::ActiveModel = room.into();
        active.history_complete = Set(true);
        active.updated_at = Set(Utc::now());
        active.update(db).await?;
    }
    Ok(())
}

/// Extract room fields from a DZMM API chat dict. Mirrors Python `Room.from_api`.
fn parse_room_fields(data: &serde_json::Value) -> Option<ParsedRoom> {
    let room_id = data
        .get("id")
        .and_then(|v| v.as_str())
        .or_else(|| data.get("chatroomId").and_then(|v| v.as_str()))
        .and_then(|s| Uuid::parse_str(s).ok())?;
    let title = data
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Room")
        .to_owned();
    let chat_type = data
        .get("chatType")
        .and_then(|v| v.as_str())
        .map(str::to_owned);
    let avatar_url = data
        .get("avatarUrl")
        .and_then(|v| v.as_str())
        .or_else(|| data.get("avatar").and_then(|v| v.as_str()))
        .map(str::to_owned);
    let member_count = data
        .get("memberCount")
        .and_then(|v| v.as_i64())
        .map(|n| n as i32);
    let tags = data.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|t| t.as_str().map(str::to_owned))
            .collect()
    });
    let is_public = data.get("isPublic").and_then(|v| v.as_bool());
    let creator_id = data
        .get("creatorId")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let last_message_at = parse_datetime(data.get("lastMessageAt").and_then(|v| v.as_str()));
    let dissolved_at = parse_datetime(data.get("dissolvedAt").and_then(|v| v.as_str()));

    Some(ParsedRoom {
        room_id,
        title,
        chat_type,
        avatar_url,
        member_count,
        tags,
        is_public,
        creator_id,
        last_message_at,
        dissolved_at,
    })
}

/// Parse an ISO 8601 datetime string to a UTC DateTime, mirroring Python
/// `parse_datetime` (handles trailing `Z` and naive datetimes).
/// Delegates to `lilium_models::dzmm::parse_optional_datetime`.
pub(crate) fn parse_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    lilium_models::dzmm::parse_optional_datetime(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lilium_test_fixtures::FixtureProfile;
    use lilium_test_fixtures::test_uuid;

    fn chat_data(room_id: Uuid, title: &str) -> serde_json::Value {
        serde_json::json!({
            "chatroomId": room_id.to_string(),
            "title": title,
            "chatType": "group",
            "memberCount": 5,
            "isPublic": true,
            "lastMessageAt": "2024-11-16T04:27:00.123Z",
        })
    }

    #[tokio::test]
    async fn upsert_room_from_dict_inserts_new_room() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");

        lilium_database::transaction!(test_db.database(), |tx| {
            let is_new = upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-1"), "Room One"),
                Some(test_uuid("acct-a")),
            )
            .await
            .unwrap();
            assert!(is_new);
            let room = get_by_id(tx, test_uuid("room-1")).await.unwrap().unwrap();
            assert_eq!(room.title, "Room One");
            assert_eq!(room.chat_type.as_deref(), Some("group"));
            assert_eq!(room.account_ids, vec![test_uuid("acct-a")]);
            assert!(room.is_active);
            Ok(())
        })
        .await
        .expect("insert");
    }

    #[tokio::test]
    async fn upsert_room_from_dict_appends_account_on_update() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");

        lilium_database::transaction!(test_db.database(), |tx| {
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-2"), "Room Two"),
                Some(test_uuid("acct-a")),
            )
            .await
            .unwrap();
            let is_new = upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-2"), "Room Two Updated"),
                Some(test_uuid("acct-b")),
            )
            .await
            .unwrap();
            assert!(!is_new);
            let room = get_by_id(tx, test_uuid("room-2")).await.unwrap().unwrap();
            assert_eq!(room.title, "Room Two Updated");
            assert_eq!(
                room.account_ids,
                vec![test_uuid("acct-a"), test_uuid("acct-b")]
            );
            Ok(())
        })
        .await
        .expect("append account");
    }

    #[tokio::test]
    async fn mark_inactive_rooms_deactivates_when_account_ids_empty() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");

        lilium_database::transaction!(test_db.database(), |tx| {
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-3"), "Room Three"),
                Some(test_uuid("acct-a")),
            )
            .await
            .unwrap();
            // acct-a no longer sees room-3 → mark inactive.
            let inactive = mark_inactive_rooms(tx, &[], Some(test_uuid("acct-a")))
                .await
                .unwrap();
            assert_eq!(inactive, 1);
            let room = get_by_id(tx, test_uuid("room-3")).await.unwrap().unwrap();
            assert!(!room.is_active);
            assert!(room.account_ids.is_empty());
            Ok(())
        })
        .await
        .expect("mark inactive");
    }

    #[tokio::test]
    async fn mark_inactive_rooms_keeps_active_when_other_accounts_remain() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");

        lilium_database::transaction!(test_db.database(), |tx| {
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-4"), "Room Four"),
                Some(test_uuid("acct-a")),
            )
            .await
            .unwrap();
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-4"), "Room Four"),
                Some(test_uuid("acct-b")),
            )
            .await
            .unwrap();
            // acct-a leaves, but acct-b remains.
            let inactive = mark_inactive_rooms(tx, &[], Some(test_uuid("acct-a")))
                .await
                .unwrap();
            assert_eq!(inactive, 0);
            let room = get_by_id(tx, test_uuid("room-4")).await.unwrap().unwrap();
            assert!(room.is_active);
            assert_eq!(room.account_ids, vec![test_uuid("acct-b")]);
            Ok(())
        })
        .await
        .expect("keep active");
    }

    #[tokio::test]
    async fn get_all_rooms_filters_by_account_id() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");

        lilium_database::transaction!(test_db.database(), |tx| {
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-5"), "Room Five"),
                Some(test_uuid("acct-a")),
            )
            .await
            .unwrap();
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-6"), "Room Six"),
                Some(test_uuid("acct-b")),
            )
            .await
            .unwrap();
            let filters = RoomFilters {
                account_id: Some(test_uuid("acct-a")),
                ..Default::default()
            };
            let rooms = get_all_rooms(tx, Some(&filters)).await.unwrap();
            assert_eq!(rooms.len(), 1);
            assert_eq!(rooms[0].room_id, test_uuid("room-5"));
            Ok(())
        })
        .await
        .expect("filter");
    }

    #[tokio::test]
    async fn update_backfill_progress_and_history_complete() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");

        lilium_database::transaction!(test_db.database(), |tx| {
            upsert_room_from_dict(
                tx,
                &chat_data(test_uuid("room-7"), "Room Seven"),
                Some(test_uuid("acct-a")),
            )
            .await
            .unwrap();
            let ts = parse_datetime(Some("2024-01-01T00:00:00Z")).unwrap();
            update_backfill_progress(tx, test_uuid("room-7"), ts)
                .await
                .unwrap();
            let room = get_by_id(tx, test_uuid("room-7")).await.unwrap().unwrap();
            assert_eq!(room.backfill_until, Some(ts));
            assert!(!room.history_complete);
            mark_history_complete(tx, test_uuid("room-7"))
                .await
                .unwrap();
            let room = get_by_id(tx, test_uuid("room-7")).await.unwrap().unwrap();
            assert!(room.history_complete);
            Ok(())
        })
        .await
        .expect("backfill progress");
    }

    #[test]
    fn parse_datetime_handles_z_suffix_and_naive() {
        let dt = parse_datetime(Some("2024-11-16T04:27:00.123Z")).unwrap();
        assert_eq!(dt.to_rfc3339(), "2024-11-16T04:27:00.123+00:00");
        let dt2 = parse_datetime(Some("2024-01-01T00:00:00")).unwrap();
        assert_eq!(dt2.to_rfc3339(), "2024-01-01T00:00:00+00:00");
        assert!(parse_datetime(None).is_none());
        assert!(parse_datetime(Some("")).is_none());
    }
}
