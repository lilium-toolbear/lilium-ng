// This module is new Rust code with no direct Python equivalent.
// It combines media download logic from multiple Python sources into a single service.
// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 core/media.py
use crate::user::AvatarDownload;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use exif::{In, Reader as ExifReader, Tag, Value};
use lilium_models::dzmm::{image_gps, message as messages, user as users};
use lofty::file::AudioFile;
use reqwest::{
    Client, Method, Url,
    header::{CONTENT_TYPE, LOCATION},
    redirect::Policy,
};
use sea_orm::sea_query::{Alias, Expr, extension::postgres::PgExpr};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use std::fs::File;
use std::io::BufReader;
use std::io::ErrorKind;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::time::{Duration as TokioDuration, sleep};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaDownload {
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
    pub content_type: String,
    pub attachment_url: String,
    pub ext: String,
    pub sticker_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaFileUpdate {
    pub message_id: String,
    pub attachment_file: String,
    pub gps: Option<ImageGpsData>,
    pub metadata_patch: Option<serde_json::Value>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ImageGpsData {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvatarFileUpdate {
    pub user_id: String,
    pub avatar_file: String,
}

impl Default for MediaService {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaService {
    pub fn new() -> Self {
        Self::with_data_path(PathBuf::from("./data"))
    }

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
    #[instrument(level = "debug" skip(self, downloads), fields(message_count = downloads.len()))]
    pub async fn download_media_batch(
        &self,
        downloads: &[MediaDownload],
    ) -> Result<(Vec<MediaFileUpdate>, i64)> {
        if downloads.is_empty() {
            return Ok((Vec::new(), 0));
        }

        info!(count = downloads.len(), "Downloading media for messages");

        let semaphore = Arc::new(Semaphore::new(10));
        let mut failure_count: i64 = 0;
        let mut handles: Vec<tokio::task::JoinHandle<(String, Result<MediaFileUpdate>)>> =
            Vec::new();
        let mut updates = Vec::new();

        for download in downloads.iter().cloned() {
            let permit = semaphore.clone().acquire_owned().await?;
            let data_path = self.data_path.clone();
            let client = self.client.clone();

            let handle = tokio::spawn(async move {
                let result = download_single_media(&client, &download, &data_path).await;
                let message_id = download.message_id;
                drop(permit);
                (message_id, result)
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok((_message_id, Ok(update))) => {
                    updates.push(update);
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
            success = updates.len(),
            failure = failure_count,
            "Media download complete"
        );

        Ok((updates, failure_count))
    }

    #[instrument(level = "debug" skip(self, downloads), fields(avatar_count = downloads.len()))]
    pub async fn download_user_avatars(
        &self,
        downloads: &[AvatarDownload],
    ) -> Result<(Vec<AvatarFileUpdate>, i64)> {
        if downloads.is_empty() {
            return Ok((Vec::new(), 0));
        }

        let mut updates = Vec::new();
        let mut failure_count = 0;

        for download in downloads {
            match download_user_avatar(&self.data_path, &download.avatar_url, &download.user_id)
                .await
            {
                Ok(Some(avatar_file)) => {
                    updates.push(AvatarFileUpdate {
                        user_id: download.user_id.clone(),
                        avatar_file: avatar_file.clone(),
                    });
                    info!(
                        user_id = %download.user_id,
                        path = %avatar_file,
                        "Downloaded user avatar"
                    );
                }
                Ok(None) => {
                    failure_count += 1;
                }
                Err(error) => {
                    failure_count += 1;
                    warn!(
                        user_id = %download.user_id,
                        error = %error,
                        "Failed to download user avatar"
                    );
                }
            }
        }

        Ok((updates, failure_count))
    }
}

#[instrument(level = "debug" skip(db, message_ids), fields(message_count = message_ids.len()))]
pub async fn collect_message_media_downloads<C>(
    db: &C,
    message_ids: &[String],
) -> Result<Vec<MediaDownload>>
where
    C: ConnectionTrait,
{
    if message_ids.is_empty() {
        return Ok(Vec::new());
    }

    let downloads = messages::Entity::find()
        .filter(messages::Column::MessageId.is_in(message_ids.iter().cloned()))
        .all(db)
        .await?
        .into_iter()
        .filter_map(|message| {
            if message.attachment_file.is_some() {
                return None;
            }
            let attachment_url = message.attachment_url?;
            let content_type = message.content_type;
            let ext = content_type_ext(&content_type).to_string();
            Some(MediaDownload {
                message_id: message.message_id,
                sent_at: message.sent_at,
                content_type,
                attachment_url,
                ext,
                sticker_id: message.sticker_id,
            })
        })
        .collect();

    Ok(downloads)
}

#[instrument(level = "debug" skip(db, updates), fields(update_count = updates.len()))]
pub async fn persist_message_media_files<C>(db: &C, updates: &[MediaFileUpdate]) -> Result<i64>
where
    C: ConnectionTrait,
{
    let mut updated_count = 0;
    for update in updates {
        let result = messages::Entity::update_many()
            .set(messages::ActiveModel {
                attachment_file: Set(Some(update.attachment_file.clone())),
                ..Default::default()
            })
            .filter(messages::Column::MessageId.eq(update.message_id.clone()))
            .exec(db)
            .await?;
        updated_count += result.rows_affected as i64;

        if let Some(metadata_patch) = &update.metadata_patch {
            messages::Entity::update_many()
                .col_expr(
                    messages::Column::Metadata,
                    Expr::cust("COALESCE(metadata, '{}'::jsonb)")
                        .concat(Expr::value(metadata_patch.clone()).cast_as(Alias::new("jsonb"))),
                )
                .filter(messages::Column::MessageId.eq(update.message_id.clone()))
                .exec(db)
                .await?;
        }

        if let Some(gps) = &update.gps {
            image_gps::Entity::insert(image_gps::ActiveModel {
                message_id: Set(update.message_id.clone()),
                latitude: Set(gps.latitude),
                longitude: Set(gps.longitude),
                altitude: Set(gps.altitude),
                timestamp: Set(gps.timestamp),
                created_at: Set(Utc::now()),
            })
            .on_conflict_do_nothing()
            .exec(db)
            .await?;
        }
        info!(
            message_id = %update.message_id,
            path = %update.attachment_file,
            "Persisted media attachment path"
        );
    }
    Ok(updated_count)
}

#[instrument(level = "debug" skip(db, updates), fields(update_count = updates.len()))]
pub async fn persist_user_avatar_files<C>(db: &C, updates: &[AvatarFileUpdate]) -> Result<i64>
where
    C: ConnectionTrait,
{
    let mut updated_count = 0;
    for update in updates {
        let result = users::Entity::update_many()
            .set(users::ActiveModel {
                avatar_file: Set(Some(update.avatar_file.clone())),
                updated_at: Set(Utc::now()),
                ..Default::default()
            })
            .filter(users::Column::UserId.eq(update.user_id.clone()))
            .exec(db)
            .await?;
        updated_count += result.rows_affected as i64;
        info!(
            user_id = %update.user_id,
            path = %update.avatar_file,
            "Persisted user avatar path"
        );
    }
    Ok(updated_count)
}

pub fn normalize_url(url: &str) -> String {
    let mut sanitized = url.trim().to_string();
    for hidden_char in ['\u{200b}', '\u{200c}', '\u{200d}', '\u{feff}'] {
        sanitized = sanitized.replace(hidden_char, "");
    }
    sanitized.trim().to_string()
}

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
        "voice" => "m4a",
        "audio/mpeg" => "mp3",
        "audio/ogg" => "ogg",
        "audio/wav" => "wav",
        "audio/webm" => "webm",
        "sticker" => "png",
        _ => "bin",
    }
}

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

pub fn sticker_attachment_path(data_path: &Path, sticker_id: &str, ext: &str) -> PathBuf {
    data_path
        .join("attachments")
        .join("stickers")
        .join(format!("{}.{}", sticker_id, ext))
}

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

pub fn extract_avatar_id(url: &str) -> Option<String> {
    let parsed = Url::parse(&normalize_url(url)).ok()?;
    let filename = parsed.path_segments()?.next_back()?.to_string();
    Some(
        filename
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(&filename)
            .to_string(),
    )
}

#[instrument(level = "debug" skip(data_path, url), fields(user_id = %user_id, input_len = url.len()))]
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
    download: &MediaDownload,
    data_path: &Path,
) -> Result<MediaFileUpdate> {
    let normalized_url = validate_remote_url(&download.attachment_url).await?;
    let ext = match extension_from_url(normalized_url.as_str()) {
        Some(ext) => ext,
        None => head_content_type_ext(client, normalized_url.as_str(), &download.ext).await?,
    };

    let file_path = media_attachment_path(data_path, download, &ext)?;
    let attachment_file = relative_data_path(data_path, &file_path)?;
    if file_path.exists() {
        return Ok(media_file_update(download, attachment_file, &file_path));
    }

    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let bytes = download_bytes(client, normalized_url.as_str()).await?;
    tokio::fs::write(&file_path, bytes).await?;

    Ok(media_file_update(download, attachment_file, &file_path))
}

fn media_attachment_path(data_path: &Path, download: &MediaDownload, ext: &str) -> Result<PathBuf> {
    if download.content_type == "sticker" {
        let sticker_id = download
            .sticker_id
            .as_deref()
            .context("Sticker media download requires sticker_id")?;
        return Ok(sticker_attachment_path(data_path, sticker_id, ext));
    }

    Ok(message_attachment_path(
        data_path,
        &download.message_id,
        download.sent_at,
        ext,
    ))
}

fn media_file_update(
    download: &MediaDownload,
    attachment_file: String,
    file_path: &Path,
) -> MediaFileUpdate {
    let gps = if download.content_type == "image" {
        match extract_image_gps_data(file_path) {
            Ok(value) => value,
            Err(error) => {
                warn!(message_id = %download.message_id, error = %error, "Failed to extract image GPS data");
                None
            }
        }
    } else {
        None
    };

    let metadata_patch = if download.content_type == "voice" {
        extract_audio_duration(file_path)
            .map(|duration| serde_json::json!({ "audio_duration": duration }))
    } else {
        None
    };

    MediaFileUpdate {
        message_id: download.message_id.clone(),
        attachment_file,
        gps,
        metadata_patch,
    }
}

fn extract_image_gps_data(file_path: &Path) -> Result<Option<ImageGpsData>> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut reader = BufReader::new(file);
    let exif = match ExifReader::new().read_from_container(&mut reader) {
        Ok(exif) => exif,
        Err(_) => return Ok(None),
    };

    let latitude = gps_coordinate(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef, "S")?;
    let longitude = gps_coordinate(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef, "W")?;
    let (Some(latitude), Some(longitude)) = (latitude, longitude) else {
        return Ok(None);
    };
    let altitude = exif
        .get_field(Tag::GPSAltitude, In::PRIMARY)
        .and_then(|field| rational_value(&field.value, 0));
    let timestamp = gps_timestamp(&exif);

    Ok(Some(ImageGpsData {
        latitude,
        longitude,
        altitude,
        timestamp,
    }))
}

fn gps_coordinate(
    exif: &exif::Exif,
    value_tag: Tag,
    ref_tag: Tag,
    negative_ref: &str,
) -> Result<Option<f64>> {
    let Some(field) = exif.get_field(value_tag, In::PRIMARY) else {
        return Ok(None);
    };
    let Some(mut value) = rational_triplet_to_degrees(&field.value) else {
        return Ok(None);
    };
    if exif
        .get_field(ref_tag, In::PRIMARY)
        .and_then(|field| ascii_value(&field.value))
        .as_deref()
        == Some(negative_ref)
    {
        value = -value;
    }
    Ok(Some(value))
}

fn rational_triplet_to_degrees(value: &Value) -> Option<f64> {
    let Value::Rational(values) = value else {
        return None;
    };
    let degrees = values.first()?.to_f64();
    let minutes = values.get(1)?.to_f64();
    let seconds = values.get(2)?.to_f64();
    Some(degrees + minutes / 60.0 + seconds / 3600.0)
}

fn rational_value(value: &Value, index: usize) -> Option<f64> {
    match value {
        Value::Rational(values) => values.get(index).map(|value| value.to_f64()),
        _ => None,
    }
}

fn ascii_value(value: &Value) -> Option<String> {
    let Value::Ascii(values) = value else {
        return None;
    };
    let bytes = values.first()?;
    String::from_utf8(bytes.clone()).ok()
}

fn gps_timestamp(exif: &exif::Exif) -> Option<DateTime<Utc>> {
    let date = exif
        .get_field(Tag::GPSDateStamp, In::PRIMARY)
        .and_then(|field| ascii_value(&field.value))?;
    let time = exif
        .get_field(Tag::GPSTimeStamp, In::PRIMARY)
        .and_then(|field| match &field.value {
            Value::Rational(values) if values.len() >= 3 => Some((
                values[0].to_f64() as u32,
                values[1].to_f64() as u32,
                values[2].to_f64() as u32,
            )),
            _ => None,
        })?;
    let mut parts = date.split(':');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    chrono::NaiveDate::from_ymd_opt(year, month, day)?
        .and_hms_opt(time.0, time.1, time.2)
        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

pub fn extract_audio_duration(file_path: &Path) -> Option<f64> {
    let tagged_file = match lofty::read_from_path(file_path) {
        Ok(file) => file,
        Err(error) => {
            warn!(path = %file_path.display(), error = %error, "Failed to extract audio duration");
            return None;
        }
    };
    let duration = tagged_file.properties().duration();
    if duration.is_zero() {
        None
    } else {
        Some(duration.as_secs_f64())
    }
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
    let filename = parsed.path_segments()?.next_back()?;
    let ext = filename.rsplit_once('.').map(|(_, ext)| ext)?;
    if ext.is_empty() {
        None
    } else {
        Some(ext.to_ascii_lowercase())
    }
}

#[instrument(level = "debug" skip(client, url), fields(method = ?method, max_attempts))]
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

#[instrument(level = "debug" skip(client, url), fields(method = ?method))]
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

#[instrument(level = "debug" skip(client, url), fields(url_len = url.len()))]
async fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    let response = request_with_retry(client, Method::GET, url, 3).await?;
    let response = response.error_for_status()?;
    Ok(response.bytes().await?.to_vec())
}

#[instrument(level = "debug" skip(client, url), fields(url_len = url.len(), default_ext = %default_ext))]
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
        assert_eq!(content_type_ext("voice"), "m4a");
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
    fn sticker_attachment_path_uses_global_sticker_identity() {
        let path = sticker_attachment_path(Path::new("/tmp/data"), "sticker-1", "webp");
        assert_eq!(
            path,
            Path::new("/tmp/data/attachments/stickers/sticker-1.webp")
        );
    }

    #[test]
    fn media_download_path_uses_sticker_identity_for_stickers() {
        let sent_at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let download = MediaDownload {
            message_id: "message-1".into(),
            sent_at,
            content_type: "sticker".into(),
            attachment_url: "https://example.com/sticker.webp".into(),
            ext: "png".into(),
            sticker_id: Some("sticker-1".into()),
        };

        let path = media_attachment_path(Path::new("/tmp/data"), &download, "webp").unwrap();
        assert_eq!(
            path,
            Path::new("/tmp/data/attachments/stickers/sticker-1.webp")
        );
    }

    #[test]
    fn media_download_path_keeps_message_partition_for_images() {
        let sent_at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).unwrap();
        let download = MediaDownload {
            message_id: "message-1".into(),
            sent_at,
            content_type: "image".into(),
            attachment_url: "https://example.com/image.jpg".into(),
            ext: "jpg".into(),
            sticker_id: None,
        };

        let path = media_attachment_path(Path::new("/tmp/data"), &download, "jpg").unwrap();
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
    async fn avatar_download_batch_empty_does_not_require_db_session() {
        let media_service = MediaService::with_data_path(PathBuf::from("/tmp/data"));
        let (updates, failure_count) = media_service.download_user_avatars(&[]).await.unwrap();
        assert!(updates.is_empty());
        assert_eq!(failure_count, 0);
    }

    #[tokio::test]
    async fn media_download_batch_empty_does_not_require_db_session() {
        let media_service = MediaService::with_data_path(PathBuf::from("/tmp/data"));
        let (updates, failure_count) = media_service.download_media_batch(&[]).await.unwrap();
        assert!(updates.is_empty());
        assert_eq!(failure_count, 0);
    }

    #[tokio::test]
    async fn collect_media_downloads_preserves_sticker_identity_from_database() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Empty)
                .await
                .expect("init media db");

        lilium_database::transaction!(test_db.database(), |session| {
            let sent_at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
            let message = lilium_models::dzmm::message::Message {
                message_id: "sticker-message-1".into(),
                room_id: "room-1".into(),
                sent_at,
                sent_by: "user-1".into(),
                content_type: "sticker".into(),
                content_text: None,
                attachment_url: Some("https://example.com/sticker.webp".into()),
                attachment_file: None,
                sticker_id: Some("sticker-1".into()),
                alt_text: None,
                metadata: None,
                raw_data: serde_json::json!({
                    "message": {
                        "content": {
                            "type": "sticker",
                            "stickerId": "sticker-1",
                            "url": "https://example.com/sticker.webp"
                        }
                    }
                }),
                source: "spider".into(),
                created_at: Utc::now(),
                updated_at: None,
                is_deleted: false,
                deleted_at: None,
                deleted_by: None,
                is_recalled: false,
                is_edited: false,
                history: None,
                reference_message_id: None,
                reference_data: None,
            };

            crate::message::create_message_if_missing(session, &message)
                .await
                .expect("insert sticker message");

            let downloads =
                collect_message_media_downloads(session, std::slice::from_ref(&message.message_id))
                    .await
                    .expect("collect media downloads");

            assert_eq!(downloads.len(), 1);
            assert_eq!(downloads[0].content_type, "sticker");
            assert_eq!(downloads[0].sticker_id.as_deref(), Some("sticker-1"));

            Ok(())
        })
        .await
        .expect("collect sticker media download")
    }

    #[tokio::test]
    async fn persist_media_files_writes_gps_and_audio_duration_metadata() {
        let test_db =
            lilium_test_fixtures::TestDb::acquire(lilium_test_fixtures::FixtureProfile::Empty)
                .await
                .expect("init media db");

        lilium_database::transaction!(test_db.database(), |session| {
            let sent_at = Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap();
            for (message_id, content_type) in [
                ("image-with-gps", "image"),
                ("voice-with-duration", "voice"),
            ] {
                let message = lilium_models::dzmm::message::Message {
                    message_id: message_id.into(),
                    room_id: "room-1".into(),
                    sent_at,
                    sent_by: "user-1".into(),
                    content_type: content_type.into(),
                    content_text: None,
                    attachment_url: Some(format!("https://example.com/{message_id}")),
                    attachment_file: None,
                    sticker_id: None,
                    alt_text: None,
                    metadata: None,
                    raw_data: serde_json::json!({}),
                    source: "spider".into(),
                    created_at: Utc::now(),
                    updated_at: None,
                    is_deleted: false,
                    deleted_at: None,
                    deleted_by: None,
                    is_recalled: false,
                    is_edited: false,
                    history: None,
                    reference_message_id: None,
                    reference_data: None,
                };
                crate::message::create_message_if_missing(session, &message)
                    .await
                    .expect("insert media message");
            }

            persist_message_media_files(
                session,
                &[
                    MediaFileUpdate {
                        message_id: "image-with-gps".into(),
                        attachment_file: "attachments/messages/2024/01/02/image-with-gps.jpg"
                            .into(),
                        gps: Some(ImageGpsData {
                            latitude: 35.0,
                            longitude: 139.0,
                            altitude: Some(42.0),
                            timestamp: Some(sent_at),
                        }),
                        metadata_patch: None,
                    },
                    MediaFileUpdate {
                        message_id: "voice-with-duration".into(),
                        attachment_file: "attachments/messages/2024/01/02/voice-with-duration.m4a"
                            .into(),
                        gps: None,
                        metadata_patch: Some(serde_json::json!({"audio_duration": 12.5})),
                    },
                ],
            )
            .await
            .expect("persist media side effects");

            let gps = image_gps::Entity::find_by_id("image-with-gps".to_owned())
                .one(session)
                .await?
                .expect("gps row");
            assert_eq!(gps.latitude, 35.0);
            assert_eq!(gps.longitude, 139.0);
            assert_eq!(gps.altitude, Some(42.0));
            assert_eq!(gps.timestamp, Some(sent_at));

            let metadata_message = messages::Entity::find()
                .filter(messages::Column::MessageId.eq("voice-with-duration"))
                .one(session)
                .await?
                .expect("metadata row");
            assert_eq!(
                metadata_message.metadata,
                Some(serde_json::json!({"audio_duration": 12.5}))
            );

            Ok(())
        })
        .await
        .expect("persist media files with post processing")
    }

    #[test]
    fn extract_audio_duration_reads_wav_properties() {
        let path = std::env::temp_dir().join(format!(
            "lilium_voice_duration_{}_{}.wav",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let sample_rate = 8_000u32;
        let seconds = 1u32;
        let data_len = sample_rate * seconds * 2;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.resize(bytes.len() + data_len as usize, 0);
        std::fs::write(&path, bytes).expect("write wav fixture");

        let duration = extract_audio_duration(&path).expect("duration");
        let _ = std::fs::remove_file(&path);
        assert!((duration - 1.0).abs() < 0.01);
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
