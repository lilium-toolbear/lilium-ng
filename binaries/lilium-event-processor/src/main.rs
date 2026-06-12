use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod processor;

#[derive(Parser)]
#[command(name = "lilium-event-processor")]
#[command(about = "Lilium Event Processor - Batch queue consumer")]
struct Cli {
    #[arg(short, long, default_value = "config/spider.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let config = config::Config::load(&cli.config)?;
    tracing::info!("Starting lilium-event-processor");

    // Connect to database - do NOT run migrations
    let pool = sqlx::PgPool::connect(&config.database.url).await?;

    let processor = processor::EventProcessor::new(
        pool,
        "event_processor_main".to_string(),
        config.processor.batch_size,
        config.processor.polling_interval_secs,
    );

    processor.run().await
}
