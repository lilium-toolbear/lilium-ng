// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 cli/explore.py
//
// Ports the argparse CLI to clap. Note the Python `--backfill` flag uses
// `action="store_false"`: backfill defaults to True, and passing `--backfill`
// sets it to False (i.e. "stop on known content" mode).
use anyhow::Result;
use clap::Args;
use lilium_database::Database;
use lilium_services::{account::AuthClientFactory, explore};
use std::path::PathBuf;

#[derive(Args)]
pub struct SyncExploreArgs {
    /// Sort method (default: recent)
    #[arg(long, default_value = "recent")]
    pub sort: String,
    /// Maximum pages to fetch (default: unlimited until known content)
    #[arg(long = "max-pages")]
    pub max_pages: Option<u32>,
    /// Poll mode: run fetch every 5 minutes continuously
    #[arg(long)]
    pub poll: bool,
    /// Disable backfill mode (stop on known content). Python's --backfill uses
    /// store_false, so passing this flag turns backfill OFF.
    #[arg(long)]
    pub backfill: bool,
    /// Initial offset to start fetching from (default: 0)
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
    /// Number of items per page (default: 100)
    #[arg(long = "page-size", default_value_t = 100)]
    pub page_size: u64,
    /// Comma-separated content types to fetch
    #[arg(
        long,
        default_value = "cards,novels,tweets,checkpoints,galleries,gamefy"
    )]
    pub types: String,
}

impl SyncExploreArgs {
    /// Execute the explore subcommand. Returns a process exit code.
    ///
    /// `data_path` is the global data directory from `Config.cli.data_path`
    /// (sourced from the `DATA_PATH` env var).
    pub async fn run(self, db: &Database, data_path: &str) -> Result<u8> {
        let data_path = PathBuf::from(data_path);
        std::fs::create_dir_all(&data_path).ok();
        let auth_clients = AuthClientFactory::new(db.clone());

        // Python: --backfill store_false → backfill = not args.backfill.
        let backfill = !self.backfill;

        let config = explore::ExploreFetchConfig {
            sort: self.sort.clone(),
            max_pages: self.max_pages,
            initial_offset: self.offset,
            page_size: self.page_size,
            content_types: self
                .types
                .split(',')
                .map(|t| t.trim().to_owned())
                .filter(|t| !t.is_empty())
                .collect(),
            ..Default::default()
        };

        if self.poll {
            tracing::info!("Poll mode enabled: fetching every 5 minutes (Ctrl+C to stop)");
            let mut poll_count = 0u32;
            loop {
                poll_count += 1;
                tracing::info!("[Poll #{poll_count}] starting fetch");
                if let Err(e) = run_fetch(db, &auth_clients, &data_path, &config, backfill).await {
                    tracing::error!("❌ Fetch error: {e}");
                }
                tracing::info!("Waiting 5 minutes until next fetch...");
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        tracing::info!("\n⚠️  Interrupted by user");
                        return Ok(130);
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {}
                }
            }
        } else {
            run_fetch(db, &auth_clients, &data_path, &config, backfill)
                .await
                .map(|_| 0u8)
                .or_else(|e| {
                    tracing::error!("❌ Fetch error: {e}");
                    Ok(1)
                })
        }
    }
}

async fn run_fetch(
    db: &Database,
    auth_clients: &AuthClientFactory,
    data_path: &std::path::Path,
    config: &explore::ExploreFetchConfig,
    backfill: bool,
) -> Result<()> {
    tracing::info!("Starting explore feed fetch");
    tracing::debug!(sort = %config.sort, max_pages = ?config.max_pages, "Config");

    let conn = db.orm();
    let Some(api) = explore::next_auth_client(conn, auth_clients)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
    else {
        tracing::error!("❌ No available accounts. Add an account first.");
        anyhow::bail!("No available accounts");
    };

    let mut fetcher =
        explore::ExploreFetcher::new(&api, data_path.to_owned(), config.clone(), backfill);
    let stats = fetcher
        .fetch_and_process(conn)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    print_stats(&stats);
    if stats.stopped_early {
        tracing::info!("✓ Sync complete: reached known content");
    } else {
        tracing::info!("✓ Sync complete: processed all available pages");
    }
    Ok(())
}

fn print_stats(stats: &explore::ExploreFetchStats) {
    tracing::info!("{}", "=".repeat(50));
    tracing::info!("Explore Feed Fetching Statistics");
    tracing::info!("{}", "=".repeat(50));
    tracing::info!("Pages fetched: {}", stats.pages_fetched);
    tracing::info!("Tweets saved: {}", stats.tweets_saved);
    tracing::info!("Tweets updated: {}", stats.tweets_updated);
    tracing::info!("Cards saved: {}", stats.cards_saved);
    tracing::info!("Galleries saved: {}", stats.galleries_saved);
    tracing::info!("Checkpoints saved: {}", stats.checkpoints_saved);
    tracing::info!("Books saved: {}", stats.books_saved);
    tracing::info!("Chapters saved: {}", stats.chapters_saved);
    tracing::info!("Images downloaded: {}", stats.images_downloaded);
    tracing::info!("Other content skipped: {}", stats.other_content_skipped);
    tracing::info!("Errors: {}", stats.errors);
    if stats.stopped_early {
        tracing::info!("Status: Stopped early (reached known content)");
    } else {
        tracing::info!("Status: Completed all available pages");
    }
    tracing::info!("{}", "=".repeat(50));
}
