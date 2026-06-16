use std::collections::HashMap;

use crate::Result;
use chrono::{DateTime, Utc};
use lilium_models::dzmm::room_member as room_members;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use tracing::instrument;

type RoomMember = room_members::Model;

#[instrument(skip(db), fields(room_id = %room_id, user_id = %user_id))]
pub async fn get_member_info(
    db: &impl ConnectionTrait,
    room_id: &str,
    user_id: &str,
) -> Result<Option<RoomMember>> {
    let member = room_members::Entity::find_by_id((room_id.to_owned(), user_id.to_owned()))
        .one(db)
        .await?;
    Ok(member)
}

#[instrument(skip(db), fields(room_id = %room_id, user_id = %user_id))]
pub async fn is_member(db: &impl ConnectionTrait, room_id: &str, user_id: &str) -> Result<bool> {
    let member = room_members::Entity::find()
        .filter(room_members::Column::RoomId.eq(room_id))
        .filter(room_members::Column::UserId.eq(user_id))
        .filter(room_members::Column::LeftAt.is_null())
        .one(db)
        .await?;
    Ok(member.is_some())
}

#[instrument(skip(db, user_ids), fields(room_id = %room_id, user_count = user_ids.len(), has_account_user_id = _account_user_id.is_some()))]
pub async fn get_active_members_by_ids(
    db: &impl ConnectionTrait,
    room_id: &str,
    user_ids: &[String],
    _account_user_id: Option<&str>,
) -> Result<HashMap<String, RoomMember>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let members: Vec<RoomMember> = room_members::Entity::find()
        .filter(room_members::Column::RoomId.eq(room_id))
        .filter(room_members::Column::UserId.is_in(user_ids.iter().cloned()))
        .filter(room_members::Column::LeftAt.is_null())
        .all(db)
        .await?
        .into_iter()
        .collect();
    let map = members
        .into_iter()
        .map(|m| (m.user_id.clone(), m))
        .collect();
    Ok(map)
}

#[instrument(skip(db), fields(room_id = %room_id, user_id = %user_id, role = %role, has_joined_at = joined_at.is_some()))]
pub async fn upsert_member(
    db: &impl ConnectionTrait,
    room_id: &str,
    user_id: &str,
    role: &str,
    joined_at: Option<DateTime<Utc>>,
) -> Result<()> {
    let now = Utc::now();
    room_members::Entity::insert(room_members::ActiveModel {
        room_id: Set(room_id.to_owned()),
        user_id: Set(user_id.to_owned()),
        role: Set(Some(role.to_owned())),
        joined_at: Set(joined_at),
        raw_data: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        left_at: Set(None),
    })
    .on_conflict(
        OnConflict::columns([room_members::Column::RoomId, room_members::Column::UserId])
            .update_columns([
                room_members::Column::Role,
                room_members::Column::JoinedAt,
                room_members::Column::LeftAt,
                room_members::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(db)
    .await?;
    Ok(())
}

#[instrument(skip(db), fields(room_id = %room_id, user_id = %user_id, role = %role, has_joined_at = joined_at.is_some()))]
pub async fn upsert_member_simple(
    db: &impl ConnectionTrait,
    room_id: &str,
    user_id: &str,
    role: &str,
    joined_at: Option<DateTime<Utc>>,
) -> Result<()> {
    upsert_member(db, room_id, user_id, role, joined_at).await
}

#[instrument(skip(db), fields(room_id = %room_id, user_id = %user_id, has_left_at = left_at.is_some()))]
pub async fn mark_member_left(
    db: &impl ConnectionTrait,
    room_id: &str,
    user_id: &str,
    left_at: Option<DateTime<Utc>>,
) -> Result<bool> {
    if let Some(member) = room_members::Entity::find_by_id((room_id.to_owned(), user_id.to_owned()))
        .filter(room_members::Column::LeftAt.is_null())
        .one(db)
        .await?
    {
        let mut active: room_members::ActiveModel = member.into();
        active.left_at = Set(left_at);
        active.update(db).await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[instrument(skip(db), fields(room_id = %room_id))]
pub async fn get_member_count(db: &impl ConnectionTrait, room_id: &str) -> Result<i64> {
    let count = room_members::Entity::find()
        .select_only()
        .column_as(room_members::Column::UserId.count(), "count")
        .filter(room_members::Column::RoomId.eq(room_id))
        .filter(room_members::Column::LeftAt.is_null())
        .into_tuple::<(i64,)>()
        .one(db)
        .await?
        .map(|(count,)| count)
        .unwrap_or(0);
    Ok(count)
}

#[instrument(skip(db), fields(room_id = %room_id, limit, offset))]
pub async fn get_room_members(
    db: &impl ConnectionTrait,
    room_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<RoomMember>> {
    let members: Vec<RoomMember> = room_members::Entity::find()
        .filter(room_members::Column::RoomId.eq(room_id))
        .filter(room_members::Column::LeftAt.is_null())
        .order_by_asc(room_members::Column::JoinedAt)
        .limit(limit as u64)
        .offset(offset as u64)
        .all(db)
        .await?
        .into_iter()
        .collect();
    Ok(members)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    mod room_member_struct {
        use super::*;

        #[test]
        fn construction() {
            let now = Utc::now();
            let m = lilium_models::dzmm::room_member::RoomMember {
                room_id: "r1".into(),
                user_id: "u1".into(),
                role: Some("member".into()),
                joined_at: Some(now),
                left_at: None,
                raw_data: None,
                created_at: now,
                updated_at: now,
            };
            assert_eq!(m.room_id, "r1");
            assert_eq!(m.user_id, "u1");
            assert_eq!(m.role.as_deref(), Some("member"));
            assert!(m.joined_at.is_some());
            assert!(m.left_at.is_none());
        }

        #[test]
        fn admin_role() {
            let now = Utc::now();
            let m = lilium_models::dzmm::room_member::RoomMember {
                room_id: "r1".into(),
                user_id: "u1".into(),
                role: Some("admin".into()),
                joined_at: None,
                left_at: None,
                raw_data: None,
                created_at: now,
                updated_at: now,
            };
            assert_eq!(m.role.as_deref(), Some("admin"));
        }

        #[test]
        fn creator_role() {
            let now = Utc::now();
            let m = lilium_models::dzmm::room_member::RoomMember {
                room_id: "r1".into(),
                user_id: "u1".into(),
                role: Some("creator".into()),
                joined_at: None,
                left_at: None,
                raw_data: None,
                created_at: now,
                updated_at: now,
            };
            assert_eq!(m.role.as_deref(), Some("creator"));
        }

        #[test]
        fn left_member_has_left_at() {
            let now = Utc::now();
            let left_at = Utc::now();
            let m = lilium_models::dzmm::room_member::RoomMember {
                room_id: "r1".into(),
                user_id: "u2".into(),
                role: Some("member".into()),
                joined_at: Some(now),
                left_at: Some(left_at),
                raw_data: None,
                created_at: now,
                updated_at: now,
            };
            assert!(m.left_at.is_some());
        }
    }
    mod room_member_integration {
        use super::*;
        use chrono::Utc;

        #[tokio::test]
        async fn get_member_info_existing() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_r", "test_u", "member", Some(now))
                .await
                .expect("upsert");
            let member = get_member_info(db, "test_r", "test_u")
                .await
                .expect("query");
            assert!(member.is_some());
            if let Some(m) = member {
                assert_eq!(m.room_id, "test_r");
                assert_eq!(m.user_id, "test_u");
                assert_eq!(m.role.as_deref(), Some("member"));
            }
        }

        #[tokio::test]
        async fn get_member_info_nonexistent() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let member = get_member_info(db, "__no_room__", "__no_user__")
                .await
                .expect("query");
            assert!(member.is_none());
        }

        #[tokio::test]
        async fn get_member_info_wrong_room() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_r1", "test_u", "member", Some(now))
                .await
                .expect("upsert");
            let member = get_member_info(db, "test_r2", "test_u")
                .await
                .expect("query");
            assert!(member.is_none());
        }

        #[tokio::test]
        async fn is_member_true() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_is_member", "test_u", "member", Some(now))
                .await
                .expect("upsert");
            assert!(
                is_member(db, "test_is_member", "test_u")
                    .await
                    .expect("query")
            );
        }

        #[tokio::test]
        async fn is_member_false() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            assert!(
                !is_member(db, "__no_room__", "__no_user__")
                    .await
                    .expect("query")
            );
        }

        #[tokio::test]
        async fn get_active_members_by_ids_deduplicates() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_active", "test_u1", "member", Some(now))
                .await
                .expect("upsert");
            let members = get_active_members_by_ids(
                db,
                "test_active",
                &["test_u1".into(), "test_u1".into()],
                None,
            )
            .await
            .expect("query");
            assert_eq!(members.len(), 1);
            assert!(members.contains_key("test_u1"));
        }

        #[tokio::test]
        async fn get_active_members_by_ids_empty() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let members = get_active_members_by_ids(db, "test_r", &[], None)
                .await
                .expect("query");
            assert!(members.is_empty());
        }

        #[tokio::test]
        async fn upsert_new_member() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_new_r", "test_new_u", "member", Some(now))
                .await
                .expect("upsert");
            let member = get_member_info(db, "test_new_r", "test_new_u")
                .await
                .expect("query");
            assert!(member.is_some());
        }

        #[tokio::test]
        async fn upsert_existing_member_updates_role() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_upd_r", "test_upd_u", "member", Some(now))
                .await
                .expect("upsert member");
            upsert_member(db, "test_upd_r", "test_upd_u", "admin", Some(now))
                .await
                .expect("upsert admin");
            let member = get_member_info(db, "test_upd_r", "test_upd_u")
                .await
                .expect("query");
            assert_eq!(member.unwrap().role.as_deref(), Some("admin"));
        }

        #[tokio::test]
        async fn upsert_member_simple_new() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member_simple(db, "test_simple_r", "test_simple_u", "creator", Some(now))
                .await
                .expect("upsert");
            let member = get_member_info(db, "test_simple_r", "test_simple_u")
                .await
                .expect("query");
            assert_eq!(member.unwrap().role.as_deref(), Some("creator"));
        }

        #[tokio::test]
        async fn upsert_member_simple_reactivates_left_member() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_rejoin_r", "test_rejoin_u", "member", Some(now))
                .await
                .expect("upsert");
            mark_member_left(db, "test_rejoin_r", "test_rejoin_u", Some(Utc::now()))
                .await
                .expect("mark left");
            upsert_member_simple(
                db,
                "test_rejoin_r",
                "test_rejoin_u",
                "member",
                Some(Utc::now()),
            )
            .await
            .expect("rejoin");
            let member = get_member_info(db, "test_rejoin_r", "test_rejoin_u")
                .await
                .expect("query");
            assert!(member.unwrap().left_at.is_none());
        }

        #[tokio::test]
        async fn mark_member_left_sets_left_at() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_leave_r", "test_leave_u", "member", Some(now))
                .await
                .expect("upsert");
            let left_at = Utc::now();
            let marked = mark_member_left(db, "test_leave_r", "test_leave_u", Some(left_at))
                .await
                .expect("mark left");
            assert!(marked);
            let member = get_member_info(db, "test_leave_r", "test_leave_u")
                .await
                .expect("query");
            assert!(member.unwrap().left_at.is_some());
        }

        #[tokio::test]
        async fn mark_member_left_nonexistent_returns_false() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let marked = mark_member_left(db, "__no_room__", "__no_user__", None)
                .await
                .expect("mark left");
            assert!(!marked);
        }

        #[tokio::test]
        async fn get_member_count_zero_empty_room() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let count = get_member_count(db, "__empty_room__").await.expect("count");
            assert_eq!(count, 0);
        }

        #[tokio::test]
        async fn get_member_count_correct() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_count_r", "test_u1", "member", Some(now))
                .await
                .expect("upsert u1");
            upsert_member(db, "test_count_r", "test_u2", "member", Some(now))
                .await
                .expect("upsert u2");
            upsert_member(db, "test_count_r", "test_u3", "member", Some(now))
                .await
                .expect("upsert u3");
            let count = get_member_count(db, "test_count_r").await.expect("count");
            assert_eq!(count, 3);
        }

        #[tokio::test]
        async fn get_room_members_all() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_list_r", "test_u1", "member", Some(now))
                .await
                .expect("upsert");
            upsert_member(db, "test_list_r", "test_u2", "admin", Some(now))
                .await
                .expect("upsert");
            let members = get_room_members(db, "test_list_r", 100, 0)
                .await
                .expect("query");
            assert_eq!(members.len(), 2);
            let ids: std::collections::HashSet<_> =
                members.into_iter().map(|m| m.user_id).collect();
            assert!(ids.contains("test_u1"));
            assert!(ids.contains("test_u2"));
        }

        #[tokio::test]
        async fn get_room_members_empty_room() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let members = get_room_members(db, "__empty_room__", 100, 0)
                .await
                .expect("query");
            assert!(members.is_empty());
        }

        #[tokio::test]
        async fn get_room_members_does_not_cross_rooms() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");
            let db = test_db.database().orm();
            let now = Utc::now();
            upsert_member(db, "test_room_a", "test_u1", "member", Some(now))
                .await
                .expect("upsert");
            upsert_member(db, "test_room_b", "test_u2", "member", Some(now))
                .await
                .expect("upsert");
            let members = get_room_members(db, "test_room_a", 100, 0)
                .await
                .expect("query");
            assert_eq!(members.len(), 1);
            assert_eq!(members[0].user_id, "test_u1");
        }
    }
}
