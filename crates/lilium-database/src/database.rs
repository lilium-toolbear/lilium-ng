use crate::pool::DbPool;
use crate::pool::DbSession;
use crate::pool::normalize_database_url;
use crate::transaction::TransactionFuture;
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use sqlx::{pool::PoolConnection, Postgres};
use sqlx::postgres::PgPoolOptions;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tracing::instrument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn from_url(url: impl Into<String>, max_connections: u32) -> Self {
        Self {
            url: url.into(),
            max_connections,
        }
    }

    pub fn normalized_url(&self) -> String {
        normalize_database_url(&self.url)
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    orm: DatabaseConnection,
    raw_pool: DbPool,
}

#[derive(Debug)]
pub struct RawDbConnection {
    inner: PoolConnection<Postgres>,
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
    #[instrument(skip(config), fields(max_connections = config.max_connections))]
    pub async fn create(config: DatabaseConfig) -> Result<Self> {
        let normalized_url = config.normalized_url();
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(30))
            .connect(&normalized_url)
            .await
            .with_context(|| "connect database pool")?;
        let orm = SqlxPostgresConnector::from_sqlx_postgres_pool(pool.clone());
        let raw_pool = DbPool::from_pg_pool(pool);

        Ok(Self { orm, raw_pool })
    }

    pub fn orm(&self) -> &DatabaseConnection {
        &self.orm
    }

    pub fn raw_pool(&self) -> &DbPool {
        &self.raw_pool
    }

    #[instrument(skip(self, f))]
    pub async fn transaction<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> TransactionFuture<'a, T>,
    {
        self.raw_pool.with_session(f).await
    }

    #[instrument(skip(self))]
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

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::ConnectionTrait;

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

    #[tokio::test]
    #[ignore = "requires TEST_DATABASE_URL"]
    async fn create_uses_one_shared_pg_pool_budget() {
        dotenvy::dotenv().ok();
        let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
        let db = Database::create(DatabaseConfig::from_url(url, 1))
            .await
            .unwrap();

        let raw_pool = db.raw_pool().inner();
        let sea_pool = db.orm().get_postgres_connection_pool();

        assert_eq!(raw_pool.options().get_max_connections(), 1);
        assert_eq!(sea_pool.options().get_max_connections(), 1);

        db.orm().execute_unprepared("SELECT 1").await.unwrap();

        let _raw_conn = raw_pool.acquire().await.unwrap();

        assert!(sea_pool.try_acquire().is_none());
    }
}
