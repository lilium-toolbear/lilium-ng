// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/send_command.py
//
// Ports the Python `click` group `cli` (commands: send, status, join-room,
// heartbeat, reconnect, list-pending, send-message, send-reply, leave-room,
// start-match, cancel-match, fetch-match-limit, edit-message, recall-message,
// delete-message, mark-read, send-image, send-voice) to a clap subcommand
// tree. JSON payloads are preserved verbatim from the Python source.
use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use lilium_api_client::http::DzmmApi;
use lilium_database::{
    NotificationConnection, NotificationDatabaseConfig, NotificationDatabaseConfig as NotifCfg,
};
use lilium_models::dzmm::outgoing_command::{self as outgoing_commands, status};
use lilium_services::{account, outgoing_command as cmd_service};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{Value, json};
use std::time::Duration;
use uuid::Uuid;

/// Terminal command statuses that end wait_for_result.
const TERMINAL_STATUSES: &[&str] = status::TERMINAL_STATUSES;

#[derive(Subcommand)]
pub enum SendCommand {
    /// Send a custom event with JSON data
    Send(SendArgs),
    /// Check status of a command by ID
    Status(StatusArgs),
    /// Join a room (shortcut for message:join-room)
    JoinRoom(RoomAccountArgs),
    /// Send a heartbeat (fire-and-forget)
    Heartbeat(AccountArgs),
    /// Trigger a hot-swap WebSocket reconnection (system:reconnect, fire-and-forget)
    Reconnect(ReconnectArgs),
    /// List pending commands in queue
    ListPending(ListPendingArgs),
    /// Send a text message to a room (message:send)
    SendMessage(SendMessageArgs),
    /// Send a reply to a specific message (message:send with reference)
    SendReply(SendReplyArgs),
    /// Leave a room (message:leave-room)
    LeaveRoom(RoomAccountArgs),
    /// Start random matching (match:start, fire-and-forget)
    StartMatch(StartMatchArgs),
    /// Cancel ongoing match (match:cancel, fire-and-forget)
    CancelMatch(AccountArgs),
    /// Fetch match limit info (match:fetch-limit, fire-and-forget)
    FetchMatchLimit(AccountArgs),
    /// Edit a message content (message:edit)
    EditMessage(EditMessageArgs),
    /// Recall a message (message:recall)
    RecallMessage(RoomMessageArgs),
    /// Delete a message (message:delete)
    DeleteMessage(RoomMessageArgs),
    /// Mark messages as read up to the specified message (message:read)
    MarkRead(RoomMessageArgs),
    /// Send an image message to a room (uploads via DZMM API first)
    SendImage(SendImageArgs),
    /// Send a voice message to a room (uploads via DZMM API first)
    SendVoice(SendVoiceArgs),
}

#[derive(Args)]
pub struct SendArgs {
    /// Socket.IO event name (e.g. 'message:send')
    pub event: String,
    /// JSON payload (e.g. '{"chatroomId": "abc123"}')
    pub data: String,
    #[arg(short, long, required = true)]
    pub account: String,
    /// Do not wait for the command result
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    /// Seconds to wait for result
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
    /// Fire-and-forget (no acknowledgment)
    #[arg(long = "no-ack")]
    pub no_ack: bool,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Command ID to check
    pub command_id: i32,
}

#[derive(Args)]
pub struct RoomAccountArgs {
    pub room_id: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct AccountArgs {
    #[arg(short, long, required = true)]
    pub account: String,
}

#[derive(Args)]
pub struct ReconnectArgs {
    #[arg(short, long, required = true)]
    pub account: String,
    /// Reason for reconnection
    #[arg(short, long, default_value = "cli request")]
    pub reason: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct ListPendingArgs {
    /// Filter by account user_id
    #[arg(short, long)]
    pub account: Option<String>,
    /// Max commands to show
    #[arg(short = 'n', long, default_value_t = 20)]
    pub limit: i64,
}

#[derive(Args)]
pub struct SendMessageArgs {
    #[arg(short, long, required = true)]
    pub room: String,
    #[arg(short, long, required = true)]
    pub message: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct SendReplyArgs {
    #[arg(short, long, required = true)]
    pub room: String,
    #[arg(short = 'm', long = "message-id", required = true)]
    pub message_id: String,
    #[arg(short, long, required = true)]
    pub sender: String,
    #[arg(short, long, required = true)]
    pub text: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct StartMatchArgs {
    #[arg(short, long, required = true)]
    pub account: String,
    /// Match type
    #[arg(short, long, default_value = "random")]
    pub match_type: String,
}

#[derive(Args)]
pub struct EditMessageArgs {
    pub room_id: String,
    pub message_id: String,
    pub new_content: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct RoomMessageArgs {
    pub room_id: String,
    pub message_id: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct SendImageArgs {
    #[arg(short, long = "room", required = true)]
    pub room_id: String,
    #[arg(short, long = "file", required = true)]
    pub image_file: String,
    /// Alt text for image
    #[arg(long, default_value = "")]
    pub alt: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

#[derive(Args)]
pub struct SendVoiceArgs {
    #[arg(short, long = "room", required = true)]
    pub room_id: String,
    #[arg(short, long = "file", required = true)]
    pub voice_file: String,
    #[arg(short, long, required = true)]
    pub account: String,
    #[arg(long = "no-wait")]
    pub no_wait: bool,
    #[arg(short, long, default_value_t = 30)]
    pub timeout: u64,
}

impl SendCommand {
    /// Execute the send-command subcommand. Returns a process exit code.
    pub async fn run(
        self,
        db: &impl ConnectionTrait,
        notification_config: NotificationDatabaseConfig,
    ) -> Result<u8> {
        match self {
            SendCommand::Send(a) => {
                let data: Value = serde_json::from_str(&a.data)
                    .with_context(|| format!("Invalid JSON data: {}", a.data))?;
                send(
                    db,
                    notification_config,
                    &a.account,
                    &a.event,
                    data,
                    !a.no_ack,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::Status(a) => {
                let cmd = cmd_service::get_command_result(db, a.command_id)
                    .await
                    .context("fetch command")?;
                let Some(cmd) = cmd else {
                    tracing::error!("Command {} not found", a.command_id);
                    return Ok(1);
                };
                print_command(&cmd);
                Ok(0)
            }
            SendCommand::JoinRoom(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:join-room",
                    json!({"chatroomId": a.room_id}),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::Heartbeat(a) => {
                let ts = now_millis();
                send(
                    db,
                    notification_config,
                    &a.account,
                    "heartbeat",
                    json!({"timestamp": ts}),
                    false,
                    false,
                    5,
                )
                .await
            }
            SendCommand::Reconnect(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "system:reconnect",
                    json!({"reason": a.reason}),
                    false,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::ListPending(a) => list_pending(db, a.account, a.limit).await,
            SendCommand::SendMessage(a) => {
                let message_id = Uuid::new_v4().to_string();
                let sent_at = now_iso();
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:send",
                    json!({
                        "chatroomId": a.room,
                        "message": {
                            "content": {"type": "text", "text": a.message},
                            "message_id": message_id,
                            "chatroom_id": a.room,
                            "sent_at": sent_at,
                        }
                    }),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::SendReply(a) => {
                let new_message_id = Uuid::new_v4().to_string();
                let sent_at = now_iso();
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:send",
                    json!({
                        "chatroomId": a.room,
                        "message": {
                            "content": {
                                "type": "text",
                                "text": a.text,
                                "reference": {
                                    "id": a.message_id,
                                    "content": {"type": "text", "text": "(original message)"},
                                    "sentBy": a.sender,
                                }
                            },
                            "message_id": new_message_id,
                            "chatroom_id": a.room,
                            "sent_at": sent_at,
                        }
                    }),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::LeaveRoom(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:leave-room",
                    json!({"chatroomId": a.room_id}),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::StartMatch(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "match:start",
                    json!({"type": a.match_type}),
                    false,
                    false,
                    5,
                )
                .await?;
                tracing::info!("Match started (fire-and-forget)");
                Ok(0)
            }
            SendCommand::CancelMatch(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "match:cancel",
                    json!({}),
                    false,
                    false,
                    5,
                )
                .await?;
                tracing::info!("Match cancelled (fire-and-forget)");
                Ok(0)
            }
            SendCommand::FetchMatchLimit(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "match:fetch-limit",
                    json!({}),
                    false,
                    false,
                    5,
                )
                .await?;
                tracing::info!("Fetch match limit sent (fire-and-forget)");
                Ok(0)
            }
            SendCommand::EditMessage(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:edit",
                    json!({
                        "chatroomId": a.room_id,
                        "messageId": a.message_id,
                        "message": {
                            "content": {"type": "text", "text": a.new_content},
                        }
                    }),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::RecallMessage(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:recall",
                    json!({"chatroomId": a.room_id, "messageId": a.message_id}),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::DeleteMessage(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:delete",
                    json!({"chatroomId": a.room_id, "messageId": a.message_id}),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::MarkRead(a) => {
                send(
                    db,
                    notification_config,
                    &a.account,
                    "message:read",
                    json!({"chatroomId": a.room_id, "messageId": a.message_id}),
                    true,
                    !a.no_wait,
                    a.timeout,
                )
                .await
            }
            SendCommand::SendImage(a) => send_image(db, notification_config, a).await,
            SendCommand::SendVoice(a) => send_voice(db, notification_config, a).await,
        }
    }
}

/// Shared send helper: create the outgoing command, optionally wait for result.
/// Mirrors Python `_send`.
#[allow(clippy::too_many_arguments)]
async fn send(
    db: &impl ConnectionTrait,
    notification_config: NotificationDatabaseConfig,
    account: &str,
    event: &str,
    data: Value,
    require_ack: bool,
    wait: bool,
    timeout_secs: u64,
) -> Result<u8> {
    let cmd = cmd_service::create_command(db, account, event, data, require_ack, None).await?;

    tracing::info!("Command queued: id={}", cmd.id);
    tracing::info!("   Event: {}", event);
    tracing::info!("   Account: {}", account);
    tracing::info!("   Require ack: {}", require_ack);

    if !wait {
        tracing::info!("--no-wait specified, not waiting for result");
        tracing::info!(
            "Check status with: lilium send-command status {}",
            cmd.id
        );
        return Ok(0);
    }

    tracing::info!("Waiting for result (timeout: {}s)...", timeout_secs);
    let outcome = wait_for_result(db, notification_config, cmd.id, timeout_secs).await?;

    match outcome {
        WaitOutcome::Success(cmd) => {
            tracing::info!("✅ Command succeeded");
            if let Some(resp) = cmd.ack_response {
                tracing::info!("   Response: {}", serde_json::to_string_pretty(&resp)?);
            }
            Ok(0)
        }
        WaitOutcome::Failed(cmd) => {
            tracing::error!("❌ Command failed: {:?}", cmd.error_message);
            tracing::error!("   Attempts: {}", cmd.attempt_count);
            Ok(1)
        }
        WaitOutcome::Timeout(cmd) => {
            tracing::error!("⏱️ Command timed out on server: {:?}", cmd.error_message);
            Ok(1)
        }
        WaitOutcome::PollTimeout(cmd) => {
            tracing::warn!("⚠️ CLI poll timeout (command may still be pending)");
            tracing::info!(
                "Check status with: lilium send-command status {}",
                cmd.id
            );
            Ok(1)
        }
        WaitOutcome::NotFound => {
            tracing::error!("❌ Command {} not found", cmd.id);
            Ok(1)
        }
    }
}

enum WaitOutcome {
    Success(outgoing_commands::Model),
    Failed(outgoing_commands::Model),
    Timeout(outgoing_commands::Model),
    PollTimeout(outgoing_commands::Model),
    NotFound,
}

/// Wait for a command to reach a terminal status using PostgreSQL LISTEN/NOTIFY
/// on the `outgoing_command_updated` channel, with a DB poll fallback.
/// Mirrors Python `wait_for_result`.
async fn wait_for_result(
    db: &impl ConnectionTrait,
    notification_config: NotifCfg,
    command_id: i32,
    timeout_secs: u64,
) -> Result<WaitOutcome> {
    // Race guard: command may already be terminal.
    if let Some(cmd) = cmd_service::get_command_result(db, command_id).await? {
        if let Some(outcome) = terminal_outcome(cmd) {
            return Ok(outcome);
        }
    } else {
        return Ok(WaitOutcome::NotFound);
    }

    let mut listener = NotificationConnection::connect(notification_config)
        .await
        .context("connect outgoing_command_updated listener")?;
    listener
        .listen("outgoing_command_updated")
        .await
        .context("listen outgoing_command_updated")?;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            break;
        }
        let remaining = deadline - now;
        tokio::select! {
            payload = listener.recv_payload() => {
                let payload = match payload {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!("notification listener error: {e}");
                        break;
                    }
                };
                if let Some((id, status_str)) = parse_notify_payload(&payload)
                    && id == command_id && TERMINAL_STATUSES.contains(&status_str.as_str()) {
                        break;
                    }
            }
            _ = tokio::time::sleep(remaining) => break,
        }
    }

    // Final authoritative fetch.
    match cmd_service::get_command_result(db, command_id).await? {
        Some(cmd) => Ok(match cmd.status.as_str() {
            status::SUCCESS => WaitOutcome::Success(cmd),
            status::FAILED => WaitOutcome::Failed(cmd),
            status::TIMEOUT => WaitOutcome::Timeout(cmd),
            _ => WaitOutcome::PollTimeout(cmd),
        }),
        None => Ok(WaitOutcome::NotFound),
    }
}

fn terminal_outcome(cmd: outgoing_commands::Model) -> Option<WaitOutcome> {
    match cmd.status.as_str() {
        status::SUCCESS => Some(WaitOutcome::Success(cmd)),
        status::FAILED => Some(WaitOutcome::Failed(cmd)),
        status::TIMEOUT => Some(WaitOutcome::Timeout(cmd)),
        _ => None,
    }
}

/// Parse a NOTIFY payload `{"id":N,"account_user_id":"...","status":"..."}`.
fn parse_notify_payload(payload: &str) -> Option<(i32, String)> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let id = value.get("id")?.as_i64()?;
    let status_str = value.get("status")?.as_str()?;
    Some((id as i32, status_str.to_owned()))
}

async fn list_pending(
    db: &impl ConnectionTrait,
    account_filter: Option<String>,
    limit: i64,
) -> Result<u8> {
    let mut query = outgoing_commands::Entity::find()
        .filter(outgoing_commands::Column::Status.eq(status::PENDING))
        .order_by_asc(outgoing_commands::Column::Id);
    if let Some(account_id) = account_filter.as_deref() {
        query = query.filter(outgoing_commands::Column::AccountUserId.eq(account_id));
    }
    let commands = query.limit(limit as u64).all(db).await?;

    if commands.is_empty() {
        tracing::info!("No pending commands");
        return Ok(0);
    }
    tracing::info!("Pending commands ({}):", commands.len());
    for cmd in commands {
        tracing::info!(
            "  #{}: {} (account: {})",
            cmd.id,
            cmd.event,
            cmd.account_user_id
        );
    }
    Ok(0)
}

async fn send_image(
    db: &impl ConnectionTrait,
    notification_config: NotificationDatabaseConfig,
    a: SendImageArgs,
) -> Result<u8> {
    let path = std::path::Path::new(&a.image_file);
    if !path.exists() {
        bail!("Image file not found: {}", a.image_file);
    }
    let alt = if a.alt.is_empty() {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image")
            .to_owned()
    } else {
        a.alt
    };

    let api = get_api_client(db, &a.account).await?;
    tracing::info!("Uploading image: {}", path.display());
    let image_url = api
        .upload_chat_image(&a.image_file)
        .await
        .context("upload image")?;
    tracing::info!("Image uploaded: {}", image_url);

    let message_id = Uuid::new_v4().to_string();
    let sent_at = now_iso();
    send(
        db,
        notification_config,
        &a.account,
        "message:send",
        json!({
            "chatroomId": a.room_id,
            "message": {
                "content": {"type": "image", "url": image_url, "alt": alt},
                "message_id": message_id,
                "chatroom_id": a.room_id,
                "sent_at": sent_at,
            }
        }),
        true,
        !a.no_wait,
        a.timeout,
    )
    .await
}

async fn send_voice(
    db: &impl ConnectionTrait,
    notification_config: NotificationDatabaseConfig,
    a: SendVoiceArgs,
) -> Result<u8> {
    let path = std::path::Path::new(&a.voice_file);
    if !path.exists() {
        bail!("Voice file not found: {}", a.voice_file);
    }

    let duration = lilium_services::media::extract_audio_duration(path);
    let duration = match duration {
        Some(d) => d,
        None => bail!("Could not detect audio duration for {}", a.voice_file),
    };
    tracing::info!("Detected duration: {:.2}s", duration);

    let api = get_api_client(db, &a.account).await?;
    tracing::info!("Uploading voice: {}", path.display());
    let result = api
        .upload_voice_message(&a.voice_file, Some(duration))
        .await
        .context("upload voice")?;
    let voice_url = result
        .get("url")
        .and_then(|u| u.as_str())
        .with_context(|| format!("upload response missing 'url': {result}"))?
        .to_owned();
    tracing::info!("Voice uploaded: {}", voice_url);

    let message_id = Uuid::new_v4().to_string();
    let sent_at = now_iso();
    send(
        db,
        notification_config,
        &a.account,
        "message:send",
        json!({
            "chatroomId": a.room_id,
            "message": {
                "content": {"type": "voice", "url": voice_url, "duration": duration},
                "message_id": message_id,
                "chatroom_id": a.room_id,
                "sent_at": sent_at,
            }
        }),
        true,
        !a.no_wait,
        a.timeout,
    )
    .await
}

/// Build an authenticated DZMM API client for the given account.
/// Mirrors Python `get_api_client`.
async fn get_api_client(db: &impl ConnectionTrait, account_user_id: &str) -> Result<DzmmApi> {
    let account = account::get_account(db, account_user_id)
        .await
        .context("fetch account")?
        .with_context(|| format!("Account '{}' not found", account_user_id))?;
    account::create_auth_client(account).map_err(Into::into)
}

fn print_command(cmd: &outgoing_commands::Model) {
    tracing::info!("Command #{}", cmd.id);
    tracing::info!("   Event: {}", cmd.event);
    tracing::info!("   Account: {}", cmd.account_user_id);
    tracing::info!("   Status: {}", cmd.status);
    tracing::info!("   Require ack: {}", cmd.require_ack);
    tracing::info!("   Attempts: {}/{}", cmd.attempt_count, cmd.max_attempts);
    tracing::info!("   Created: {}", cmd.created_at.to_rfc3339());
    if let Some(processed_at) = cmd.processed_at {
        tracing::info!("   Processed: {}", processed_at.to_rfc3339());
    }
    if let Some(resp) = cmd.ack_response.as_ref() {
        tracing::info!(
            "   Response: {}",
            serde_json::to_string_pretty(resp).unwrap_or_default()
        );
    }
    if let Some(err) = cmd.error_message.as_ref() {
        tracing::error!("   Error: {}", err);
    }
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_notify_payload_decodes_id_and_status() {
        let payload = r#"{"id":42,"account_user_id":"user1","status":"success"}"#;
        let (id, status_str) = parse_notify_payload(payload).expect("parsed");
        assert_eq!(id, 42);
        assert_eq!(status_str, "success");
    }

    #[test]
    fn parse_notify_payload_rejects_invalid_json() {
        assert!(parse_notify_payload("not json").is_none());
    }

    #[test]
    fn terminal_outcome_classifies_status() {
        let make = |status_str: &str| outgoing_commands::Model {
            id: 1,
            created_at: chrono::Utc::now(),
            account_user_id: "u".into(),
            event: "e".into(),
            data: json!({}),
            require_ack: true,
            status: status_str.into(),
            processed_at: None,
            ack_response: None,
            error_message: None,
            attempt_count: 0,
            max_attempts: 3,
        };
        assert!(matches!(
            terminal_outcome(make(status::SUCCESS)),
            Some(WaitOutcome::Success(_))
        ));
        assert!(matches!(
            terminal_outcome(make(status::FAILED)),
            Some(WaitOutcome::Failed(_))
        ));
        assert!(matches!(
            terminal_outcome(make(status::TIMEOUT)),
            Some(WaitOutcome::Timeout(_))
        ));
        assert!(terminal_outcome(make(status::PENDING)).is_none());
    }
}
