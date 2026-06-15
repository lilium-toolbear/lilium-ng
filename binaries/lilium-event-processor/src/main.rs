use anyhow::Result;
use lilium_database::Database;
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

    // Create database runtime - do NOT run migrations
    let db = Database::create(config.database.clone().into()).await?;

    let processor = processor::EventProcessor::new(
        db,
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
