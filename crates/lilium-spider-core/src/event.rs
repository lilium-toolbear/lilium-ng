use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Normalized websocket event before durable queue insertion.
/// This is the core data type that flows through the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Account that received this event
    pub account_user_id: String,
    /// Event type (e.g., "message:new", "message:updated")
    pub event_type: String,
    /// Event payload
    pub payload: serde_json::Value,
    /// When the event was received
    pub received_at: DateTime<Utc>,
    /// Source of the event (socket or disk_replay)
    pub source: EventSource,
}

/// Where the event came from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    /// Received from WebSocket
    Socket,
    /// Replayed from disk buffer
    DiskReplay,
}

/// Event type classification for routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    MessageNew,
    MessageUpdated,
    MessageDeleted,
    MessageRecalled,
    PresenceUserOnline,
    GroupMemberJoined,
    GroupMemberLeft,
    Unknown,
}

impl EventType {
    /// Parse event type from string
    pub fn from_str(s: &str) -> Self {
        match s {
            "message:new" => Self::MessageNew,
            "message:updated" => Self::MessageUpdated,
            "message:deleted" => Self::MessageDeleted,
            "message:recalled" => Self::MessageRecalled,
            "presence:user-online" => Self::PresenceUserOnline,
            "group:member-joined" => Self::GroupMemberJoined,
            "group:member-left" => Self::GroupMemberLeft,
            _ => Self::Unknown,
        }
    }

    /// Check if this event type should be processed
    pub fn should_process(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessageNew => write!(f, "message:new"),
            Self::MessageUpdated => write!(f, "message:updated"),
            Self::MessageDeleted => write!(f, "message:deleted"),
            Self::MessageRecalled => write!(f, "message:recalled"),
            Self::PresenceUserOnline => write!(f, "presence:user-online"),
            Self::GroupMemberJoined => write!(f, "group:member-joined"),
            Self::GroupMemberLeft => write!(f, "group:member-left"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Disk spill record format (schema_version = 2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpillRecord {
    pub schema_version: u32,
    pub account_user_id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub received_at: DateTime<Utc>,
    pub source: String,
}

impl From<&EventEnvelope> for SpillRecord {
    fn from(e: &EventEnvelope) -> Self {
        Self {
            schema_version: 2,
            account_user_id: e.account_user_id.clone(),
            event_type: e.event_type.clone(),
            payload: e.payload.clone(),
            received_at: e.received_at,
            source: e.source.to_string(),
        }
    }
}

impl SpillRecord {
    pub fn to_event_envelope(&self) -> Option<EventEnvelope> {
        let source = match self.source.as_str() {
            "socket" => EventSource::Socket,
            "disk_replay" => EventSource::DiskReplay,
            _ => return None,
        };
        Some(EventEnvelope {
            account_user_id: self.account_user_id.clone(),
            event_type: self.event_type.clone(),
            payload: self.payload.clone(),
            received_at: self.received_at,
            source,
        })
    }
}

impl std::fmt::Display for EventSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Socket => write!(f, "socket"),
            Self::DiskReplay => write!(f, "disk_replay"),
        }
    }
}
