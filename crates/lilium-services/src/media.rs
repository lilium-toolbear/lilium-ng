use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use lilium_database::DbSession;
use reqwest::{
    header::{CONTENT_TYPE, LOCATION},
    redirect::Policy,
    Client, Method, Url,
};
use std::io::ErrorKind;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration as TokioDuration};
use tracing::instrument;
use tracing::{info, warn};

/// Service for downloading media.
///
/// This mirrors the Python pipeline where it can:
/// - normalize and validate remote URLs before downloading
/// - infer file extensions from URLs or HEAD responses
/// - store attachment paths relative to `data_path`
///
/// Limitation: the Python pipeline also performs GPS extraction and audio
/// duration metadata enrichment. That requires other crates and is not part of
/// this service yet.
pub struct MediaService {
    data_path: PathBuf,
    client: Client,
}

impl MediaService {
    pub fn new() -> Self {
        Self::with_data_path(PathBuf::from("./data"))
    }

    #[instrument(fields(data_path = %data_path.display()))]
    pub fn with_data_path(data_path: PathBuf) -> Self {
        Self {
            data_path,
            client: build_http_client(),
        }
    }

    /// Download media for multiple messages in parallel.
    ///
    /// The result paths are stored relative to `data_path`, which matches the
    /// Python contract more closely than the old absolute-path behavior.
    #[instrument(skip(self, session, message_ids), fields(message_count = message_ids.len()))]
    pub async fn download_media_batch(
        &self,
        session: &mut DbSession,
        message_ids: &[String],
    ) -> Result<(i64, i64)> {
        if message_ids.is_empty() {
            return Ok((0, 0));
        }

        info!(count = message_ids.len(), "Downloading media for messages");

        let media_rows = sqlx::query_as::<
            _,
            (
                String,
                DateTime<Utc>,
                Option<String>,
                Option<String>,
                Option<String>,
            ),
        >(
            r#"SELECT message_id, sent_at, content_type, attachment_url, attachment_file
               FROM messages WHERE message_id = ANY($1)"#,
        )
        .bind(message_ids)
        .fetch_all(session.as_mut())
        .await?;

        let mut to_download: Vec<(String, DateTime<Utc>, String, String)> = Vec::new();
        for (message_id, sent_at, content_type, attachment_url, attachment_file) in media_rows {
            if attachment_url.is_none() || attachment_file.is_some() {
                continue;
            }
            let ext = content_type_ext(content_type.as_deref().unwrap_or("other"));
            to_download.push((
                message_id,
                sent_at,
                attachment_url.expect("checked above"),
                ext.to_string(),
            ));
        }

        if to_download.is_empty() {
            return Ok((0, 0));
        }

        let semaphore = Arc::new(Semaphore::new(10));
        let mut success_count: i64 = 0;
        let mut failure_count: i64 = 0;
        let mut handles: Vec<tokio::task::JoinHandle<(String, Result<String>)>> = Vec::new();
        let mut downloaded_files: Vec<(String, String)> = Vec::new();

        for (message_id, sent_at, attachment_url, ext) in to_download {
            let permit = semaphore.clone().acquire_owned().await?;
            let data_path = self.data_path.clone();
            let client = self.client.clone();

            let handle = tokio::spawn(async move {
                let result = download_single_media(
                    &client,
                    &message_id,
                    &attachment_url,
                    sent_at,
                    &ext,
                    &data_path,
                )
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
                .execute(session.as_mut())
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
}

#[instrument(skip(url), fields(input_len = url.len()))]
pub fn normalize_url(url: &str) -> String {
    let mut sanitized = url.trim().to_string();
    for hidden_char in ['\u{200b}', '\u{200c}', '\u{200d}', '\u{feff}'] {
        sanitized = sanitized.replace(hidden_char, "");
    }
    sanitized.trim().to_string()
}

#[instrument(fields(ip = %ip))]
pub fn is_public_ip_address(ip: &str) -> bool {
    match ip.parse::<IpAddr>() {
        Ok(addr) => is_public_ip_addr(addr),
        Err(_) => false,
    }
}

fn is_public_ip_addr(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(addr) => {
            let octets = addr.octets();
            if addr.is_loopback()
                || addr.is_link_local()
                || addr.is_private()
                || addr.is_multicast()
                || addr.is_broadcast()
                || addr.is_unspecified()
            {
                return false;
            }

            !matches!(
                octets,
                [100, 64..=127, _, _]
                    | [192, 0, 0, _]
                    | [192, 0, 2, _]
                    | [198, 18..=19, _, _]
                    | [198, 51, 100, _]
                    | [203, 0, 113, _]
            )
        }
        IpAddr::V6(addr) => {
            let segments = addr.segments();
            if addr.is_loopback()
                || addr.is_multicast()
                || addr.is_unspecified()
                || addr.is_unique_local()
                || addr.is_unicast_link_local()
            {
                return false;
            }

            !(segments[0] == 0x2001 && segments[1] == 0x0db8)
        }
    }
}

#[instrument(skip(url), fields(input_len = url.len()))]
pub async fn validate_remote_url(url: &str) -> Result<Url> {
    let normalized_url = normalize_url(url);
    let parsed = Url::parse(&normalized_url)
        .with_context(|| format!("Invalid media download URL: {}", url))?;

    if !matches!(
        parsed.scheme().to_ascii_lowercase().as_str(),
        "http" | "https"
    ) {
        bail!("Unsupported URL scheme for media download: {}", url);
    }

    let host = parsed
        .host_str()
        .context("Media download URL is missing hostname")?;

    if let Ok(ip) = host.parse::<IpAddr>() {
        if !is_public_ip_addr(ip) {
            bail!("Blocked media download to non-public address: {}", ip);
        }
    } else {
        let port = parsed.port_or_known_default().unwrap_or(80);
        let resolved = tokio::net::lookup_host((host, port))
            .await
            .with_context(|| format!("Unable to resolve media host: {}", host))?;

        for addr in resolved {
            if !is_public_ip_addr(addr.ip()) {
                bail!(
                    "Blocked media download to non-public address: {}",
                    addr.ip()
                );
            }
        }
    }

    Ok(parsed)
}

#[instrument(fields(content_type = %content_type))]
pub fn content_type_ext(content_type: &str) -> &'static str {
    let normalized = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase();

    match normalized.as_str() {
        "image" | "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "video" | "video/mp4" => "mp4",
        "video/quicktime" => "mov",
        "video/webm" => "webm",
        "audio" | "audio/m4a" | "audio/mp4" => "m4a",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/webm" => "webm",
        "sticker" => "png",
        _ => "bin",
    }
}

#[instrument(fields(data_path = %data_path.display(), message_id = %message_id, sent_at = %sent_at, ext = %ext))]
pub fn message_attachment_path(
    data_path: &Path,
    message_id: &str,
    sent_at: DateTime<Utc>,
    ext: &str,
) -> PathBuf {
    data_path
        .join("attachments")
        .join("messages")
        .join(sent_at.format("%Y/%m/%d").to_string())
        .join(format!("{}.{}", message_id, ext))
}

#[instrument(fields(data_path = %data_path.display(), user_id = %user_id, avatar_id = %avatar_id, ext = %ext))]
pub fn avatar_attachment_path(
    data_path: &Path,
    user_id: &str,
    avatar_id: &str,
    ext: &str,
) -> PathBuf {
    data_path
        .join("attachments")
        .join("avatars")
        .join(format!("{}_{}.{}", user_id, avatar_id, ext))
}

#[instrument(skip(url), fields(input_len = url.len()))]
pub fn transform_avatar_url(url: &str) -> Option<String> {
    let normalized = normalize_url(url);
    let parsed = Url::parse(&normalized).ok()?;
    if parsed.host_str()? != "rls.cheggpt.com" {
        return None;
    }

    let mut path = parsed.path().to_string();
    if path.contains("/render/image/") {
        path = path.replace("/render/image/", "/object/");
    } else if !path.contains("/object/") {
        return None;
    }

    let mut transformed = parsed;
    transformed.set_path(&path);
    transformed.set_query(None);
    Some(transformed.to_string())
}

#[instrument(skip(url), fields(input_len = url.len()))]
pub fn extract_avatar_id(url: &str) -> Option<String> {
    let parsed = Url::parse(&normalize_url(url)).ok()?;
    let filename = parsed.path_segments()?.last()?.to_string();
    Some(
        filename
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(&filename)
            .to_string(),
    )
}

pub async fn download_user_avatar(
    data_path: &Path,
    url: &str,
    user_id: &str,
) -> Result<Option<String>> {
    let result: Result<Option<String>> = async {
        let download_url = match transform_avatar_url(url) {
            Some(url) => url,
            None => return Ok(None),
        };

        let avatar_id = match extract_avatar_id(&download_url) {
            Some(id) => id,
            None => return Ok(None),
        };

        let client = build_http_client();
        let ext = match extension_from_url(&download_url) {
            Some(ext) => ext,
            None => head_content_type_ext(&client, &download_url, "png").await?,
        };
        let file_path = avatar_attachment_path(data_path, user_id, &avatar_id, &ext);

        if file_path.exists() {
            return Ok(Some(relative_data_path(data_path, &file_path)?));
        }

        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let bytes = download_bytes(&client, &download_url).await?;
        tokio::fs::write(&file_path, bytes).await?;

        Ok(Some(relative_data_path(data_path, &file_path)?))
    }
    .await;

    match result {
        Ok(value) => Ok(value),
        Err(e) => {
            warn!(user_id = %user_id, error = %e, "Failed to download avatar");
            Ok(None)
        }
    }
}

async fn download_single_media(
    client: &Client,
    message_id: &str,
    attachment_url: &str,
    sent_at: DateTime<Utc>,
    default_ext: &str,
    data_path: &Path,
) -> Result<String> {
    let normalized_url = validate_remote_url(attachment_url).await?;
    let ext = match extension_from_url(normalized_url.as_str()) {
        Some(ext) => ext,
        None => head_content_type_ext(client, normalized_url.as_str(), default_ext).await?,
    };

    let file_path = message_attachment_path(data_path, message_id, sent_at, &ext);
    if file_path.exists() {
        return relative_data_path(data_path, &file_path);
    }

    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let bytes = download_bytes(client, normalized_url.as_str()).await?;
    tokio::fs::write(&file_path, bytes).await?;

    relative_data_path(data_path, &file_path)
}

fn build_http_client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(Policy::none())
        .build()
        .expect("Failed to build media HTTP client")
}

fn extension_from_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let filename = parsed.path_segments()?.last()?;
    let ext = filename.rsplit_once('.').map(|(_, ext)| ext)?;
    if ext.is_empty() {
        None
    } else {
        Some(ext.to_ascii_lowercase())
    }
}

#[instrument(skip(client, url), fields(method = ?method, max_attempts))]
async fn request_with_retry(
    client: &Client,
    method: Method,
    url: &str,
    max_attempts: usize,
) -> Result<reqwest::Response> {
    let normalized_url = validate_remote_url(url).await?;

    for attempt in 1..=max_attempts {
        match send_follow_redirects(client, method.clone(), normalized_url.clone()).await {
            Ok(response) => return Ok(response),
            Err(error) if is_retryable_request_error(&error) && attempt < max_attempts => {
                let delay_ms = 500 * 2u64.pow((attempt - 1) as u32);
                sleep(TokioDuration::from_millis(delay_ms)).await;
            }
            Err(error) => return Err(error),
        }
    }

    bail!("request retry loop exited unexpectedly")
}

#[instrument(skip(client, url), fields(method = ?method))]
async fn send_follow_redirects(
    client: &Client,
    method: Method,
    url: Url,
) -> Result<reqwest::Response> {
    let mut current_url = url;
    let mut redirect_hops = 0usize;

    loop {
        let response = client
            .request(method.clone(), current_url.clone())
            .send()
            .await?;

        if response.status().is_redirection() {
            redirect_hops += 1;
            if redirect_hops > 5 {
                bail!("Exceeded max redirects for {}", current_url);
            }

            let location = response
                .headers()
                .get(LOCATION)
                .context("Redirect response missing Location header")?
                .to_str()
                .context("Invalid redirect Location header")?;
            let next_url = current_url
                .join(location)
                .context("Failed to resolve redirect target")?;
            validate_remote_url(next_url.as_str()).await?;
            current_url = next_url;
            continue;
        }

        return Ok(response);
    }
}

fn is_retryable_request_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        if let Some(reqwest_error) = cause.downcast_ref::<reqwest::Error>() {
            return reqwest_error.is_timeout() || reqwest_error.is_connect();
        }

        if let Some(io_error) = cause.downcast_ref::<std::io::Error>() {
            return matches!(
                io_error.kind(),
                ErrorKind::TimedOut
                    | ErrorKind::ConnectionRefused
                    | ErrorKind::ConnectionAborted
                    | ErrorKind::ConnectionReset
                    | ErrorKind::NotConnected
                    | ErrorKind::BrokenPipe
                    | ErrorKind::UnexpectedEof
            ) || io_error.raw_os_error() == Some(8);
        }

        false
    })
}

#[instrument(skip(client, url), fields(url_len = url.len()))]
async fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = request_with_retry(client, Method::GET, url, 3).await?;
    let response = response.error_for_status()?;
    Ok(response.bytes().await?.to_vec())
}

#[instrument(skip(client, url), fields(url_len = url.len(), default_ext = %default_ext))]
async fn head_content_type_ext(client: &Client, url: &str, default_ext: &str) -> Result<String> {
    let response = request_with_retry(client, Method::HEAD, url, 3).await?;
    let response = response.error_for_status()?;
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let ext = content_type_ext(content_type);
    if ext == "bin" {
        Ok(default_ext.to_string())
    } else {
        Ok(ext.to_string())
    }
}

fn relative_data_path(data_path: &Path, file_path: &Path) -> Result<String> {
    Ok(file_path
        .strip_prefix(data_path)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn normalize_url_strips_whitespace_and_hidden_separators() {
        let normalized = normalize_url(" \u{200b}https://example.com/a/b.png?x=1\u{feff} ");
        assert_eq!(normalized, "https://example.com/a/b.png?x=1");
    }

    #[test]
    fn content_type_ext_handles_python_media_types() {
        assert_eq!(content_type_ext("image/jpeg"), "jpg");
        assert_eq!(content_type_ext("video/mp4"), "mp4");
        assert_eq!(content_type_ext("audio/mp4"), "m4a");
        assert_eq!(content_type_ext("sticker"), "png");
        assert_eq!(content_type_ext("other"), "bin");
    }

    #[test]
    fn media_attachment_path_uses_date_partition() {
        let sent_at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let path = message_attachment_path(Path::new("/tmp/data"), "message-1", sent_at, "jpg");
        assert_eq!(
            path,
            Path::new("/tmp/data/attachments/messages/2026/01/02/message-1.jpg")
        );
    }

    #[test]
    fn avatar_path_keeps_avatar_history_identity() {
        let path = avatar_attachment_path(Path::new("/tmp/data"), "user-1", "avatar-uuid", "png");
        assert_eq!(
            path,
            Path::new("/tmp/data/attachments/avatars/user-1_avatar-uuid.png")
        );
    }

    #[tokio::test]
    async fn validate_remote_url_rejects_loopback_targets() {
        let err = validate_remote_url("http://127.0.0.1/avatar.png")
            .await
            .expect_err("loopback must be blocked");
        let msg = err.to_string();
        assert!(msg.contains("non-public") || msg.contains("Unsupported"));
    }

    #[test]
    fn transform_avatar_url_rewrites_render_urls() {
        let url = "https://rls.cheggpt.com/storage/v1/render/image/public/avatar/id.png?width=128&height=128";
        assert_eq!(
            transform_avatar_url(url).as_deref(),
            Some("https://rls.cheggpt.com/storage/v1/object/public/avatar/id.png")
        );
    }

    #[test]
    fn extract_avatar_id_strips_extension() {
        let url = "https://rls.cheggpt.com/storage/v1/object/public/avatar/0ec8f6c4-a83e-43b6-8567-fb253fe34c38.png";
        assert_eq!(
            extract_avatar_id(url).as_deref(),
            Some("0ec8f6c4-a83e-43b6-8567-fb253fe34c38")
        );
    }
}
