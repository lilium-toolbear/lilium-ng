use sqlx::postgres::{PgPool, PgPoolOptions};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct DbPool {
    inner: PgPool,
}

impl DbPool {
    pub async fn connect(url: &str, pool_size: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(url)
            .await?;
        Ok(Self { inner: pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.inner).await?;
        Ok(())
    }

    pub fn inner(&self) -> &PgPool {
        &self.inner
    }
}
