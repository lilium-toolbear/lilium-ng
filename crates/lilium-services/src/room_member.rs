use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use lilium_database::DbSessionContext;

use lilium_models::dzmm::room_member::RoomMember;

pub struct RoomMemberService<'a> {
    session: DbSessionContext<'a>,
}

impl<'a> RoomMemberService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    pub async fn get_member_info(
        &mut self,
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
        .fetch_optional(self.session.as_mut())
        .await?;
        Ok(member)
    }

    pub async fn is_member(&mut self, room_id: &str, user_id: &str) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM room_members
               WHERE room_id = $1 AND user_id = $2 AND left_at IS NULL"#,
        )
        .bind(room_id)
        .bind(user_id)
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(count > 0)
    }

    pub async fn get_active_members_by_ids(
        &mut self,
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
        .fetch_all(self.session.as_mut())
        .await?;
        let map = members
            .into_iter()
            .map(|m| (m.user_id.clone(), m))
            .collect();
        Ok(map)
    }

    pub async fn upsert_member(
        &mut self,
        room_id: &str,
        user_id: &str,
        role: &str,
        joined_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO room_members (room_id, user_id, role, joined_at)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (room_id, user_id) DO UPDATE SET
                   role = $3,
                   joined_at = $4,
                   left_at = NULL"#,
        )
        .bind(room_id)
        .bind(user_id)
        .bind(role)
        .bind(joined_at)
        .execute(self.session.as_mut())
        .await?;
        Ok(())
    }

    pub async fn upsert_member_simple(
        &mut self,
        room_id: &str,
        user_id: &str,
        role: &str,
        joined_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.upsert_member(room_id, user_id, role, joined_at).await
    }

    pub async fn mark_member_left(
        &mut self,
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
        .execute(self.session.as_mut())
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_member_count(&mut self, room_id: &str) -> Result<i64> {
        let count: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM room_members
               WHERE room_id = $1 AND left_at IS NULL"#,
        )
        .bind(room_id)
        .fetch_one(self.session.as_mut())
        .await?;
        Ok(count)
    }

    pub async fn get_room_members(
        &mut self,
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
        .fetch_all(self.session.as_mut())
        .await?;
        Ok(members)
    }
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
        async fn service_struct_can_be_created() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut _svc = RoomMemberService::new(session);
                        Ok(())
                    })
                },
            )
            .await
            .expect("service struct create")
        }

        #[tokio::test]
        async fn get_member_info_existing() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_r", "test_u", "member", Some(now))
                            .await
                            .expect("upsert");
                        let member = svc
                            .get_member_info("test_r", "test_u")
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
                },
            )
            .await
            .expect("member_info existing")
        }

        #[tokio::test]
        async fn get_member_info_nonexistent() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let member = svc
                            .get_member_info("__no_room__", "__no_user__")
                            .await
                            .expect("query");
                        assert!(member.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("member_info nonexistent")
        }

        #[tokio::test]
        async fn get_member_info_wrong_room() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_r1", "test_u", "member", Some(now))
                            .await
                            .expect("upsert");
                        let member = svc
                            .get_member_info("test_r2", "test_u")
                            .await
                            .expect("query");
                        assert!(member.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("member_info wrong room")
        }

        #[tokio::test]
        async fn is_member_true() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_is_member", "test_u", "member", Some(now))
                            .await
                            .expect("upsert");
                        assert!(svc
                            .is_member("test_is_member", "test_u")
                            .await
                            .expect("query"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("is_member true")
        }

        #[tokio::test]
        async fn is_member_false() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        assert!(!svc
                            .is_member("__no_room__", "__no_user__")
                            .await
                            .expect("query"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("is_member false")
        }

        #[tokio::test]
        async fn get_active_members_by_ids_deduplicates() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_active", "test_u1", "member", Some(now))
                            .await
                            .expect("upsert");
                        let members = svc
                            .get_active_members_by_ids(
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
                },
            )
            .await
            .expect("deduplicate members")
        }

        #[tokio::test]
        async fn get_active_members_by_ids_empty() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let members = svc
                            .get_active_members_by_ids("test_r", &[], None)
                            .await
                            .expect("query");
                        assert!(members.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("members_by_ids empty")
        }

        #[tokio::test]
        async fn upsert_new_member() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_new_r", "test_new_u", "member", Some(now))
                            .await
                            .expect("upsert");
                        let member = svc
                            .get_member_info("test_new_r", "test_new_u")
                            .await
                            .expect("query");
                        assert!(member.is_some());
                        Ok(())
                    })
                },
            )
            .await
            .expect("upsert new member")
        }

        #[tokio::test]
        async fn upsert_existing_member_updates_role() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_upd_r", "test_upd_u", "member", Some(now))
                            .await
                            .expect("upsert member");
                        svc.upsert_member("test_upd_r", "test_upd_u", "admin", Some(now))
                            .await
                            .expect("upsert admin");
                        let member = svc
                            .get_member_info("test_upd_r", "test_upd_u")
                            .await
                            .expect("query");
                        assert_eq!(member.unwrap().role.as_deref(), Some("admin"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("upsert existing")
        }

        #[tokio::test]
        async fn upsert_member_simple_new() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member_simple(
                            "test_simple_r",
                            "test_simple_u",
                            "creator",
                            Some(now),
                        )
                        .await
                        .expect("upsert");
                        let member = svc
                            .get_member_info("test_simple_r", "test_simple_u")
                            .await
                            .expect("query");
                        assert_eq!(member.unwrap().role.as_deref(), Some("creator"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("upsert member simple")
        }

        #[tokio::test]
        async fn upsert_member_simple_reactivates_left_member() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_rejoin_r", "test_rejoin_u", "member", Some(now))
                            .await
                            .expect("upsert");
                        svc.mark_member_left("test_rejoin_r", "test_rejoin_u", Some(Utc::now()))
                            .await
                            .expect("mark left");
                        svc.upsert_member_simple(
                            "test_rejoin_r",
                            "test_rejoin_u",
                            "member",
                            Some(Utc::now()),
                        )
                        .await
                        .expect("rejoin");
                        let member = svc
                            .get_member_info("test_rejoin_r", "test_rejoin_u")
                            .await
                            .expect("query");
                        assert!(member.unwrap().left_at.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("reactivates left member")
        }

        #[tokio::test]
        async fn mark_member_left_sets_left_at() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_leave_r", "test_leave_u", "member", Some(now))
                            .await
                            .expect("upsert");
                        let left_at = Utc::now();
                        let marked = svc
                            .mark_member_left("test_leave_r", "test_leave_u", Some(left_at))
                            .await
                            .expect("mark left");
                        assert!(marked);
                        let member = svc
                            .get_member_info("test_leave_r", "test_leave_u")
                            .await
                            .expect("query");
                        assert!(member.unwrap().left_at.is_some());
                        Ok(())
                    })
                },
            )
            .await
            .expect("mark left at")
        }

        #[tokio::test]
        async fn mark_member_left_nonexistent_returns_false() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let marked = svc
                            .mark_member_left("__no_room__", "__no_user__", None)
                            .await
                            .expect("mark left");
                        assert!(!marked);
                        Ok(())
                    })
                },
            )
            .await
            .expect("mark left nonexistent")
        }

        #[tokio::test]
        async fn get_member_count_zero_empty_room() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let count = svc.get_member_count("__empty_room__").await.expect("count");
                        assert_eq!(count, 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("member_count empty")
        }

        #[tokio::test]
        async fn get_member_count_correct() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_count_r", "test_u1", "member", Some(now))
                            .await
                            .expect("upsert u1");
                        svc.upsert_member("test_count_r", "test_u2", "member", Some(now))
                            .await
                            .expect("upsert u2");
                        svc.upsert_member("test_count_r", "test_u3", "member", Some(now))
                            .await
                            .expect("upsert u3");
                        let count = svc.get_member_count("test_count_r").await.expect("count");
                        assert_eq!(count, 3);
                        Ok(())
                    })
                },
            )
            .await
            .expect("member_count")
        }

        #[tokio::test]
        async fn get_room_members_all() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_list_r", "test_u1", "member", Some(now))
                            .await
                            .expect("upsert");
                        svc.upsert_member("test_list_r", "test_u2", "admin", Some(now))
                            .await
                            .expect("upsert");
                        let members = svc
                            .get_room_members("test_list_r", 100, 0)
                            .await
                            .expect("query");
                        assert_eq!(members.len(), 2);
                        let ids: std::collections::HashSet<_> =
                            members.into_iter().map(|m| m.user_id).collect();
                        assert!(ids.contains("test_u1"));
                        assert!(ids.contains("test_u2"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("room_members all")
        }

        #[tokio::test]
        async fn get_room_members_empty_room() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let members = svc
                            .get_room_members("__empty_room__", 100, 0)
                            .await
                            .expect("query");
                        assert!(members.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("room_members empty")
        }

        #[tokio::test]
        async fn get_room_members_does_not_cross_rooms() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::RoomMember,
                |session| {
                    Box::pin(async move {
                        let mut svc = RoomMemberService::new(session);
                        let now = Utc::now();
                        svc.upsert_member("test_room_a", "test_u1", "member", Some(now))
                            .await
                            .expect("upsert");
                        svc.upsert_member("test_room_b", "test_u2", "member", Some(now))
                            .await
                            .expect("upsert");
                        let members = svc
                            .get_room_members("test_room_a", 100, 0)
                            .await
                            .expect("query");
                        assert_eq!(members.len(), 1);
                        assert_eq!(members[0].user_id, "test_u1");
                        Ok(())
                    })
                },
            )
            .await
            .expect("room_members no cross-room")
        }
    }
}
