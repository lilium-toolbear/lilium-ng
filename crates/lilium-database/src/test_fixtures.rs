use crate::pool::{DbPool, DbSession, DbSessionContext, SessionFuture};
use std::borrow::Borrow;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::{Mutex, OwnedMutexGuard};

const TEST_DATABASE_URL_ENV_FILE: &str = "/Users/bearice/Working/github/dzmm_archive/.env";
const TEST_DATABASE_URL_ENV_KEY: &str = "TEST_DATABASE_URL";

pub async fn connect_test_db() -> DbPool {
    let db_url = resolve_test_database_url();
    DbPool::connect(&db_url, 4).await.expect("connect")
}

static TEST_DB_MUTEX: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

fn test_db_mutex() -> &'static Arc<Mutex<()>> {
    TEST_DB_MUTEX.get_or_init(|| Arc::new(Mutex::new(())))
}

pub async fn connect_test_db_lazy() -> DbPool {
    let db_url = resolve_test_database_url();
    DbPool::connect_lazy(&db_url).await.expect("connect")
}

pub struct LockedTestPool {
    pub pool: DbPool,
    _guard: OwnedMutexGuard<()>,
}

impl Deref for LockedTestPool {
    type Target = DbPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl Borrow<DbPool> for LockedTestPool {
    fn borrow(&self) -> &DbPool {
        &self.pool
    }
}

pub async fn connect_test_db_locked() -> LockedTestPool {
    let guard = test_db_mutex().clone().lock_owned().await;
    let pool = connect_test_db().await;
    LockedTestPool {
        pool,
        _guard: guard,
    }
}

pub async fn connect_shared_test_db() -> LockedTestPool {
    let pool = connect_test_db_locked().await;
    pool.with_session_context(|mut session| {
        Box::pin(async move {
            init_shared_test_db(&mut session).await;
            Ok(())
        })
    })
    .await
    .expect("init shared test db");
    pool
}

pub fn test_database_url() -> String {
    resolve_test_database_url()
}

#[derive(Copy, Clone, Debug)]
pub enum TestServiceFixture {
    Shared,
    RoomMember,
    WebsocketConnection,
    OutgoingCommand,
    Event,
    User,
    Account,
    Message,
    Notification,
}

pub type DbTestSessionContext<'a> = DbSessionContext<'a>;

async fn init_test_fixture(session: &mut DbSession, service: TestServiceFixture) {
    match service {
        TestServiceFixture::Shared => {
            init_shared_test_db(session).await;
        }
        TestServiceFixture::RoomMember => {
            init_room_member_db(session).await;
        }
        TestServiceFixture::WebsocketConnection => {
            init_websocket_service_db(session).await;
        }
        TestServiceFixture::OutgoingCommand => {
            init_outgoing_command_service_db(session).await;
        }
        TestServiceFixture::Event => {
            init_event_service_db(session).await;
        }
        TestServiceFixture::User => {
            init_user_service_db(session).await;
        }
        TestServiceFixture::Account => {
            init_account_service_db(session).await;
        }
        TestServiceFixture::Message => {
            init_message_service_db(session).await;
        }
        TestServiceFixture::Notification => {
            init_notification_service_db(session).await;
        }
    }
}

/// Run a transactional test fixture around one async callback.
///
/// The callback receives a `DbSessionContext` that is a session tied to a single
/// database transaction. All SQL operations execute inside this transaction and are
/// rolled back after the callback returns.
pub async fn with_db_session<T, F>(service: TestServiceFixture, f: F) -> anyhow::Result<T>
where
    F: for<'a> FnOnce(DbTestSessionContext<'a>) -> SessionFuture<'a, T> + 'static,
{
    let pool = connect_test_db_locked().await;
    pool.with_rollback_session_context(|session| {
        Box::pin(async move {
            let mut session = session;
            init_test_fixture(&mut session, service).await;
            f(session).await
        })
    })
    .await
}

/// Same fixture transaction scope as [`with_db_session`], while also exposing the
/// shared test pool for callers that need to construct production objects requiring
/// a pool value.
pub async fn with_db_session_and_pool<T, F>(service: TestServiceFixture, f: F) -> anyhow::Result<T>
where
    F: for<'a> FnOnce(DbTestSessionContext<'a>, DbPool) -> SessionFuture<'a, T> + 'static,
{
    let pool = connect_test_db_locked().await;
    let pool_for_callback = pool.pool.clone();

    pool.with_rollback_session_context(move |session| {
        let pool_for_callback = pool_for_callback.clone();
        Box::pin(async move {
            let mut session = session;
            init_test_fixture(&mut session, service).await;
            f(session, pool_for_callback).await
        })
    })
    .await
}

fn resolve_test_database_url() -> String {
    if let Ok(url) = std::env::var(TEST_DATABASE_URL_ENV_KEY) {
        return url;
    }

    read_env_file_test_database_url(TEST_DATABASE_URL_ENV_FILE).unwrap_or_else(|| {
        panic!(
            "TEST_DATABASE_URL is not set and not found in {}",
            TEST_DATABASE_URL_ENV_FILE
        )
    })
}

fn read_env_file_test_database_url(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != TEST_DATABASE_URL_ENV_KEY {
            continue;
        }
        let mut value = value.trim().trim();
        if value.len() >= 2 {
            let first = value.as_bytes().first().copied();
            let last = value.as_bytes().last().copied();
            if (first == Some(b'\'') && last == Some(b'\''))
                || (first == Some(b'\"') && last == Some(b'\"'))
            {
                value = &value[1..value.len() - 1];
            }
        }
        return Some(value.to_string());
    }
    None
}

pub async fn init_room_member_db(session: &mut DbSession) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS room_members (
            room_id VARCHAR NOT NULL,
            user_id VARCHAR NOT NULL,
            role VARCHAR,
            joined_at TIMESTAMP WITH TIME ZONE,
            left_at TIMESTAMP WITH TIME ZONE,
            raw_data JSONB,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            PRIMARY KEY (room_id, user_id)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create room_members table");
    sqlx::query("ALTER TABLE room_members ALTER COLUMN created_at SET DEFAULT NOW()")
        .execute(session.as_mut())
        .await
        .expect("ensure room_members.created_at default");
    sqlx::query("ALTER TABLE room_members ALTER COLUMN updated_at SET DEFAULT NOW()")
        .execute(session.as_mut())
        .await
        .expect("ensure room_members.updated_at default");
    sqlx::query("TRUNCATE TABLE room_members")
        .execute(session.as_mut())
        .await
        .expect("truncate room_members");
}

pub async fn init_websocket_like_db(session: &mut DbSession) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            user_id VARCHAR NOT NULL,
            full_name VARCHAR,
            name_tsv TSVECTOR,
            avatar_url VARCHAR,
            avatar_file VARCHAR,
            bio VARCHAR,
            birthday VARCHAR,
            birthday_public BOOLEAN,
            quirk VARCHAR,
            is_bot BOOLEAN,
            gender VARCHAR,
            metadata JSONB,
            raw_data JSONB,
            last_seen TIMESTAMP WITH TIME ZONE,
            message_count INTEGER NOT NULL,
            deleted_count INTEGER NOT NULL,
            recalled_count INTEGER NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
            PRIMARY KEY (user_id)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create users table");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dzmm_account (
            user_id VARCHAR NOT NULL,
            user_profile JSONB NOT NULL,
            email VARCHAR,
            password VARCHAR,
            signin_code VARCHAR,
            signin_code_image BYTEA,
            signin_code_image_mime VARCHAR,
            cookies VARCHAR,
            is_enabled BOOLEAN NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
            PRIMARY KEY (user_id)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create dzmm_account table");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS websocket_connections (
            lock_id BIGINT PRIMARY KEY,
            account_user_id VARCHAR NOT NULL,
            connected_at TIMESTAMP WITH TIME ZONE NOT NULL,
            last_heartbeat TIMESTAMP WITH TIME ZONE NOT NULL
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create websocket_connections table");
}

pub async fn init_websocket_service_db(session: &mut DbSession) {
    init_websocket_like_db(session).await;
    sqlx::query(
        "INSERT INTO users (
            user_id, message_count, deleted_count, recalled_count, created_at, updated_at
        ) VALUES
            ('user_test_acquire', 0, 0, 0, NOW(), NOW()),
            ('user_test_release', 0, 0, 0, NOW(), NOW()),
            ('user_test_heartbeat', 0, 0, 0, NOW(), NOW()),
            ('user_test_active1', 0, 0, 0, NOW(), NOW()),
            ('user_test_active2', 0, 0, 0, NOW(), NOW()),
            ('user_test_filter1', 0, 0, 0, NOW(), NOW()),
            ('user_test_filter2', 0, 0, 0, NOW(), NOW()),
            ('user_test_in_use', 0, 0, 0, NOW(), NOW()),
            ('user_test_not_in_use', 0, 0, 0, NOW(), NOW()),
            ('user_test_fresh', 0, 0, 0, NOW(), NOW()),
            ('user_test_stale', 0, 0, 0, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
            SET updated_at = EXCLUDED.updated_at",
    )
    .execute(session.as_mut())
    .await
    .expect("seed users");
    sqlx::query(
        "INSERT INTO dzmm_account (
            user_id, user_profile, is_enabled, created_at, updated_at
        ) VALUES
            ('user_test_acquire', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_release', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_heartbeat', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_active1', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_active2', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_filter1', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_filter2', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_in_use', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_not_in_use', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_fresh', '{}'::jsonb, true, NOW(), NOW()),
            ('user_test_stale', '{}'::jsonb, true, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
            SET updated_at = EXCLUDED.updated_at",
    )
    .execute(session.as_mut())
    .await
    .expect("seed dzmm_account");
    sqlx::query("TRUNCATE TABLE websocket_connections")
        .execute(session.as_mut())
        .await
        .expect("truncate websocket_connections");
    sqlx::query("TRUNCATE TABLE users")
        .execute(session.as_mut())
        .await
        .expect("truncate users");
    sqlx::query("TRUNCATE TABLE dzmm_account")
        .execute(session.as_mut())
        .await
        .expect("truncate dzmm_account");
    sqlx::query("SELECT pg_advisory_unlock_all()")
        .execute(session.as_mut())
        .await
        .expect("unlock advisory locks");
}

pub async fn init_outgoing_command_service_db(session: &mut DbSession) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS outgoing_commands (
            id SERIAL PRIMARY KEY,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            account_user_id VARCHAR NOT NULL,
            event VARCHAR NOT NULL,
            data JSONB NOT NULL,
            require_ack BOOLEAN NOT NULL,
            status VARCHAR NOT NULL,
            processed_at TIMESTAMP WITH TIME ZONE,
            ack_response JSONB,
            error_message VARCHAR,
            attempt_count INTEGER NOT NULL,
            max_attempts INTEGER NOT NULL
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create outgoing_commands");
    sqlx::query("TRUNCATE TABLE outgoing_commands")
        .execute(session.as_mut())
        .await
        .expect("truncate outgoing_commands");
}

pub async fn init_event_service_db(session: &mut DbSession) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS websocket_events (
            id BIGINT NULL,
            user_id TEXT NOT NULL,
            event TEXT NOT NULL,
            data JSONB NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create websocket_events");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS event_processor_offsets (
            processor_id TEXT PRIMARY KEY,
            last_processed_id BIGINT NOT NULL,
            last_processed_timestamp TIMESTAMPTZ NULL,
            last_processed_at TIMESTAMPTZ NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create event_processor_offsets");
    sqlx::query("TRUNCATE TABLE websocket_events")
        .execute(session.as_mut())
        .await
        .expect("truncate websocket_events");
    sqlx::query("TRUNCATE TABLE event_processor_offsets")
        .execute(session.as_mut())
        .await
        .expect("truncate event_processor_offsets");
}

pub async fn init_user_service_db(session: &mut DbSession) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            user_id VARCHAR NOT NULL,
            full_name VARCHAR,
            name_tsv TSVECTOR,
            avatar_url VARCHAR,
            avatar_file VARCHAR,
            bio VARCHAR,
            birthday VARCHAR,
            birthday_public BOOLEAN,
            quirk VARCHAR,
            is_bot BOOLEAN,
            gender VARCHAR,
            metadata JSONB,
            raw_data JSONB,
            last_seen TIMESTAMP WITH TIME ZONE,
            message_count INTEGER NOT NULL,
            deleted_count INTEGER NOT NULL,
            recalled_count INTEGER NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
            PRIMARY KEY (user_id)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create users table");
    sqlx::query("TRUNCATE TABLE users")
        .execute(session.as_mut())
        .await
        .expect("truncate users");
    sqlx::query(
        "INSERT INTO users (
            user_id, full_name, message_count, deleted_count, recalled_count, created_at, updated_at
        ) VALUES
            ('user1', 'User One', 3, 1, 2, NOW(), NOW()),
            ('user2', 'User Two', 0, 0, 0, NOW(), NOW()),
            ('test_user', 'Test User', 0, 0, 0, NOW(), NOW()),
            ('test_user_1', 'Another User', 0, 0, 0, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
            SET full_name = EXCLUDED.full_name",
    )
    .execute(session.as_mut())
    .await
    .expect("seed users");
}

pub async fn init_account_service_db(session: &mut DbSession) {
    init_websocket_like_db(session).await;
    sqlx::query("TRUNCATE TABLE dzmm_account")
        .execute(session.as_mut())
        .await
        .expect("truncate dzmm_account");
    sqlx::query("TRUNCATE TABLE websocket_connections")
        .execute(session.as_mut())
        .await
        .expect("truncate websocket_connections");
}

pub async fn init_message_service_db(session: &mut DbSession) {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            user_id VARCHAR NOT NULL,
            full_name VARCHAR,
            name_tsv TSVECTOR,
            avatar_url VARCHAR,
            avatar_file VARCHAR,
            bio VARCHAR,
            birthday VARCHAR,
            birthday_public BOOLEAN,
            quirk VARCHAR,
            is_bot BOOLEAN,
            gender VARCHAR,
            metadata JSONB,
            raw_data JSONB,
            last_seen TIMESTAMP WITH TIME ZONE,
            message_count INTEGER NOT NULL,
            deleted_count INTEGER NOT NULL,
            recalled_count INTEGER NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
            PRIMARY KEY (user_id)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create users table");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS rooms (
            room_id VARCHAR NOT NULL,
            title VARCHAR NOT NULL,
            chat_type VARCHAR,
            avatar_url VARCHAR,
            member_count INTEGER,
            tags TEXT[],
            is_public BOOLEAN,
            creator_id VARCHAR,
            account_ids TEXT[] NOT NULL DEFAULT '{}',
            last_message_at TIMESTAMP WITH TIME ZONE,
            first_message_at TIMESTAMP WITH TIME ZONE,
            backfill_until TIMESTAMP WITH TIME ZONE,
            history_complete BOOLEAN NOT NULL,
            message_count INTEGER NOT NULL,
            deleted_count INTEGER NOT NULL,
            recalled_count INTEGER NOT NULL,
            edited_count INTEGER NOT NULL,
            image_count INTEGER NOT NULL,
            is_active BOOLEAN NOT NULL,
            dissolved_at TIMESTAMP WITH TIME ZONE,
            raw_data JSONB,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
            PRIMARY KEY (room_id)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create rooms table");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            message_id VARCHAR NOT NULL,
            room_id VARCHAR NOT NULL,
            sent_at TIMESTAMP WITH TIME ZONE NOT NULL,
            sent_by VARCHAR NOT NULL,
            content_type VARCHAR NOT NULL,
            content_text VARCHAR,
            content_tsv TSVECTOR,
            attachment_url VARCHAR,
            attachment_file VARCHAR,
            sticker_id VARCHAR,
            alt_text VARCHAR,
            metadata JSONB,
            raw_data JSONB NOT NULL,
            source VARCHAR NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE,
            is_deleted BOOLEAN NOT NULL,
            deleted_at TIMESTAMP WITH TIME ZONE,
            deleted_by VARCHAR,
            is_recalled BOOLEAN NOT NULL,
            is_edited BOOLEAN NOT NULL,
            history JSONB,
            reference_message_id VARCHAR,
            reference_data JSONB,
            PRIMARY KEY (message_id, sent_at)
        )",
    )
    .execute(session.as_mut())
    .await
    .expect("create messages table");
    sqlx::query("TRUNCATE TABLE messages")
        .execute(session.as_mut())
        .await
        .expect("truncate messages");
    sqlx::query(
        "INSERT INTO users (
            user_id, full_name, message_count, deleted_count, recalled_count, created_at, updated_at
        ) VALUES
            ('user1', 'User One', 5, 1, 1, NOW(), NOW()),
            ('user2', 'User Two', 0, 0, 0, NOW(), NOW()),
            ('test_user', 'Test User', 0, 0, 0, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
            SET full_name = EXCLUDED.full_name",
    )
    .execute(session.as_mut())
    .await
    .expect("seed users");
    sqlx::query(
        "INSERT INTO rooms (
            room_id, title, history_complete, message_count, deleted_count, recalled_count,
            edited_count, image_count, is_active, created_at, updated_at
        ) VALUES
            ('room1', 'Room 1', true, 0, 0, 0, 0, 0, true, NOW(), NOW())
        ON CONFLICT (room_id) DO UPDATE
            SET title = EXCLUDED.title",
    )
    .execute(session.as_mut())
    .await
    .expect("seed rooms");
}

pub async fn init_notification_service_db(_session: &mut DbSession) {}

pub async fn init_shared_test_db(session: &mut DbSession) {
    init_room_member_db(session).await;
    init_websocket_like_db(session).await;
    init_message_service_db(session).await;
    init_outgoing_command_service_db(session).await;
    init_event_service_db(session).await;
    sqlx::query(
        "TRUNCATE TABLE room_members, users, dzmm_account, websocket_connections, rooms, messages, outgoing_commands, websocket_events, event_processor_offsets"
    )
    .execute(session.as_mut())
    .await
    .expect("truncate all service tables");
    sqlx::query(
        "INSERT INTO users (
            user_id, full_name, message_count, deleted_count, recalled_count, created_at, updated_at
        ) VALUES
            ('user1', 'User One', 3, 1, 2, NOW(), NOW()),
            ('user2', 'User Two', 0, 0, 0, NOW(), NOW()),
            ('test_user', 'Test User', 0, 0, 0, NOW(), NOW()),
            ('test_user_1', 'Another User', 0, 0, 0, NOW(), NOW())
        ON CONFLICT (user_id) DO UPDATE
            SET full_name = EXCLUDED.full_name",
    )
    .execute(session.as_mut())
    .await
    .expect("seed users");
    sqlx::query(
        "INSERT INTO rooms (
            room_id, title, history_complete, message_count, deleted_count, recalled_count,
            edited_count, image_count, is_active, created_at, updated_at
        ) VALUES
            ('room1', 'Room 1', true, 0, 0, 0, 0, 0, true, NOW(), NOW())
        ON CONFLICT (room_id) DO UPDATE
            SET title = EXCLUDED.title",
    )
    .execute(session.as_mut())
    .await
    .expect("seed rooms");
    sqlx::query("SELECT pg_advisory_unlock_all()")
        .execute(session.as_mut())
        .await
        .expect("unlock advisory locks");
}
