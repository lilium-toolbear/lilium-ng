use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

pub struct Worker {
    account_id: String,
    #[allow(dead_code)]
    pool: PgPool,
}

impl Worker {
    pub fn new(account_id: String, pool: PgPool) -> Self {
        Self { account_id, pool }
    }

    pub async fn run(&self) -> Result<()> {
        info!(account = %self.account_id, "Worker running");
        // TODO: Connect to DZMM WebSocket
        // TODO: Process events
        // TODO: Write to database
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }
}
