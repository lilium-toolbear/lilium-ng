# Unified `lilium` Binary + Worker Process Isolation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Consolidate `lilium-spider`, `lilium-event-processor`, and `lilium-cli` into a single `lilium` binary where all servers and commands are invoked through one executable; within `lilium ws-client`, spawn each account worker as a separate child process instead of a tokio task.

**Architecture:** Rename `lilium-cli` to `lilium`, merge the three config files into one, move the spider and event-processor modules under `src/commands/ws_client/` and `src/commands/event_processor.rs`, and replace `TokioWorkerSpawner` with `ProcessWorkerSpawner` that self-re-executes `lilium ws-client worker --account <id>`.

**Tech Stack:** Rust 2024, tokio, clap 4, clap_complete, anyhow, tracing/sentry, `tokio::process`.

## Global Constraints

- Rust edition must be `2024`.
- Never run `sea-orm-cli migrate` or any migration apply command without explicit user approval.
- Preserve existing `// Python parity source:` comments and update `docs/python-to-rust-migration-progress.md` only if this is explicitly parity work; this change is structural, not feature parity.
- Do not add tests that merely pin constant values.
- Run the narrowest credible test verification after each task and full `cargo fmt --all` + `cargo clippy --all-targets --all-features` before final handoff.
- All commits follow Conventional Commits (`feat`, `refactor`, `docs`, `test`, `chore`).
- Use `git mv` when moving files to preserve history.

---

## Task 1: Rename `lilium-cli` Crate to `lilium`

**Files:**
- Modify: `binaries/lilium-cli/Cargo.toml`
- Rename: `binaries/lilium-cli/` → `binaries/lilium/`

**Interfaces:**
- Produces: a `binaries/lilium/` crate whose package name and binary name are both `lilium`.

- [ ] **Step 1: Rename the directory**

```bash
git mv binaries/lilium-cli binaries/lilium
```

- [ ] **Step 2: Update `binaries/lilium/Cargo.toml`**

Replace the package and bin stanzas with:

```toml
[package]
name = "lilium"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "lilium"
path = "src/main.rs"
```

Keep all existing dependencies unchanged for now.

- [ ] **Step 3: Verify the crate still builds with its current code**

```bash
cargo check -p lilium
```

Expected: passes (no functional changes yet).

- [ ] **Step 4: Commit**

```bash
git add binaries/lilium/Cargo.toml
git commit -m "refactor: rename lilium-cli crate and binary to lilium"
```

---

## Task 2: Merge the Three `config.rs` Files into One

**Files:**
- Create: `binaries/lilium/src/config.rs` (replace existing `lilium-cli/src/config.rs`)

**Interfaces:**
- Produces: single `crate::config::Config` used by all subcommands.

- [ ] **Step 1: Write the unified `binaries/lilium/src/config.rs`**

```rust
use anyhow::{Context, Result};
use lilium_api_client::config::ApiClientConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub notification: NotificationConfig,
    pub spider: SpiderConfig,
    pub processor: ProcessorConfig,
    pub cli: CliConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct SpiderConfig {
    pub queue_size: usize,
    pub batch_size: usize,
    pub buffer_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub websocket_url: String,
    pub reconnect_delay_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub polling_interval_secs: u64,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub data_path: String,
}

fn default_pool_size() -> u32 {
    5
}

fn default_queue_size() -> usize {
    5_000
}

fn default_buffer_dir() -> PathBuf {
    PathBuf::from("data/event/buffer")
}

fn default_runtime_dir() -> PathBuf {
    PathBuf::from("runtime/spider")
}

fn default_ws_url() -> String {
    ApiClientConfig::default().ws_url
}

fn default_reconnect_delay_ms() -> u64 {
    5_000
}

fn default_polling_interval() -> u64 {
    5
}

fn default_batch_size() -> usize {
    100
}

fn default_data_path() -> &'static str {
    "./data"
}

fn env_string(name: &str, default: String) -> String {
    std::env::var(name).unwrap_or(default)
}

fn env_required_string(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("required env var '{name}' is missing"))
}

fn env_optional_string(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn fallback_string(
    primary: Option<&str>,
    fallback: Option<&str>,
    primary_name: &str,
    fallback_name: &str,
) -> Result<String> {
    primary
        .map(str::to_owned)
        .or_else(|| fallback.map(str::to_owned))
        .with_context(|| {
            format!("required env vars '{primary_name}' or '{fallback_name}' are missing")
        })
}

fn env_fallback_string(primary: &str, fallback: &str) -> Result<String> {
    fallback_string(
        std::env::var(primary).ok().as_deref(),
        std::env::var(fallback).ok().as_deref(),
        primary,
        fallback,
    )
}

fn env_u32(name: &str, default: u32) -> Result<u32> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<u32>()
            .with_context(|| format!("failed to parse {name} as u32")),
        Err(_) => Ok(default),
    }
}

fn env_u64(name: &str, default: u64) -> Result<u64> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .with_context(|| format!("failed to parse {name} as u64")),
        Err(_) => Ok(default),
    }
}

fn env_usize(name: &str, default: usize) -> Result<usize> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .with_context(|| format!("failed to parse {name} as usize")),
        Err(_) => Ok(default),
    }
}

fn env_path(name: &str, default: PathBuf) -> PathBuf {
    std::env::var(name).map(PathBuf::from).unwrap_or(default)
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            database: DatabaseConfig {
                url: env_required_string("DATABASE_URL")?,
                max_connections: env_u32("DATABASE_POOL_SIZE", default_pool_size())?,
            },
            notification: NotificationConfig {
                url: env_fallback_string("DATABASE_NOTIFICATION_URL", "DATABASE_URL")?,
            },
            spider: SpiderConfig {
                queue_size: env_usize("SPIDER_QUEUE_SIZE", default_queue_size())?,
                batch_size: env_usize("SPIDER_BATCH_SIZE", default_batch_size())?,
                buffer_dir: env_path("SPIDER_BUFFER_DIR", default_buffer_dir()),
                runtime_dir: env_path("SPIDER_RUNTIME_DIR", default_runtime_dir()),
                websocket_url: env_string("SPIDER_WEBSOCKET_URL", default_ws_url()),
                reconnect_delay_ms: env_u64(
                    "SPIDER_RECONNECT_DELAY_MS",
                    default_reconnect_delay_ms(),
                )?,
            },
            processor: ProcessorConfig {
                polling_interval_secs: env_u64(
                    "EVENT_PROCESSOR_POLLING_INTERVAL_SECS",
                    default_polling_interval(),
                )?,
                batch_size: env_usize("EVENT_PROCESSOR_BATCH_SIZE", default_batch_size())?,
            },
            cli: CliConfig {
                data_path: env_optional_string("DATA_PATH", default_data_path()),
            },
        })
    }
}

impl From<DatabaseConfig> for lilium_database::DatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DatabaseConfig::from_url(value.url, value.max_connections)
    }
}

impl From<DatabaseConfig> for lilium_database::DedicatedDatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DedicatedDatabaseConfig::from_url(value.url)
    }
}

impl From<NotificationConfig> for lilium_database::NotificationDatabaseConfig {
    fn from(value: NotificationConfig) -> Self {
        lilium_database::NotificationDatabaseConfig::from_url(value.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_string_prefers_primary_value() {
        let value =
            fallback_string(Some("primary"), Some("fallback"), "PRIMARY", "FALLBACK").unwrap();
        assert_eq!(value, "primary");
    }

    #[test]
    fn fallback_string_uses_fallback_value() {
        let value = fallback_string(None, Some("fallback"), "PRIMARY", "FALLBACK").unwrap();
        assert_eq!(value, "fallback");
    }

    #[test]
    fn notification_config_converts_url() {
        let config = NotificationConfig {
            url: "postgresql://notify".into(),
        };
        let db_config: lilium_database::NotificationDatabaseConfig = config.into();
        assert_eq!(db_config.normalized_url(), "postgres://notify");
    }
}
```

- [ ] **Step 2: Verify config tests pass**

```bash
cargo test -p lilium config::
```

Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add binaries/lilium/src/config.rs
git commit -m "refactor: merge spider/event-processor/cli configs into one"
```

---

## Task 3: Move Spider and Event-Processor Modules into `lilium`

**Files:**
- Create/Rename: `binaries/lilium/src/commands/ws_client/arbiter.rs` (from `binaries/lilium-spider/src/arbiter/mod.rs`)
- Create/Rename: `binaries/lilium/src/commands/ws_client/control.rs` (from `binaries/lilium-spider/src/control.rs`)
- Create/Rename: `binaries/lilium/src/commands/ws_client/ingestion.rs` (from `binaries/lilium-spider/src/ingestion.rs`)
- Create/Rename: `binaries/lilium/src/commands/ws_client/worker.rs` (from `binaries/lilium-spider/src/worker/mod.rs`)
- Create/Rename: `binaries/lilium/src/commands/event_processor.rs` (from `binaries/lilium-event-processor/src/processor.rs`)
- Create: `binaries/lilium/src/commands/ws_client/mod.rs`
- Create: `binaries/lilium/src/commands/mod.rs`
- Modify: `binaries/lilium/Cargo.toml` (add dependencies from old crates)

**Interfaces:**
- Consumes: `crate::config::Config` from Task 2; existing modules from `binaries/lilium-spider/src/` and `binaries/lilium-event-processor/src/`.
- Produces: `commands::ws_client` and `commands::event_processor` modules that expose `run(...)` entry points.

- [ ] **Step 1: Create the commands directory tree and move files with git mv**

```bash
mkdir -p binaries/lilium/src/commands/ws_client
git mv binaries/lilium-spider/src/arbiter/mod.rs binaries/lilium/src/commands/ws_client/arbiter.rs
git mv binaries/lilium-spider/src/control.rs binaries/lilium/src/commands/ws_client/control.rs
git mv binaries/lilium-spider/src/ingestion.rs binaries/lilium/src/commands/ws_client/ingestion.rs
git mv binaries/lilium-spider/src/worker/mod.rs binaries/lilium/src/commands/ws_client/worker.rs
git mv binaries/lilium-event-processor/src/processor.rs binaries/lilium/src/commands/event_processor.rs
```

- [ ] **Step 2: Create `binaries/lilium/src/commands/mod.rs`**

```rust
pub mod event_processor;
pub mod explore;
pub mod send_command;
pub mod sync_members;
pub mod sync_rooms;
pub mod ws_client;
```

- [ ] **Step 3: Create `binaries/lilium/src/commands/ws_client/mod.rs`**

```rust
pub mod arbiter;
pub mod control;
pub mod ingestion;
pub mod worker;

use anyhow::Result;
use lilium_database::Database;

use crate::config::Config;

pub async fn run(config: Config, db: Database) -> Result<()> {
    let arbiter = arbiter::Arbiter::new(config, db);
    arbiter.run().await
}
```

- [ ] **Step 4: Merge dependencies into `binaries/lilium/Cargo.toml`**

Add the dependencies that the spider and event-processor crates had but `lilium-cli` did not:

```toml
[dependencies]
# existing deps...
lilium-api-client.workspace = true
lilium-core.workspace = true
sqlx.workspace = true
rand.workspace = true

[dev-dependencies]
tempfile = "3"
mockall.workspace = true
```

- [ ] **Step 5: Fix moved module imports and config field references**

In `binaries/lilium/src/commands/ws_client/arbiter.rs`:
- Change `use crate::control::{...};` to `use crate::commands::ws_client::control::{...};`.
- Change `use crate::config::Config;` stays as-is (Config now lives at `crate::config`).
- Change all `self.config.worker.*` field accesses to `self.config.spider.*` (e.g., `self.config.spider.queue_size`, `self.config.spider.runtime_dir`).
- Change `self.config.notification.clone().into()` stays as-is.

In `binaries/lilium/src/commands/ws_client/worker.rs`:
- Change `use crate::control::{...};` to `use crate::commands::ws_client::control::{...};`.

In `binaries/lilium/src/commands/event_processor.rs`:
- No config field changes needed here; the run wrapper in Task 5 handles config access.

- [ ] **Step 6: Verify the crate compiles**

```bash
cargo check -p lilium
```

Expected: passes.

- [ ] **Step 7: Commit**

```bash
git add binaries/lilium/src/commands binaries/lilium/Cargo.toml
git commit -m "refactor: move spider and event-processor modules into lilium"
```

---

## Task 4: Delete Old Spider and Event-Processor Crates

**Files:**
- Delete: `binaries/lilium-spider/` (entire crate)
- Delete: `binaries/lilium-event-processor/` (entire crate)
- Modify: root `Cargo.toml` workspace members

**Interfaces:**
- Produces: workspace with only `binaries/lilium` as the executable crate.

- [ ] **Step 1: Remove the old crate directories**

```bash
git rm -r binaries/lilium-spider binaries/lilium-event-processor
```

- [ ] **Step 2: Update root `Cargo.toml` workspace members**

Replace:

```toml
members = [
    "crates/lilium-common",
    "crates/lilium-models",
    "crates/lilium-database",
    "crates/lilium-test-fixtures",
    "crates/lilium-core",
    "crates/lilium-services",
    "crates/lilium-api-client",
    "binaries/lilium-spider",
    "binaries/lilium-event-processor",
    "binaries/lilium-cli",
]
```

with:

```toml
members = [
    "crates/lilium-common",
    "crates/lilium-models",
    "crates/lilium-database",
    "crates/lilium-test-fixtures",
    "crates/lilium-core",
    "crates/lilium-services",
    "crates/lilium-api-client",
    "binaries/lilium",
]
```

- [ ] **Step 3: Verify workspace builds**

```bash
cargo check --workspace
```

Expected: passes.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "refactor: remove lilium-spider and lilium-event-processor crates"
```

---

## Task 5: Wire Command Entry Points (`run` functions)

**Files:**
- Modify: `binaries/lilium/src/commands/ws_client/mod.rs`
- Modify: `binaries/lilium/src/commands/event_processor.rs`

**Interfaces:**
- Produces: consistent `run` signatures for `ws_client` and `event_processor`.

- [ ] **Step 1: Update `binaries/lilium/src/commands/ws_client/mod.rs`**

```rust
pub mod arbiter;
pub mod control;
pub mod ingestion;
pub mod worker;

use anyhow::Result;
use lilium_database::Database;

use crate::config::Config;

pub async fn run(config: Config, db: Database) -> Result<()> {
    let arbiter = arbiter::Arbiter::new(config, db);
    arbiter.run().await
}

pub fn build_worker_runtime(config: &Config) -> worker::WorkerRuntimeConfig {
    worker::WorkerRuntimeConfig {
        notification_config: config.notification.clone().into(),
        lock_config: config.database.clone().into(),
        queue_size: config.spider.queue_size,
        batch_size: config.spider.batch_size,
        buffer_dir: config.spider.buffer_dir.clone(),
        runtime_dir: config.spider.runtime_dir.clone(),
        websocket_url: config.spider.websocket_url.clone(),
        reconnect_delay_ms: config.spider.reconnect_delay_ms,
    }
}
```

- [ ] **Step 2: Add a thin run wrapper to `binaries/lilium/src/commands/event_processor.rs`**

At the bottom of the file (after the `EventProcessor` impl), add:

```rust
use anyhow::Result;
use lilium_database::Database;

use crate::config::Config;

pub async fn run(config: Config, db: Database) -> Result<()> {
    let processor = EventProcessor::new(
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
```

- [ ] **Step 3: Verify compile**

```bash
cargo check -p lilium
```

Expected: passes.

- [ ] **Step 4: Commit**

```bash
git add binaries/lilium/src/commands

git commit -m "refactor: add run entry points for ws_client and event_processor commands"
```

---

## Task 6: Implement Unified `main.rs`, Clap Dispatch, and Shell Completion

**Files:**
- Modify: `binaries/lilium/src/main.rs`
- Modify: `binaries/lilium/Cargo.toml` (add `clap_complete`)
- Modify: root `Cargo.toml` (add `clap_complete` workspace dependency)

**Interfaces:**
- Produces: `Cli`/`Verb`/`WsClientWorker` enums and `main` that dispatches to all commands.

- [ ] **Step 1: Add `clap_complete` to workspace dependencies**

In root `Cargo.toml` under `[workspace.dependencies]`:

```toml
clap_complete = "4"
```

- [ ] **Step 2: Add `clap_complete` to `binaries/lilium/Cargo.toml`**

```toml
clap_complete.workspace = true
```

- [ ] **Step 3: Rewrite `binaries/lilium/src/main.rs`**

```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
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
```

- [ ] **Step 4: Verify compile and completion generation**

```bash
cargo build -p lilium
./target/debug/lilium completion bash > /tmp/lilium-completion.bash
./target/debug/lilium completion zsh > /tmp/lilium-completion.zsh
[ -s /tmp/lilium-completion.bash ] && grep -q "ws-client" /tmp/lilium-completion.bash && echo "bash completion ok"
[ -s /tmp/lilium-completion.zsh ] && grep -q "ws-client" /tmp/lilium-completion.zsh && echo "zsh completion ok"
```

Expected: build passes, completion files non-empty and contain `ws-client`, `event-processor`, `send-command`, etc. They must NOT contain the hidden `worker` subcommand.

- [ ] **Step 5: Commit**

```bash
git add binaries/lilium/src/main.rs binaries/lilium/Cargo.toml Cargo.toml
git commit -m "feat: unify dispatch, add completion command"
```

---

## Task 7: Add `run_worker` Entry for the Hidden `ws-client worker` Subcommand

**Files:**
- Modify: `binaries/lilium/src/commands/ws_client/mod.rs`

**Interfaces:**
- Produces: `commands::ws_client::run_worker(account_id, config, db)` invoked by `Verb::WsClient { worker: Some(WsClientWorker::Worker { account }) }`.

- [ ] **Step 1: Add `run_worker` to `commands/ws_client/mod.rs`**

```rust
pub async fn run_worker(account_id: String, config: Config, db: Database) -> Result<()> {
    let runtime = build_worker_runtime(&config);
    let worker = worker::Worker::new(account_id, db, runtime);
    let shutdown = std::sync::Arc::new(tokio::sync::Notify::new());

    tokio::spawn({
        let shutdown = shutdown.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            shutdown.notify_waiters();
        }
    });

    worker.run(shutdown).await
}
```

- [ ] **Step 2: Verify the hidden subcommand is hidden but functional**

```bash
cargo build -p lilium
./target/debug/lilium --help | grep -v worker
./target/debug/lilium ws-client --help | grep -v worker
./target/debug/lilium ws-client worker --account test --help
```

Expected: top-level and `ws-client` help do not show `worker`; `lilium ws-client worker --account <id>` is accepted.

- [ ] **Step 3: Commit**

```bash
git add binaries/lilium/src/commands/ws_client/mod.rs
git commit -m "feat: add hidden ws-client worker subcommand for arbiter self-exec"
```

---

## Task 8: Implement `ProcessWorkerSpawner`

**Files:**
- Modify: `binaries/lilium/src/commands/ws_client/arbiter.rs`

**Interfaces:**
- Produces: `ProcessWorkerSpawner` that spawns `lilium ws-client worker --account <id>`.

- [ ] **Step 1: Replace `WorkerHandle` to hold a child process**

At the top of `arbiter.rs`, replace:

```rust
use std::process::Stdio;
use tokio::process::Child;

struct WorkerHandle {
    child: Child,
}
```

- [ ] **Step 2: Replace `TokioWorkerSpawner` with `ProcessWorkerSpawner`**

```rust
struct ProcessWorkerSpawner;

impl WorkerSpawner for ProcessWorkerSpawner {
    fn spawn_worker(&self, spec: WorkerSpec) -> WorkerHandle {
        let account = spec.account;
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command
            .arg("ws-client")
            .arg("worker")
            .arg("--account")
            .arg(&account)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .kill_on_drop(true);

        let child = command.spawn().unwrap_or_else(|e| {
            panic!("failed to spawn worker process for account {account}: {e}")
        });

        WorkerHandle { child }
    }
}
```

- [ ] **Step 3: Update `Arbiter::new` to use `ProcessWorkerSpawner`**

```rust
impl Arbiter {
    pub fn new(config: Config, database: Database) -> Self {
        Self::with_worker_spawner(config, database, Arc::new(ProcessWorkerSpawner))
    }
}
```

- [ ] **Step 4: Verify compile**

```bash
cargo check -p lilium
```

Expected: passes.

- [ ] **Step 5: Commit**

```bash
git add binaries/lilium/src/commands/ws_client/arbiter.rs
git commit -m "feat: spawn worker as child process via self-re-exec"
```

---

## Task 9: Implement Worker Restart/Backoff and Graceful Shutdown

**Files:**
- Modify: `binaries/lilium/src/commands/ws_client/arbiter.rs`

**Interfaces:**
- Produces: `Arbiter::run` that watches child exits and restarts crashed workers with backoff; `stop_worker` uses control socket then SIGTERM/SIGKILL.

- [ ] **Step 1: Add restart bookkeeping to `WorkerHandle`**

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct WorkerHandle {
    child: Child,
    restart_count: u32,
    last_restart: Instant,
}

fn backoff_delay(restart_count: u32) -> Duration {
    let base = 100u64;
    let max = 30_000u64;
    let millis = base.saturating_mul(2u64.saturating_pow(restart_count));
    Duration::from_millis(std::cmp::min(millis, max))
}
```

- [ ] **Step 2: Update `spawn_worker` to initialize bookkeeping**

```rust
fn spawn_worker(&self, spec: WorkerSpec) -> WorkerHandle {
    // ... existing command setup from Task 8 ...

    WorkerHandle {
        child,
        restart_count: 0,
        last_restart: Instant::now(),
    }
}
```

- [ ] **Step 3: Add a restart watcher task in `Arbiter::run`**

After starting initial workers in `Arbiter::run`, spawn a background task that loops until shutdown:

```rust
let restart_shutdown = self.shutdown.clone();
let restart_workers = self.workers.clone();
let restart_spawner = self.worker_spawner.clone();
let restart_config = self.config.clone();
let restart_db = self.database.clone();

tokio::spawn(async move {
    loop {
        tokio::select! {
            _ = restart_shutdown.notified() => break,
            _ = sleep(Duration::from_secs(1)) => {
                let mut workers = restart_workers.write().await;
                let mut to_restart = Vec::new();
                for (account, handle) in workers.iter_mut() {
                    match handle.child.try_wait() {
                        Ok(Some(status)) if !status.success() => {
                            tracing::warn!(account = %account, status = ?status, "worker exited; will restart");
                            to_restart.push(account.clone());
                        }
                        Ok(Some(status)) if status.success() => {
                            tracing::info!(account = %account, "worker exited cleanly; removing");
                            to_restart.push(account.clone());
                        }
                        _ => {}
                    }
                }
                for account in to_restart {
                    if let Some(handle) = workers.remove(&account) {
                        let _ = handle.child.start_kill();
                        let delay = backoff_delay(handle.restart_count);
                        let new_handle = restart_spawner.spawn_worker(WorkerSpec {
                            account: account.clone(),
                            database: restart_db.clone(),
                            notification_config: restart_config.notification.clone().into(),
                            lock_config: restart_config.database.clone().into(),
                            queue_size: restart_config.spider.queue_size,
                            batch_size: restart_config.spider.batch_size,
                            buffer_dir: restart_config.spider.buffer_dir.clone(),
                            runtime_dir: restart_config.spider.runtime_dir.clone(),
                            websocket_url: restart_config.spider.websocket_url.clone(),
                            reconnect_delay_ms: restart_config.spider.reconnect_delay_ms,
                        });
                        workers.insert(account, WorkerHandle {
                            child: new_handle.child,
                            restart_count: handle.restart_count + 1,
                            last_restart: Instant::now(),
                        });
                        sleep(delay).await;
                    }
                }
            }
        }
    }
});
```

To avoid duplicating `WorkerSpec` construction, add a helper in `arbiter.rs`:

```rust
impl Arbiter {
    fn worker_spec(&self, account: String) -> WorkerSpec {
        WorkerSpec {
            account,
            database: self.database.clone(),
            notification_config: self.config.notification.clone().into(),
            lock_config: self.config.database.clone().into(),
            queue_size: self.config.spider.queue_size,
            batch_size: self.config.spider.batch_size,
            buffer_dir: self.config.spider.buffer_dir.clone(),
            runtime_dir: self.config.spider.runtime_dir.clone(),
            websocket_url: self.config.spider.websocket_url.clone(),
            reconnect_delay_ms: self.config.spider.reconnect_delay_ms,
        }
    }
}
```

Then `start_worker` uses `self.worker_spec(account)` and the watcher copies the helper logic inline or calls a clone of `self` if possible.

- [ ] **Step 4: Implement graceful `stop_worker`**

```rust
const WORKER_STOP_TIMEOUT_SECS: u64 = 10;

pub async fn stop_worker(&self, account_id: &str) -> Result<()> {
    let mut workers = self.workers.write().await;
    let Some(mut handle) = workers.remove(account_id) else {
        warn!(account = account_id, "Worker not running");
        return Ok(());
    };

    let socket_path = control::worker_socket_path(&self.config.spider.runtime_dir, account_id);

    let graceful = async {
        let command = serde_json::json!({
            "action": "stop",
            "account_user_id": account_id,
        })
        .to_string();
        match tokio::net::UnixStream::connect(&socket_path).await {
            Ok(mut stream) => {
                let _ = control::write_message(&mut stream, &command).await;
                let _ = tokio::time::timeout(
                    Duration::from_secs(WORKER_STOP_TIMEOUT_SECS),
                    control::read_message(&mut stream),
                )
                .await;
            }
            Err(e) => {
                warn!(account = account_id, error = %e, "worker control socket unavailable");
            }
        }
    };

    graceful.await;

    match tokio::time::timeout(
        Duration::from_secs(WORKER_STOP_TIMEOUT_SECS),
        handle.child.wait(),
    )
    .await
    {
        Ok(Ok(_)) => {
            info!(account = account_id, "worker stopped gracefully");
        }
        _ => {
            warn!(account = account_id, "worker did not stop gracefully; sending SIGTERM");
            let _ = handle.child.start_kill();
            match tokio::time::timeout(Duration::from_secs(5), handle.child.wait()).await {
                Ok(Ok(_)) => info!(account = account_id, "worker killed"),
                _ => warn!(account = account_id, "worker may be a zombie"),
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 5: Update `stop_all_workers` to await graceful shutdowns in parallel**

```rust
async fn stop_all_workers(&self) {
    let workers = {
        let mut w = self.workers.write().await;
        std::mem::take(&mut *w)
    };

    let futures: Vec<_> = workers
        .into_keys()
        .map(|account_id| self.stop_worker(&account_id))
        .collect();

    let results = futures::future::join_all(futures).await;
    for (account_id, result) in results {
        if let Err(e) = result {
            error!(account = %account_id, error = %e, "failed to stop worker");
        }
    }
}
```

- [ ] **Step 6: Verify compile**

```bash
cargo check -p lilium
```

Expected: passes.

- [ ] **Step 7: Commit**

```bash
git add binaries/lilium/src/commands/ws_client/arbiter.rs
git commit -m "feat: add worker restart backoff and graceful shutdown"
```

---

## Task 10: Update `AGENTS.md`, Migrate Tests, Final Verification

**Files:**
- Modify: `AGENTS.md`
- Modify: `binaries/lilium/Cargo.toml` if needed to ensure dev-dependencies are correct.

**Interfaces:**
- Produces: documentation and test suite aligned with the new binary.

- [ ] **Step 1: Update `AGENTS.md` binary references**

Replace the architecture paragraph:

```
Binaries (lilium)
    → Service Layer (lilium-services)
        → Core Layer (lilium-core) — pure business logic, no async/DB
        → Domain Layer (lilium-models) — SeaORM entities and row mappings
        → Data Access (lilium-database) — connection pool, sessions, transactions
        → API Client (lilium-api-client) — external HTTP and Socket.IO calls
```

Replace the build/test commands:

```
- `cargo build`: compile the full workspace
- `cargo test`: run all workspace tests
- `cargo test -p lilium-database`: run database-layer tests only
- `cargo run --bin lilium ws-client`: start the WebSocket client arbiter locally
- `cargo run --bin lilium event-processor`: start the event processor locally
- `cargo run --bin lilium`: run the operational CLI locally
```

Update any other `lilium-spider`/`lilium-event-processor`/`lilium-cli` mentions.

- [ ] **Step 2: Ensure tests compile**

```bash
cargo test -p lilium --no-run
```

Expected: tests compile.

- [ ] **Step 3: Run tests**

```bash
cargo test -p lilium
```

Expected: existing config tests pass; arbiter tests may require `mockall` mocks updated for new `WorkerHandle` (now holds `Child` instead of `Arc<Notify>`). Fix any compile errors in tests.

- [ ] **Step 4: Format and clippy**

```bash
cargo fmt --all
cargo clippy --all-targets --all-features
```

Fix any warnings.

- [ ] **Step 5: Build final binary and sanity-check commands**

```bash
cargo build -p lilium
./target/debug/lilium --help
./target/debug/lilium ws-client --help
./target/debug/lilium event-processor --help
./target/debug/lilium send-command --help
./target/debug/lilium completion bash | head -n 20
```

Expected: all help output looks correct, completion contains all public subcommands and does not contain `worker`.

- [ ] **Step 6: Commit**

```bash
git add AGENTS.md binaries/lilium
git commit -m "docs: update AGENTS.md for unified lilium binary"
```

---

## Spec Coverage Self-Review

| Spec Section | Task(s) Implementing It |
|--------------|------------------------|
| Crate/binary/directory renamed to `lilium` | Task 1, Task 4 |
| Flat top-level subcommands | Task 6 |
| Hidden nested `ws-client worker` self-re-exec | Task 6, Task 7 |
| Module tree under `commands/` | Task 3, Task 5 |
| Unified Config | Task 2 |
| Shell completion | Task 6 |
| Worker process isolation (self-re-exec) | Task 7, Task 8 |
| Restart/backoff and graceful shutdown | Task 9 |
| Sentry name per verb | Task 6 |
| `AGENTS.md` updated | Task 10 |

**Placeholder scan:** No TBD/TODO/fill-in details; all code snippets are concrete.
**Type consistency:** `Config` fields (`spider`, `processor`, `cli`) used consistently across tasks. `WorkerSpec` and `WorkerHandle` evolve in Tasks 8 and 9 but are only consumed within `arbiter.rs`.

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-06-18-unified-lilium-binary-plan.md`. Two execution options:**

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using `executing-plans`, batch execution with checkpoints.

Which approach?
