use anyhow::{Context, Result};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{pool::PoolConnection, PgPool, Postgres};
use std::borrow::Borrow;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use tracing::instrument;

pub type SessionFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct DbPool {
    inner: PgPool,
}

#[derive(Copy, Clone, Debug)]
enum SessionFinish {
    Commit,
    Rollback,
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

#[derive(Debug)]
pub struct DbSessionContext<'a> {
    session: &'a mut DbSession,
}

impl<'a> DbSessionContext<'a> {
    pub fn new(session: &'a mut DbSession) -> Self {
        Self { session }
    }
}

impl<'a> Deref for DbSessionContext<'a> {
    type Target = DbSession;

    fn deref(&self) -> &Self::Target {
        self.session
    }
}

impl<'a> DerefMut for DbSessionContext<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session
    }
}

impl DbPool {
    fn normalize_database_url(url: &str) -> String {
        let url = url.trim();
        if let Some(rest) = url.strip_prefix("postgresql://") {
            format!("postgres://{rest}")
        } else {
            url.to_string()
        }
    }

    #[instrument(skip(url), fields(pool_size))]
    pub async fn connect_with_session(url: &str, pool_size: u32) -> Result<Self> {
        Self::connect(url, pool_size).await
    }

    #[instrument(skip(url), fields(pool_size))]
    pub async fn connect(url: &str, pool_size: u32) -> Result<Self> {
        let normalized_url = Self::normalize_database_url(url);
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(&normalized_url)
            .await
            .with_context(|| format!("failed to connect database: {normalized_url}"))?;
        Ok(Self { inner: pool })
    }

    #[instrument(skip(options), fields(pool_size))]
    pub async fn connect_with_options(options: PgConnectOptions, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect_with(options)
            .await
            .with_context(|| "failed to connect database with explicit options")?;
        Ok(Self { inner: pool })
    }

    #[instrument(fields(env_var = var_name, pool_size))]
    pub async fn connect_from_env(var_name: &str, pool_size: u32) -> Result<Self> {
        let db_url = std::env::var(var_name)
            .with_context(|| format!("required env var '{var_name}' is missing"))?;
        Self::connect(&db_url, pool_size).await
    }

    #[instrument(skip(url))]
    pub async fn connect_lazy(url: &str) -> Result<Self> {
        let normalized_url = Self::normalize_database_url(url);
        let pool = PgPool::connect_lazy(&normalized_url)
            .with_context(|| "failed to create lazy database pool")?;
        Ok(Self { inner: pool })
    }

    #[instrument(skip(options))]
    pub async fn connect_lazy_with_options(options: PgConnectOptions) -> Result<Self> {
        let pool = PgPoolOptions::new().connect_lazy_with(options);
        Ok(Self { inner: pool })
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

    async fn finish_session(session: &mut DbSession, finish: SessionFinish) -> Result<()> {
        match finish {
            SessionFinish::Commit => Self::commit_session(session).await,
            SessionFinish::Rollback => Self::rollback_session(session).await,
        }
    }

    async fn run_session<T, F>(&self, finish: SessionFinish, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        let mut session = self.begin_session().await?;
        let result = f(&mut session).await;
        match result {
            Ok(value) => {
                Self::finish_session(&mut session, finish).await?;
                Ok(value)
            }
            Err(error) => {
                Self::rollback_session(&mut session).await.ok();
                Err(error)
            }
        }
    }

    async fn run_session_context<T, F>(&self, finish: SessionFinish, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(DbSessionContext<'a>) -> SessionFuture<'a, T>,
    {
        self.run_session(finish, |session| {
            let context = DbSessionContext::new(session);
            f(context)
        })
        .await
    }

    #[instrument(skip(self, f))]
    pub async fn with_session<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        self.run_session(SessionFinish::Commit, f).await
    }

    #[instrument(skip(self, f))]
    pub async fn with_session_context<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(DbSessionContext<'a>) -> SessionFuture<'a, T>,
    {
        self.run_session_context(SessionFinish::Commit, f).await
    }

    #[instrument(skip(self, f))]
    pub async fn with_rollback_session<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        self.run_session(SessionFinish::Rollback, f).await
    }

    #[instrument(skip(self, f))]
    pub async fn with_rollback_session_context<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(DbSessionContext<'a>) -> SessionFuture<'a, T>,
    {
        self.run_session_context(SessionFinish::Rollback, f).await
    }

    #[instrument(skip(self, f))]
    pub async fn with_session_boxed<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        self.with_session(f).await
    }

    #[instrument(skip(self, f))]
    pub async fn with_rollback_session_boxed<T, F>(&self, f: F) -> Result<T>
    where
        F: for<'a> FnOnce(&'a mut DbSession) -> SessionFuture<'a, T>,
    {
        self.with_rollback_session(f).await
    }
}

impl Borrow<PgPool> for DbPool {
    fn borrow(&self) -> &PgPool {
        self.inner()
    }
}

impl Deref for DbPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}
