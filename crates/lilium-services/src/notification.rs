use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use crate::Result;
use lilium_database::{NotificationConnection, NotificationDatabaseConfig};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio::time::timeout;
use tracing::instrument;

struct Subscriber {
    id: usize,
    channel: String,
    sender: broadcast::Sender<String>,
}

pub struct NotificationService {
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    listener: Option<NotificationConnection>,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationService {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            listener: None,
        }
    }

    #[instrument(level = "debug" skip(self, config))]
    pub async fn attach_listener(&mut self, config: NotificationDatabaseConfig) -> Result<()> {
        let listener = NotificationConnection::connect(config)
            .await
            .map_err(|error| lilium_common::LiliumError::database(error.to_string()))?;
        self.listener = Some(listener);
        Ok(())
    }

    #[instrument(level = "debug" skip(self), fields(channel = %channel))]
    pub async fn listen_channel(&mut self, channel: &str) -> Result<()> {
        let listener = self.listener.as_mut().ok_or_else(|| {
            lilium_common::LiliumError::config("notification listener is not attached")
        })?;
        listener
            .listen(channel)
            .await
            .map_err(|error| lilium_common::LiliumError::database(error.to_string()))?;
        Ok(())
    }

    #[instrument(level = "debug" skip(self))]
    pub async fn receive_payload(&mut self) -> Result<Option<String>> {
        let listener = self.listener.as_mut().ok_or_else(|| {
            lilium_common::LiliumError::config("notification listener is not attached")
        })?;
        listener
            .try_recv_payload()
            .await
            .map_err(|error| lilium_common::LiliumError::database(error.to_string()))
    }

    pub async fn subscribe(
        &mut self,
        channel: &str,
    ) -> Result<(usize, broadcast::Receiver<String>)> {
        let mut subs = self.subscribers.write().await;
        let sender = if let Some(existing) = subs.iter().find(|s| s.channel == channel) {
            existing.sender.clone()
        } else {
            let (sender, _receiver) = broadcast::channel(100);
            sender
        };
        let id = subs.len();
        subs.push(Subscriber {
            id,
            channel: channel.to_string(),
            sender: sender.clone(),
        });
        let receiver = sender.subscribe();
        Ok((id, receiver))
    }

    pub async fn unsubscribe(&mut self, id: usize) -> Result<()> {
        let mut subs = self.subscribers.write().await;
        subs.retain(|s| s.id != id);
        Ok(())
    }

    pub async fn wait_for_notification(
        &mut self,
        channel: &str,
        timeout_duration: Option<Duration>,
    ) -> Result<bool> {
        let (_id, mut receiver) = self.subscribe(channel).await?;

        let result = match timeout_duration {
            Some(dur) => timeout(dur, receiver.recv()).await,
            None => Ok(receiver.recv().await),
        };

        self.unsubscribe(_id).await?;

        match result {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    pub async fn wait_for_multiple(
        &mut self,
        channels: &[&str],
        timeout_duration: Option<Duration>,
    ) -> Result<Option<String>> {
        let mut ids = Vec::new();
        let mut receivers = Vec::new();

        for channel in channels {
            let (id, receiver) = self.subscribe(channel).await?;
            ids.push(id);
            receivers.push((channel.to_string(), receiver));
        }

        if receivers.is_empty() {
            for id in ids {
                self.unsubscribe(id).await?;
            }
            return Ok(None);
        }

        let (result_tx, mut result_rx) = mpsc::unbounded_channel::<String>();
        let mut joiners = Vec::new();
        for (channel_name, mut receiver) in receivers {
            let tx = result_tx.clone();
            joiners.push(tokio::spawn(async move {
                if receiver.recv().await.is_ok() {
                    let _ = tx.send(channel_name);
                }
            }));
        }
        drop(result_tx);

        let result = match timeout_duration {
            Some(dur) => match timeout(dur, result_rx.recv()).await {
                Ok(Some(c)) => Some(c),
                _ => None,
            },
            None => result_rx.recv().await,
        };

        for id in ids {
            self.unsubscribe(id).await?;
        }

        for joiner in joiners {
            joiner.abort();
        }

        Ok(result)
    }

    #[instrument(level = "debug" skip(self, state, poll_callback, stop_signal), fields(channel = %channel, polling_ms = polling_interval.as_millis()))]
    pub async fn stream_with_polling<F, Fut, T, S>(
        &mut self,
        channel: &str,
        state: S,
        poll_callback: F,
        polling_interval: Duration,
        stop_signal: Arc<tokio::sync::Notify>,
    ) -> Result<()>
    where
        F: Fn(S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(Vec<T>, S)>> + Send + 'static,
        T: Send + 'static,
        S: Send + 'static,
    {
        let (id, mut receiver) = self.subscribe(channel).await?;

        let subscribers = self.subscribers.clone();
        let _channel = channel.to_string();
        let mut state = Some(state);
        let poll_callback = Arc::new(poll_callback);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = stop_signal.notified() => {
                        break;
                    }
                    _ = receiver.recv() => {
                        let callback = poll_callback.clone();
                        match callback(state.take().unwrap()).await {
                            Ok((items, new_state)) => {
                                state = Some(new_state);
                                if !items.is_empty() {
                                    tracing::info!("Found {} items via NOTIFY", items.len());
                                }
                            }
                            Err(e) => {
                                state = None;
                                tracing::error!("Error in poll callback: {}", e);
                            }
                        }
                    }
                    _ = tokio::time::sleep(polling_interval) => {
                        let callback = poll_callback.clone();
                        match callback(state.take().unwrap()).await {
                            Ok((items, new_state)) => {
                                state = Some(new_state);
                                if !items.is_empty() {
                                    tracing::info!("Found {} items via polling", items.len());
                                }
                            }
                            Err(e) => {
                                state = None;
                                tracing::error!("Error in poll callback: {}", e);
                            }
                        }
                    }
                }
            }

            let mut subs = subscribers.write().await;
            subs.retain(|s| s.id != id);
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_subscribe_returns_id_and_receiver() {
        let mut service = NotificationService::new();
        let (id, _receiver) = service.subscribe("test_channel").await.unwrap();
        assert_eq!(id, 0);
    }

    #[tokio::test]
    async fn test_subscribe_increments_ids() {
        let mut service = NotificationService::new();
        let (id1, _) = service.subscribe("ch").await.unwrap();
        let (id2, _) = service.subscribe("ch").await.unwrap();
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_subscriber() {
        let mut service = NotificationService::new();
        let (id, _receiver) = service.subscribe("ch").await.unwrap();
        service.unsubscribe(id).await.unwrap();
        let (new_id, _) = service.subscribe("ch").await.unwrap();
        assert_eq!(new_id, 0);
    }

    #[tokio::test]
    async fn test_wait_for_notification_times_out() {
        let mut service = NotificationService::new();
        let result = service
            .wait_for_notification("ch", Some(Duration::from_millis(10)))
            .await
            .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_wait_for_notification_receives() {
        let mut service = NotificationService::new();
        let (id, _receiver) = service.subscribe("ch").await.unwrap();

        let subscribers = service.subscribers.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let subs = subscribers.read().await;
            let _ = subs[id].sender.send("msg".into());
        });

        let result = service
            .wait_for_notification("ch", Some(Duration::from_secs(1)))
            .await
            .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_wait_for_multiple_empty_channels_returns_none() {
        let mut service = NotificationService::new();
        let result = service
            .wait_for_multiple(&[], Some(Duration::from_millis(10)))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_wait_for_multiple_receives_channel_name() {
        let mut service = NotificationService::new();
        let (id, _receiver) = service.subscribe("channel_a").await.unwrap();

        let subscribers = service.subscribers.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let subs = subscribers.read().await;
            let _ = subs[id].sender.send("msg".into());
        });

        let result = service
            .wait_for_multiple(&["channel_a"], Some(Duration::from_secs(1)))
            .await
            .unwrap();
        assert_eq!(result, Some("channel_a".to_string()));
    }

    #[tokio::test]
    async fn test_wait_for_multiple_times_out() {
        let mut service = NotificationService::new();
        let result = service
            .wait_for_multiple(&["ch"], Some(Duration::from_millis(10)))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_receive_payload_without_listener_errors() {
        let mut service = NotificationService::new();
        let error = service.receive_payload().await.unwrap_err();

        assert!(
            error
                .to_string()
                .contains("notification listener is not attached")
        );
    }
}
