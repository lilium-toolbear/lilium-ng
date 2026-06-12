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

    #[error("Business error: {0}")]
    Business(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),
}
