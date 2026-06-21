// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 core/sync.py
// Ports RoomSyncer and MemberSyncer orchestration. Per the migration SOP,
// orchestration lives in lilium-services (not lilium-core, which is reserved
// for pure calculations). DB CRUD is delegated to the `room` and `room_member`
// services; API transport to `DzmmApi`.
use crate::{room, room_member, user};
use lilium_api_client::http::DzmmApi;
use sea_orm::ConnectionTrait;
use serde_json::Value;
use std::time::Duration;
use tracing::instrument;
use uuid::Uuid;

/// Map an `anyhow` API error into a `LiliumError` service error.
fn api_err<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> crate::Result<T> {
    result.map_err(|e| lilium_common::LiliumError::service("SYNC_API_ERROR", e.to_string()))
}

/// Statistics returned by [`RoomSyncer::sync_rooms`]. Mirrors Python
/// `RoomSyncStats`. `user_chats` carries the raw API chat list so callers can
/// derive the synced room-id set for gap detection.
#[derive(Debug, Clone, Default)]
pub struct RoomSyncStats {
    pub new_rooms: usize,
    pub updated_rooms: usize,
    pub marked_inactive: i64,
    pub total_active: usize,
    pub user_chats: Vec<Value>,
}

/// Syncs the room list from the DZMM API into the `rooms` table.
/// Mirrors Python `core.sync.RoomSyncer`.
pub struct RoomSyncer<'a> {
    auth: &'a DzmmApi,
    account_user_id: Option<Uuid>,
}

impl<'a> RoomSyncer<'a> {
    pub fn new(auth: &'a DzmmApi, account_user_id: Option<Uuid>) -> Self {
        Self {
            auth,
            account_user_id,
        }
    }

    #[instrument(level = "debug", skip(self, db), fields(account_user_id = self.account_user_id.is_some()))]
    pub async fn sync_rooms<C>(&self, db: &C) -> crate::Result<RoomSyncStats>
    where
        C: ConnectionTrait,
    {
        tracing::info!("Fetching room list from DZMM API");
        let mut stats = RoomSyncStats::default();

        let user_chats = api_err(self.auth.fetch_user_chats().await)?;
        if user_chats.is_empty() {
            tracing::warn!("No rooms found in API response");
            return Ok(stats);
        }
        stats.user_chats = user_chats.clone();

        let mut active_room_ids: Vec<Uuid> = Vec::new();
        for chat in &user_chats {
            let chat_data = chat.get("data").unwrap_or(&Value::Null);
            let room_id_str = chat
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| chat_data.get("chatroomId").and_then(|v| v.as_str()));
            let Some(room_id_str) = room_id_str else {
                continue;
            };
            let Ok(room_id) = Uuid::parse_str(room_id_str) else {
                tracing::warn!(room_id = %room_id_str, "chat has non-UUID id, skipping");
                continue;
            };
            active_room_ids.push(room_id);

            if !chat_data.is_object() {
                tracing::warn!(room_id = %room_id, "chat has no 'data' object, skipping upsert");
                continue;
            }

            let is_new = room::upsert_room_from_dict(db, chat_data, self.account_user_id).await?;
            if is_new {
                stats.new_rooms += 1;
            } else {
                stats.updated_rooms += 1;
            }
        }

        stats.marked_inactive =
            room::mark_inactive_rooms(db, &active_room_ids, self.account_user_id).await?;
        stats.total_active = active_room_ids.len();

        tracing::info!(
            "Room sync complete: {} new, {} updated, {} marked inactive, {} total active",
            stats.new_rooms,
            stats.updated_rooms,
            stats.marked_inactive,
            stats.total_active
        );
        Ok(stats)
    }
}

/// Configuration for [`MemberSyncer`]. Mirrors Python `MemberSyncConfig`.
#[derive(Debug, Clone)]
pub struct MemberSyncConfig {
    /// Specific room to sync; `None` syncs all rooms visible to the account.
    pub room_id: Option<Uuid>,
    /// Force re-sync by clearing existing members first.
    pub force: bool,
    /// Delay between API calls when syncing all rooms.
    pub rate_limit_delay: Duration,
}

impl Default for MemberSyncConfig {
    fn default() -> Self {
        Self {
            room_id: None,
            force: false,
            rate_limit_delay: Duration::from_millis(500),
        }
    }
}

/// Statistics returned by [`MemberSyncer::sync_members`]. Mirrors Python
/// `MemberSyncStats`.
#[derive(Debug, Clone, Default)]
pub struct MemberSyncStats {
    pub rooms_processed: usize,
    pub rooms_skipped: usize,
    pub members_new: usize,
    pub members_updated: usize,
    pub members_left: usize,
    pub users_synced: usize,
    pub errors: usize,
}

/// Syncs room members from the DZMM API into `room_members` (+ `users`).
/// Mirrors Python `core.sync.MemberSyncer`.
pub struct MemberSyncer<'a> {
    auth: &'a DzmmApi,
    config: MemberSyncConfig,
}

impl<'a> MemberSyncer<'a> {
    pub fn new(auth: &'a DzmmApi, config: MemberSyncConfig) -> Self {
        Self { auth, config }
    }

    #[instrument(level = "debug", skip(self, db), fields(room_id = ?self.config.room_id, force = self.config.force))]
    pub async fn sync_members<C>(&self, db: &C) -> crate::Result<MemberSyncStats>
    where
        C: ConnectionTrait,
    {
        if let Some(room_id) = &self.config.room_id {
            let mut stats = MemberSyncStats::default();
            self.sync_room_members(db, *room_id, &mut stats).await?;
            Ok(stats)
        } else {
            self.sync_all_rooms(db).await
        }
    }

    async fn sync_all_rooms<C>(&self, db: &C) -> crate::Result<MemberSyncStats>
    where
        C: ConnectionTrait,
    {
        let mut stats = MemberSyncStats::default();
        let user_chats = api_err(self.auth.fetch_user_chats().await)?;
        for chat in &user_chats {
            let chat_data = chat.get("data").unwrap_or(&Value::Null);
            let room_id_str = chat
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| chat_data.get("chatroomId").and_then(|v| v.as_str()));
            let Some(room_id_str) = room_id_str else {
                continue;
            };
            let Ok(room_id) = Uuid::parse_str(room_id_str) else {
                tracing::warn!(room_id = %room_id_str, "chat has non-UUID id, skipping member sync");
                continue;
            };

            self.sync_room_members(db, room_id, &mut stats).await?;
            tokio::time::sleep(self.config.rate_limit_delay).await;
        }
        Ok(stats)
    }

    async fn sync_room_members<C>(
        &self,
        db: &C,
        room_id: Uuid,
        stats: &mut MemberSyncStats,
    ) -> crate::Result<()>
    where
        C: ConnectionTrait,
    {
        if self.config.force {
            room_member::clear_room_members(db, room_id).await?;
        }

        let members = api_err(
            self.auth
                .fetch_all_room_members(&room_id.to_string(), Some(500))
                .await,
        )?;
        let result = room_member::batch_upsert_members(db, room_id, &members).await?;

        stats.rooms_processed += 1;
        stats.members_new += result.new;
        stats.members_updated += result.updated;
        stats.members_left += result.left;

        // Fetch full profiles for newly-seen users (1-hour cache).
        if !result.new_user_ids.is_empty() {
            let pairs: Vec<(Uuid, Uuid)> = result
                .new_user_ids
                .iter()
                .map(|uid| (*uid, room_id))
                .collect();
            let fetched =
                api_err(user::batch_fetch_and_update_with_auth(db, self.auth, &pairs, 1).await)?;
            stats.users_synced += (fetched.new_count + fetched.updated_count) as usize;
        }
        Ok(())
    }
}
