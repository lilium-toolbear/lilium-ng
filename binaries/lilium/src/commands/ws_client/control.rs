// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 spider/ws_control.py
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub fn arbiter_socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("ws_arbiter.sock")
}

pub fn worker_socket_path(runtime_dir: &Path, account_user_id: &str) -> PathBuf {
    runtime_dir.join(format!("ws_worker_{}.sock", account_user_id))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlAction {
    Status,
    Reconnect,
    Reload,
    Stop,
    Start,
    Restart,
    Rescan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCommand {
    pub action: ControlAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_user_id: Option<String>,
    #[serde(default = "default_reason")]
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

fn default_reason() -> String {
    "requested".to_string()
}

impl ControlCommand {
    pub fn from_json(s: &str) -> Result<Self, String> {
        let payload: serde_json::Value =
            serde_json::from_str(s).map_err(|e| format!("Invalid JSON: {}", e))?;

        let action_str = payload
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'action' field")?;

        let action: ControlAction =
            serde_json::from_value(serde_json::Value::String(action_str.to_string()))
                .map_err(|e| format!("Invalid action: {}", e))?;

        let account_user_id = payload
            .get("account_user_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(ref uid) = account_user_id {
            validate_account_user_id(uid)?;
        }

        let reason = payload
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("requested")
            .to_string();

        let data = payload.get("data").cloned();
        if let Some(ref value) = data
            && !value.is_object()
        {
            return Err("Control command data must be a JSON object".to_string());
        }

        Ok(Self {
            action,
            account_user_id,
            reason,
            data,
        })
    }
}

pub fn validate_account_user_id(account_user_id: &str) -> Result<(), String> {
    let parts: Vec<&str> = account_user_id.split('-').collect();
    if parts.len() != 5 {
        return Err("account_user_id must be a canonical UUID string".to_string());
    }
    if parts[0].len() != 8
        || parts[1].len() != 4
        || parts[2].len() != 4
        || parts[3].len() != 4
        || parts[4].len() != 12
    {
        return Err("account_user_id must be a canonical UUID string".to_string());
    }
    for part in &parts {
        if !part
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        {
            return Err("account_user_id must be a canonical UUID string".to_string());
        }
    }
    Ok(())
}

pub fn remove_stale_or_refuse_unix_socket(socket_path: &Path) -> bool {
    if !socket_path.exists() {
        return true;
    }

    match std::os::unix::net::UnixStream::connect(socket_path) {
        Ok(_) => false,
        Err(_) => std::fs::remove_file(socket_path).is_ok() || !socket_path.exists(),
    }
}

pub async fn bind_unix_control_socket(
    socket_path: &Path,
) -> Result<(tokio::net::UnixListener, UnixSocketIdentity), std::io::Error> {
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    if socket_path.exists() && !remove_stale_or_refuse_unix_socket(socket_path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            format!(
                "control socket is already active: {}",
                socket_path.display()
            ),
        ));
    }

    let listener = tokio::net::UnixListener::bind(socket_path)?;
    let metadata = std::fs::metadata(socket_path)?;
    let identity = UnixSocketIdentity::from_metadata(&metadata);
    Ok((listener, identity))
}

pub fn unlink_bound_unix_socket(socket_path: &Path, identity: UnixSocketIdentity) {
    let Ok(metadata) = std::fs::metadata(socket_path) else {
        return;
    };

    let current = UnixSocketIdentity::from_metadata(&metadata);
    if current.dev == identity.dev && current.ino == identity.ino {
        let _ = std::fs::remove_file(socket_path);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ControlResponse {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
            data: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct UnixSocketIdentity {
    pub dev: u64,
    pub ino: u64,
}

impl UnixSocketIdentity {
    pub fn from_metadata(meta: &std::fs::Metadata) -> Self {
        use std::os::unix::fs::MetadataExt;
        Self {
            dev: meta.dev(),
            ino: meta.ino(),
        }
    }
}

pub async fn write_message(
    writer: &mut tokio::net::UnixStream,
    msg: &str,
) -> Result<(), std::io::Error> {
    use tokio::io::AsyncWriteExt;
    writer.write_all(msg.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_message(reader: &mut tokio::net::UnixStream) -> Result<String, std::io::Error> {
    use tokio::io::AsyncBufReadExt;
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;
    Ok(line.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid_valid() {
        assert!(validate_account_user_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid_format() {
        assert!(validate_account_user_id("not-a-uuid").is_err());
        assert!(validate_account_user_id("550e8400-e29b-41d4-a716").is_err());
        assert!(validate_account_user_id("550e8400-e29b-41d4-a716-446655440000-extra").is_err());
    }

    #[test]
    fn test_validate_uuid_invalid_hex() {
        assert!(validate_account_user_id("550e8400-e29b-41d4-a716-44665544000g").is_err());
    }

    #[test]
    fn test_validate_uuid_rejects_uppercase() {
        assert!(validate_account_user_id("550E8400-e29b-41d4-a716-446655440000").is_err());
    }

    #[test]
    fn test_control_socket_paths_match_python_runtime_names() {
        let runtime_dir = Path::new("/tmp/lilium-ws");

        assert_eq!(
            arbiter_socket_path(runtime_dir),
            PathBuf::from("/tmp/lilium-ws/ws_arbiter.sock")
        );
        assert_eq!(
            worker_socket_path(runtime_dir, "550e8400-e29b-41d4-a716-446655440000"),
            PathBuf::from("/tmp/lilium-ws/ws_worker_550e8400-e29b-41d4-a716-446655440000.sock")
        );
    }

    #[test]
    fn test_from_json_rejects_invalid_action() {
        let json = r#"{"action": "invalid_action"}"#;
        assert!(ControlCommand::from_json(json).is_err());
    }

    #[test]
    fn test_from_json_rejects_invalid_uuid() {
        let json = r#"{"action": "reload", "account_user_id": "not-a-uuid"}"#;
        assert!(ControlCommand::from_json(json).is_err());
    }

    #[test]
    fn test_from_json_accepts_valid_uuid() {
        let json =
            r#"{"action": "reload", "account_user_id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let cmd = ControlCommand::from_json(json).unwrap();
        assert_eq!(cmd.action, ControlAction::Reload);
    }

    #[test]
    fn test_default_reason() {
        let json = r#"{"action": "status"}"#;
        let cmd = ControlCommand::from_json(json).unwrap();
        assert_eq!(cmd.reason, "requested");
    }
}
