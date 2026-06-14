use anyhow::Result;
use lilium_database::DbSessionContext;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{info, warn};

/// Service for downloading media
pub struct MediaService<'a> {
    session: DbSessionContext<'a>,
    data_path: PathBuf,
}

impl<'a> MediaService<'a> {
    pub fn new(session: DbSessionContext<'a>) -> Self {
        Self {
            session,
            data_path: PathBuf::from("./data"),
        }
    }

    pub fn with_data_path(session: DbSessionContext<'a>, data_path: PathBuf) -> Self {
        Self { session, data_path }
    }

    /// Download media for multiple messages in parallel
    /// Returns (success_count, failure_count)
    pub async fn download_media_batch(&mut self, message_ids: &[String]) -> Result<(i64, i64)> {
        if message_ids.is_empty() {
            return Ok((0, 0));
        }

        info!(count = message_ids.len(), "Downloading media for messages");

        let media_rows =
            sqlx::query_as::<_, (String, Option<String>, Option<String>, Option<String>)>(
                r#"SELECT message_id, content_type, attachment_url, attachment_file
               FROM messages WHERE message_id = ANY($1)"#,
            )
            .bind(message_ids)
            .fetch_all(self.session.as_mut())
            .await?;

        let mut to_download: Vec<(String, String, String)> = Vec::new();
        for (message_id, content_type, attachment_url, attachment_file) in media_rows {
            if attachment_url.is_none() || attachment_file.is_some() {
                continue;
            }
            let ext = Self::content_type_ext(content_type.as_deref().unwrap_or("other"));
            to_download.push((message_id, attachment_url.unwrap(), ext.to_string()));
        }

        if to_download.is_empty() {
            return Ok((0, 0));
        }

        let semaphore = Arc::new(Semaphore::new(10));
        let mut success_count: i64 = 0;
        let mut failure_count: i64 = 0;
        let mut handles: Vec<tokio::task::JoinHandle<(String, Result<String>)>> = Vec::new();
        let mut downloaded_files: Vec<(String, String)> = Vec::new();

        for (message_id, attachment_url, ext) in to_download {
            let permit = semaphore.clone().acquire_owned().await?;
            let data_path = self.data_path.clone();

            let handle = tokio::spawn(async move {
                let result =
                    Self::download_single_media(&message_id, &attachment_url, &ext, &data_path)
                        .await;
                drop(permit);
                (message_id, result)
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok((message_id, Ok(file_path))) => {
                    success_count += 1;
                    downloaded_files.push((message_id, file_path));
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

        for (message_id, file_path) in downloaded_files {
            match sqlx::query("UPDATE messages SET attachment_file = $1 WHERE message_id = $2")
                .bind(&file_path)
                .bind(&message_id)
                .execute(self.session.as_mut())
                .await
            {
                Ok(_) => {
                    info!(message_id = %message_id, path = %file_path, "Persisted media attachment path");
                }
                Err(e) => {
                    success_count = success_count.saturating_sub(1);
                    failure_count += 1;
                    warn!(message_id = %message_id, error = %e, "Failed to persist attachment file path");
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

    fn content_type_ext(content_type: &str) -> &'static str {
        match content_type {
            "image" => "jpg",
            "video" => "mp4",
            "voice" => "m4a",
            "sticker" => "png",
            _ => "bin",
        }
    }

    /// Download media for a single message
    async fn download_single_media(
        message_id: &str,
        attachment_url: &str,
        ext: &str,
        data_path: &Path,
    ) -> Result<String> {
        let media_dir = data_path.join("media").join(message_id);
        tokio::fs::create_dir_all(&media_dir).await?;

        let file_path = media_dir.join(format!("attachment.{}", ext));

        let response = reqwest::get(attachment_url).await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(&file_path, &bytes).await?;

        Ok(file_path.to_string_lossy().to_string())
    }
}
