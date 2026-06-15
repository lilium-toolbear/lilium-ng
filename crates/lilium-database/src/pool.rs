use anyhow::{Context, Result};
use sqlx::{pool::PoolConnection, PgPool, Postgres};
use std::future::Future;
use std::pin::Pin;
use tracing::instrument;

pub type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct DbPool {
    inner: PgPool,
}

#[derive(Debug)]
pub struct DbSession {
    inner: PoolConnection<Postgres>,
}

impl DbSession {
    fn new(inner: PoolConnection<Postgres>) -> Self {
        Self { inner }
    }
}

impl std::ops::Deref for DbSession {
    type Target = PoolConnection<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for DbSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub(crate) fn normalize_database_url(url: &str) -> String {
    let url = url.trim();
    if let Some(rest) = url.strip_prefix("postgresql://") {
        format!("postgres://{rest}")
    } else {
        url.to_string()
    }
}

impl DbPool {
    pub(crate) fn from_pg_pool(inner: PgPool) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &PgPool {
        &self.inner
    }

    #[instrument(skip(self))]
    pub async fn begin_session(&self) -> Result<DbSession> {
        let mut conn = self
            .inner
            .acquire()
            .await
            .with_context(|| "failed to begin database transaction")?;
        sqlx::query("BEGIN")
            .execute(conn.as_mut())
            .await
            .with_context(|| "failed to begin database transaction")?;
        Ok(DbSession::new(conn))
    }

    async fn commit_session(session: &mut DbSession) -> Result<()> {
        sqlx::query("COMMIT")
            .execute(session.inner.as_mut())
            .await
            .with_context(|| "failed to commit session")?;
        Ok(())
    }

    async fn rollback_session(session: &mut DbSession) -> Result<()> {
        sqlx::query("ROLLBACK")
            .execute(session.inner.as_mut())
            .await
            .with_context(|| "failed to rollback session")?;
        Ok(())
    }

    async fn run_session<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        let mut session = self.begin_session().await?;
        let result = f(&mut session).await;
        match result {
            Ok(value) => {
                Self::commit_session(&mut session).await?;
                Ok(value)
            }
            Err(error) => {
                Self::rollback_session(&mut session).await.ok();
                Err(error)
            }
        }
    }

    #[instrument(skip(self, f))]
    pub async fn with_session<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        self.run_session(f).await
    }
}
