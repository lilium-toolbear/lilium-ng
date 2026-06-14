use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use lilium_database::DbSessionContext;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::timeout;

struct Subscriber {
    id: usize,
    channel: String,
    sender: broadcast::Sender<String>,
}

pub struct NotificationService<'a> {
    session: DbSessionContext<'a>,
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
}

impl<'a> NotificationService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self {
            session,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
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
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
                    let (id, _receiver) = service.subscribe("test_channel").await.unwrap();
                    assert_eq!(id, 0);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_subscribe_returns_id_and_receiver");
    }

    #[tokio::test]
    async fn test_subscribe_increments_ids() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
                    let (id1, _) = service.subscribe("ch").await.unwrap();
                    let (id2, _) = service.subscribe("ch").await.unwrap();
                    assert_eq!(id1, 0);
                    assert_eq!(id2, 1);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_subscribe_increments_ids");
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_subscriber() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
                    let (id, _receiver) = service.subscribe("ch").await.unwrap();
                    service.unsubscribe(id).await.unwrap();
                    let (new_id, _) = service.subscribe("ch").await.unwrap();
                    assert_eq!(new_id, 0);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_unsubscribe_removes_subscriber");
    }

    #[tokio::test]
    async fn test_wait_for_notification_times_out() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
                    let result = service
                        .wait_for_notification("ch", Some(Duration::from_millis(10)))
                        .await
                        .unwrap();
                    assert!(!result);
                    Ok(())
                })
            },
        )
        .await
        .expect("test_wait_for_notification_times_out");
    }

    #[tokio::test]
    async fn test_wait_for_notification_receives() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
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
                    Ok(())
                })
            },
        )
        .await
        .expect("test_wait_for_notification_receives");
    }

    #[tokio::test]
    async fn test_wait_for_multiple_empty_channels_returns_none() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
                    let result = service
                        .wait_for_multiple(&[], Some(Duration::from_millis(10)))
                        .await
                        .unwrap();
                    assert!(result.is_none());
                    Ok(())
                })
            },
        )
        .await
        .expect("test_wait_for_multiple_empty_channels_returns_none");
    }

    #[tokio::test]
    async fn test_wait_for_multiple_receives_channel_name() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
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
                    Ok(())
                })
            },
        )
        .await
        .expect("test_wait_for_multiple_receives_channel_name");
    }

    #[tokio::test]
    async fn test_wait_for_multiple_times_out() {
        lilium_database::test_fixtures::with_db_session(
            lilium_database::test_fixtures::TestServiceFixture::Notification,
            |session| {
                Box::pin(async move {
                    let mut service = NotificationService::new(session);
                    let result = service
                        .wait_for_multiple(&["ch"], Some(Duration::from_millis(10)))
                        .await
                        .unwrap();
                    assert!(result.is_none());
                    Ok(())
                })
            },
        )
        .await
        .expect("test_wait_for_multiple_times_out");
    }
}
