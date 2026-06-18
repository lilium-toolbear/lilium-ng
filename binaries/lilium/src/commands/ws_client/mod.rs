pub mod arbiter;
pub mod control;
pub mod ingestion;
pub mod worker;

use anyhow::Result;
use lilium_database::Database;

use crate::config::Config;

pub async fn run(config: Config, db: Database) -> Result<()> {
    let arbiter = arbiter::Arbiter::new(config, db);
    arbiter.run().await
}
