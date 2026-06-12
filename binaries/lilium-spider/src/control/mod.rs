use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCommand {
    pub action: String,
    pub account_user_id: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResponse {
    pub ok: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_roundtrip() {
        let cmd = ControlCommand {
            action: "status".to_string(),
            account_user_id: Some("user1".to_string()),
            reason: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: ControlCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.action, "status");
        assert_eq!(parsed.account_user_id.as_deref(), Some("user1"));
    }

    #[test]
    fn test_control_response() {
        let resp = ControlResponse {
            ok: true,
            message: "ok".to_string(),
            data: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: ControlResponse = serde_json::from_str(&json).unwrap();
        assert!(parsed.ok);
    }
}
