use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod arbiter;
mod worker;
mod control;
mod ingestion;

#[derive(Parser)]
#[command(name = "lilium-spider")]
#[command(about = "Lilium Spider - WebSocket ingestion service")]
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
    tracing::info!("Starting lilium-spider");

    // Connect to database - do NOT run migrations
    // Migrations are managed separately by Alembic in the Python project
    let pool = sqlx::PgPool::connect(&config.database.url).await?;

    let arbiter = arbiter::Arbiter::new(config, pool);
    arbiter.run().await
}
