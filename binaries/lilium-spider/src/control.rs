use serde::{Deserialize, Serialize};

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

pub const ACCOUNT_ACTIONS: &[ControlAction] = &[
    ControlAction::Reconnect,
    ControlAction::Reload,
    ControlAction::Stop,
    ControlAction::Start,
    ControlAction::Restart,
];
pub const ARBITER_ACTIONS: &[ControlAction] = &[ControlAction::Status, ControlAction::Rescan];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCommand {
    pub action: ControlAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_user_id: Option<String>,
    #[serde(default)]
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ControlCommand {
    pub fn requires_account(&self) -> bool {
        ACCOUNT_ACTIONS.contains(&self.action)
    }

    pub fn is_arbiter_action(&self) -> bool {
        ARBITER_ACTIONS.contains(&self.action)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
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
        Self { ok: true, message: message.into(), data: None }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self { ok: false, message: message.into(), data: None }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
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
        Self { dev: meta.dev(), ino: meta.ino() }
    }
}

pub async fn write_message(writer: &mut tokio::net::UnixStream, msg: &str) -> Result<(), std::io::Error> {
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
    fn test_control_roundtrip() {
        let cmd = ControlCommand {
            action: ControlAction::Status,
            account_user_id: Some("user1".to_string()),
            reason: String::new(),
            data: None,
        };
        let json = cmd.to_json();
        let parsed = ControlCommand::from_json(&json).unwrap();
        assert_eq!(parsed.action, ControlAction::Status);
        assert_eq!(parsed.account_user_id.as_deref(), Some("user1"));
    }

    #[test]
    fn test_control_response() {
        let resp = ControlResponse {
            ok: true,
            message: "ok".to_string(),
            data: None,
        };
        let json = resp.to_json();
        let parsed = ControlResponse::from_json(&json).unwrap();
        assert!(parsed.ok);
        assert_eq!(parsed.message, "ok");
    }

    #[test]
    fn test_requires_account() {
        assert!(ControlCommand {
            action: ControlAction::Reload,
            account_user_id: None,
            reason: String::new(),
            data: None,
        }.requires_account());

        assert!(!ControlCommand {
            action: ControlAction::Status,
            account_user_id: None,
            reason: String::new(),
            data: None,
        }.requires_account());
    }

    #[test]
    fn test_is_arbiter_action() {
        assert!(ControlCommand {
            action: ControlAction::Status,
            account_user_id: None,
            reason: String::new(),
            data: None,
        }.is_arbiter_action());

        assert!(!ControlCommand {
            action: ControlAction::Reload,
            account_user_id: None,
            reason: String::new(),
            data: None,
        }.is_arbiter_action());
    }
}
