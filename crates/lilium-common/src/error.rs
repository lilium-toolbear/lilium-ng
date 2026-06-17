// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 services/errors.py
// NOTE: LiliumError enum and is_retryable (401/403 check on Http variant) are Rust extensions not in Python.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("[{code}] {message}")]
pub struct ServiceError {
    pub code: String,
    pub message: String,
    pub status_code: Option<u16>,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
    pub headers: Option<HashMap<String, String>>,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Error, Debug)]
#[error("[{code}] {message}")]
pub struct DomainServiceError {
    pub code: String,
    pub message: String,
    pub status_code: u16,
}

#[derive(Error, Debug)]
#[error("[{code}] {message}")]
pub struct ConnectionConflictError {
    pub code: String,
    pub message: String,
    pub lock_id: Option<i64>,
}

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

    #[error(transparent)]
    Service(Box<ServiceError>),

    #[error(transparent)]
    DomainService(Box<DomainServiceError>),

    #[error(transparent)]
    ConnectionConflict(Box<ConnectionConflictError>),
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
        LiliumError::Service(Box::new(ServiceError {
            code: code.into(),
            message: message.into(),
            status_code: None,
            retryable: false,
            details: None,
            headers: None,
            source: None,
        }))
    }

    pub fn domain_service_with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        LiliumError::DomainService(Box::new(DomainServiceError {
            code: code.into(),
            message: message.into(),
            status_code: 400,
        }))
    }

    pub fn domain_service(message: impl Into<String>) -> Self {
        Self::domain_service_with_code("INVALID_REQUEST", message)
    }

    pub fn connection_conflict(message: impl Into<String>, lock_id: Option<i64>) -> Self {
        LiliumError::ConnectionConflict(Box::new(ConnectionConflictError {
            code: "WEBSOCKET_CONNECTION_LOCK_CONFLICT".to_string(),
            message: message.into(),
            lock_id,
        }))
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            LiliumError::Http(msg) => msg.contains("401") || msg.contains("403"),
            LiliumError::Service(error) => error.retryable,
            _ => false,
        }
    }

    pub fn status_code(&self) -> Option<u16> {
        match self {
            LiliumError::Service(error) => error.status_code,
            LiliumError::DomainService(error) => Some(error.status_code),
            _ => None,
        }
    }

    pub fn code(&self) -> Option<&str> {
        match self {
            LiliumError::Service(error) => Some(&error.code),
            LiliumError::DomainService(error) => Some(&error.code),
            LiliumError::ConnectionConflict(error) => Some(&error.code),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for LiliumError {
    fn from(value: sqlx::Error) -> Self {
        LiliumError::Database(value.to_string())
    }
}

impl From<sea_orm::DbErr> for LiliumError {
    fn from(value: sea_orm::DbErr) -> Self {
        LiliumError::Database(value.to_string())
    }
}
