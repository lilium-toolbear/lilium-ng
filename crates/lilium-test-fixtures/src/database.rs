use anyhow::{Context, Result};
use lilium_database::DbPool;
use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::borrow::Borrow;
use std::ops::Deref;
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::OnceCell;
use url::Url;

use crate::reset::reset_database;

const TEST_DATABASE_URL_ENV_KEY: &str = "TEST_DATABASE_URL";

static TEST_DOTENV_LOADED: OnceLock<()> = OnceLock::new();
static TEST_DATABASE_POOL: OnceCell<Arc<TestDatabasePool>> = OnceCell::const_new();
static TEST_DATABASE_POOL_SIZE: OnceLock<u32> = OnceLock::new();
static TEST_DATABASE_NAME_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct TestDatabase {
    base_url: String,
    name: String,
}

#[derive(Debug)]
struct TestDatabasePool {
    base_url: String,
    connection_pool_size: u32,
    idle_databases: Mutex<Vec<TestDatabase>>,
}

#[derive(Debug)]
pub(crate) struct TestDatabaseLease {
    pool: Arc<TestDatabasePool>,
    database: Mutex<Option<TestDatabase>>,
}

#[derive(Debug, Clone)]
pub struct TestDatabaseConnection {
    inner: DbPool,
    database_url: String,
    max_connections: u32,
    _lease: Arc<TestDatabaseLease>,
}

impl TestDatabaseConnection {
    fn new(
        inner: DbPool,
        database_url: String,
        max_connections: u32,
        lease: Arc<TestDatabaseLease>,
    ) -> Self {
        Self {
            inner,
            database_url,
            max_connections,
            _lease: lease,
        }
    }

    pub fn inner(&self) -> &DbPool {
        &self.inner
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    pub fn database_config(&self) -> lilium_database::DatabaseConfig {
        lilium_database::DatabaseConfig::from_url(self.database_url.clone(), self.max_connections)
    }
}

impl Deref for TestDatabaseConnection {
    type Target = DbPool;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Borrow<DbPool> for TestDatabaseConnection {
    fn borrow(&self) -> &DbPool {
        &self.inner
    }
}

impl TestDatabaseLease {
    fn new(pool: Arc<TestDatabasePool>, database: TestDatabase) -> Self {
        Self {
            pool,
            database: Mutex::new(Some(database)),
        }
    }
}

impl Drop for TestDatabaseLease {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }

        let mut database = self.database.lock().expect("lock leased test database");
        if let Some(database) = database.take() {
            self.pool.return_database(database);
        }
    }
}

impl TestDatabasePool {
    async fn acquire(self: &Arc<Self>) -> Result<TestDatabase> {
        loop {
            let database = {
                self.idle_databases
                    .lock()
                    .expect("lock idle test databases")
                    .pop()
            };

            if let Some(database) = database {
                if let Err(error) = self.prepare_database(&database).await {
                    eprintln!(
                        "failed to prepare leased test database {}: {error:#}",
                        database.name
                    );
                    let _ = cleanup_test_database(&database).await;
                    continue;
                }

                return Ok(database);
            }

            return self.create_database().await;
        }
    }

    fn return_database(&self, database: TestDatabase) {
        self.idle_databases
            .lock()
            .expect("lock idle test databases")
            .push(database);
    }

    async fn create_database(self: &Arc<Self>) -> Result<TestDatabase> {
        let name = build_test_database_name();
        create_test_database(&self.base_url, &name, self.connection_pool_size).await?;
        Ok(TestDatabase {
            base_url: self.base_url.clone(),
            name,
        })
    }

    async fn prepare_database(self: &Arc<Self>, database: &TestDatabase) -> Result<()> {
        let database_url = build_test_database_url(&database.base_url, &database.name)?;
        let options = build_test_database_options(&database_url)?;
        let pool = DbPool::connect_with_options(options, 1).await?;

        pool.with_session_context(|mut session| {
            Box::pin(async move { reset_database(&mut session).await })
        })
        .await
    }
}

pub async fn connect_test_database() -> TestDatabaseConnection {
    connect_test_database_with_pool_size(4).await
}

pub async fn connect_test_database_with_pool_size(pool_size: u32) -> TestDatabaseConnection {
    load_test_env();

    let pool = ensure_test_database_pool(pool_size)
        .await
        .expect("initialize test database pool");
    let database = pool.acquire().await.expect("acquire test database");
    let database_url = match build_test_database_url(&database.base_url, &database.name) {
        Ok(database_url) => database_url,
        Err(error) => {
            pool.return_database(database);
            panic!("build test database url: {error:#}");
        }
    };
    let options = match build_test_database_options(&database_url) {
        Ok(options) => options,
        Err(error) => {
            pool.return_database(database);
            panic!("build test database options: {error:#}");
        }
    };
    let inner = match DbPool::connect_with_options(options, pool_size).await {
        Ok(inner) => inner,
        Err(error) => {
            pool.return_database(database);
            panic!("connect to test database: {error:#}");
        }
    };

    TestDatabaseConnection::new(
        inner,
        database_url,
        pool_size,
        Arc::new(TestDatabaseLease::new(pool, database)),
    )
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
    load_test_env();

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
        connection_pool_size: pool_size,
        idle_databases: Mutex::new(Vec::new()),
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
    let options = PgConnectOptions::from_str(base_url).context("parse TEST_DATABASE_URL")?;
    Ok(options.database("postgres"))
}

fn build_test_database_url(base_url: &str, db_name: &str) -> Result<String> {
    let mut url = Url::parse(base_url).context("parse TEST_DATABASE_URL")?;
    if url.scheme() == "postgresql" {
        url.set_scheme("postgres").expect("set postgres scheme");
    }
    url.set_path(&format!("/{db_name}"));
    Ok(url.to_string())
}

fn build_test_database_options(database_url: &str) -> Result<PgConnectOptions> {
    let options = PgConnectOptions::from_str(database_url).context("parse TEST_DATABASE_URL")?;
    Ok(options)
}

async fn create_test_database(base_url: &str, db_name: &str, pool_size: u32) -> Result<()> {
    let database = TestDatabase {
        base_url: base_url.to_string(),
        name: db_name.to_string(),
    };
    let admin_options = build_admin_database_options(base_url)?;
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options)
        .await
        .with_context(|| format!("connect to test database admin URL for {db_name}"))?;

    drop_database_if_exists(&admin_pool, db_name).await;

    sqlx::query(&format!("CREATE DATABASE {db_name}"))
        .execute(&admin_pool)
        .await
        .with_context(|| format!("create test database {db_name}"))?;

    let test_database_url = build_test_database_url(base_url, db_name)?;
    let test_options = build_test_database_options(&test_database_url)?;
    let test_pool = PgPoolOptions::new()
        .max_connections(pool_size)
        .connect_with(test_options)
        .await
        .with_context(|| format!("connect to test database {db_name}"))?;

    if let Err(error) = bootstrap_test_schema(&test_pool).await {
        let _ = cleanup_test_database(&database).await;
        return Err(error);
    }

    Ok(())
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
        .context("apply live schema bootstrap")
}

async fn cleanup_test_database(database: &TestDatabase) -> Result<()> {
    let admin_options = build_admin_database_options(&database.base_url)?;
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(admin_options)
        .await
        .context("connect to test database admin URL for cleanup")?;

    drop_database_if_exists(&admin_pool, &database.name).await;
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

        if active_connections == 0 {
            drop_database_if_exists(admin_pool, &stale_db).await;
        }
    }

    Ok(())
}

async fn drop_database_if_exists(admin_pool: &PgPool, db_name: &str) {
    let _ = sqlx::query(&format!(
        "ALTER DATABASE {db_name} WITH ALLOW_CONNECTIONS false"
    ))
    .execute(admin_pool)
    .await;
    let _ = sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
    ))
    .execute(admin_pool)
    .await;
    let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)"))
        .execute(admin_pool)
        .await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use url::Url;

    #[test]
    fn generated_database_names_are_unique_and_prefixed() {
        let names: HashSet<_> = (0..16).map(|_| build_test_database_name()).collect();

        assert_eq!(names.len(), 16);
        assert!(names.iter().all(|name| name.starts_with("lilium_test_")));
    }

    #[test]
    fn build_test_database_url_replaces_path_and_preserves_query_params() {
        let database_url = build_test_database_url(
            "postgresql://user:pass@localhost:5432/source_db?sslmode=require&application_name=lilium",
            "lilium_test_123",
        )
        .unwrap();
        let parsed = Url::parse(&database_url).unwrap();

        assert_eq!(parsed.scheme(), "postgres");
        assert_eq!(parsed.username(), "user");
        assert_eq!(parsed.password(), Some("pass"));
        assert_eq!(parsed.host_str(), Some("localhost"));
        assert_eq!(parsed.port(), Some(5432));
        assert_eq!(parsed.path(), "/lilium_test_123");
        assert_eq!(
            parsed.query(),
            Some("sslmode=require&application_name=lilium")
        );
    }
}
