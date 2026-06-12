use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Service for downloading media
#[derive(Clone)]
pub struct MediaService {
    pool: PgPool,
}

impl MediaService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Download media for multiple messages in parallel
    /// Returns (success_count, failure_count)
    pub async fn download_media_batch(&self, message_ids: &[String]) -> Result<(i64, i64)> {
        if message_ids.is_empty() {
            return Ok((0, 0));
        }

        info!(count = message_ids.len(), "Downloading media for messages");

        let semaphore = Arc::new(Semaphore::new(10));
        let mut success_count = 0;
        let mut failure_count = 0;

        let mut handles = Vec::new();
        for message_id in message_ids {
            let permit = semaphore.clone().clone().acquire_owned().await?;
            let message_id = message_id.clone();
            let pool = self.pool.clone();

            let handle = tokio::spawn(async move {
                let result = Self::download_single_media(&pool, &message_id).await;
                drop(permit);
                (message_id, result)
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok((_message_id, Ok(()))) => {
                    success_count += 1;
                }
                Ok((message_id, Err(e))) => {
                    failure_count += 1;
                    warn!(message_id = %message_id, error = %e, "Failed to download media");
                }
                Err(e) => {
                    failure_count += 1;
                    warn!(error = %e, "Media download task failed");
                }
            }
        }

        info!(
            success = success_count,
            failure = failure_count,
            "Media download complete"
        );

        Ok((success_count, failure_count))
    }

    /// Download media for a single message
    async fn download_single_media(pool: &PgPool, message_id: &str) -> Result<()> {
        // Get message with media content
        let message = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            r#"SELECT message_id, attachment_url, attachment_file
               FROM messages WHERE message_id = $1"#,
        )
        .bind(message_id)
        .fetch_optional(pool)
        .await?;

        if let Some((_, Some(url), _)) = message {
            // Download media from URL
            // In a real implementation, this would download the file
            // For now, just log that we would download
            tracing::debug!(message_id = %message_id, url = %url, "Would download media");
        }

        Ok(())
    }
}
