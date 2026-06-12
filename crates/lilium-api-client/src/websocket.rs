use anyhow::Result;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{info, warn, error};
use std::sync::Arc;

use lilium_models::ingestion::EventEnvelope;

/// WebSocket client for connecting to DZMM.ai
pub struct WsClient {
    account_id: String,
    url: String,
}

impl WsClient {
    pub fn new(account_id: String, url: String) -> Self {
        Self { account_id, url }
    }

    /// Connect to WebSocket and stream events
    pub async fn run(&self, sender: mpsc::Sender<EventEnvelope>) -> Result<()> {
        info!(account = %self.account_id, url = %self.url, "Connecting to WebSocket");

        let (ws_stream, _) = connect_async(&self.url).await?;
        let (_, mut read) = ws_stream.split();

        info!(account = %self.account_id, "WebSocket connected");

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match self.parse_event(&text) {
                        Ok(event) => {
                            if sender.send(event).await.is_err() {
                                warn!(account = %self.account_id, "Event channel closed");
                                break;
                            }
                        }
                        Err(e) => {
                            warn!(account = %self.account_id, error = %e, "Failed to parse event");
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    // Handle ping
                }
                Ok(Message::Pong(_)) => {
                    // Handle pong
                }
                Ok(Message::Close(_)) => {
                    info!(account = %self.account_id, "WebSocket closed");
                    break;
                }
                Err(e) => {
                    error!(account = %self.account_id, error = %e, "WebSocket error");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Parse a raw WebSocket message into an EventEnvelope
    fn parse_event(&self, text: &str) -> Result<EventEnvelope> {
        let data: serde_json::Value = serde_json::from_str(text)?;
        
        let event_type = data.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let payload = data.get("data")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        Ok(EventEnvelope {
            account_user_id: self.account_id.clone(),
            event_type,
            payload,
            received_at: chrono::Utc::now(),
            source: lilium_models::ingestion::EventSource::Socket,
        })
    }
}
