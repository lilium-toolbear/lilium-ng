// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/send_command.py
use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use lilium_database::Database;

mod commands;
mod config;

#[derive(Parser)]
#[command(
    name = "lilium",
    about = "Lilium unified binary for servers and operational commands"
)]
struct Cli {
    #[command(subcommand)]
    command: Verb,
}

#[derive(Subcommand)]
enum Verb {
    /// Run the WebSocket client arbiter
    #[command(name = "ws-client")]
    WsClient {
        #[command(subcommand)]
        worker: Option<WsClientWorker>,
    },

    /// Run the event processor
    #[command(name = "event-processor")]
    EventProcessor,

    /// Send commands to the spider via the database queue
    #[command(name = "send-command")]
    SendCommand {
        #[command(subcommand)]
        cmd: commands::send_command::SendCommand,
    },

    /// Sync room member information from DZMM API to database
    #[command(name = "sync-members")]
    SyncMembers(commands::sync_members::SyncMembersArgs),

    /// Sync room list from DZMM API to database
    #[command(name = "sync-rooms")]
    SyncRooms(commands::sync_rooms::SyncRoomsArgs),

    /// Fetch DZMM explore feed content
    #[command(name = "explore")]
    Explore(commands::explore::ExploreArgs),

    /// Generate shell completion scripts
    #[command(name = "completion")]
    Completion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand)]
enum WsClientWorker {
    /// Internal: run a single account worker (spawned by the arbiter)
    #[command(hide = true)]
    Worker {
        #[arg(long)]
        account: String,
    },
}

fn sentry_name(verb: &Verb) -> &'static str {
    match verb {
        Verb::WsClient { .. } => "ws_arbiter",
        Verb::EventProcessor => "event_processor",
        Verb::SendCommand { .. }
        | Verb::SyncMembers(_)
        | Verb::SyncRooms(_)
        | Verb::Explore(_)
        | Verb::Completion { .. } => "lilium_cli",
    }
}

async fn async_main() -> Result<u8> {
    let cli = Cli::parse();

    let sentry_name = sentry_name(&cli.command);
    let _sentry_guard = lilium_common::observability::init_backend_sentry(sentry_name);

    if let Verb::Completion { shell } = cli.command {
        let mut app = Cli::command();
        clap_complete::generate(shell, &mut app, "lilium", &mut std::io::stdout());
        return Ok(0);
    }

    let config = config::Config::load()?;
    let db = Database::create(config.database.clone().into()).await?;

    match cli.command {
        Verb::WsClient { worker: None } => {
            commands::ws_client::run(config, db).await?;
        }
        Verb::WsClient {
            worker: Some(WsClientWorker::Worker { account }),
        } => {
            commands::ws_client::run_worker(account, config, db).await?;
        }
        Verb::EventProcessor => {
            commands::event_processor::run(config, db).await?;
        }
        Verb::SendCommand { cmd } => {
            return cmd.run(db.orm(), config.notification.into()).await;
        }
        Verb::SyncMembers(args) => {
            args.run(&db).await?;
        }
        Verb::SyncRooms(args) => {
            args.run(&db).await?;
        }
        Verb::Explore(args) => {
            args.run(&db).await?;
        }
        Verb::Completion { .. } => unreachable!(),
    }

    Ok(0)
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
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
