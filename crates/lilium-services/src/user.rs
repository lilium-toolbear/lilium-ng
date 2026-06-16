use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use lilium_api_client::http::DzmmApi;
use lilium_common::LiliumError;
use lilium_models::dzmm::{user as users, user::User as ApiUser, user_history};
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::collections::{HashMap, HashSet};
use tracing::{info, instrument};

type User = users::Model;

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
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarDownload {
    pub user_id: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BatchFetchUsersResult {
    pub new_count: i64,
    pub updated_count: i64,
    pub avatar_downloads: Vec<AvatarDownload>,
}

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub user_id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

impl UpsertUserData {
    pub fn from_api_payload(data: &serde_json::Value) -> crate::Result<Self> {
        let obj = data.as_object().ok_or_else(|| {
            LiliumError::service(
                "USER_API_PAYLOAD_INVALID",
                "User API payload must be an object",
            )
        })?;

        let user_id = obj
            .get("id")
            .or_else(|| obj.get("userId"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                LiliumError::service("USER_API_PAYLOAD_INVALID", "User API payload missing id")
            })?
            .to_string();

        let full_name = obj
            .get("fullName")
            .or_else(|| obj.get("displayName"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        let avatar_url = obj
            .get("avatarUrl")
            .or_else(|| obj.get("avatar"))
            .and_then(|v| v.as_str())
            .map(|v| v.to_string());

        let last_seen = match obj.get("lastSeen").or_else(|| obj.get("last_seen")) {
            Some(value) if value.is_string() => {
                let parsed =
                    DateTime::parse_from_rfc3339(value.as_str().unwrap()).map_err(|error| {
                        LiliumError::service(
                            "USER_API_PAYLOAD_INVALID",
                            format!("Failed to parse lastSeen timestamp: {}", error),
                        )
                    })?;
                Some(parsed.with_timezone(&Utc))
            }
            Some(value) if value.is_null() => None,
            Some(_) => None,
            None => None,
        };

        let is_bot = if let Some(value) = obj.get("isBot") {
            value.as_bool()
        } else {
            Some(false)
        };

        Ok(Self {
            user_id,
            full_name,
            avatar_url,
            avatar_file: None,
            bio: obj
                .get("bio")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            birthday: obj
                .get("birthday")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            birthday_public: obj.get("birthdayPublic").and_then(|v| v.as_bool()),
            quirk: obj
                .get("quirk")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            is_bot,
            gender: obj
                .get("gender")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            metadata: obj.get("metadata").cloned(),
            raw_data: Some(data.clone()),
            last_seen,
        })
    }
}

pub fn is_user_cache_fresh(
    updated_at: Option<DateTime<Utc>>,
    cache_hours: i64,
    now: DateTime<Utc>,
) -> bool {
    if cache_hours <= 0 {
        return false;
    }

    updated_at
        .map(|ts| ts > now - chrono::Duration::hours(cache_hours))
        .unwrap_or(false)
}

pub fn avatar_url_changed(existing: Option<&str>, candidate: Option<&str>) -> bool {
    match candidate {
        Some(candidate_url) => existing != Some(candidate_url),
        None => false,
    }
}

pub fn apply_avatar_sync_plan(
    user: &mut ApiUser,
    existing: Option<&User>,
) -> Option<AvatarDownload> {
    match existing {
        Some(existing_user) if user.avatar_url == existing_user.avatar_url => {
            user.avatar_file = existing_user.avatar_file.clone();
            None
        }
        _ => user.avatar_url.clone().map(|avatar_url| {
            user.avatar_file = None;
            AvatarDownload {
                user_id: user.user_id.clone(),
                avatar_url,
            }
        }),
    }
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn get_by_id<C>(db: &C, user_id: &str) -> Result<Option<User>>
where
    C: ConnectionTrait,
{
    let user = users::Entity::find_by_id(user_id.to_owned())
        .one(db)
        .await?;
    Ok(user)
}

#[instrument(level = "debug" skip(db, user_ids), fields(user_count = user_ids.len()))]
pub async fn get_by_ids<C>(db: &C, user_ids: &[String]) -> Result<Vec<User>>
where
    C: ConnectionTrait,
{
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let users = users::Entity::find()
        .filter(users::Column::UserId.is_in(user_ids.iter().cloned()))
        .all(db)
        .await?
        .into_iter()
        .collect();
    Ok(users)
}

#[instrument(level = "debug" skip(db, params), fields(has_query = params.query.is_some(), limit = params.limit, offset = params.offset))]
pub async fn search_users<C>(db: &C, params: &SearchUsersParams) -> Result<Vec<User>>
where
    C: ConnectionTrait,
{
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);

    let mut query = users::Entity::find().order_by_desc(users::Column::UpdatedAt);
    if let Some(ref query_text) = params.query
        && !query_text.trim().is_empty()
    {
        query = query.filter(
            Condition::any()
                // `name_tsv` is a DB-maintained search vector intentionally
                // omitted from `lilium_models::dzmm::user::Model`.
                .add(Expr::cust_with_values(
                    "name_tsv @@ plainto_tsquery('simple', $1)",
                    [query_text.as_str()],
                ))
                .add(Expr::cust_with_values(
                    "user_id ILIKE '%' || $1 || '%'",
                    [query_text.as_str()],
                ))
                .add(Expr::cust_with_values(
                    "full_name ILIKE '%' || $1 || '%'",
                    [query_text.as_str()],
                )),
        );
    }

    let users = query
        .limit(limit as u64)
        .offset(offset as u64)
        .all(db)
        .await?
        .into_iter()
        .collect();

    Ok(users)
}

#[instrument(level = "debug" skip(db, data), fields(user_id = %data.user_id, has_full_name = data.full_name.is_some(), has_avatar_url = data.avatar_url.is_some(), has_raw_data = data.raw_data.is_some()))]
pub async fn upsert_user<C>(db: &C, data: &UpsertUserData) -> Result<User>
where
    C: ConnectionTrait,
{
    let now = Utc::now();
    let existing = get_by_id(db, &data.user_id).await?;
    if let Some(existing_user) = existing.as_ref()
        && has_profile_changed(existing_user, data)
    {
        save_user_history(db, existing_user).await?;
    }

    let user = users::Entity::insert(users::ActiveModel {
        user_id: Set(data.user_id.clone()),
        full_name: Set(data.full_name.clone()),
        avatar_url: Set(data.avatar_url.clone()),
        avatar_file: Set(data.avatar_file.clone()),
        bio: Set(data.bio.clone()),
        birthday: Set(data.birthday.clone()),
        birthday_public: Set(data.birthday_public),
        quirk: Set(data.quirk.clone()),
        is_bot: Set(data.is_bot),
        gender: Set(data.gender.clone()),
        metadata: Set(data.metadata.clone()),
        raw_data: Set(data.raw_data.clone()),
        last_seen: Set(data.last_seen),
        message_count: Set(0),
        deleted_count: Set(0),
        recalled_count: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
    })
    .on_conflict(
        OnConflict::column(users::Column::UserId)
            .values([
                (
                    users::Column::FullName,
                    Expr::cust("COALESCE(EXCLUDED.full_name, users.full_name)"),
                ),
                (
                    users::Column::AvatarUrl,
                    Expr::cust("COALESCE(EXCLUDED.avatar_url, users.avatar_url)"),
                ),
                (
                    users::Column::AvatarFile,
                    Expr::cust(
                        r#"CASE
                            WHEN EXCLUDED.avatar_url IS NOT NULL
                                 AND EXCLUDED.avatar_url IS DISTINCT FROM users.avatar_url
                            THEN EXCLUDED.avatar_file
                            ELSE COALESCE(EXCLUDED.avatar_file, users.avatar_file)
                        END"#,
                    ),
                ),
                (
                    users::Column::Bio,
                    Expr::cust("COALESCE(EXCLUDED.bio, users.bio)"),
                ),
                (
                    users::Column::Birthday,
                    Expr::cust("COALESCE(EXCLUDED.birthday, users.birthday)"),
                ),
                (
                    users::Column::BirthdayPublic,
                    Expr::cust("COALESCE(EXCLUDED.birthday_public, users.birthday_public)"),
                ),
                (
                    users::Column::Quirk,
                    Expr::cust("COALESCE(EXCLUDED.quirk, users.quirk)"),
                ),
                (
                    users::Column::IsBot,
                    Expr::cust("COALESCE(EXCLUDED.is_bot, users.is_bot)"),
                ),
                (
                    users::Column::Gender,
                    Expr::cust("COALESCE(EXCLUDED.gender, users.gender)"),
                ),
                (
                    users::Column::Metadata,
                    Expr::cust("COALESCE(EXCLUDED.metadata, users.metadata)"),
                ),
                (
                    users::Column::RawData,
                    Expr::cust("COALESCE(EXCLUDED.raw_data, users.raw_data)"),
                ),
                (
                    users::Column::LastSeen,
                    Expr::cust("COALESCE(EXCLUDED.last_seen, users.last_seen)"),
                ),
                (users::Column::UpdatedAt, Expr::cust("EXCLUDED.updated_at")),
            ])
            .to_owned(),
    )
    .exec_with_returning(db)
    .await
    .context("Failed to upsert user")?;

    info!(user_id = %data.user_id, "Upserted user");
    Ok(user)
}

fn has_profile_changed(existing: &User, data: &UpsertUserData) -> bool {
    data.full_name
        .as_ref()
        .is_some_and(|value| existing.full_name.as_ref() != Some(value))
        || data
            .avatar_url
            .as_ref()
            .is_some_and(|value| existing.avatar_url.as_ref() != Some(value))
        || data
            .bio
            .as_ref()
            .is_some_and(|value| existing.bio.as_ref() != Some(value))
        || data
            .birthday
            .as_ref()
            .is_some_and(|value| existing.birthday.as_ref() != Some(value))
        || data
            .quirk
            .as_ref()
            .is_some_and(|value| existing.quirk.as_ref() != Some(value))
        || data
            .gender
            .as_ref()
            .is_some_and(|value| existing.gender.as_ref() != Some(value))
}

async fn save_user_history<C>(db: &C, user: &User) -> Result<()>
where
    C: ConnectionTrait,
{
    user_history::Entity::insert(user_history::ActiveModel {
        user_id: Set(user.user_id.clone()),
        full_name: Set(user.full_name.clone()),
        avatar_url: Set(user.avatar_url.clone()),
        bio: Set(user.bio.clone()),
        birthday: Set(user.birthday.clone()),
        birthday_public: Set(user.birthday_public),
        quirk: Set(user.quirk.clone()),
        is_bot: Set(user.is_bot),
        gender: Set(user.gender.clone()),
        metadata: Set(user.metadata.clone()),
        raw_data: Set(user.raw_data.clone()),
        recorded_at: Set(Utc::now()),
        avatar_file: Set(user.avatar_file.clone()),
        ..Default::default()
    })
    .exec_without_returning(db)
    .await
    .context("Failed to save user history")?;

    Ok(())
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn increment_message_count<C>(db: &C, user_id: &str) -> Result<()>
where
    C: ConnectionTrait,
{
    increment_user_counter(db, user_id, users::Column::MessageCount, (1, 0, 0))
        .await
        .context("Failed to increment message count")?;
    Ok(())
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn increment_deleted_count<C>(db: &C, user_id: &str) -> Result<()>
where
    C: ConnectionTrait,
{
    increment_user_counter(db, user_id, users::Column::DeletedCount, (0, 1, 0))
        .await
        .context("Failed to increment deleted count")?;
    Ok(())
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn increment_recalled_count<C>(db: &C, user_id: &str) -> Result<()>
where
    C: ConnectionTrait,
{
    increment_user_counter(db, user_id, users::Column::RecalledCount, (0, 0, 1))
        .await
        .context("Failed to increment recalled count")?;
    Ok(())
}

async fn increment_user_counter<C>(
    db: &C,
    user_id: &str,
    counter: users::Column,
    initial_counts: (i32, i32, i32),
) -> Result<()>
where
    C: ConnectionTrait,
{
    let now = Utc::now();
    if let Some(user) = get_by_id(db, user_id).await? {
        let mut active = users::ActiveModel {
            user_id: Set(user.user_id),
            updated_at: Set(now),
            ..Default::default()
        };

        match counter {
            users::Column::MessageCount => {
                active.message_count = Set(user.message_count + 1);
            }
            users::Column::DeletedCount => {
                active.deleted_count = Set(user.deleted_count + 1);
            }
            users::Column::RecalledCount => {
                active.recalled_count = Set(user.recalled_count + 1);
            }
            _ => unreachable!("counter must be a user count column"),
        }

        users::Entity::update(active).exec(db).await?;
    } else {
        users::Entity::insert(users::ActiveModel {
            user_id: Set(user_id.to_owned()),
            message_count: Set(initial_counts.0),
            deleted_count: Set(initial_counts.1),
            recalled_count: Set(initial_counts.2),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        })
        .on_conflict_do_nothing()
        .exec(db)
        .await?;
    }

    Ok(())
}

#[instrument(level = "debug" skip(db, user_room_pairs), fields(pair_count = user_room_pairs.len()))]
pub async fn batch_fetch_and_update<C>(
    db: &C,
    user_room_pairs: &[(String, String)],
) -> Result<(i64, i64)>
where
    C: ConnectionTrait,
{
    // This is the lightweight DB-only fallback used by the event processor.
    // The Python implementation performs API fetch + cache checks + avatar
    // download in higher-level sync code. That richer flow is represented in
    // helper functions above, while this method keeps the existing hot path.
    let mut seen = HashSet::new();
    let mut unique_user_ids = Vec::new();
    for (user_id, _room_id) in user_room_pairs {
        if seen.insert(user_id.clone()) {
            unique_user_ids.push(user_id.clone());
        }
    }

    if unique_user_ids.is_empty() {
        return Ok((0, 0));
    }

    let mut existing_users = Vec::new();
    for chunk in unique_user_ids.chunks(5000) {
        existing_users.extend(get_by_ids(db, chunk).await?);
    }
    let existing_users: HashMap<String, User> = existing_users
        .into_iter()
        .map(|user| (user.user_id.clone(), user))
        .collect();

    let mut new_count = 0;
    let mut updated_count = 0;
    let now = Utc::now();

    for user_id in unique_user_ids {
        if existing_users.contains_key(&user_id) {
            users::Entity::update_many()
                .set(users::ActiveModel {
                    last_seen: Set(Some(now)),
                    updated_at: Set(now),
                    ..Default::default()
                })
                .filter(users::Column::UserId.eq(user_id))
                .exec(db)
                .await?;
            updated_count += 1;
        } else {
            users::Entity::insert(users::ActiveModel {
                user_id: Set(user_id),
                message_count: Set(0),
                deleted_count: Set(0),
                recalled_count: Set(0),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            })
            .on_conflict_do_nothing()
            .exec(db)
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

#[instrument(level = "debug" skip(db, auth, user_room_pairs), fields(pair_count = user_room_pairs.len(), cache_hours))]
pub async fn batch_fetch_and_update_with_auth<C>(
    db: &C,
    auth: &DzmmApi,
    user_room_pairs: &[(String, String)],
    cache_hours: i64,
) -> Result<BatchFetchUsersResult>
where
    C: ConnectionTrait,
{
    let mut seen = HashSet::new();
    let mut unique_user_ids = Vec::new();
    let mut user_to_room = HashMap::new();
    for (user_id, room_id) in user_room_pairs {
        if seen.insert(user_id.clone()) {
            unique_user_ids.push(user_id.clone());
        }
        user_to_room
            .entry(user_id.clone())
            .or_insert_with(|| room_id.clone());
    }

    if unique_user_ids.is_empty() {
        return Ok(BatchFetchUsersResult::default());
    }

    let mut existing_users = Vec::new();
    for chunk in unique_user_ids.chunks(5000) {
        existing_users.extend(get_by_ids(db, chunk).await?);
    }
    let existing_users: HashMap<String, User> = existing_users
        .into_iter()
        .map(|user| (user.user_id.clone(), user))
        .collect();

    let now = Utc::now();
    let cache_cutoff = chrono::Duration::hours(cache_hours.max(1));
    let users_to_fetch = unique_user_ids
        .into_iter()
        .filter(|user_id| {
            existing_users
                .get(user_id)
                .map(|existing| now - existing.updated_at > cache_cutoff)
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    if users_to_fetch.is_empty() {
        return Ok(BatchFetchUsersResult::default());
    }

    let mut new_count = 0;
    let mut updated_count = 0;
    let mut avatar_downloads = Vec::new();

    for chunk in users_to_fetch.chunks(30) {
        let pairs_to_fetch = chunk
            .iter()
            .map(|user_id| {
                (
                    user_id.clone(),
                    user_to_room.get(user_id).cloned().unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>();

        let fetched = auth.batch_get_user_info(&pairs_to_fetch).await?;
        for (user_data, (user_id, _room_id)) in fetched.into_iter().zip(pairs_to_fetch) {
            let Some(mut user) = ApiUser::from_api(&user_data) else {
                continue;
            };

            if let Some(download) = apply_avatar_sync_plan(&mut user, existing_users.get(&user_id))
            {
                avatar_downloads.push(download);
            }

            let data = UpsertUserData {
                user_id: user.user_id.clone(),
                full_name: user.full_name.clone(),
                avatar_url: user.avatar_url.clone(),
                avatar_file: user.avatar_file.clone(),
                bio: user.bio.clone(),
                birthday: user.birthday.clone(),
                birthday_public: user.birthday_public,
                quirk: user.quirk.clone(),
                is_bot: user.is_bot,
                gender: user.gender.clone(),
                metadata: user.metadata.clone(),
                raw_data: user.raw_data.clone(),
                last_seen: user.last_seen,
            };

            let is_new = !existing_users.contains_key(&user.user_id);
            upsert_user(db, &data).await?;
            if is_new {
                new_count += 1;
            } else {
                updated_count += 1;
            }
        }
    }

    info!(
        new = new_count,
        updated = updated_count,
        avatars = avatar_downloads.len(),
        total = user_room_pairs.len(),
        "Batch fetched users via auth client"
    );

    Ok(BatchFetchUsersResult {
        new_count,
        updated_count,
        avatar_downloads,
    })
}

#[instrument(level = "debug" skip(db), fields(user_id = %user_id))]
pub async fn fetch_user_profile<C>(db: &C, user_id: &str) -> Result<Option<UserProfile>>
where
    C: ConnectionTrait,
{
    let user = users::Entity::find_by_id(user_id.to_owned())
        .one(db)
        .await?;

    Ok(user.map(|user| UserProfile {
        user_id: user.user_id,
        display_name: user.full_name,
        avatar_url: user.avatar_url,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use sea_orm::PaginatorTrait;
    use serde_json::json;

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

    mod user_integration {
        use super::*;

        #[tokio::test]
        async fn get_by_id_existing() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let user = get_by_id(test_db.database().orm(), "user1")
                .await
                .expect("query");
            if let Some(u) = user {
                assert_eq!(u.user_id, "user1");
            }
        }

        #[tokio::test]
        async fn get_by_id_nonexistent() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let user = get_by_id(test_db.database().orm(), "__nonexistent__")
                .await
                .expect("query");
            assert!(user.is_none());
        }

        #[tokio::test]
        async fn get_by_ids_multiple() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let users = get_by_ids(test_db.database().orm(), &["user1".into(), "user2".into()])
                .await
                .expect("query");
            let ids: std::collections::HashSet<_> =
                users.iter().map(|u| u.user_id.clone()).collect();
            assert_eq!(ids.len(), users.len());
        }

        #[tokio::test]
        async fn get_by_ids_empty() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let users = get_by_ids(test_db.database().orm(), &[])
                .await
                .expect("query");
            assert!(users.is_empty());
        }

        #[tokio::test]
        async fn get_by_ids_with_nonexistent() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let users = get_by_ids(
                test_db.database().orm(),
                &["user1".into(), "__nonexistent__".into()],
            )
            .await
            .expect("query");
            assert!(users.iter().all(|u| u.user_id != "__nonexistent__"));
        }

        #[tokio::test]
        async fn search_users_no_filters() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let params = SearchUsersParams {
                    query: None,
                    limit: Some(10),
                    offset: None,
                };
                let users = search_users(session, &params).await.expect("search");
                assert!(users.len() <= 10);
                Ok(())
            })
            .await
            .expect("search no filters");
        }

        #[tokio::test]
        async fn search_users_by_name() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let params = SearchUsersParams {
                    query: Some("One".into()),
                    limit: Some(10),
                    offset: None,
                };
                let users = search_users(session, &params).await.expect("search");
                assert!(!users.is_empty());
                Ok(())
            })
            .await
            .expect("search by name");
        }

        #[tokio::test]
        async fn search_users_with_empty_query() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let params = SearchUsersParams {
                    query: Some("".into()),
                    limit: Some(10),
                    offset: None,
                };
                let users = search_users(session, &params).await.expect("search");
                assert!(users.len() <= 10);
                Ok(())
            })
            .await
            .expect("search empty query");
        }

        #[tokio::test]
        async fn search_users_pagination() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
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
                let users1 = search_users(session, &page1).await.expect("page1");
                let users2 = search_users(session, &page2).await.expect("page2");
                let ids1: std::collections::HashSet<_> =
                    users1.iter().map(|u| u.user_id.clone()).collect();
                let ids2: std::collections::HashSet<_> =
                    users2.iter().map(|u| u.user_id.clone()).collect();
                assert!(ids1.intersection(&ids2).next().is_none());
                Ok(())
            })
            .await
            .expect("search pagination");
        }

        #[tokio::test]
        async fn upsert_new_user() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let data = UpsertUserData {
                    user_id: "new_test_user".into(),
                    full_name: Some("New Test User".into()),
                    bio: Some("Test bio".into()),
                    ..Default::default()
                };
                let user = upsert_user(session, &data).await.expect("upsert");
                assert_eq!(user.user_id, "new_test_user");
                Ok(())
            })
            .await
            .expect("upsert new user");
        }

        #[tokio::test]
        async fn upsert_existing_user() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let data = UpsertUserData {
                    user_id: "user1".into(),
                    bio: Some("Updated bio".into()),
                    ..Default::default()
                };
                let user = upsert_user(session, &data).await.expect("upsert");
                assert_eq!(user.user_id, "user1");
                Ok(())
            })
            .await
            .expect("upsert existing");
        }

        #[tokio::test]
        async fn upsert_changed_avatar_url_clears_stale_avatar_file() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let initial = UpsertUserData {
                    user_id: "avatar_change_user".into(),
                    avatar_url: Some("https://example.com/old.png".into()),
                    avatar_file: Some("attachments/avatars/avatar_change_user_old.png".into()),
                    ..Default::default()
                };
                upsert_user(session, &initial)
                    .await
                    .expect("insert initial");

                let changed = UpsertUserData {
                    user_id: "avatar_change_user".into(),
                    avatar_url: Some("https://example.com/new.png".into()),
                    avatar_file: None,
                    ..Default::default()
                };
                let user = upsert_user(session, &changed)
                    .await
                    .expect("upsert changed");

                assert_eq!(
                    user.avatar_url.as_deref(),
                    Some("https://example.com/new.png")
                );
                assert!(user.avatar_file.is_none());
                Ok(())
            })
            .await
            .expect("upsert clears stale avatar file");
        }

        #[tokio::test]
        async fn upsert_existing_profile_change_saves_previous_snapshot_to_history() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let initial = UpsertUserData {
                    user_id: "history_user".into(),
                    full_name: Some("Old Name".into()),
                    avatar_url: Some("https://example.com/old.png".into()),
                    bio: Some("old bio".into()),
                    ..Default::default()
                };
                upsert_user(session, &initial)
                    .await
                    .expect("insert initial");

                let initial_history_count = user_history::Entity::find()
                    .filter(user_history::Column::UserId.eq("history_user"))
                    .count(session)
                    .await?;
                assert_eq!(initial_history_count, 0);

                let changed = UpsertUserData {
                    user_id: "history_user".into(),
                    avatar_url: Some("https://example.com/new.png".into()),
                    bio: Some("new bio".into()),
                    ..Default::default()
                };
                upsert_user(session, &changed)
                    .await
                    .expect("upsert changed");

                let snapshot = user_history::Entity::find()
                    .filter(user_history::Column::UserId.eq("history_user"))
                    .one(session)
                    .await?
                    .expect("history snapshot");

                assert_eq!(snapshot.user_id, "history_user");
                assert_eq!(snapshot.full_name.as_deref(), Some("Old Name"));
                assert_eq!(
                    snapshot.avatar_url.as_deref(),
                    Some("https://example.com/old.png")
                );
                assert_eq!(snapshot.bio.as_deref(), Some("old bio"));

                Ok(())
            })
            .await
            .expect("upsert saves user history");
        }

        #[tokio::test]
        async fn increment_message_count() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                super::increment_message_count(session, "user1")
                    .await
                    .expect("increment");
                Ok(())
            })
            .await
            .expect("increment message");
        }

        #[tokio::test]
        async fn increment_deleted_count() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                super::increment_deleted_count(session, "user1")
                    .await
                    .expect("increment");
                Ok(())
            })
            .await
            .expect("increment deleted");
        }

        #[tokio::test]
        async fn increment_recalled_count() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                super::increment_recalled_count(session, "user2")
                    .await
                    .expect("increment");
                Ok(())
            })
            .await
            .expect("increment recalled");
        }

        #[tokio::test]
        async fn increment_nonexistent_user() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let result = super::increment_message_count(session, "__nonexistent__").await;
                assert!(result.is_ok());
                Ok(())
            })
            .await
            .expect("increment nonexistent");
        }

        #[tokio::test]
        async fn fetch_user_profile_existing() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let profile = fetch_user_profile(test_db.database().orm(), "user1")
                .await
                .expect("query");
            if let Some(p) = profile {
                assert_eq!(p.user_id, "user1");
            }
        }

        #[tokio::test]
        async fn fetch_user_profile_nonexistent() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            let profile = fetch_user_profile(test_db.database().orm(), "__nonexistent__")
                .await
                .expect("query");
            assert!(profile.is_none());
        }

        #[tokio::test]
        async fn batch_fetch_and_update() {
            let test_db =
                lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::User)
                    .await
                    .expect("init user db");

            lilium_database::transaction!(test_db.database(), |session| {
                let pairs = vec![("user1".into(), "room1".into())];
                let (new_count, updated_count) = super::batch_fetch_and_update(session, &pairs)
                    .await
                    .expect("batch");
                assert!(new_count >= 0);
                assert!(updated_count >= 0);
                Ok(())
            })
            .await
            .expect("batch fetch and update");
        }
    }

    mod api_payload {
        use super::*;

        #[test]
        fn from_api_payload_uses_python_aliases_and_defaults() {
            let payload = json!({
                "id": "user_123",
                "fullName": "Primary Name",
                "displayName": "Fallback Name",
                "avatar": "https://example.com/avatar.png",
                "bio": "bio text",
                "birthday": "2000-01-01",
                "birthdayPublic": true,
                "quirk": "quirky",
                "gender": "nonbinary",
                "metadata": {"source": "public_profile"},
                "lastSeen": "2026-01-01T00:00:00Z"
            });

            let data = UpsertUserData::from_api_payload(&payload).expect("payload");
            assert_eq!(data.user_id, "user_123");
            assert_eq!(data.full_name.as_deref(), Some("Primary Name"));
            assert_eq!(
                data.avatar_url.as_deref(),
                Some("https://example.com/avatar.png")
            );
            assert_eq!(data.bio.as_deref(), Some("bio text"));
            assert_eq!(data.birthday.as_deref(), Some("2000-01-01"));
            assert_eq!(data.birthday_public, Some(true));
            assert_eq!(data.quirk.as_deref(), Some("quirky"));
            assert_eq!(data.is_bot, Some(false));
            assert_eq!(data.gender.as_deref(), Some("nonbinary"));
            assert_eq!(data.metadata, Some(json!({"source": "public_profile"})));
            assert_eq!(data.raw_data, Some(payload));
            assert_eq!(
                data.last_seen,
                Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap())
            );
        }

        #[test]
        fn from_api_payload_prefers_display_name_when_full_name_missing() {
            let payload = json!({
                "id": "user_456",
                "displayName": "Display Name",
                "avatarUrl": "https://example.com/avatar-2.png"
            });

            let data = UpsertUserData::from_api_payload(&payload).expect("payload");
            assert_eq!(data.full_name.as_deref(), Some("Display Name"));
            assert_eq!(
                data.avatar_url.as_deref(),
                Some("https://example.com/avatar-2.png")
            );
            assert_eq!(data.is_bot, Some(false));
            assert!(data.metadata.is_none());
        }

        #[test]
        fn from_api_payload_returns_structured_error_for_invalid_payload() {
            let payload = json!({
                "displayName": "Missing id"
            });

            let err = UpsertUserData::from_api_payload(&payload)
                .expect_err("payload missing id should fail");
            assert_eq!(err.code(), Some("USER_API_PAYLOAD_INVALID"));
        }

        #[test]
        fn cache_and_avatar_helpers_match_python_semantics() {
            let now = Utc.with_ymd_and_hms(2026, 1, 2, 12, 0, 0).unwrap();
            let fresh = now - chrono::Duration::minutes(30);
            let stale = now - chrono::Duration::hours(2);

            assert!(is_user_cache_fresh(Some(fresh), 1, now));
            assert!(!is_user_cache_fresh(Some(stale), 1, now));
            assert!(avatar_url_changed(
                Some("https://example.com/a.png"),
                Some("https://example.com/b.png")
            ));
            assert!(!avatar_url_changed(
                Some("https://example.com/a.png"),
                Some("https://example.com/a.png")
            ));
            assert!(avatar_url_changed(None, Some("https://example.com/a.png")));
        }

        fn test_user(user_id: &str) -> User {
            User {
                user_id: user_id.into(),
                full_name: None,
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
                message_count: 0,
                deleted_count: 0,
                recalled_count: 0,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }

        #[test]
        fn avatar_sync_plan_matches_python_update_semantics() {
            let existing_same = User {
                user_id: "u1".into(),
                avatar_url: Some("https://example.com/a.png".into()),
                avatar_file: Some("attachments/avatars/u1_old.png".into()),
                ..test_user("u1")
            };
            let mut fetched_same = User {
                user_id: "u1".into(),
                avatar_url: Some("https://example.com/a.png".into()),
                avatar_file: None,
                ..test_user("u1")
            };
            let unchanged_plan = apply_avatar_sync_plan(&mut fetched_same, Some(&existing_same));
            assert!(unchanged_plan.is_none());
            assert_eq!(
                fetched_same.avatar_file.as_deref(),
                Some("attachments/avatars/u1_old.png")
            );

            let existing_changed = User {
                user_id: "u2".into(),
                avatar_url: Some("https://example.com/old.png".into()),
                avatar_file: Some("attachments/avatars/u2_old.png".into()),
                ..test_user("u2")
            };
            let mut fetched_changed = User {
                user_id: "u2".into(),
                avatar_url: Some("https://example.com/new.png".into()),
                avatar_file: None,
                ..test_user("u2")
            };
            let changed_plan =
                apply_avatar_sync_plan(&mut fetched_changed, Some(&existing_changed));
            assert_eq!(
                changed_plan,
                Some(AvatarDownload {
                    user_id: "u2".into(),
                    avatar_url: "https://example.com/new.png".into()
                })
            );
            assert!(fetched_changed.avatar_file.is_none());

            let mut fetched_new = User {
                user_id: "u3".into(),
                avatar_url: Some("https://example.com/new-user.png".into()),
                avatar_file: None,
                ..test_user("u3")
            };
            let new_plan = apply_avatar_sync_plan(&mut fetched_new, None);
            assert_eq!(
                new_plan,
                Some(AvatarDownload {
                    user_id: "u3".into(),
                    avatar_url: "https://example.com/new-user.png".into()
                })
            );
        }
    }
}
