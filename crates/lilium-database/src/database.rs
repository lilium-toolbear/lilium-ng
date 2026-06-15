use crate::pool::DbPool;
use crate::pool::DbSession;
use crate::pool::normalize_database_url;
use crate::transaction::TransactionFuture;
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Postgres, pool::PoolConnection};
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
    use crate as lilium_database;
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

        let sea_pool = db.orm().get_postgres_connection_pool();

        assert_eq!(sea_pool.options().get_max_connections(), 1);

        db.orm().execute_unprepared("SELECT 1").await.unwrap();

        let _raw_conn = db.raw_connection().await.unwrap();

        assert!(sea_pool.try_acquire().is_none());
    }

    #[tokio::test]
    #[ignore = "requires TEST_DATABASE_URL"]
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
                sqlx::query(&format!(
                    "INSERT INTO {committed_table_name} (id, note) VALUES (2, 'committed row')"
                ))
                .execute(session.as_mut())
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
                        sqlx::query(&format!(
                            "INSERT INTO {rollback_table_name} (id, note) VALUES (3, 'rollback row')"
                        ))
                        .execute(session.as_mut())
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
