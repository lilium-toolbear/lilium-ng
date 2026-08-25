// Python parity source: dzmm_archive@18fdefbc0b6979178d7f1eb4ce0624ec4a60a2f2 core/history.py
// Ports HistoryFetcher orchestration: backfill_to_start, fetch_room_history,
// save_messages, ensure_room_info. Per the migration SOP, orchestration lives in
// lilium-services. Auth is selected per-room from room.account_ids.
//
// Divergence: the Python fetcher fires background media downloads for messages
// with attachments. This port saves messages and records progress but does not
// download media (the spider/media pipeline handles attachment enrichment
// separately). Media download for backfilled messages is a remaining gap.
use crate::{
    account::{self, AuthClientFactory},
    message, room,
};
use anyhow::Result as AnyhowResult;
use lilium_api_client::http::DzmmApi;
use lilium_models::dzmm::message::Message;
use lilium_models::dzmm::room::Model as Room;
use sea_orm::ConnectionTrait;
use std::path::PathBuf;
use tracing::instrument;
use uuid::Uuid;

/// Map an `anyhow` API error into a `LiliumError` service error.
fn api_err<T>(result: AnyhowResult<T>) -> crate::Result<T> {
    result.map_err(|e| lilium_common::LiliumError::service("HISTORY_API_ERROR", e.to_string()))
}

/// Fetches and stores historical room messages. Mirrors Python
/// `core.history.HistoryFetcher`.
pub struct HistoryFetcher {
    #[allow(dead_code)]
    data_path: PathBuf,
    batch_size: u64,
}

impl Default for HistoryFetcher {
    fn default() -> Self {
        Self::from_env()
    }
}

impl HistoryFetcher {
    pub fn from_env() -> Self {
        let data_path = std::env::var("DATA_PATH").unwrap_or_else(|_| "./data".to_owned());
        let batch_size = std::env::var("BATCH_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);
        Self {
            data_path: PathBuf::from(data_path),
            batch_size,
        }
    }

    pub fn with_batch_size(mut self, batch_size: u64) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Select an authenticated API client for a room from `room.account_ids`
    /// (first enabled account, else any). Mirrors Python `_get_auth_for_room`.
    pub async fn auth_for_room<C>(
        db: &C,
        auth_clients: &AuthClientFactory,
        room_id: Uuid,
    ) -> crate::Result<DzmmApi>
    where
        C: ConnectionTrait,
    {
        let room = room::get_by_id(db, room_id).await?.ok_or_else(|| {
            lilium_common::LiliumError::domain_service_with_code(
                "HISTORY_ROOM_NOT_FOUND",
                format!("Room {room_id} not found"),
            )
        })?;
        let account_ids = room.account_ids;
        if account_ids.is_empty() {
            return Err(lilium_common::LiliumError::domain_service_with_code(
                "HISTORY_NO_ACCOUNT_ACCESS",
                format!("No accounts have access to room {room_id}"),
            ));
        }

        for acc_uid in &account_ids {
            if let Some(account) = account::get_account(db, *acc_uid).await?
                && account.is_enabled
            {
                return auth_clients.create(account);
            }
        }
        // Fall back to any account with access.
        for acc_uid in &account_ids {
            if let Some(account) = account::get_account(db, *acc_uid).await? {
                return auth_clients.create(account);
            }
        }
        Err(lilium_common::LiliumError::domain_service_with_code(
            "HISTORY_NO_ACCOUNT_ACCESS",
            format!("No usable account for room {room_id}"),
        ))
    }

    /// Ensure a room record exists; fetch from API and upsert if missing.
    /// Mirrors Python `ensure_room_info`.
    #[instrument(level = "debug", skip(db, auth_clients), fields(room_id = %room_id))]
    pub async fn ensure_room_info<C>(
        db: &C,
        auth_clients: &AuthClientFactory,
        room_id: Uuid,
    ) -> crate::Result<Option<Room>>
    where
        C: ConnectionTrait,
    {
        if let Some(existing) = room::get_by_id(db, room_id).await? {
            return Ok(Some(existing));
        }
        let api = Self::auth_for_room(db, auth_clients, room_id).await?;
        let room_info = api_err(api.get_room_info(&room_id.to_string()).await)?;
        let Some(room_info) = room_info else {
            tracing::warn!(room_id = %room_id, "Could not fetch room info");
            return Ok(None);
        };
        let room_data = room_info
            .get("data")
            .filter(|v| v.is_object())
            .cloned()
            .unwrap_or(room_info.clone());
        // upsert_room_from_dict requires an id/chatroomId; normalize like Python.
        let mut normalized = room_data;
        if normalized.get("id").is_none()
            && normalized.get("chatroomId").is_none()
            && let Some(obj) = normalized.as_object_mut()
        {
            let rid = obj
                .get("roomId")
                .and_then(|v| v.as_str())
                .unwrap_or(&room_id.to_string())
                .to_owned();
            obj.insert("chatroomId".to_string(), serde_json::json!(rid));
        }
        room::upsert_room_from_dict(db, &normalized, None).await?;
        room::get_by_id(db, room_id).await
    }

    /// Save a batch of API message dicts. Returns `(new, existing)`.
    /// Mirrors Python `save_messages` (without background media download).
    #[instrument(level = "debug", skip(db, messages), fields(room_id = %room_id, count = messages.len()))]
    pub async fn save_messages<C>(
        db: &C,
        room_id: Uuid,
        messages: &[serde_json::Value],
    ) -> crate::Result<(usize, usize)>
    where
        C: ConnectionTrait,
    {
        let mut models: Vec<Message> = Vec::with_capacity(messages.len());
        for msg in messages {
            if let Some(model) = Message::from_api(msg, room_id) {
                models.push(model);
            } else {
                tracing::warn!(room_id = %room_id, "Skipping invalid message (no message_id)");
            }
        }
        if models.is_empty() {
            return Ok((0, 0));
        }
        let total = models.len();
        let inserted = message::batch_create_if_missing(db, &models).await?;
        let new = inserted.len();
        let existing = total - new;
        tracing::info!(
            room_id = %room_id,
            "Batch processed {} messages: {} new, {} skipped",
            total,
            new,
            existing
        );
        Ok((new, existing))
    }

    /// Backfill room history to the beginning, saving progress every 10 batches
    /// and marking `history_complete` on completion. Mirrors Python
    /// `backfill_to_start`.
    #[instrument(level = "debug", skip(db, auth_clients), fields(room_id = %room_id))]
    pub async fn backfill_to_start<C>(
        db: &C,
        auth_clients: &AuthClientFactory,
        room_id: Uuid,
    ) -> crate::Result<()>
    where
        C: ConnectionTrait,
    {
        let fetcher = Self::from_env();
        let room = room::get_by_id(db, room_id).await?;
        if let Some(ref room) = room
            && room.history_complete
        {
            tracing::info!(room_id = %room_id, "Room already has complete history, skipping");
            return Ok(());
        }
        tracing::info!(room_id = %room_id, "Starting backfill to beginning");

        let start_before = room
            .as_ref()
            .and_then(|r| r.backfill_until)
            .map(|dt| dt.to_rfc3339());
        if let Some(ref ts) = start_before {
            tracing::info!(room_id = %room_id, "Continuing backfill from {ts}");
        }

        let api = Self::auth_for_room(db, auth_clients, room_id).await?;
        Self::ensure_room_info(db, auth_clients, room_id).await?;

        let mut before = start_before;
        let mut batch_count: u32 = 0;
        let mut total_messages: usize = 0;
        let mut oldest_timestamp: Option<String> = None;

        loop {
            batch_count += 1;
            tracing::info!(room_id = %room_id, batch = batch_count, "Fetching batch");
            let messages = api_err(
                api.fetch_room_messages(
                    &room_id.to_string(),
                    before.as_deref(),
                    Some(fetcher.batch_size),
                )
                .await,
            )?;
            if messages.is_empty() {
                tracing::info!(room_id = %room_id, "No more messages. Reached beginning.");
                break;
            }

            let (new, _existing) = Self::save_messages(db, room_id, &messages).await?;
            total_messages += new;

            // Oldest message in batch drives the next `before` cursor.
            let oldest = messages
                .iter()
                .filter_map(|m| m.get("sent_at").and_then(|v| v.as_str()).map(str::to_owned))
                .min();
            oldest_timestamp = oldest.clone();
            before = oldest;
            tracing::info!(
                room_id = %room_id,
                batch = batch_count,
                fetched = messages.len(),
                new,
                total = total_messages,
                "Batch complete"
            );

            // Save progress every 10 batches.
            if batch_count.is_multiple_of(10)
                && let Some(ref ts) = oldest_timestamp
                && let Some(dt) = room::parse_datetime(Some(ts))
                && let Err(e) = room::update_backfill_progress(db, room_id, dt).await
            {
                tracing::warn!(room_id = %room_id, "Failed to update backfill_until: {e}");
            }
        }

        tracing::info!(room_id = %room_id, total = total_messages, "Backfill complete");
        if let Some(ref ts) = oldest_timestamp
            && let Some(dt) = room::parse_datetime(Some(ts))
        {
            let _ = room::update_backfill_progress(db, room_id, dt).await;
        }
        room::mark_history_complete(db, room_id).await?;
        tracing::info!(room_id = %room_id, "Marked history_complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lilium_test_fixtures::{FixtureProfile, test_uuid};

    #[tokio::test]
    async fn ensure_room_info_no_account_errors() {
        let test_db = lilium_test_fixtures::TestDb::acquire(FixtureProfile::RoomMember)
            .await
            .expect("acquire room db");
        let db = test_db.database().orm();
        let auth_clients = AuthClientFactory::new(test_db.database().clone());
        // Room doesn't exist and no accounts seeded → should error cleanly.
        let err =
            HistoryFetcher::auth_for_room(db, &auth_clients, test_uuid("nonexistent-room")).await;
        assert!(err.is_err());
    }
}
