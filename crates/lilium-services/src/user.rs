use anyhow::{Context, Result};
use chrono::Utc;
use lilium_database::DbSessionContext;
use tracing::info;

use lilium_models::dzmm::user::User;

pub struct UserService<'a> {
    session: DbSessionContext<'a>,
}

#[derive(Debug, Clone)]
pub struct SearchUsersParams {
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct UpsertUserData {
    pub user_id: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_file: Option<String>,
    pub bio: Option<String>,
    pub birthday: Option<String>,
    pub birthday_public: Option<bool>,
    pub quirk: Option<String>,
    pub is_bot: Option<bool>,
    pub gender: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub raw_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

impl<'a> UserService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self { session }
    }

    pub async fn get_by_id(&mut self, user_id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT user_id, full_name, avatar_url, avatar_file, bio, birthday,
                      birthday_public, quirk, is_bot, gender, metadata, raw_data,
                      last_seen, message_count, deleted_count, recalled_count,
                      created_at, updated_at
               FROM users WHERE user_id = $1"#,
        )
        .bind(user_id)
        .fetch_optional(self.session.as_mut())
        .await?;
        Ok(user)
    }

    pub async fn get_by_ids(&mut self, user_ids: &[String]) -> Result<Vec<User>> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }
        let users = sqlx::query_as::<_, User>(
            r#"SELECT user_id, full_name, avatar_url, avatar_file, bio, birthday,
                      birthday_public, quirk, is_bot, gender, metadata, raw_data,
                      last_seen, message_count, deleted_count, recalled_count,
                      created_at, updated_at
               FROM users WHERE user_id = ANY($1)"#,
        )
        .bind(user_ids)
        .fetch_all(self.session.as_mut())
        .await?;
        Ok(users)
    }

    pub async fn search_users(&mut self, params: &SearchUsersParams) -> Result<Vec<User>> {
        let limit = params.limit.unwrap_or(50).min(200);
        let offset = params.offset.unwrap_or(0);

        let users = if let Some(ref query) = params.query {
            if query.trim().is_empty() {
                sqlx::query_as::<_, User>(
                    r#"SELECT user_id, full_name, avatar_url, avatar_file, bio, birthday,
                              birthday_public, quirk, is_bot, gender, metadata, raw_data,
                              last_seen, message_count, deleted_count, recalled_count,
                              created_at, updated_at
                       FROM users
                       ORDER BY updated_at DESC
                       LIMIT $1 OFFSET $2"#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(self.session.as_mut())
                .await?
            } else {
                sqlx::query_as::<_, User>(
                    r#"SELECT user_id, full_name, avatar_url, avatar_file, bio, birthday,
                              birthday_public, quirk, is_bot, gender, metadata, raw_data,
                              last_seen, message_count, deleted_count, recalled_count,
                              created_at, updated_at
                       FROM users
                       WHERE name_tsv @@ plainto_tsquery('simple', $1)
                          OR user_id ILIKE '%' || $1 || '%'
                          OR full_name ILIKE '%' || $1 || '%'
                       ORDER BY updated_at DESC
                       LIMIT $2 OFFSET $3"#,
                )
                .bind(query)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.session.as_mut())
                .await?
            }
        } else {
            sqlx::query_as::<_, User>(
                r#"SELECT user_id, full_name, avatar_url, avatar_file, bio, birthday,
                          birthday_public, quirk, is_bot, gender, metadata, raw_data,
                          last_seen, message_count, deleted_count, recalled_count,
                          created_at, updated_at
                   FROM users
                   ORDER BY updated_at DESC
                   LIMIT $1 OFFSET $2"#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.session.as_mut())
            .await?
        };

        Ok(users)
    }

    pub async fn upsert_user(&mut self, data: &UpsertUserData) -> Result<User> {
        let now = Utc::now();
        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (
                    user_id, full_name, avatar_url, avatar_file, bio, birthday,
                    birthday_public, quirk, is_bot, gender, metadata, raw_data,
                    message_count, deleted_count, recalled_count,
                    created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10, $11, $12,
                    0, 0, 0,
                    $13, $13
                )
                ON CONFLICT (user_id) DO UPDATE SET
                    full_name = COALESCE(EXCLUDED.full_name, users.full_name),
                    avatar_url = COALESCE(EXCLUDED.avatar_url, users.avatar_url),
                    avatar_file = COALESCE(EXCLUDED.avatar_file, users.avatar_file),
                    bio = COALESCE(EXCLUDED.bio, users.bio),
                    birthday = COALESCE(EXCLUDED.birthday, users.birthday),
                    birthday_public = COALESCE(EXCLUDED.birthday_public, users.birthday_public),
                    quirk = COALESCE(EXCLUDED.quirk, users.quirk),
                    is_bot = COALESCE(EXCLUDED.is_bot, users.is_bot),
                    gender = COALESCE(EXCLUDED.gender, users.gender),
                    metadata = COALESCE(EXCLUDED.metadata, users.metadata),
                    raw_data = COALESCE(EXCLUDED.raw_data, users.raw_data),
                    last_seen = $13,
                    updated_at = $13
                RETURNING user_id, full_name, avatar_url, avatar_file, bio, birthday,
                          birthday_public, quirk, is_bot, gender, metadata, raw_data,
                          last_seen, message_count, deleted_count, recalled_count,
                          created_at, updated_at"#,
        )
        .bind(&data.user_id)
        .bind(&data.full_name)
        .bind(&data.avatar_url)
        .bind(&data.avatar_file)
        .bind(&data.bio)
        .bind(&data.birthday)
        .bind(data.birthday_public)
        .bind(&data.quirk)
        .bind(data.is_bot)
        .bind(&data.gender)
        .bind(&data.metadata)
        .bind(&data.raw_data)
        .bind(now)
        .fetch_one(self.session.as_mut())
        .await
        .context("Failed to upsert user")?;

        info!(user_id = %data.user_id, "Upserted user");
        Ok(user)
    }

    pub async fn increment_message_count(&mut self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO users (user_id, message_count, deleted_count, recalled_count, created_at, updated_at)
               VALUES ($1, 1, 0, 0, NOW(), NOW())
               ON CONFLICT (user_id) DO UPDATE SET
                   message_count = users.message_count + 1,
                   updated_at = NOW()"#,
        )
        .bind(user_id)
        .execute(self.session.as_mut())
        .await
        .context("Failed to increment message count")?;
        Ok(())
    }

    pub async fn increment_deleted_count(&mut self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO users (user_id, message_count, deleted_count, recalled_count, created_at, updated_at)
               VALUES ($1, 0, 1, 0, NOW(), NOW())
               ON CONFLICT (user_id) DO UPDATE SET
                   deleted_count = users.deleted_count + 1,
                   updated_at = NOW()"#,
        )
        .bind(user_id)
        .execute(self.session.as_mut())
        .await
        .context("Failed to increment deleted count")?;
        Ok(())
    }

    pub async fn increment_recalled_count(&mut self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO users (user_id, message_count, deleted_count, recalled_count, created_at, updated_at)
               VALUES ($1, 0, 0, 1, NOW(), NOW())
               ON CONFLICT (user_id) DO UPDATE SET
                   recalled_count = users.recalled_count + 1,
                   updated_at = NOW()"#,
        )
        .bind(user_id)
        .execute(self.session.as_mut())
        .await
        .context("Failed to increment recalled count")?;
        Ok(())
    }

    pub async fn batch_fetch_and_update(
        &mut self,
        user_room_pairs: &[(String, String)],
    ) -> Result<(i64, i64)> {
        let mut new_count = 0;
        let mut updated_count = 0;

        for (user_id, _room_id) in user_room_pairs {
            let exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM users WHERE user_id = $1)",
            )
            .bind(user_id)
            .fetch_one(self.session.as_mut())
            .await?;

            if exists {
                sqlx::query(
                    "UPDATE users SET last_seen = NOW(), updated_at = NOW() WHERE user_id = $1",
                )
                .bind(user_id)
                .execute(self.session.as_mut())
                .await?;
                updated_count += 1;
            } else {
                sqlx::query(
                    r#"INSERT INTO users (user_id, created_at, updated_at)
                       VALUES ($1, NOW(), NOW())
                       ON CONFLICT (user_id) DO NOTHING"#,
                )
                .bind(user_id)
                .execute(self.session.as_mut())
                .await?;
                new_count += 1;
            }
        }

        info!(
            new = new_count,
            updated = updated_count,
            total = user_room_pairs.len(),
            "Batch fetched users"
        );

        Ok((new_count, updated_count))
    }

    pub async fn fetch_user_profile(&mut self, user_id: &str) -> Result<Option<UserProfile>> {
        let user = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT user_id, full_name, avatar_url FROM users WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(self.session.as_mut())
        .await?;

        Ok(user.map(|(uid, name, avatar)| UserProfile {
            user_id: uid,
            display_name: name,
            avatar_url: avatar,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod user_struct {
        #[test]
        fn test_user() {
            let user = lilium_models::dzmm::user::User {
                user_id: "u1".into(),
                full_name: Some("U".into()),
                message_count: 0,
                deleted_count: 0,
                recalled_count: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                avatar_url: None,
                avatar_file: None,
                bio: None,
                birthday: None,
                birthday_public: None,
                quirk: None,
                is_bot: None,
                gender: None,
                metadata: None,
                raw_data: None,
                last_seen: None,
            };
            assert_eq!(user.user_id, "u1");
        }
    }

    mod user_profile {
        use super::UserProfile;

        #[test]
        fn construction_and_field_access() {
            let p = UserProfile {
                user_id: "u1".into(),
                display_name: Some("Alice".into()),
                avatar_url: Some("https://example.com/avatar.jpg".into()),
            };
            assert_eq!(p.user_id, "u1");
            assert_eq!(p.display_name.as_deref(), Some("Alice"));
            assert_eq!(
                p.avatar_url.as_deref(),
                Some("https://example.com/avatar.jpg")
            );
        }

        #[test]
        fn with_none_fields() {
            let p = UserProfile {
                user_id: "u2".into(),
                display_name: None,
                avatar_url: None,
            };
            assert_eq!(p.user_id, "u2");
            assert!(p.display_name.is_none());
            assert!(p.avatar_url.is_none());
        }
    }

    mod user_service_integration {
        use super::*;

        #[tokio::test]
        async fn service_struct_can_be_created() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let _svc = UserService::new(session);
                        Ok(())
                    })
                },
            )
            .await
            .expect("service struct can be created");
        }

        #[tokio::test]
        async fn get_by_id_existing() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let user = svc.get_by_id("user1").await.expect("query");
                        if let Some(u) = user {
                            assert_eq!(u.user_id, "user1");
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_by_id existing");
        }

        #[tokio::test]
        async fn get_by_id_nonexistent() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let user = svc.get_by_id("__nonexistent__").await.expect("query");
                        assert!(user.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_by_id nonexistent");
        }

        #[tokio::test]
        async fn get_by_ids_multiple() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let users = svc
                            .get_by_ids(&["user1".into(), "user2".into()])
                            .await
                            .expect("query");
                        let ids: std::collections::HashSet<_> =
                            users.iter().map(|u| u.user_id.clone()).collect();
                        assert_eq!(ids.len(), users.len());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_by_ids multiple");
        }

        #[tokio::test]
        async fn get_by_ids_empty() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let users = svc.get_by_ids(&[]).await.expect("query");
                        assert!(users.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_by_ids empty");
        }

        #[tokio::test]
        async fn get_by_ids_with_nonexistent() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let users = svc
                            .get_by_ids(&["user1".into(), "__nonexistent__".into()])
                            .await
                            .expect("query");
                        assert!(users.iter().all(|u| u.user_id != "__nonexistent__"));
                        Ok(())
                    })
                },
            )
            .await
            .expect("get_by_ids_with_nonexistent");
        }

        #[tokio::test]
        async fn search_users_no_filters() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let params = SearchUsersParams {
                            query: None,
                            limit: Some(10),
                            offset: None,
                        };
                        let users = svc.search_users(&params).await.expect("search");
                        assert!(users.len() <= 10);
                        Ok(())
                    })
                },
            )
            .await
            .expect("search no filters");
        }

        #[tokio::test]
        async fn search_users_by_name() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let params = SearchUsersParams {
                            query: Some("One".into()),
                            limit: Some(10),
                            offset: None,
                        };
                        let users = svc.search_users(&params).await.expect("search");
                        assert!(!users.is_empty());
                        Ok(())
                    })
                },
            )
            .await
            .expect("search by name");
        }

        #[tokio::test]
        async fn search_users_with_empty_query() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let params = SearchUsersParams {
                            query: Some("".into()),
                            limit: Some(10),
                            offset: None,
                        };
                        let users = svc.search_users(&params).await.expect("search");
                        assert!(users.len() <= 10);
                        Ok(())
                    })
                },
            )
            .await
            .expect("search empty query");
        }

        #[tokio::test]
        async fn search_users_pagination() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let page1 = SearchUsersParams {
                            query: None,
                            limit: Some(2),
                            offset: Some(0),
                        };
                        let page2 = SearchUsersParams {
                            query: None,
                            limit: Some(2),
                            offset: Some(2),
                        };
                        let users1 = svc.search_users(&page1).await.expect("page1");
                        let users2 = svc.search_users(&page2).await.expect("page2");
                        let ids1: std::collections::HashSet<_> =
                            users1.iter().map(|u| u.user_id.clone()).collect();
                        let ids2: std::collections::HashSet<_> =
                            users2.iter().map(|u| u.user_id.clone()).collect();
                        assert!(ids1.intersection(&ids2).next().is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("search pagination");
        }

        #[tokio::test]
        async fn upsert_new_user() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let data = UpsertUserData {
                            user_id: "new_test_user".into(),
                            full_name: Some("New Test User".into()),
                            bio: Some("Test bio".into()),
                            ..Default::default()
                        };
                        let user = svc.upsert_user(&data).await.expect("upsert");
                        assert_eq!(user.user_id, "new_test_user");
                        Ok(())
                    })
                },
            )
            .await
            .expect("upsert new user");
        }

        #[tokio::test]
        async fn upsert_existing_user() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let data = UpsertUserData {
                            user_id: "user1".into(),
                            bio: Some("Updated bio".into()),
                            ..Default::default()
                        };
                        let user = svc.upsert_user(&data).await.expect("upsert");
                        assert_eq!(user.user_id, "user1");
                        Ok(())
                    })
                },
            )
            .await
            .expect("upsert existing");
        }

        #[tokio::test]
        async fn increment_message_count() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        svc.increment_message_count("user1")
                            .await
                            .expect("increment");
                        Ok(())
                    })
                },
            )
            .await
            .expect("increment message");
        }

        #[tokio::test]
        async fn increment_deleted_count() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        svc.increment_deleted_count("user1")
                            .await
                            .expect("increment");
                        Ok(())
                    })
                },
            )
            .await
            .expect("increment deleted");
        }

        #[tokio::test]
        async fn increment_recalled_count() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        svc.increment_recalled_count("user2")
                            .await
                            .expect("increment");
                        Ok(())
                    })
                },
            )
            .await
            .expect("increment recalled");
        }

        #[tokio::test]
        async fn increment_nonexistent_user() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let result = svc.increment_message_count("__nonexistent__").await;
                        assert!(result.is_ok());
                        Ok(())
                    })
                },
            )
            .await
            .expect("increment nonexistent");
        }

        #[tokio::test]
        async fn fetch_user_profile_existing() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let profile = svc.fetch_user_profile("user1").await.expect("query");
                        if let Some(p) = profile {
                            assert_eq!(p.user_id, "user1");
                        }
                        Ok(())
                    })
                },
            )
            .await
            .expect("fetch user profile existing");
        }

        #[tokio::test]
        async fn fetch_user_profile_nonexistent() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let profile = svc
                            .fetch_user_profile("__nonexistent__")
                            .await
                            .expect("query");
                        assert!(profile.is_none());
                        Ok(())
                    })
                },
            )
            .await
            .expect("fetch user profile nonexistent");
        }

        #[tokio::test]
        async fn batch_fetch_and_update() {
            lilium_database::test_fixtures::with_db_session(
                lilium_database::test_fixtures::TestServiceFixture::User,
                |session| {
                    Box::pin(async move {
                        let mut svc = UserService::new(session);
                        let pairs = vec![("user1".into(), "room1".into())];
                        let (new_count, updated_count) =
                            svc.batch_fetch_and_update(&pairs).await.expect("batch");
                        assert!(new_count >= 0);
                        assert!(updated_count >= 0);
                        Ok(())
                    })
                },
            )
            .await
            .expect("batch fetch and update");
        }
    }
}
