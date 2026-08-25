// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/sync_rooms.py
//
// Ports the argparse CLI to clap. `sync_once` is the single complete room-sync
// iteration; one-shot mode calls it once and poll mode loops over it. Each
// successful iteration selects current rooms that still need backfill, syncs
// members, backfills history, and queues `system:reconnect` commands for
// affected accounts.
use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::Args;
use lilium_api_client::http::DzmmApi;
use lilium_database::Database;
use lilium_services::{
    account::{self, AuthClientFactory},
    history, outgoing_command as cmd_service, room, sync,
};
use sea_orm::ConnectionTrait;
use tokio::signal;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

#[derive(Args)]
pub struct SyncRoomsArgs {
    /// Account user_id to use for sync (default: sync all enabled accounts)
    #[arg(short, long)]
    pub account: Option<String>,
    /// List all available accounts and exit
    #[arg(short, long = "list-accounts")]
    pub list_accounts: bool,
    /// Run in polling mode, syncing rooms periodically and processing rooms needing backfill
    #[arg(short, long)]
    pub poll: bool,
    /// Interval between syncs in polling mode (minutes)
    #[arg(long = "poll-interval", default_value_t = 5)]
    pub poll_interval: u64,
}

impl SyncRoomsArgs {
    /// Execute the sync-rooms subcommand. Returns a process exit code.
    pub async fn run(self, db: &Database) -> Result<u8> {
        let auth_clients = AuthClientFactory::new(db.clone());
        let account = match self.account {
            Some(s) => Some(Uuid::parse_str(&s).context(format!("invalid account user id: {s}"))?),
            None => None,
        };
        if self.list_accounts {
            list_accounts(db).await?;
            return Ok(0);
        }
        if self.poll {
            return poll_mode(db, &auth_clients, account, self.poll_interval).await;
        }
        if sync_once(db, &auth_clients, account).await? {
            Ok(0)
        } else {
            Ok(1)
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
    account_id: Option<Uuid>,
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

/// Fetch and persist the room list for one iteration.
/// Returns the set of room IDs visible to the selected account(s), or `None`
/// when no account can be selected. The complete iteration, including selecting
/// and processing rooms needing backfill, is implemented by [`sync_once`].
async fn sync_room_ids_once(
    db: &Database,
    auth_clients: &AuthClientFactory,
    account_id: Option<Uuid>,
) -> Result<Option<HashSet<Uuid>>> {
    let accounts = match get_accounts(db, account_id).await? {
        Some(a) => a,
        None => return Ok(None),
    };

    tracing::info!("🔄 Syncing {} account(s)...", accounts.len());
    let mut all_room_ids: HashSet<Uuid> = HashSet::new();
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

        let auth = match auth_clients.create(account.clone()) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("   ❌ Failed to build auth client for {user_id}: {e}");
                failed_accounts.push(user_id.to_string());
                continue;
            }
        };
        let syncer = sync::RoomSyncer::new(&auth, Some(*user_id));
        match syncer.sync_rooms(db.orm()).await {
            Ok(stats) => {
                for chat in &stats.user_chats {
                    let room_id_str = chat.get("id").and_then(|v| v.as_str()).or_else(|| {
                        chat.get("data")
                            .and_then(|d| d.get("chatroomId"))
                            .and_then(|v| v.as_str())
                    });
                    if let Some(rid) = room_id_str
                        && let Ok(rid) = Uuid::parse_str(rid)
                    {
                        all_room_ids.insert(rid);
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
                failed_accounts.push(user_id.to_string());
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

/// Execute one complete room-sync iteration for either one-shot or poll mode.
/// The API result identifies rooms visible in this iteration; the database
/// `history_complete` flag is the durable work queue for member sync/backfill.
/// Returns `false` when no room set could be produced; callers decide whether
/// that should terminate or retry.
async fn sync_once(
    db: &Database,
    auth_clients: &AuthClientFactory,
    account_id: Option<Uuid>,
) -> Result<bool> {
    let Some(current_room_ids) = sync_room_ids_once(db, auth_clients, account_id).await? else {
        return Ok(false);
    };
    let rooms_needing_backfill =
        room::get_rooms_needing_backfill(db.orm(), &current_room_ids, account_id)
            .await
            .context("select rooms needing backfill")?;
    let room_ids: HashSet<Uuid> = rooms_needing_backfill
        .into_iter()
        .map(|room| room.room_id)
        .collect();
    process_rooms_needing_backfill(db, auth_clients, &room_ids).await;
    Ok(true)
}

/// Process rooms selected from the durable backfill queue: sync members,
/// backfill history, and queue reconnect commands for affected accounts.
/// Reconnect is queued only once per affected account, after all rooms in this
/// iteration have been processed.
async fn process_rooms_needing_backfill(
    db: &Database,
    auth_clients: &AuthClientFactory,
    room_ids: &HashSet<Uuid>,
) {
    if room_ids.is_empty() {
        return;
    }
    tracing::info!(
        "\n📚 Processing {} room(s) needing backfill...",
        room_ids.len()
    );
    let mut accounts_to_reconnect: HashSet<Uuid> = HashSet::new();

    for room_id in room_ids {
        tracing::info!("\n📦 Room needing backfill: {room_id}");
        if let Err(e) = sync_room_members(db, auth_clients, *room_id).await {
            tracing::error!("   ❌ Failed member sync for {room_id}: {e}");
        }
        tracing::info!("   📜 Backfilling history for {room_id}...");
        if let Err(e) =
            history::HistoryFetcher::backfill_to_start(db.orm(), auth_clients, *room_id).await
        {
            tracing::error!("   ❌ Failed to backfill history for {room_id}: {e}");
        } else {
            tracing::info!("   ✅ History backfill complete for {room_id}");
        }
        if let Ok(Some(room)) = room::get_by_id(db.orm(), *room_id).await {
            for uid in room.account_ids {
                accounts_to_reconnect.insert(uid);
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
                *account_user_id,
                "system:reconnect",
                serde_json::json!({"reason": format!("rooms needing backfill: {}", room_ids.len())}),
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

async fn sync_room_members(
    db: &Database,
    auth_clients: &AuthClientFactory,
    room_id: Uuid,
) -> Result<()> {
    let conn = db.orm();
    let room = room::get_by_id(conn, room_id)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .with_context(|| format!("No accounts have access to room {room_id}"))?;
    if room.account_ids.is_empty() {
        anyhow::bail!("No accounts have access to room {room_id}, skipping member sync");
    }
    let auth = auth_for_room(conn, auth_clients, &room).await?;
    let config = sync::MemberSyncConfig {
        room_id: Some(room_id),
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
    auth_clients: &AuthClientFactory,
    room: &lilium_models::dzmm::room::Model,
) -> Result<DzmmApi> {
    for uid in &room.account_ids {
        if let Some(account) = account::get_account(conn, *uid)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            && account.is_enabled
        {
            return Ok(auth_clients.create(account)?);
        }
    }
    let uid = room.account_ids.first().context("no accounts")?;
    let account = account::get_account(conn, *uid)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .context("account missing")?;
    Ok(auth_clients.create(account)?)
}

async fn poll_mode(
    db: &Database,
    auth_clients: &AuthClientFactory,
    account_id: Option<Uuid>,
    poll_interval_minutes: u64,
) -> Result<u8> {
    let poll_interval = Duration::from_secs(poll_interval_minutes * 60);

    tracing::info!("{}", "=".repeat(60));
    tracing::info!("🔄 Starting room sync in POLLING mode");
    tracing::info!("   Poll interval: {poll_interval_minutes} minute(s)");
    tracing::info!("   Press Ctrl+C to stop");
    tracing::info!("{}", "=".repeat(60));

    let mut poll_count = 0u32;
    loop {
        poll_count += 1;
        tracing::info!("\n{}", "=".repeat(60));
        tracing::info!("🔄 Poll #{poll_count}");
        tracing::info!("{}", "=".repeat(60));

        match sync_once(db, auth_clients, account_id).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::warn!("⚠️  Sync returned no room set, will retry next cycle");
            }
            Err(error) => {
                tracing::warn!(error = %error, "⚠️  Sync failed, will retry next cycle");
            }
        }

        tracing::info!("\n⏱️  Waiting {poll_interval_minutes} minute(s) until next sync...");
        tokio::select! {
            _ = signal::ctrl_c() => break,
            _ = sleep(poll_interval) => {}
        }
    }

    tracing::info!("\n{}", "=".repeat(60));
    tracing::info!("🛑 Polling mode stopped gracefully");
    tracing::info!("{}", "=".repeat(60));
    Ok(0)
}
