use crate::pool::DbPool;
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use sqlx::postgres::PgPoolOptions;
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
}

fn normalize_database_url(url: &str) -> String {
    let url = url.trim();
    if let Some(rest) = url.strip_prefix("postgresql://") {
        format!("postgres://{rest}")
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
