// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/sync_rooms.py
//
// Ports the argparse CLI to clap. One-shot and poll modes mirror the Python.
// Poll mode diffs the synced room-id set, and for new rooms: syncs members,
// backfills history, and queues `system:reconnect` commands for affected
// accounts.
use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::Args;
use lilium_api_client::http::DzmmApi;
use lilium_database::Database;
use lilium_services::{account, history, outgoing_command as cmd_service, room, sync};
use sea_orm::ConnectionTrait;
use tokio::signal;
use tokio::time::{Duration, sleep};

#[derive(Args)]
pub struct SyncRoomsArgs {
    /// Account user_id to use for sync (default: sync all enabled accounts)
    #[arg(short, long)]
    pub account: Option<String>,
    /// List all available accounts and exit
    #[arg(short, long = "list-accounts")]
    pub list_accounts: bool,
    /// Run in polling mode, syncing rooms periodically and processing new rooms
    #[arg(short, long)]
    pub poll: bool,
    /// Interval between syncs in polling mode (minutes)
    #[arg(long = "poll-interval", default_value_t = 5)]
    pub poll_interval: u64,
}

impl SyncRoomsArgs {
    /// Execute the sync-rooms subcommand. Returns a process exit code.
    pub async fn run(self, db: &Database) -> Result<u8> {
        if self.list_accounts {
            list_accounts(db).await?;
            return Ok(0);
        }
        if self.poll {
            return poll_mode(db, self.account, self.poll_interval).await;
        }
        match sync_once(db, self.account).await? {
            Some(_) => Ok(0),
            None => Ok(1),
        }
    }
}

async fn list_accounts(db: &Database) -> Result<()> {
    let accounts = account::list_accounts(db.orm(), false)
        .await
        .context("list accounts")?;
    if accounts.is_empty() {
        tracing::error!("❌ No accounts found in database");
        return Ok(());
    }
    tracing::info!("{}", "=".repeat(60));
    tracing::info!("Available Accounts");
    tracing::info!("{}", "=".repeat(60));
    for account in accounts {
        let status = if account.is_enabled {
            "✅ Enabled"
        } else {
            "❌ Disabled"
        };
        let full_name = account
            .user_profile
            .get("fullName")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let email = account
            .user_profile
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let auth = if account.email.is_some() {
            "Email/Password"
        } else {
            "QR Code"
        };
        tracing::info!("\nUser ID: {}", account.user_id);
        tracing::info!("  Status: {status}");
        tracing::info!("  Name:   {full_name}");
        tracing::info!("  Email:  {email}");
        tracing::info!("  Auth:   {auth}");
    }
    tracing::info!("{}", "=".repeat(60));
    Ok(())
}

async fn get_accounts(
    db: &Database,
    account_id: Option<&str>,
) -> Result<Option<Vec<lilium_models::dzmm::account::Model>>> {
    if let Some(account_id) = account_id {
        let account = account::get_account(db.orm(), account_id)
            .await
            .context("get account")?;
        let Some(account) = account else {
            tracing::error!("❌ Account '{account_id}' not found in database");
            return Ok(None);
        };
        if !account.is_enabled {
            tracing::warn!("⚠️  Account '{account_id}' is disabled but will be used");
        }
        return Ok(Some(vec![account]));
    }
    let accounts = account::list_accounts(db.orm(), true)
        .await
        .context("list enabled accounts")?;
    if accounts.is_empty() {
        tracing::error!("❌ No enabled accounts found in database");
        return Ok(None);
    }
    Ok(Some(accounts))
}

/// Perform a single room sync iteration across the selected account(s).
/// Returns the set of room IDs synced, or `None` on failure. Mirrors Python
/// `sync_once`.
async fn sync_once(db: &Database, account_id: Option<String>) -> Result<Option<HashSet<String>>> {
    let accounts = match get_accounts(db, account_id.as_deref()).await? {
        Some(a) => a,
        None => return Ok(None),
    };

    tracing::info!("🔄 Syncing {} account(s)...", accounts.len());
    let mut all_room_ids: HashSet<String> = HashSet::new();
    let mut failed_accounts: Vec<String> = Vec::new();
    let mut total_new: usize = 0;
    let mut total_updated: usize = 0;
    let mut total_active: usize = 0;
    let mut total_inactive: i64 = 0;
    let count = accounts.len();

    for (idx, account) in accounts.iter().enumerate() {
        let i = idx + 1;
        let user_id = &account.user_id;
        tracing::info!("\n[{i}/{count}] 🔑 Account: {user_id}");
        let full_name = account
            .user_profile
            .get("fullName")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        let email = account
            .user_profile
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A");
        tracing::info!("   Name:  {full_name}");
        tracing::info!("   Email: {email}");

        let auth = match account::create_auth_client(account.clone()) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("   ❌ Failed to build auth client for {user_id}: {e}");
                failed_accounts.push(user_id.clone());
                continue;
            }
        };
        let syncer = sync::RoomSyncer::new(&auth, Some(user_id.as_str()));
        match syncer.sync_rooms(db.orm()).await {
            Ok(stats) => {
                for chat in &stats.user_chats {
                    let room_id = chat.get("id").and_then(|v| v.as_str()).or_else(|| {
                        chat.get("data")
                            .and_then(|d| d.get("chatroomId"))
                            .and_then(|v| v.as_str())
                    });
                    if let Some(rid) = room_id {
                        all_room_ids.insert(rid.to_owned());
                    }
                }
                total_new += stats.new_rooms;
                total_updated += stats.updated_rooms;
                total_active += stats.total_active;
                total_inactive += stats.marked_inactive;
                tracing::info!(
                    "   ✅ Synced: {} new, {} updated, {} total active",
                    stats.new_rooms,
                    stats.updated_rooms,
                    stats.total_active
                );
            }
            Err(e) => {
                tracing::error!("   ❌ Failed to sync account {user_id}: {e}");
                failed_accounts.push(user_id.clone());
            }
        }
    }

    tracing::info!("\n{}", "=".repeat(60));
    if count > 1 {
        tracing::info!("Aggregate Statistics (All Accounts)");
        tracing::info!("{}", "=".repeat(60));
        tracing::info!(
            "Accounts synced:  {}/{}",
            count - failed_accounts.len(),
            count
        );
        tracing::info!("New rooms:        {total_new}");
        tracing::info!("Updated rooms:    {total_updated}");
        tracing::info!("Total active:     {total_active}");
        if total_inactive > 0 {
            tracing::info!("Marked inactive:  {total_inactive} (user quit or removed)");
        }
        tracing::info!("{}", "=".repeat(60));
    }
    if failed_accounts.is_empty() {
        tracing::info!("✓ All accounts synced successfully");
    } else {
        tracing::warn!("⚠️  Failed accounts: {}", failed_accounts.join(", "));
    }
    Ok(Some(all_room_ids))
}

/// Process newly detected rooms: sync members, backfill history, and queue
/// reconnect commands for affected accounts. Mirrors Python `process_new_rooms`.
async fn process_new_rooms(db: &Database, new_room_ids: &HashSet<String>) {
    if new_room_ids.is_empty() {
        return;
    }
    tracing::info!("\n🆕 Processing {} new room(s)...", new_room_ids.len());
    let mut accounts_to_reconnect: HashSet<String> = HashSet::new();

    for room_id in new_room_ids {
        tracing::info!("\n📦 New room: {room_id}");
        if let Err(e) = sync_room_members(db, room_id).await {
            tracing::error!("   ❌ Failed member sync for {room_id}: {e}");
        }
        tracing::info!("   📜 Backfilling history for {room_id}...");
        if let Err(e) = history::HistoryFetcher::backfill_to_start(db.orm(), room_id).await {
            tracing::error!("   ❌ Failed to backfill history for {room_id}: {e}");
        } else {
            tracing::info!("   ✅ History backfill complete for {room_id}");
        }
        if let Ok(Some(room)) = room::get_by_id(db.orm(), room_id).await {
            for acc in room.account_ids {
                accounts_to_reconnect.insert(acc);
            }
        }
    }

    if !accounts_to_reconnect.is_empty() {
        tracing::info!(
            "\n📡 Triggering reconnect for {} account(s)...",
            accounts_to_reconnect.len()
        );
        for account_user_id in &accounts_to_reconnect {
            match cmd_service::create_command(
                db.orm(),
                account_user_id,
                "system:reconnect",
                serde_json::json!({"reason": format!("new rooms detected: {}", new_room_ids.len())}),
                false,
                Some(1),
            )
            .await
            {
                Ok(_) => tracing::info!("   ✅ Reconnect command queued for {account_user_id}"),
                Err(e) => tracing::error!("   ❌ Failed to queue reconnect for {account_user_id}: {e}"),
            }
        }
    }
}

async fn sync_room_members(db: &Database, room_id: &str) -> Result<()> {
    let conn = db.orm();
    let room = room::get_by_id(conn, room_id)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .with_context(|| format!("No accounts have access to room {room_id}"))?;
    if room.account_ids.is_empty() {
        anyhow::bail!("No accounts have access to room {room_id}, skipping member sync");
    }
    let auth = auth_for_room(conn, &room).await?;
    let config = sync::MemberSyncConfig {
        room_id: Some(room_id.to_string()),
        force: false,
        ..Default::default()
    };
    let syncer = sync::MemberSyncer::new(&auth, config);
    let stats = syncer
        .sync_members(conn)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    tracing::info!(
        "   ✅ Synced members: {} new, {} updated",
        stats.members_new,
        stats.members_updated
    );
    Ok(())
}

async fn auth_for_room(
    conn: &impl ConnectionTrait,
    room: &lilium_models::dzmm::room::Model,
) -> Result<DzmmApi> {
    for acc_id in &room.account_ids {
        if let Some(account) = account::get_account(conn, acc_id)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            && account.is_enabled
        {
            return Ok(account::create_auth_client(account)?);
        }
    }
    let acc_id = room.account_ids.first().context("no accounts")?.clone();
    let account = account::get_account(conn, &acc_id)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .context("account missing")?;
    Ok(account::create_auth_client(account)?)
}

async fn poll_mode(
    db: &Database,
    account_id: Option<String>,
    poll_interval_minutes: u64,
) -> Result<u8> {
    let poll_interval = Duration::from_secs(poll_interval_minutes * 60);
    let mut known_room_ids: HashSet<String>;
    let mut shutdown = false;

    tracing::info!("{}", "=".repeat(60));
    tracing::info!("🔄 Starting room sync in POLLING mode");
    tracing::info!("   Poll interval: {poll_interval_minutes} minute(s)");
    tracing::info!("   Press Ctrl+C to stop");
    tracing::info!("{}", "=".repeat(60));

    tracing::info!("\n📋 Initial sync...");
    let current = match sync_once(db, account_id.clone()).await? {
        Some(set) => set,
        None => {
            tracing::error!("❌ Initial sync failed");
            return Ok(1);
        }
    };
    known_room_ids = current.clone();
    tracing::info!("\n📊 Tracking {} room(s)", known_room_ids.len());

    let mut poll_count = 0u32;
    loop {
        poll_count += 1;
        tracing::info!("\n⏱️  Waiting {poll_interval_minutes} minute(s) until next sync...");

        // Wait in small increments for graceful shutdown.
        let mut waited = Duration::ZERO;
        while waited < poll_interval {
            tokio::select! {
                _ = signal::ctrl_c() => { shutdown = true; break; }
                _ = sleep(Duration::from_secs(1)) => { waited += Duration::from_secs(1); }
            }
        }
        if shutdown {
            break;
        }

        tracing::info!("\n{}", "=".repeat(60));
        tracing::info!("🔄 Poll #{poll_count}");
        tracing::info!("{}", "=".repeat(60));

        let current = match sync_once(db, account_id.clone()).await? {
            Some(set) => set,
            None => {
                tracing::warn!("⚠️  Sync failed, will retry next cycle");
                continue;
            }
        };

        let new_room_ids: HashSet<String> = current.difference(&known_room_ids).cloned().collect();
        if new_room_ids.is_empty() {
            tracing::info!("\n📊 No new rooms detected");
        } else {
            tracing::info!("\n🆕 Detected {} new room(s)!", new_room_ids.len());
            process_new_rooms(db, &new_room_ids).await;
            known_room_ids = current;
        }
        tracing::info!("📊 Total tracked rooms: {}", known_room_ids.len());
    }

    tracing::info!("\n{}", "=".repeat(60));
    tracing::info!("🛑 Polling mode stopped gracefully");
    tracing::info!("{}", "=".repeat(60));
    Ok(0)
}
