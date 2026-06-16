use anyhow::Result;
use lilium_database::Database;
use tracing::Instrument;

mod arbiter;
mod config;
mod control;
mod ingestion;
mod worker;

async fn async_main() -> Result<()> {
    let config = config::Config::load()?;
    tracing::info!("Starting lilium-spider");

    // Create database runtime - do NOT run migrations
    // Migrations are managed separately by Alembic in the Python project
    let db = Database::create(config.database.clone().into()).await?;

    let arbiter = arbiter::Arbiter::new(config, db);
    arbiter.run().await
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _sentry_guard = lilium_common::observability::init_backend_sentry("ws_arbiter");
    let root_span = tracing::info_span!(
        "lilium-spider.run",
        sentry.name = "spider run",
        sentry.op = "spider.run",
    );
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main().instrument(root_span))
}
