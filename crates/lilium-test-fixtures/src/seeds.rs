use anyhow::{Context, Result};
use chrono::Utc;
use lilium_database::DbSession;
use sqlx::QueryBuilder;

#[derive(Copy, Clone, Debug)]
struct UserSeed {
    user_id: &'static str,
    full_name: Option<&'static str>,
    message_count: i32,
    deleted_count: i32,
    recalled_count: i32,
}

impl UserSeed {
    const fn named(
        user_id: &'static str,
        full_name: &'static str,
        message_count: i32,
        deleted_count: i32,
        recalled_count: i32,
    ) -> Self {
        Self {
            user_id,
            full_name: Some(full_name),
            message_count,
            deleted_count,
            recalled_count,
        }
    }

    const fn anonymous(
        user_id: &'static str,
        message_count: i32,
        deleted_count: i32,
        recalled_count: i32,
    ) -> Self {
        Self {
            user_id,
            full_name: None,
            message_count,
            deleted_count,
            recalled_count,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct RoomSeed {
    room_id: &'static str,
    title: &'static str,
    history_complete: bool,
    message_count: i32,
    deleted_count: i32,
    recalled_count: i32,
    edited_count: i32,
    image_count: i32,
    is_active: bool,
}

const DEFAULT_TEST_USERS: &[UserSeed] = &[
    UserSeed::named("user1", "User One", 3, 1, 2),
    UserSeed::named("user2", "User Two", 0, 0, 0),
    UserSeed::named("test_user", "Test User", 0, 0, 0),
    UserSeed::named("test_user_1", "Another User", 0, 0, 0),
];

const MESSAGE_SERVICE_TEST_USERS: &[UserSeed] = &[
    UserSeed::named("user1", "User One", 5, 1, 1),
    UserSeed::named("user2", "User Two", 0, 0, 0),
    UserSeed::named("test_user", "Test User", 0, 0, 0),
];

const WEBSOCKET_SERVICE_TEST_USERS: &[UserSeed] = &[
    UserSeed::anonymous("user_test_acquire", 0, 0, 0),
    UserSeed::anonymous("user_test_release", 0, 0, 0),
    UserSeed::anonymous("user_test_heartbeat", 0, 0, 0),
    UserSeed::anonymous("user_test_active1", 0, 0, 0),
    UserSeed::anonymous("user_test_active2", 0, 0, 0),
    UserSeed::anonymous("user_test_filter1", 0, 0, 0),
    UserSeed::anonymous("user_test_filter2", 0, 0, 0),
    UserSeed::anonymous("user_test_in_use", 0, 0, 0),
    UserSeed::anonymous("user_test_not_in_use", 0, 0, 0),
    UserSeed::anonymous("user_test_fresh", 0, 0, 0),
    UserSeed::anonymous("user_test_stale", 0, 0, 0),
];

const WEBSOCKET_SERVICE_ACCOUNT_USER_IDS: &[&str] = &[
    "user_test_acquire",
    "user_test_release",
    "user_test_heartbeat",
    "user_test_active1",
    "user_test_active2",
    "user_test_filter1",
    "user_test_filter2",
    "user_test_in_use",
    "user_test_not_in_use",
    "user_test_fresh",
    "user_test_stale",
];

const MESSAGE_SERVICE_TEST_ROOMS: &[RoomSeed] = &[RoomSeed {
    room_id: "room1",
    title: "Room 1",
    history_complete: true,
    message_count: 0,
    deleted_count: 0,
    recalled_count: 0,
    edited_count: 0,
    image_count: 0,
    is_active: true,
}];

pub async fn seed_shared_profile(session: &mut DbSession) -> Result<()> {
    seed_users(session, DEFAULT_TEST_USERS).await
}

pub async fn seed_user_profile(session: &mut DbSession) -> Result<()> {
    seed_users(session, DEFAULT_TEST_USERS).await
}

pub async fn seed_message_profile(session: &mut DbSession) -> Result<()> {
    seed_users(session, MESSAGE_SERVICE_TEST_USERS).await?;
    seed_rooms(session, MESSAGE_SERVICE_TEST_ROOMS).await
}

pub async fn seed_websocket_profile(session: &mut DbSession) -> Result<()> {
    seed_users(session, WEBSOCKET_SERVICE_TEST_USERS).await?;
    seed_dzmm_accounts(session, WEBSOCKET_SERVICE_ACCOUNT_USER_IDS).await
}

pub async fn seed_test_users(session: &mut DbSession, user_ids: &[&str]) -> Result<()> {
    if user_ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    let mut query = QueryBuilder::new(
        "INSERT INTO users (\
            user_id, full_name, message_count, deleted_count, recalled_count, created_at, updated_at\
        ) ",
    );
    query.push_values(user_ids, |mut row, user_id| {
        row.push_bind(user_id);
        let full_name: Option<&str> = None;
        row.push_bind(full_name);
        row.push_bind(0_i32);
        row.push_bind(0_i32);
        row.push_bind(0_i32);
        row.push_bind(now);
        row.push_bind(now);
    });
    query.push(" ON CONFLICT (user_id) DO UPDATE SET full_name = EXCLUDED.full_name");

    query
        .build()
        .execute(session.as_mut())
        .await
        .context("seed test users")?;

    Ok(())
}

async fn seed_users(session: &mut DbSession, users: &[UserSeed]) -> Result<()> {
    if users.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    let mut query = QueryBuilder::new(
        "INSERT INTO users (\
            user_id, full_name, message_count, deleted_count, recalled_count, created_at, updated_at\
        ) ",
    );
    query.push_values(users, |mut row, user| {
        row.push_bind(user.user_id);
        row.push_bind(user.full_name);
        row.push_bind(user.message_count);
        row.push_bind(user.deleted_count);
        row.push_bind(user.recalled_count);
        row.push_bind(now);
        row.push_bind(now);
    });
    query.push(" ON CONFLICT (user_id) DO UPDATE SET full_name = EXCLUDED.full_name");

    query
        .build()
        .execute(session.as_mut())
        .await
        .context("seed users")?;

    Ok(())
}

async fn seed_rooms(session: &mut DbSession, rooms: &[RoomSeed]) -> Result<()> {
    if rooms.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    let mut query = QueryBuilder::new(
        "INSERT INTO rooms (\
            room_id, title, history_complete, message_count, deleted_count, recalled_count,\
            edited_count, image_count, is_active, created_at, updated_at\
        ) ",
    );
    query.push_values(rooms, |mut row, room| {
        row.push_bind(room.room_id);
        row.push_bind(room.title);
        row.push_bind(room.history_complete);
        row.push_bind(room.message_count);
        row.push_bind(room.deleted_count);
        row.push_bind(room.recalled_count);
        row.push_bind(room.edited_count);
        row.push_bind(room.image_count);
        row.push_bind(room.is_active);
        row.push_bind(now);
        row.push_bind(now);
    });
    query.push(" ON CONFLICT (room_id) DO UPDATE SET title = EXCLUDED.title");

    query
        .build()
        .execute(session.as_mut())
        .await
        .context("seed rooms")?;

    Ok(())
}

async fn seed_dzmm_accounts(session: &mut DbSession, user_ids: &[&str]) -> Result<()> {
    if user_ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    let mut query = QueryBuilder::new(
        "INSERT INTO dzmm_account (user_id, user_profile, is_enabled, created_at, updated_at) ",
    );
    query.push_values(user_ids, |mut row, user_id| {
        row.push_bind(user_id);
        row.push_bind(serde_json::json!({}));
        row.push_bind(true);
        row.push_bind(now);
        row.push_bind(now);
    });
    query.push(" ON CONFLICT (user_id) DO UPDATE SET updated_at = EXCLUDED.updated_at");

    query
        .build()
        .execute(session.as_mut())
        .await
        .context("seed dzmm_account")?;

    Ok(())
}
