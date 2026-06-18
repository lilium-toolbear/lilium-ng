pub mod account;
pub mod book;
pub mod card;
pub mod chapter;
pub mod checkpoint;
pub mod gallery;
pub mod image_gps;
pub mod message;
pub mod outgoing_command;
pub mod room;
pub mod room_member;
pub mod tweet;
pub mod user;
pub mod user_history;
pub mod websocket_connection;

use chrono::{DateTime, Utc};

/// Parse an ISO 8601 datetime string to a UTC DateTime.
/// Handles trailing `Z`, `+00:00`, and naive datetimes.
/// Python parity: `datetime.fromisoformat(value.replace("Z", "+00:00"))`
pub(crate) fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    let normalized = value.replace('Z', "+00:00");
    if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(ndt) = chrono::NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(ndt.and_utc());
    }
    None
}

/// Parse an optional ISO 8601 datetime string.
pub fn parse_optional_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    let v = value?.trim();
    if v.is_empty() {
        return None;
    }
    parse_datetime(v)
}

/// Extract a boolean field from JSON, checking snake_case then camelCase.
pub(crate) fn bool_field(
    data: &serde_json::Value,
    snake: &str,
    camel: &str,
    default: bool,
) -> bool {
    data.get(snake)
        .or_else(|| data.get(camel))
        .and_then(|v| v.as_bool())
        .unwrap_or(default)
}

/// Extract an integer field from JSON, checking snake_case then camelCase.
pub(crate) fn int_field(data: &serde_json::Value, snake: &str, camel: &str, default: i32) -> i32 {
    data.get(snake)
        .or_else(|| data.get(camel))
        .and_then(|v| v.as_i64())
        .map(|n| n as i32)
        .unwrap_or(default)
}
