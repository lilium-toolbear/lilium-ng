use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LiliumError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("[{code}] {message}")]
    Service {
        code: String,
        message: String,
        status_code: Option<u16>,
        retryable: bool,
        details: Option<serde_json::Value>,
        headers: Option<HashMap<String, String>>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("[{code}] {message}")]
    DomainService {
        code: String,
        message: String,
        status_code: u16,
    },

    #[error("[{code}] {message}")]
    ConnectionConflict {
        code: String,
        message: String,
        lock_id: Option<i64>,
    },
}

impl LiliumError {
    pub fn database(message: impl Into<String>) -> Self {
        LiliumError::Database(message.into())
    }

    pub fn http(message: impl Into<String>) -> Self {
        LiliumError::Http(message.into())
    }

    pub fn websocket(message: impl Into<String>) -> Self {
        LiliumError::WebSocket(message.into())
    }

    pub fn config(message: impl Into<String>) -> Self {
        LiliumError::Config(message.into())
    }

    pub fn service(code: impl Into<String>, message: impl Into<String>) -> Self {
        LiliumError::Service {
            code: code.into(),
            message: message.into(),
            status_code: None,
            retryable: false,
            details: None,
            headers: None,
            source: None,
        }
    }

    pub fn domain_service_with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        LiliumError::DomainService {
            code: code.into(),
            message: message.into(),
            status_code: 400,
        }
    }

    pub fn domain_service(message: impl Into<String>) -> Self {
        Self::domain_service_with_code("INVALID_REQUEST", message)
    }

    pub fn connection_conflict(message: impl Into<String>, lock_id: Option<i64>) -> Self {
        LiliumError::ConnectionConflict {
            code: "WEBSOCKET_CONNECTION_LOCK_CONFLICT".to_string(),
            message: message.into(),
            lock_id,
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            LiliumError::Http(msg) => msg.contains("401") || msg.contains("403"),
            LiliumError::Service { retryable, .. } => *retryable,
            _ => false,
        }
    }

    pub fn status_code(&self) -> Option<u16> {
        match self {
            LiliumError::Service { status_code, .. } => *status_code,
            LiliumError::DomainService { status_code, .. } => Some(*status_code),
            _ => None,
        }
    }

    pub fn code(&self) -> Option<&str> {
        match self {
            LiliumError::Service { code, .. } => Some(code),
            LiliumError::DomainService { code, .. } => Some(code),
            LiliumError::ConnectionConflict { code, .. } => Some(code),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for LiliumError {
    fn from(value: sqlx::Error) -> Self {
        LiliumError::Database(value.to_string())
    }
}
