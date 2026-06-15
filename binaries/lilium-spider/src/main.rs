use anyhow::Result;
use lilium_database::Database;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod arbiter;
mod config;
mod control;
mod ingestion;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _sentry_guard = lilium_common::observability::init_backend_sentry("ws_arbiter");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(sentry_tracing::layer())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::load()?;
    tracing::info!("Starting lilium-spider");

    // Create database runtime - do NOT run migrations
    // Migrations are managed separately by Alembic in the Python project
    let db = Database::create(config.database.clone().into()).await?;
    let pool = db.raw_pool().clone();

    let arbiter = arbiter::Arbiter::new(config, pool);
    arbiter.run().await
}
