use anyhow::{Context, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use lilium_database::pool::SessionFuture;
use lilium_database::{DbPool, DbSession, DbSessionContext};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use sqlx::QueryBuilder;
use std::borrow::Borrow;
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;

const TEST_DATABASE_URL_ENV_KEY: &str = "TEST_DATABASE_URL";
static TEST_DOTENV_LOADED: OnceLock<()> = OnceLock::new();
static TEST_DATABASE_POOL: OnceCell<Arc<TestDatabasePool>> = OnceCell::const_new();
static TEST_DATABASE_POOL_SIZE: OnceLock<u32> = OnceLock::new();
static TEST_DATABASE_NAME_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct TestDatabaseState {
    base_url: String,
    db_name: String,
}

#[derive(Debug)]
struct TestDatabasePool {
    base_url: String,
    pool_size: u32,
    free_instances: Mutex<Vec<TestDatabaseState>>,
    all_instances: Mutex<Vec<TestDatabaseState>>,
}

#[derive(Debug)]
pub(crate) struct TestDatabaseLease {
    pool: Arc<TestDatabasePool>,
    state: Mutex<Option<TestDatabaseState>>,
}

#[derive(Debug, Clone)]
pub struct TestDbPool {
    inner: DbPool,
    _lease: Arc<TestDatabaseLease>,
}

impl TestDbPool {
    fn new(inner: DbPool, lease: Arc<TestDatabaseLease>) -> Self {
        Self {
            inner,
            _lease: lease,
        }
    }

    pub fn inner(&self) -> &DbPool {
        &self.inner
    }
}

impl Deref for TestDbPool {
    type Target = DbPool;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Borrow<DbPool> for TestDbPool {
    fn borrow(&self) -> &DbPool {
        &self.inner
    }
}

impl TestDatabaseLease {
    fn new(pool: Arc<TestDatabasePool>, state: TestDatabaseState) -> Self {
        Self {
            pool,
            state: Mutex::new(Some(state)),
        }
    }

    fn release_state(&self, state: TestDatabaseState) {
        self.pool.return_instance(state);
    }
}

impl Drop for TestDatabaseLease {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }

        let mut state = self.state.lock().expect("lock leased test database");
        if let Some(state) = state.take() {
            self.release_state(state);
        }
    }
}

impl TestDatabasePool {
    async fn acquire(self: &Arc<Self>) -> Result<TestDatabaseState> {
        loop {
            if let Some(state) = self
                .free_instances
                .lock()
                .expect("lock free test databases")
                .pop()
            {
                if let Err(error) = self.prepare_instance(&state).await {
                    eprintln!(
                        "failed to prepare leased test database {}: {error:#}",
                        state.db_name
                    );
                    let _ = cleanup_test_database_async(&state).await;
                    continue;
                }

                return Ok(state);
            }

            let state = self.create_instance().await?;
            self.all_instances
                .lock()
                .expect("lock created test databases")
                .push(state.clone());
            return Ok(state);
        }
    }

    fn return_instance(&self, state: TestDatabaseState) {
        self.free_instances
            .lock()
            .expect("lock free test databases")
            .push(state);
    }

    async fn create_instance(self: &Arc<Self>) -> Result<TestDatabaseState> {
        let db_name = build_test_database_name();
        create_test_database_instance(&self.base_url, &db_name, self.pool_size).await?;
        Ok(TestDatabaseState {
            base_url: self.base_url.clone(),
            db_name,
        })
    }

    async fn prepare_instance(self: &Arc<Self>, state: &TestDatabaseState) -> Result<()> {
        reset_test_database_async_via_pool(state).await?;
        Ok(())
    }
}

pub async fn connect_test_db() -> TestDbPool {
    load_test_env();
    connect_test_db_with_pool_size(4).await
}

pub async fn connect_test_db_with_pool_size(pool_size: u32) -> TestDbPool {
    let pool = ensure_test_database_pool(pool_size)
        .await
        .expect("initialize test database pool");
    let state = pool.acquire().await.expect("acquire test database");
    let options = match build_test_database_options(&state.base_url, &state.db_name) {
        Ok(options) => options,
        Err(error) => {
            pool.return_instance(state);
            panic!("build test database options: {error:#}");
        }
    };
    let inner = match DbPool::connect_with_options(options, pool_size).await {
        Ok(inner) => inner,
        Err(error) => {
            pool.return_instance(state);
            panic!("connect: {error:#}");
        }
    };
    TestDbPool::new(inner, Arc::new(TestDatabaseLease::new(pool, state)))
}

pub async fn connect_test_db_lazy() -> TestDbPool {
    load_test_env();
    let pool = ensure_test_database_pool(4)
        .await
        .expect("initialize test database pool");
    let state = pool.acquire().await.expect("acquire test database");
    let options = match build_test_database_options(&state.base_url, &state.db_name) {
        Ok(options) => options,
        Err(error) => {
            pool.return_instance(state);
            panic!("build test database options: {error:#}");
        }
    };
    let inner = match DbPool::connect_lazy_with_options(options).await {
        Ok(inner) => inner,
        Err(error) => {
            pool.return_instance(state);
            panic!("connect: {error:#}");
        }
    };
    TestDbPool::new(inner, Arc::new(TestDatabaseLease::new(pool, state)))
}

pub fn test_database_url() -> String {
    load_test_env();
    resolve_test_database_url()
}

async fn ensure_test_database_pool(pool_size: u32) -> Result<Arc<TestDatabasePool>> {
    if let Some(existing_pool_size) = TEST_DATABASE_POOL_SIZE.get() {
        if *existing_pool_size != pool_size {
            return Err(anyhow::anyhow!(
                "test database pool size mismatch: existing={}, requested={}",
                existing_pool_size,
                pool_size
            ));
        }
    }

    TEST_DATABASE_POOL
        .get_or_try_init(|| async { create_test_database_pool(pool_size).await })
        .await
        .map(Arc::clone)
}

async fn create_test_database_pool(pool_size: u32) -> Result<Arc<TestDatabasePool>> {
    let base_url = resolve_test_database_url();
    let admin_options = build_admin_database_options(&base_url)?;
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options)
        .await
        .context("connect to test database admin URL")?;

    cleanup_stale_test_databases(&admin_pool).await?;

    let _ = TEST_DATABASE_POOL_SIZE.set(pool_size);

    Ok(Arc::new(TestDatabasePool {
        base_url,
        pool_size,
        free_instances: Mutex::new(Vec::new()),
        all_instances: Mutex::new(Vec::new()),
    }))
}

fn build_test_database_name() -> String {
    let pid = std::process::id();
    let sequence = TEST_DATABASE_NAME_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("lilium_test_{pid}_{sequence}_{nanos}")
}

fn build_admin_database_options(base_url: &str) -> Result<PgConnectOptions> {
    let options = PgConnectOptions::from_str(base_url)
        .with_context(|| format!("parse TEST_DATABASE_URL: {base_url}"))?;
    Ok(options.database("postgres"))
}

fn build_test_database_options(base_url: &str, db_name: &str) -> Result<PgConnectOptions> {
    let options = PgConnectOptions::from_str(base_url)
        .with_context(|| format!("parse TEST_DATABASE_URL: {base_url}"))?;
    Ok(options.database(db_name))
}

async fn bootstrap_test_schema(pool: &PgPool) -> Result<()> {
    let migrator = Migrator::new(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../lilium-database/testdata/live_schema_bootstrap"
    )))
    .await
    .context("load live schema bootstrap")?;

    migrator
        .run(pool)
        .await
        .context("apply live schema bootstrap")?;

    Ok(())
}

async fn cleanup_test_database_async(state: &TestDatabaseState) -> Result<()> {
    let admin_options = build_admin_database_options(&state.base_url)?;
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options)
        .await
        .context("connect to test database admin URL for cleanup")?;

    let db_name = &state.db_name;
    let _ = sqlx::query(&format!(
        "ALTER DATABASE {db_name} WITH ALLOW_CONNECTIONS false"
    ))
    .execute(&admin_pool)
    .await;
    let _ = sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    ))
    .execute(&admin_pool)
    .await;
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)"))
        .execute(&admin_pool)
        .await;

    Ok(())
}

async fn reset_test_database_async_via_pool(state: &TestDatabaseState) -> Result<()> {
    let options = build_test_database_options(&state.base_url, &state.db_name)?;
    let pool = DbPool::connect_with_options(options, 1).await?;

    pool.with_session_context(|mut session| {
        Box::pin(async move { reset_test_database_async(&mut session).await })
    })
    .await?;

    Ok(())
}

async fn create_test_database_instance(
    base_url: &str,
    db_name: &str,
    pool_size: u32,
) -> Result<()> {
    let state = TestDatabaseState {
        base_url: base_url.to_string(),
        db_name: db_name.to_string(),
    };
    let admin_options = build_admin_database_options(base_url)?;
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options)
        .await
        .with_context(|| format!("connect to test database admin URL for {db_name}"))?;

    let _ = sqlx::query(&format!(
        "ALTER DATABASE {db_name} WITH ALLOW_CONNECTIONS false"
    ))
    .execute(&admin_pool)
    .await;
    let _ = sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    ))
    .execute(&admin_pool)
    .await;
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)"))
        .execute(&admin_pool)
        .await;

    sqlx::query(&format!("CREATE DATABASE {db_name}"))
        .execute(&admin_pool)
        .await
        .with_context(|| format!("create test database {db_name}"))?;

    let test_options = build_test_database_options(base_url, db_name)?;
    let test_pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .connect_with(test_options)
        .await
        .with_context(|| format!("connect to test database {db_name}"))?;

    if let Err(error) = bootstrap_test_schema(&test_pool).await {
        let _ = cleanup_test_database_async(&state).await;
        return Err(error);
    }

    Ok(())
}

async fn cleanup_stale_test_databases(admin_pool: &PgPool) -> Result<()> {
    let stale_databases = sqlx::query_scalar::<_, String>(
        r#"
        SELECT datname
        FROM pg_database
        WHERE datname LIKE 'lilium_test_%'
        ORDER BY datname
        "#,
    )
    .fetch_all(admin_pool)
    .await
    .context("list stale test databases")?;

    for stale_db in stale_databases {
        let active_connections = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM pg_stat_activity
            WHERE datname = $1
              AND pid <> pg_backend_pid()
            "#,
        )
        .bind(&stale_db)
        .fetch_one(admin_pool)
        .await
        .with_context(|| format!("count connections for stale test database {stale_db}"))?;

        if active_connections > 0 {
            continue;
        }

        let _ = sqlx::query(&format!(
            "ALTER DATABASE {stale_db} WITH ALLOW_CONNECTIONS false"
        ))
        .execute(admin_pool)
        .await;
        let _ = sqlx::query(&format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{stale_db}' AND pid <> pg_backend_pid()"
        ))
        .execute(admin_pool)
        .await;
        let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {stale_db} WITH (FORCE)"))
            .execute(admin_pool)
            .await;
    }

    Ok(())
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
    F: for<'a> FnOnce(DbTestSessionContext<'a>) -> SessionFuture<'a, T> + Send + 'static,
{
    let pool = connect_test_db().await;
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
    F: for<'a> FnOnce(DbTestSessionContext<'a>, TestDbPool) -> SessionFuture<'a, T>
        + Send
        + 'static,
{
    let pool = connect_test_db().await;
    let pool_for_callback = pool.clone();

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
    std::env::var(TEST_DATABASE_URL_ENV_KEY)
        .unwrap_or_else(|_| panic!("TEST_DATABASE_URL is not set"))
}

fn load_test_env() {
    TEST_DOTENV_LOADED.get_or_init(|| {
        dotenvy::dotenv().ok();
    });
}

async fn reset_test_database_async(session: &mut DbSession) -> Result<()> {
    let table_names = public_table_names(session).await?;
    if !table_names.is_empty() {
        let truncate_sql = format!(
            "TRUNCATE TABLE {} RESTART IDENTITY CASCADE",
            table_names.join(", ")
        );
        sqlx::query(&truncate_sql)
            .execute(session.as_mut())
            .await
            .context("truncate public tables")?;
    }

    ensure_test_time_partitions_async(session).await?;

    sqlx::query("SELECT pg_advisory_unlock_all()")
        .execute(session.as_mut())
        .await
        .context("unlock advisory locks")?;

    Ok(())
}

async fn public_table_names(session: &mut DbSession) -> Result<Vec<String>> {
    let names = sqlx::query_scalar::<_, String>(
        r#"
        SELECT format('%I.%I', schemaname, tablename)
        FROM pg_tables
        WHERE schemaname = 'public'
          AND tablename <> 'sqlx_migrations'
          AND NOT EXISTS (
              SELECT 1
              FROM pg_inherits
              WHERE inhrelid = format('%I.%I', schemaname, tablename)::regclass
          )
        ORDER BY tablename
        "#,
    )
    .fetch_all(session.as_mut())
    .await
    .context("list public tables")?;

    Ok(names)
}

async fn ensure_test_time_partitions_async(session: &mut DbSession) -> Result<()> {
    let now = Utc::now();
    let messages_anchors = [
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
            .single()
            .expect("valid messages partition anchor"),
        Utc.with_ymd_and_hms(2026, 2, 28, 23, 59, 0)
            .single()
            .expect("valid messages partition anchor"),
        Utc.with_ymd_and_hms(2026, 3, 1, 0, 1, 0)
            .single()
            .expect("valid messages partition anchor"),
        now - Duration::days(40),
        now,
        now + Duration::days(40),
    ];
    let websocket_anchors = [
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
            .single()
            .expect("valid websocket partition anchor"),
        Utc.with_ymd_and_hms(2026, 3, 15, 0, 0, 0)
            .single()
            .expect("valid websocket partition anchor"),
        Utc.with_ymd_and_hms(2026, 3, 16, 0, 1, 0)
            .single()
            .expect("valid websocket partition anchor"),
        now - Duration::days(8),
        now,
        now + Duration::days(8),
    ];

    ensure_test_partitions_for_table(session, "messages", &messages_anchors).await?;
    ensure_test_partitions_for_table(session, "websocket_events", &websocket_anchors).await?;

    Ok(())
}

async fn ensure_test_partitions_for_table(
    session: &mut DbSession,
    table_name: &str,
    anchors: &[DateTime<Utc>],
) -> Result<()> {
    for anchor in anchors {
        let _: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT child_name
            FROM ensure_time_partitions(
                p_table_name => $1,
                p_anchor => $2,
                p_apply => true
            )
            "#,
        )
        .bind(table_name)
        .bind(anchor)
        .fetch_all(session.as_mut())
        .await
        .with_context(|| format!("ensure {table_name} partitions"))?;
    }

    Ok(())
}

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

impl RoomSeed {
    const fn new(
        room_id: &'static str,
        title: &'static str,
        history_complete: bool,
        message_count: i32,
        deleted_count: i32,
        recalled_count: i32,
        edited_count: i32,
        image_count: i32,
        is_active: bool,
    ) -> Self {
        Self {
            room_id,
            title,
            history_complete,
            message_count,
            deleted_count,
            recalled_count,
            edited_count,
            image_count,
            is_active,
        }
    }
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

const MESSAGE_SERVICE_TEST_ROOMS: &[RoomSeed] =
    &[RoomSeed::new("room1", "Room 1", true, 0, 0, 0, 0, 0, true)];

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

async fn seed_shared_test_db(session: &mut DbSession) -> Result<()> {
    seed_users(session, DEFAULT_TEST_USERS).await
}

async fn seed_user_service_db(session: &mut DbSession) -> Result<()> {
    seed_users(session, DEFAULT_TEST_USERS).await
}

async fn seed_message_service_db(session: &mut DbSession) -> Result<()> {
    seed_users(session, MESSAGE_SERVICE_TEST_USERS).await?;
    seed_rooms(session, MESSAGE_SERVICE_TEST_ROOMS).await
}

async fn seed_websocket_service_db(session: &mut DbSession) -> Result<()> {
    seed_users(session, WEBSOCKET_SERVICE_TEST_USERS).await?;
    seed_dzmm_accounts(session, WEBSOCKET_SERVICE_ACCOUNT_USER_IDS).await
}

pub async fn init_room_member_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
}

pub async fn init_websocket_like_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
}

pub async fn init_websocket_service_db(session: &mut DbSession) {
    init_websocket_like_db(session).await;
    seed_websocket_service_db(session)
        .await
        .expect("seed websocket service db");
}

pub async fn init_outgoing_command_service_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
}

pub async fn init_event_service_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
}

pub async fn init_user_service_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
    seed_user_service_db(session)
        .await
        .expect("seed user service db");
}

pub async fn init_account_service_db(session: &mut DbSession) {
    init_websocket_like_db(session).await;
}

pub async fn init_message_service_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
    seed_message_service_db(session)
        .await
        .expect("seed message service db");
}

pub async fn init_notification_service_db(_session: &mut DbSession) {}

pub async fn init_shared_test_db(session: &mut DbSession) {
    reset_test_database_async(session)
        .await
        .expect("reset test db");
    seed_shared_test_db(session)
        .await
        .expect("seed shared test db");
}
