use anyhow::Result;
use lilium_database::Database;

mod config;
mod processor;

async fn async_main() -> Result<()> {
    let config = config::Config::load()?;
    tracing::info!("Starting lilium-event-processor");

    // Create database runtime - do NOT run migrations
    let db = Database::create(config.database.clone().into()).await?;

    let processor = processor::EventProcessor::new(
        db,
        "event_processor_main".to_string(),
        config.processor.batch_size,
        config.processor.polling_interval_secs,
    )
    .with_notification_config(config.notification.into());

    let shutdown = processor.shutdown_handle();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.notify_waiters();
    });

    processor.run().await
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _sentry_guard = lilium_common::observability::init_backend_sentry("event_processor");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())
}
