// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/sync_members.py
//
// Ports the argparse CLI to clap. Auth clients are selected dynamically per
// room from room.account_ids (first enabled account, else any). Mirrors the
// Python single-room and all-rooms paths.
use anyhow::{Context, Result};
use clap::Args;
use lilium_api_client::http::DzmmApi;
use lilium_database::Database;
use lilium_services::{
    account::{self, AuthClientFactory},
    room, sync,
};
use uuid::Uuid;

#[derive(Args)]
pub struct SyncMembersArgs {
    /// Specific room ID to sync (omit to sync all active group rooms)
    pub room_id: Option<String>,
    /// Force re-sync by clearing existing members
    #[arg(short, long)]
    pub force: bool,
}

impl SyncMembersArgs {
    /// Execute the sync-members subcommand. Returns a process exit code.
    pub async fn run(self, db: &Database) -> Result<u8> {
        tracing::info!("🔐 Auth clients selected dynamically per room from database");
        let auth_clients = AuthClientFactory::new(db.clone());

        let room_id = match self.room_id {
            Some(s) => Some(Uuid::parse_str(&s).with_context(|| format!("invalid room id: {s}"))?),
            None => None,
        };

        match room_id {
            Some(room_id) => {
                tracing::info!("Syncing specific room: {}", room_id);
                let stats = sync_single_room(db, &auth_clients, room_id, self.force).await?;
                print_stats(&stats);
                tracing::info!("✓ Sync complete");
                Ok(0)
            }
            None => {
                let filters = room::RoomFilters {
                    chat_type: Some("group".to_string()),
                    is_active: Some(true),
                    ..Default::default()
                };
                let rooms = room::get_all_rooms(db.orm(), Some(&filters))
                    .await
                    .context("list rooms")?;
                if rooms.is_empty() {
                    tracing::warn!("⚠️  No active group rooms found. Run sync-rooms first.");
                    return Ok(1);
                }
                tracing::info!("📋 Found {} active rooms to sync", rooms.len());

                let mut total = sync::MemberSyncStats::default();
                let mut failed_rooms: Vec<String> = Vec::new();
                let count = rooms.len();
                for (idx, room) in rooms.iter().enumerate() {
                    let i = idx + 1;
                    tracing::info!("\n[{i}/{count}] 🏠 Room: {} ({})", room.title, room.room_id);
                    match sync_single_room(db, &auth_clients, room.room_id, self.force).await {
                        Ok(stats) => {
                            total.rooms_processed += stats.rooms_processed;
                            total.members_new += stats.members_new;
                            total.members_updated += stats.members_updated;
                            total.members_left += stats.members_left;
                            total.users_synced += stats.users_synced;
                            total.errors += stats.errors;
                            tracing::info!(
                                "   ✅ Synced: {} new, {} updated",
                                stats.members_new,
                                stats.members_updated
                            );
                        }
                        Err(e) => {
                            tracing::error!("   ❌ Failed: {e}");
                            failed_rooms.push(room.room_id.to_string());
                            total.errors += 1;
                        }
                    }
                }

                tracing::info!("\n{}", "=".repeat(60));
                tracing::info!("Aggregate Statistics (All Rooms)");
                print_stats(&total);
                if failed_rooms.is_empty() {
                    tracing::info!("✓ All rooms synced successfully");
                    Ok(0)
                } else {
                    tracing::warn!("⚠️  Failed rooms: {}", failed_rooms.join(", "));
                    Ok(1)
                }
            }
        }
    }
}

async fn sync_single_room(
    db: &Database,
    auth_clients: &AuthClientFactory,
    room_id: Uuid,
    force: bool,
) -> Result<sync::MemberSyncStats> {
    let auth = auth_for_room(db, auth_clients, room_id).await?;
    let config = sync::MemberSyncConfig {
        room_id: Some(room_id),
        force,
        ..Default::default()
    };
    let syncer = sync::MemberSyncer::new(&auth, config);
    syncer
        .sync_members(db.orm())
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))
}

/// Select an enabled account with access to the room (first enabled, else any).
/// Mirrors Python `get_auth_for_room`.
async fn auth_for_room(
    db: &Database,
    auth_clients: &AuthClientFactory,
    room_id: Uuid,
) -> Result<DzmmApi> {
    let conn = db.orm();
    let room = room::get_by_id(conn, room_id)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .with_context(|| {
            format!(
                "No accounts have access to room {room_id}. Run sync-rooms first to populate account_ids."
            )
        })?;
    if room.account_ids.is_empty() {
        anyhow::bail!(
            "No accounts have access to room {room_id}. Run sync-rooms first to populate account_ids."
        );
    }
    for uid in &room.account_ids {
        if let Some(account) = account::get_account(conn, *uid)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            && account.is_enabled
        {
            tracing::info!("🔑 Using account {uid} for room {room_id}");
            return Ok(auth_clients.create(account)?);
        }
    }
    let uid = room.account_ids[0];
    tracing::warn!("⚠️  No enabled accounts for room {room_id}, using {uid}");
    let account = account::get_account(conn, uid)
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?
        .context("account missing")?;
    Ok(auth_clients.create(account)?)
}

fn print_stats(stats: &sync::MemberSyncStats) {
    tracing::info!("{}", "=".repeat(60));
    tracing::info!("Member Synchronization Complete");
    tracing::info!("{}", "=".repeat(60));
    tracing::info!("Rooms processed: {}", stats.rooms_processed);
    tracing::info!("Rooms skipped: {}", stats.rooms_skipped);
    tracing::info!("New members: {}", stats.members_new);
    tracing::info!("Updated members: {}", stats.members_updated);
    tracing::info!("Left members: {}", stats.members_left);
    tracing::info!("Users synced: {}", stats.users_synced);
    tracing::info!("Errors: {}", stats.errors);
    tracing::info!("{}", "=".repeat(60));
}
