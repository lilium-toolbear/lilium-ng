// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/send_command.py
use anyhow::Result;
use clap::{Parser, Subcommand};
use lilium_database::Database;

mod config;
mod explore;
mod send_command;
mod sync_members;
mod sync_rooms;

#[derive(Parser)]
#[command(
    name = "lilium-cli",
    about = "Lilium administration CLI (port of dzmm_archive cli/*)",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Send commands to the DZMM spider via the database queue
    #[command(name = "send-command")]
    SendCommand {
        #[command(subcommand)]
        cmd: send_command::SendCommand,
    },
    /// Sync room member information from DZMM API to database
    #[command(name = "sync-members")]
    SyncMembers(sync_members::SyncMembersArgs),
    /// Sync room list from DZMM API to database
    #[command(name = "sync-rooms")]
    SyncRooms(sync_rooms::SyncRoomsArgs),
    /// Fetch DZMM explore feed content (tweets, cards, galleries, etc.)
    #[command(name = "explore")]
    Explore(explore::ExploreArgs),
}

async fn async_main() -> Result<u8> {
    let config = config::Config::load()?;
    tracing::info!("Starting lilium-cli");

    let db = Database::create(config.database.clone().into()).await?;

    let cli = Cli::parse();
    match cli.command {
        Command::SendCommand { cmd } => cmd.run(db.orm(), config.notification.into()).await,
        Command::SyncMembers(args) => args.run(&db).await,
        Command::SyncRooms(args) => args.run(&db).await,
        Command::Explore(args) => args.run(&db).await,
    }
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let _sentry_guard = lilium_common::observability::init_backend_sentry("lilium_cli");
    let code = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main())?;
    if code != 0 {
        std::process::exit(code as i32);
    }
    Ok(())
}
