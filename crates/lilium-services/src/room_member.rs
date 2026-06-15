use std::collections::HashMap;

use crate::Result;
use chrono::{DateTime, Utc};
use lilium_database::DbSession;

use lilium_models::dzmm::room_member::RoomMember;
use tracing::instrument;

#[instrument(skip(session), fields(room_id = %room_id, user_id = %user_id))]
pub async fn get_member_info(
    session: &mut DbSession,
    room_id: &str,
    user_id: &str,
) -> Result<Option<RoomMember>> {
    let member = sqlx::query_as::<_, RoomMember>(
        r#"SELECT room_id, user_id, role, joined_at, left_at, raw_data, created_at, updated_at
           FROM room_members
           WHERE room_id = $1 AND user_id = $2"#,
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_optional(session.as_mut())
    .await?;
    Ok(member)
}

#[instrument(skip(session), fields(room_id = %room_id, user_id = %user_id))]
pub async fn is_member(session: &mut DbSession, room_id: &str, user_id: &str) -> Result<bool> {
    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM room_members
           WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL"#,
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_one(session.as_mut())
    .await?;
    Ok(count > 0)
}

#[instrument(skip(session, user_ids), fields(room_id = %room_id, user_count = user_ids.len(), has_account_user_id = _account_user_id.is_some()))]
pub async fn get_active_members_by_ids(
    session: &mut DbSession,
    room_id: &str,
    user_ids: &[String],
    _account_user_id: Option<&str>,
) -> Result<HashMap<String, RoomMember>> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let members = sqlx::query_as::<_, RoomMember>(
        r#"SELECT room_id, user_id, role, joined_at, left_at, raw_data, created_at, updated_at
           FROM room_members
           WHERE room_id = $1 AND user_id = ANY($2) AND left_at IS NULL"#,
    )
    .bind(room_id)
    .bind(user_ids)
    .fetch_all(session.as_mut())
    .await?;
    let map = members
        .into_iter()
        .map(|m| (m.user_id.clone(), m))
        .collect();
    Ok(map)
}

#[instrument(skip(session), fields(room_id = %room_id, user_id = %user_id, role = %role, has_joined_at = joined_at.is_some()))]
pub async fn upsert_member(
    session: &mut DbSession,
    room_id: &str,
    user_id: &str,
    role: &str,
    joined_at: Option<DateTime<Utc>>,
) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        r#"INSERT INTO room_members (room_id, user_id, role, joined_at, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $5)
           ON CONFLICT (room_id, user_id) DO UPDATE SET
               role = $3,
               joined_at = $4,
               left_at = NULL,
               updated_at = EXCLUDED.updated_at"#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(role)
    .bind(joined_at)
    .bind(now)
    .execute(session.as_mut())
    .await?;
    Ok(())
}

#[instrument(skip(session), fields(room_id = %room_id, user_id = %user_id, role = %role, has_joined_at = joined_at.is_some()))]
pub async fn upsert_member_simple(
    session: &mut DbSession,
    room_id: &str,
    user_id: &str,
    role: &str,
    joined_at: Option<DateTime<Utc>>,
) -> Result<()> {
    upsert_member(session, room_id, user_id, role, joined_at).await
}

#[instrument(skip(session), fields(room_id = %room_id, user_id = %user_id, has_left_at = left_at.is_some()))]
pub async fn mark_member_left(
    session: &mut DbSession,
    room_id: &str,
    user_id: &str,
    left_at: Option<DateTime<Utc>>,
) -> Result<bool> {
    let result = sqlx::query(
        r#"UPDATE room_members SET left_at = $3
           WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL"#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(left_at)
    .execute(session.as_mut())
    .await?;
    Ok(result.rows_affected() > 0)
}

#[instrument(skip(session), fields(room_id = %room_id))]
pub async fn get_member_count(session: &mut DbSession, room_id: &str) -> Result<i64> {
    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM room_members
           WHERE room_id = $1 AND left_at IS NULL"#,
    )
    .bind(room_id)
    .fetch_one(session.as_mut())
    .await?;
    Ok(count)
}

#[instrument(skip(session), fields(room_id = %room_id, limit, offset))]
pub async fn get_room_members(
    session: &mut DbSession,
    room_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<RoomMember>> {
    let members = sqlx::query_as::<_, RoomMember>(
        r#"SELECT room_id, user_id, role, joined_at, left_at, raw_data, created_at, updated_at
           FROM room_members
           WHERE room_id = $1 AND left_at IS NULL
           ORDER BY joined_at ASC NULLS LAST
           LIMIT $2 OFFSET $3"#,
    )
    .bind(room_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(session.as_mut())
    .await?;
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
    mod room_member_service_integration {
        use super::*;
        use chrono::Utc;

        #[tokio::test]
        async fn get_member_info_existing() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_r", "test_u", "member", Some(now))
                    .await
                    .expect("upsert");
                let member = get_member_info(session, "test_r", "test_u")
                    .await
                    .expect("query");
                assert!(member.is_some());
                if let Some(m) = member {
                    assert_eq!(m.room_id, "test_r");
                    assert_eq!(m.user_id, "test_u");
                    assert_eq!(m.role.as_deref(), Some("member"));
                }
                Ok(())
            })
            .await
            .expect("member_info existing")
        }

        #[tokio::test]
        async fn get_member_info_nonexistent() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let member = get_member_info(session, "__no_room__", "__no_user__")
                    .await
                    .expect("query");
                assert!(member.is_none());
                Ok(())
            })
            .await
            .expect("member_info nonexistent")
        }

        #[tokio::test]
        async fn get_member_info_wrong_room() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_r1", "test_u", "member", Some(now))
                    .await
                    .expect("upsert");
                let member = get_member_info(session, "test_r2", "test_u")
                    .await
                    .expect("query");
                assert!(member.is_none());
                Ok(())
            })
            .await
            .expect("member_info wrong room")
        }

        #[tokio::test]
        async fn is_member_true() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_is_member", "test_u", "member", Some(now))
                    .await
                    .expect("upsert");
                assert!(
                    is_member(session, "test_is_member", "test_u")
                        .await
                        .expect("query")
                );
                Ok(())
            })
            .await
            .expect("is_member true")
        }

        #[tokio::test]
        async fn is_member_false() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                assert!(
                    !is_member(session, "__no_room__", "__no_user__")
                        .await
                        .expect("query")
                );
                Ok(())
            })
            .await
            .expect("is_member false")
        }

        #[tokio::test]
        async fn get_active_members_by_ids_deduplicates() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_active", "test_u1", "member", Some(now))
                    .await
                    .expect("upsert");
                let members = get_active_members_by_ids(
                    session,
                    "test_active",
                    &["test_u1".into(), "test_u1".into()],
                    None,
                )
                .await
                .expect("query");
                assert_eq!(members.len(), 1);
                assert!(members.contains_key("test_u1"));
                Ok(())
            })
            .await
            .expect("deduplicate members")
        }

        #[tokio::test]
        async fn get_active_members_by_ids_empty() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let members = get_active_members_by_ids(session, "test_r", &[], None)
                    .await
                    .expect("query");
                assert!(members.is_empty());
                Ok(())
            })
            .await
            .expect("members_by_ids empty")
        }

        #[tokio::test]
        async fn upsert_new_member() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_new_r", "test_new_u", "member", Some(now))
                    .await
                    .expect("upsert");
                let member = get_member_info(session, "test_new_r", "test_new_u")
                    .await
                    .expect("query");
                assert!(member.is_some());
                Ok(())
            })
            .await
            .expect("upsert new member")
        }

        #[tokio::test]
        async fn upsert_existing_member_updates_role() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_upd_r", "test_upd_u", "member", Some(now))
                    .await
                    .expect("upsert member");
                upsert_member(session, "test_upd_r", "test_upd_u", "admin", Some(now))
                    .await
                    .expect("upsert admin");
                let member = get_member_info(session, "test_upd_r", "test_upd_u")
                    .await
                    .expect("query");
                assert_eq!(member.unwrap().role.as_deref(), Some("admin"));
                Ok(())
            })
            .await
            .expect("upsert existing")
        }

        #[tokio::test]
        async fn upsert_member_simple_new() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member_simple(session, "test_simple_r", "test_simple_u", "creator", Some(now))
                    .await
                    .expect("upsert");
                let member = get_member_info(session, "test_simple_r", "test_simple_u")
                    .await
                    .expect("query");
                assert_eq!(member.unwrap().role.as_deref(), Some("creator"));
                Ok(())
            })
            .await
            .expect("upsert member simple")
        }

        #[tokio::test]
        async fn upsert_member_simple_reactivates_left_member() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_rejoin_r", "test_rejoin_u", "member", Some(now))
                    .await
                    .expect("upsert");
                mark_member_left(
                    session,
                    "test_rejoin_r",
                    "test_rejoin_u",
                    Some(Utc::now()),
                )
                .await
                .expect("mark left");
                upsert_member_simple(
                    session,
                    "test_rejoin_r",
                    "test_rejoin_u",
                    "member",
                    Some(Utc::now()),
                )
                .await
                .expect("rejoin");
                let member = get_member_info(session, "test_rejoin_r", "test_rejoin_u")
                    .await
                    .expect("query");
                assert!(member.unwrap().left_at.is_none());
                Ok(())
            })
            .await
            .expect("reactivates left member")
        }

        #[tokio::test]
        async fn mark_member_left_sets_left_at() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_leave_r", "test_leave_u", "member", Some(now))
                    .await
                    .expect("upsert");
                let left_at = Utc::now();
                let marked = mark_member_left(
                    session,
                    "test_leave_r",
                    "test_leave_u",
                    Some(left_at),
                )
                .await
                .expect("mark left");
                assert!(marked);
                let member = get_member_info(session, "test_leave_r", "test_leave_u")
                    .await
                    .expect("query");
                assert!(member.unwrap().left_at.is_some());
                Ok(())
            })
            .await
            .expect("mark left at")
        }

        #[tokio::test]
        async fn mark_member_left_nonexistent_returns_false() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let marked = mark_member_left(session, "__no_room__", "__no_user__", None)
                    .await
                    .expect("mark left");
                assert!(!marked);
                Ok(())
            })
            .await
            .expect("mark left nonexistent")
        }

        #[tokio::test]
        async fn get_member_count_zero_empty_room() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let count = get_member_count(session, "__empty_room__")
                    .await
                    .expect("count");
                assert_eq!(count, 0);
                Ok(())
            })
            .await
            .expect("member_count empty")
        }

        #[tokio::test]
        async fn get_member_count_correct() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_count_r", "test_u1", "member", Some(now))
                    .await
                    .expect("upsert u1");
                upsert_member(session, "test_count_r", "test_u2", "member", Some(now))
                    .await
                    .expect("upsert u2");
                upsert_member(session, "test_count_r", "test_u3", "member", Some(now))
                    .await
                    .expect("upsert u3");
                let count = get_member_count(session, "test_count_r")
                    .await
                    .expect("count");
                assert_eq!(count, 3);
                Ok(())
            })
            .await
            .expect("member_count")
        }

        #[tokio::test]
        async fn get_room_members_all() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_list_r", "test_u1", "member", Some(now))
                    .await
                    .expect("upsert");
                upsert_member(session, "test_list_r", "test_u2", "admin", Some(now))
                    .await
                    .expect("upsert");
                let members = get_room_members(session, "test_list_r", 100, 0)
                    .await
                    .expect("query");
                assert_eq!(members.len(), 2);
                let ids: std::collections::HashSet<_> =
                    members.into_iter().map(|m| m.user_id).collect();
                assert!(ids.contains("test_u1"));
                assert!(ids.contains("test_u2"));
                Ok(())
            })
            .await
            .expect("room_members all")
        }

        #[tokio::test]
        async fn get_room_members_empty_room() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let members = get_room_members(session, "__empty_room__", 100, 0)
                    .await
                    .expect("query");
                assert!(members.is_empty());
                Ok(())
            })
            .await
            .expect("room_members empty")
        }

        #[tokio::test]
        async fn get_room_members_does_not_cross_rooms() {
            let test_db = lilium_test_fixtures::TestDb::acquire(
                lilium_test_fixtures::FixtureProfile::RoomMember,
            )
            .await
            .expect("init room member db");

            lilium_database::transaction!(test_db.database(), |session| {
                let now = Utc::now();
                upsert_member(session, "test_room_a", "test_u1", "member", Some(now))
                    .await
                    .expect("upsert");
                upsert_member(session, "test_room_b", "test_u2", "member", Some(now))
                    .await
                    .expect("upsert");
                let members = get_room_members(session, "test_room_a", 100, 0)
                    .await
                    .expect("query");
                assert_eq!(members.len(), 1);
                assert_eq!(members[0].user_id, "test_u1");
                Ok(())
            })
            .await
            .expect("room_members no cross-room")
        }
    }
}
