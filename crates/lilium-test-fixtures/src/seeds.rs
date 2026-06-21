// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 tests/conftest.py

use crate::test_uuid;
use anyhow::{Context, Result};
use chrono::Utc;
use lilium_models::dzmm::{account as dzmm_account, room as rooms, user as users};
use sea_orm::{ConnectionTrait, EntityTrait, Set};
use uuid::Uuid;

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
    room_id: Uuid,
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

fn message_service_test_rooms() -> Vec<RoomSeed> {
    vec![RoomSeed {
        room_id: test_uuid("room1"),
        title: "Room 1",
        history_complete: true,
        message_count: 0,
        deleted_count: 0,
        recalled_count: 0,
        edited_count: 0,
        image_count: 0,
        is_active: true,
    }]
}

pub async fn seed_shared_profile<C: ConnectionTrait>(db: &C) -> Result<()> {
    seed_users(db, DEFAULT_TEST_USERS).await
}

pub async fn seed_user_profile<C: ConnectionTrait>(db: &C) -> Result<()> {
    seed_users(db, DEFAULT_TEST_USERS).await
}

pub async fn seed_message_profile<C: ConnectionTrait>(db: &C) -> Result<()> {
    seed_users(db, MESSAGE_SERVICE_TEST_USERS).await?;
    seed_rooms(db, &message_service_test_rooms()).await
}

pub async fn seed_websocket_profile<C: ConnectionTrait>(db: &C) -> Result<()> {
    seed_users(db, WEBSOCKET_SERVICE_TEST_USERS).await?;
    seed_dzmm_accounts(db, WEBSOCKET_SERVICE_ACCOUNT_USER_IDS).await
}

pub async fn seed_test_users<C: ConnectionTrait>(db: &C, user_ids: &[&str]) -> Result<()> {
    if user_ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    users::Entity::insert_many(user_ids.iter().map(|user_id| users::ActiveModel {
        user_id: Set(test_uuid(user_id)),
        full_name: Set(None),
        message_count: Set(0),
        deleted_count: Set(0),
        recalled_count: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }))
    .exec(db)
    .await
    .context("seed test users")?;

    Ok(())
}

async fn seed_users<C: ConnectionTrait>(db: &C, users_to_seed: &[UserSeed]) -> Result<()> {
    if users_to_seed.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    users::Entity::insert_many(users_to_seed.iter().map(|user| users::ActiveModel {
        user_id: Set(test_uuid(user.user_id)),
        full_name: Set(user.full_name.map(str::to_owned)),
        message_count: Set(user.message_count),
        deleted_count: Set(user.deleted_count),
        recalled_count: Set(user.recalled_count),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }))
    .exec(db)
    .await
    .context("seed users")?;

    Ok(())
}

async fn seed_rooms<C: ConnectionTrait>(db: &C, rooms_to_seed: &[RoomSeed]) -> Result<()> {
    if rooms_to_seed.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    rooms::Entity::insert_many(rooms_to_seed.iter().map(|room| rooms::ActiveModel {
        room_id: Set(room.room_id),
        title: Set(room.title.to_owned()),
        history_complete: Set(room.history_complete),
        message_count: Set(room.message_count),
        deleted_count: Set(room.deleted_count),
        recalled_count: Set(room.recalled_count),
        edited_count: Set(room.edited_count),
        image_count: Set(room.image_count),
        is_active: Set(room.is_active),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }))
    .exec(db)
    .await
    .context("seed rooms")?;

    Ok(())
}

async fn seed_dzmm_accounts<C: ConnectionTrait>(db: &C, user_ids: &[&str]) -> Result<()> {
    if user_ids.is_empty() {
        return Ok(());
    }

    let now = Utc::now();
    dzmm_account::Entity::insert_many(user_ids.iter().map(|user_id| dzmm_account::ActiveModel {
        user_id: Set(test_uuid(user_id)),
        user_profile: Set(serde_json::json!({})),
        is_enabled: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }))
    .exec(db)
    .await
    .context("seed dzmm_account")?;

    Ok(())
}
