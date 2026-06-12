use anyhow::Result;
use sqlx::PgPool;
use tracing::info;

pub struct EventProcessor {
    #[allow(dead_code)]
    pool: PgPool,
    batch_size: usize,
    polling_interval_secs: u64,
}

impl EventProcessor {
    pub fn new(pool: PgPool, batch_size: usize, polling_interval_secs: u64) -> Self {
        Self { pool, batch_size, polling_interval_secs }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Event processor starting, batch_size={}, poll_interval={}s",
            self.batch_size, self.polling_interval_secs);
        // TODO: Implement NOTIFY + polling fallback
        // TODO: Process events in batches
        Ok(())
    }
}
