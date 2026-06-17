// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 database/async_engine.py

use crate::DbTransaction;
use crate::observability::install_sea_orm_query_metrics;
use crate::pool::DbPool;
use crate::pool::normalize_database_url;
use crate::transaction::TransactionFuture;
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, SqlxPostgresConnector, TransactionError, TransactionTrait};
use sqlx::postgres::PgPoolOptions;
use sqlx::postgres::{PgConnection, PgListener};
use sqlx::{Connection, Postgres, pool::PoolConnection};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::time::Duration;
use tracing::{Instrument, instrument};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for the pooled application database connection.
///
/// Use this for the normal SeaORM pool that backs service-layer CRUD. It is not
/// for connection-stateful features such as `LISTEN`/`NOTIFY`, because those
/// require a physical connection to stay owned by one listener.
pub struct DatabaseConfig {
    /// PostgreSQL connection URL. `postgresql://` is normalized to
    /// `postgres://` before connecting.
    pub url: String,
    /// Maximum number of pooled connections for ordinary application queries.
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Build a normal pooled database configuration from a PostgreSQL URL.
    pub fn from_url(url: impl Into<String>, max_connections: u32) -> Self {
        Self {
            url: url.into(),
            max_connections,
        }
    }

    /// Return the URL form accepted by the underlying SQLx/Postgres connector.
    pub fn normalized_url(&self) -> String {
        normalize_database_url(&self.url)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for the dedicated PostgreSQL notification connection.
///
/// Use this for `LISTEN`/`NOTIFY` consumers. It is intentionally separate from
/// [`DatabaseConfig`] because notifications are tied to one physical
/// connection and should not be multiplexed through the normal query pool.
pub struct NotificationDatabaseConfig {
    /// PostgreSQL connection URL for the listener connection.
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Configuration for one dedicated PostgreSQL connection.
///
/// Use this when a runtime component must own PostgreSQL session state for its
/// full lifecycle, such as advisory locks. This is separate from
/// [`DatabaseConfig`] because a pool cannot model session ownership.
pub struct DedicatedDatabaseConfig {
    /// PostgreSQL connection URL for the dedicated session.
    pub url: String,
}

impl DedicatedDatabaseConfig {
    /// Build a dedicated-connection configuration from a PostgreSQL URL.
    pub fn from_url(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    /// Return the URL form accepted by SQLx direct connections.
    pub fn normalized_url(&self) -> String {
        normalize_database_url(&self.url)
    }
}

impl NotificationDatabaseConfig {
    /// Build a notification-listener configuration from a PostgreSQL URL.
    pub fn from_url(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }

    /// Return the URL form accepted by the SQLx notification listener.
    pub fn normalized_url(&self) -> String {
        normalize_database_url(&self.url)
    }
}

#[derive(Debug, Clone)]
/// Shared database runtime for ordinary application queries.
///
/// Use [`Database::orm`] when calling services or SeaORM entities. Use
/// [`Database::transaction`] when multiple service calls must commit or roll
/// back together. Avoid using this type for long-lived PostgreSQL
/// connection-stateful primitives; those need dedicated connection owners.
pub struct Database {
    orm: DatabaseConnection,
    raw_pool: DbPool,
}

#[derive(Debug)]
/// Short-lived raw SQLx connection borrowed from the normal application pool.
///
/// This is an escape hatch for database administration helpers or narrow
/// PostgreSQL operations that cannot be represented by SeaORM. Do not use it
/// for `LISTEN`/`NOTIFY` or advisory-lock ownership because pool checkout
/// lifetime does not model those long-lived connection states.
pub struct RawDbConnection {
    inner: PoolConnection<Postgres>,
}

#[derive(Debug)]
/// Dedicated physical PostgreSQL connection for session-stateful runtime work.
///
/// Use this for session-level advisory locks and similar primitives whose
/// correctness depends on keeping one PostgreSQL backend session alive. Do not
/// use this for ordinary CRUD; use [`Database`] and SeaORM for normal queries.
pub struct DedicatedDbConnection {
    inner: PgConnection,
}

/// Backend abstraction for PostgreSQL notification listeners.
///
/// Production code uses `PgListener`. Tests use this trait to provide a fake
/// listener without opening a real socket. Service code should normally depend
/// on [`NotificationConnection`] rather than implementing this trait directly.
pub trait NotificationListenerBackend: std::fmt::Debug + Send {
    /// Subscribe this physical connection to one PostgreSQL notification
    /// channel.
    fn listen<'a>(
        &'a mut self,
        channel: &'a str,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), sqlx::Error>> + Send + 'a>>;

    /// Unsubscribe this physical connection from one PostgreSQL notification
    /// channel.
    fn unlisten<'a>(
        &'a mut self,
        channel: &'a str,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), sqlx::Error>> + Send + 'a>>;

    /// Try to receive one notification payload without owning a pooled query
    /// transaction.
    fn try_recv_payload<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Option<String>, sqlx::Error>> + Send + 'a>>;

    /// Wait for the next notification payload on this physical listener.
    fn recv_payload<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<String, sqlx::Error>> + Send + 'a>>;
}

impl NotificationListenerBackend for PgListener {
    fn listen<'a>(
        &'a mut self,
        channel: &'a str,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), sqlx::Error>> + Send + 'a>> {
        Box::pin(async move { PgListener::listen(self, channel).await })
    }

    fn unlisten<'a>(
        &'a mut self,
        channel: &'a str,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<(), sqlx::Error>> + Send + 'a>> {
        Box::pin(async move { PgListener::unlisten(self, channel).await })
    }

    fn try_recv_payload<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Option<String>, sqlx::Error>> + Send + 'a>>
    {
        Box::pin(async move {
            match PgListener::try_recv(self).await? {
                Some(notification) => Ok(Some(notification.payload().to_string())),
                None => Ok(None),
            }
        })
    }

    fn recv_payload<'a>(
        &'a mut self,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<String, sqlx::Error>> + Send + 'a>> {
        Box::pin(async move { Ok(PgListener::recv(self).await?.payload().to_string()) })
    }
}

#[derive(Debug)]
/// Dedicated owner for PostgreSQL `LISTEN`/`NOTIFY` state.
///
/// Create this with [`NotificationConnection::connect`] when a worker needs to
/// receive notifications. Keep it separate from normal service CRUD because the
/// subscribed channels live on the physical PostgreSQL connection.
pub struct NotificationConnection<L = PgListener> {
    listener: L,
}

impl<L> NotificationConnection<L>
where
    L: NotificationListenerBackend,
{
    /// Subscribe this dedicated listener to `channel`.
    pub async fn listen(&mut self, channel: &str) -> Result<()> {
        self.listener
            .listen(channel)
            .await
            .with_context(|| format!("listen for notifications on channel '{channel}'"))?;
        Ok(())
    }

    /// Stop receiving notifications for `channel` on this dedicated listener.
    pub async fn unlisten(&mut self, channel: &str) -> Result<()> {
        self.listener
            .unlisten(channel)
            .await
            .with_context(|| format!("stop listening for notifications on channel '{channel}'"))?;
        Ok(())
    }

    /// Try to receive one notification payload.
    ///
    /// Returns `Ok(None)` when no payload is currently available. Use this when
    /// the caller owns its own wait/backoff loop.
    pub async fn try_recv_payload(&mut self) -> Result<Option<String>> {
        self.listener
            .try_recv_payload()
            .await
            .with_context(|| "receive PostgreSQL notification payload")
    }

    /// Wait until one notification payload is available and return it.
    ///
    /// Use this in a notification worker that owns the listener connection. Do
    /// not call it on a pooled ORM connection.
    pub async fn recv_payload(&mut self) -> Result<String> {
        self.listener
            .recv_payload()
            .await
            .with_context(|| "receive PostgreSQL notification payload")
    }
}

impl NotificationConnection<PgListener> {
    /// Open a new dedicated PostgreSQL notification listener connection.
    pub async fn connect(config: NotificationDatabaseConfig) -> Result<Self> {
        let normalized_url = config.normalized_url();
        let listener = PgListener::connect(&normalized_url)
            .await
            .with_context(|| "connect notification listener")?;
        Ok(Self { listener })
    }
}

impl DedicatedDbConnection {
    /// Open one physical PostgreSQL connection outside the ordinary pool.
    pub async fn connect(config: DedicatedDatabaseConfig) -> Result<Self> {
        let normalized_url = config.normalized_url();
        let inner = PgConnection::connect(&normalized_url)
            .await
            .with_context(|| "connect dedicated PostgreSQL connection")?;
        Ok(Self { inner })
    }

    /// Borrow the owned SQLx connection for session-stateful SQL.
    pub fn as_mut_pg_connection(&mut self) -> &mut PgConnection {
        &mut self.inner
    }
}

impl Deref for RawDbConnection {
    type Target = PoolConnection<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for RawDbConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Database {
    /// Create the shared application database runtime.
    ///
    /// This constructs one SQLx Postgres pool and wraps it as SeaORM's
    /// [`DatabaseConnection`] for normal service calls.
    #[instrument(level = "debug" skip(config), fields(max_connections = config.max_connections))]
    pub async fn create(config: DatabaseConfig) -> Result<Self> {
        let normalized_url = config.normalized_url();
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&normalized_url)
            .await
            .with_context(|| "connect database pool")?;
        let mut orm = SqlxPostgresConnector::from_sqlx_postgres_pool(pool.clone());
        install_sea_orm_query_metrics(&mut orm, database_namespace_from_url(&normalized_url));
        let raw_pool = DbPool::from_pg_pool(pool);

        Ok(Self { orm, raw_pool })
    }

    /// Return the SeaORM connection used by services and entities.
    ///
    /// Prefer this path for table CRUD, query builders, fixed projections,
    /// aggregates, and upserts.
    pub fn orm(&self) -> &DatabaseConnection {
        &self.orm
    }

    /// Run an application transaction on the SeaORM connection.
    ///
    /// Use this when multiple service calls need one commit/rollback boundary.
    /// Service functions should accept `&impl ConnectionTrait` so they can be
    /// called both inside and outside this transaction helper.
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        T: Send,
        F: for<'a> FnOnce(&'a DbTransaction) -> TransactionFuture<'a, T> + Send,
    {
        let span = tracing::info_span!(
            "lilium-database.transaction",
            sentry.name = "db transaction",
            sentry.op = "db.transaction",
            db.system = "postgresql",
            db.system.name = "postgresql",
            db.orm = "sea-orm",
        );

        async move {
            self.orm
                .transaction(|tx| f(tx))
                .await
                .map_err(|error| match error {
                    TransactionError::Connection(error) => anyhow::Error::new(error),
                    TransactionError::Transaction(error) => error,
                })
        }
        .instrument(span)
        .await
    }

    /// Acquire a short-lived raw SQLx connection from the normal query pool.
    ///
    /// This is for narrow infrastructure escape hatches. It is not a substitute
    /// for [`NotificationConnection`] and should not be used for long-lived
    /// connection-stateful primitives.
    #[instrument(level = "debug" skip(self))]
    pub async fn raw_connection(&self) -> Result<RawDbConnection> {
        let conn = self
            .raw_pool
            .inner()
            .acquire()
            .await
            .with_context(|| "acquire raw SQL connection")?;
        Ok(RawDbConnection { inner: conn })
    }
}

fn database_namespace_from_url(url: &str) -> Option<String> {
    let after_scheme = url.split_once("://")?.1;
    let path = after_scheme.split_once('/')?.1;
    let database = path
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .trim_matches('/');

    (!database.is_empty()).then(|| database.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as lilium_database;
    use sea_orm::ConnectionTrait;
    use std::collections::VecDeque;

    #[derive(Debug, Default)]
    struct FakeNotificationListener {
        listened: Vec<String>,
        unlistened: Vec<String>,
        payloads: VecDeque<String>,
    }

    impl NotificationListenerBackend for FakeNotificationListener {
        fn listen<'a>(
            &'a mut self,
            channel: &'a str,
        ) -> Pin<Box<dyn Future<Output = std::result::Result<(), sqlx::Error>> + Send + 'a>>
        {
            Box::pin(async move {
                self.listened.push(channel.to_string());
                Ok(())
            })
        }

        fn unlisten<'a>(
            &'a mut self,
            channel: &'a str,
        ) -> Pin<Box<dyn Future<Output = std::result::Result<(), sqlx::Error>> + Send + 'a>>
        {
            Box::pin(async move {
                self.unlistened.push(channel.to_string());
                Ok(())
            })
        }

        fn try_recv_payload<'a>(
            &'a mut self,
        ) -> Pin<
            Box<dyn Future<Output = std::result::Result<Option<String>, sqlx::Error>> + Send + 'a>,
        > {
            Box::pin(async move { Ok(self.payloads.pop_front()) })
        }

        fn recv_payload<'a>(
            &'a mut self,
        ) -> Pin<Box<dyn Future<Output = std::result::Result<String, sqlx::Error>> + Send + 'a>>
        {
            Box::pin(async move {
                Ok(self
                    .payloads
                    .pop_front()
                    .expect("fake notification payload"))
            })
        }
    }

    #[test]
    fn from_url_sets_connection_and_max_connections() {
        let config = DatabaseConfig::from_url("postgres://localhost/db", 12);

        assert_eq!(config.url, "postgres://localhost/db".to_string());
        assert_eq!(config.max_connections, 12);
    }

    #[test]
    fn normalized_url_accepts_postgresql_scheme() {
        let config = DatabaseConfig::from_url("  postgresql://localhost/db  ", 8);

        assert_eq!(config.normalized_url(), "postgres://localhost/db");
    }

    #[test]
    fn notification_config_uses_same_url_normalization() {
        let config = NotificationDatabaseConfig::from_url("  postgresql://localhost/notify  ");

        assert_eq!(config.normalized_url(), "postgres://localhost/notify");
    }

    #[test]
    fn dedicated_config_uses_same_url_normalization() {
        let config = DedicatedDatabaseConfig::from_url("  postgresql://localhost/lock  ");

        assert_eq!(config.normalized_url(), "postgres://localhost/lock");
    }

    #[tokio::test]
    async fn notification_connection_delegates_to_backend_without_live_db() {
        let backend = FakeNotificationListener {
            payloads: VecDeque::from(["payload-1".to_string(), "payload-2".to_string()]),
            ..Default::default()
        };
        let mut connection = NotificationConnection { listener: backend };

        connection.listen("channel_a").await.unwrap();
        connection.listen("channel_b").await.unwrap();

        let first = connection.try_recv_payload().await.unwrap();
        let second = connection.recv_payload().await.unwrap();
        connection.unlisten("channel_a").await.unwrap();

        assert_eq!(first, Some("payload-1".to_string()));
        assert_eq!(second, "payload-2".to_string());
        assert_eq!(
            connection.listener.listened,
            vec!["channel_a".to_string(), "channel_b".to_string()]
        );
        assert_eq!(
            connection.listener.unlistened,
            vec!["channel_a".to_string()]
        );
    }

    #[tokio::test]
    async fn create_uses_one_shared_pg_pool_budget() {
        dotenvy::dotenv().ok();
        let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let db = Database::create(DatabaseConfig::from_url(url, 1))
            .await
            .unwrap();

        let sea_pool = db.orm().get_postgres_connection_pool();

        assert_eq!(sea_pool.options().get_max_connections(), 1);

        db.orm().execute_unprepared("SELECT 1").await.unwrap();

        let _raw_conn = db.raw_connection().await.unwrap();

        assert!(sea_pool.try_acquire().is_none());
    }

    #[tokio::test]
    async fn transaction_macro_commits_and_raw_connection_executes() {
        dotenvy::dotenv().ok();
        let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let db = Database::create(DatabaseConfig::from_url(url, 1))
            .await
            .unwrap();
        let table_name = format!(
            "transaction_api_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        async fn run_transaction_api_scenario(
            db: &Database,
            table_name: &str,
        ) -> anyhow::Result<()> {
            {
                let mut conn = db.raw_connection().await?;
                sqlx::query(&format!(
                    "CREATE TABLE {table_name} (id BIGINT PRIMARY KEY, note TEXT NOT NULL)"
                ))
                .execute(conn.as_mut())
                .await?;
                sqlx::query(&format!(
                    "INSERT INTO {table_name} (id, note) VALUES (1, 'raw connection row')"
                ))
                .execute(conn.as_mut())
                .await?;
                let raw_count: i64 =
                    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table_name}"))
                        .fetch_one(conn.as_mut())
                        .await?;
                anyhow::ensure!(
                    raw_count == 1,
                    "raw connection did not observe its own insert"
                );
            }

            let committed_table_name = table_name.to_owned();
            lilium_database::transaction!(db, |session| {
                session
                    .execute_unprepared(&format!(
                        "INSERT INTO {committed_table_name} (id, note) VALUES (2, 'committed row')"
                    ))
                    .await?;
                Ok(())
            })
            .await?;

            {
                let mut conn = db.raw_connection().await?;
                let committed_count: i64 =
                    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table_name}"))
                        .fetch_one(conn.as_mut())
                        .await?;
                anyhow::ensure!(
                    committed_count == 2,
                    "committed transaction row was not visible through raw SQL"
                );
            }

            let rollback_table_name = table_name.to_owned();
            let rollback_result: anyhow::Result<()> = db
                .transaction(|session| {
                    Box::pin(async move {
                        session
                            .execute_unprepared(&format!(
                                "INSERT INTO {rollback_table_name} (id, note) VALUES (3, 'rollback row')"
                            ))
                            .await?;
                        Err(anyhow::anyhow!("force rollback"))
                    })
                })
                .await;
            anyhow::ensure!(
                rollback_result.is_err(),
                "rollback transaction unexpectedly succeeded"
            );

            {
                let mut conn = db.raw_connection().await?;
                let visible_count: i64 =
                    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table_name}"))
                        .fetch_one(conn.as_mut())
                        .await?;
                anyhow::ensure!(
                    visible_count == 2,
                    "rolled back row became visible through raw SQL"
                );
            }

            Ok(())
        }

        let result = run_transaction_api_scenario(&db, &table_name).await;

        {
            let mut conn = db.raw_connection().await.unwrap();
            sqlx::query(&format!("DROP TABLE IF EXISTS {table_name}"))
                .execute(conn.as_mut())
                .await
                .unwrap();
        }

        result.unwrap();
    }
}
