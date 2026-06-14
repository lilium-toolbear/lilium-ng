use anyhow::Result;
use lilium_database::DbPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod processor;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _sentry_guard = lilium_common::observability::init_backend_sentry("event_processor");
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(sentry_tracing::layer())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::load()?;
    tracing::info!("Starting lilium-event-processor");

    // Connect to database - do NOT run migrations
    let pool = DbPool::connect_from_env("DATABASE_URL", config.database.pool_size).await?;

    let processor = processor::EventProcessor::new(
        pool,
        "event_processor_main".to_string(),
        config.processor.batch_size,
        config.processor.polling_interval_secs,
    );

    let shutdown = processor.shutdown_handle();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.notify_waiters();
    });

    processor.run().await
}
