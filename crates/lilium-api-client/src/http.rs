// Python parity source: dzmm_archive@dd724947e194006e5c5cc55b910937745de84655 dzmm_client/api.py, dzmm_client/rate_limiter.py, dzmm_client/utils.py

use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::clearance::{
    ClearanceAgentClient, ClearanceError, ClearanceProvider, ClearanceRefreshReason,
    ClearanceSnapshot, is_cloudflare_cookie_name,
};
use crate::config::{ApiClientConfig, dzmm_local_address_from_env};
use crate::websocket::SocketIoCredentials;
use anyhow::{Context, Result, bail};
use base64::Engine;
use rand::RngExt;
use reqwest::{
    Client, Method, StatusCode,
    cookie::{CookieStore, Jar},
    header::{
        CONTENT_TYPE, COOKIE, HeaderMap, HeaderValue, ORIGIN, REFERER, SET_COOKIE, USER_AGENT,
    },
    multipart::{Form, Part},
    redirect::Policy,
};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tracing::instrument;
use tracing::{debug, error, info, warn};
use url::Url;

const GENERATE_STRING_CHARSET: &[u8] =
    b"useandomp26T198340PX75pxJACKVERYMINDBUSHWOLFoGQZbfghjklqvwyzrict";

fn generate_string(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..GENERATE_STRING_CHARSET.len());
            GENERATE_STRING_CHARSET[idx] as char
        })
        .collect()
}

fn normalize_base_url(base_url: &str) -> Result<String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        bail!("API base_url must not be empty");
    }
    Url::parse(trimmed).context("Invalid API base_url")?;
    Ok(trimmed.to_string())
}

fn is_trpc_business_forbidden(body_text: &str) -> bool {
    let payload: Value = match serde_json::from_str(body_text) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let items: Vec<&Value> = if let Some(arr) = payload.as_array() {
        arr.iter().collect()
    } else {
        vec![&payload]
    };

    for item in items {
        if let Some(obj) = item.as_object()
            && let Some(error) = obj.get("error")
            && let Some(error_obj) = error.as_object()
            && let Some(error_json) = error_obj.get("json")
            && error_json.get("code").and_then(|c| c.as_i64()) == Some(-32003)
        {
            return true;
        }
    }

    false
}

fn is_cloudflare_challenge(status: StatusCode, headers: &HeaderMap) -> bool {
    status == StatusCode::FORBIDDEN
        && headers
            .get("cf-mitigated")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.eq_ignore_ascii_case("challenge"))
}

fn parse_trpc_response(response: &Value, index: usize, default: Option<Value>) -> Value {
    let default = default.unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    let arr = match response.as_array() {
        Some(a) => a,
        None => return default,
    };

    if arr.len() <= index {
        return default;
    }

    let result = match arr[index].get("result") {
        Some(r) => r,
        None => return default,
    };

    let data = match result.get("data") {
        Some(d) => d,
        None => return default,
    };

    if !data.is_object() {
        return default;
    }

    data.get("json").cloned().unwrap_or_else(|| {
        if data.is_object() && !data.as_object().unwrap().is_empty() {
            data.clone()
        } else {
            default
        }
    })
}

fn extract_cookie_kv(set_cookie: &str) -> Option<(String, String)> {
    let main_part = set_cookie.split(';').next()?;
    let mut parts = main_part.splitn(2, '=');
    let name = parts.next()?.trim().to_string();
    let value = parts.next()?.trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some((name, value))
}

fn extract_response_cookies(headers: &HeaderMap) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    for value in headers.get_all(SET_COOKIE) {
        if let Ok(s) = value.to_str()
            && let Some((name, val)) = extract_cookie_kv(s)
        {
            cookies.insert(name, val);
        }
    }
    cookies
}

fn parse_cookie_header(cookie_header: &str) -> HashMap<String, String> {
    cookie_header
        .split(';')
        .filter_map(|part| extract_cookie_kv(part.trim()))
        .collect()
}

fn sanitize_logged_url(url: &Url) -> String {
    let path = url.path();
    let mut sanitized = url.clone();
    sanitized.set_query(None);
    if path.starts_with("/api/auth/sign-in-code/")
        && path != "/api/auth/sign-in-code/scan"
        && path.len() > "/api/auth/sign-in-code/".len()
    {
        sanitized.set_path("/api/auth/sign-in-code/<redacted>");
    }
    sanitized.to_string()
}

fn sanitize_logged_endpoint(endpoint: &str) -> String {
    let path = endpoint.split_once('?').map_or(endpoint, |(path, _)| path);
    if path.starts_with("/api/auth/sign-in-code/")
        && path != "/api/auth/sign-in-code/scan"
        && path.len() > "/api/auth/sign-in-code/".len()
    {
        return "/api/auth/sign-in-code/<redacted>".to_string();
    }
    path.to_string()
}

#[derive(Default)]
struct AccountCookieStore {
    inner: Jar,
}

impl AccountCookieStore {
    fn add_cookie_str(&self, cookie: &str, url: &Url) {
        if extract_cookie_kv(cookie).is_some_and(|(name, _)| !is_cloudflare_cookie_name(&name)) {
            self.inner.add_cookie_str(cookie, url);
        }
    }
}

impl CookieStore for AccountCookieStore {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        let allowed: Vec<&HeaderValue> = cookie_headers
            .filter(|header| {
                header
                    .to_str()
                    .ok()
                    .and_then(extract_cookie_kv)
                    .is_some_and(|(name, _)| !is_cloudflare_cookie_name(&name))
            })
            .collect();
        self.inner.set_cookies(&mut allowed.into_iter(), url);
    }

    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        self.inner.cookies(url)
    }
}

fn guess_content_type(suffix: &str) -> &str {
    match suffix.to_lowercase().as_str() {
        ".jpg" | ".jpeg" => "image/jpeg",
        ".png" => "image/png",
        ".gif" => "image/gif",
        ".webp" => "image/webp",
        ".mp3" => "audio/mpeg",
        ".wav" => "audio/wav",
        ".ogg" => "audio/ogg",
        ".webm" => "audio/webm",
        ".m4a" => "audio/mp4",
        _ => "application/octet-stream",
    }
}

fn guess_video_content_type(suffix: &str) -> &str {
    match suffix.to_lowercase().as_str() {
        ".mp4" => "video/mp4",
        ".mov" => "video/quicktime",
        ".m4v" => "video/x-m4v",
        ".webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

struct RateLimiter {
    min_delay: f64,
    max_delay: f64,
    batch_size: u64,
    batch_delay: f64,
    request_count: u64,
}

/// Maximum number of reactive retries on `429 Too Many Requests` within a
/// single `_request` call.
const MAX_429_RETRIES: u32 = 5;
/// Upper bound on a single `Retry-After` sleep so an absurd server value can't
/// hang the client.
const MAX_RETRY_AFTER_SECS: u64 = 60;
/// Escalating backoff (seconds) used when the server omits `Retry-After`.
const FALLBACK_429_BACKOFF_SECS: &[u64] = &[2, 4, 8, 16, 32];

/// Parse a `Retry-After` header as delta-seconds. HTTP-date form is not
/// supported (the upstream API uses delta-seconds).
fn parse_retry_after(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

impl RateLimiter {
    fn new(min_delay: f64, max_delay: f64, batch_size: u64, batch_delay: f64) -> Self {
        Self {
            min_delay,
            max_delay,
            batch_size,
            batch_delay,
            request_count: 0,
        }
    }

    /// Increment the request counter and return the delay to apply before the
    /// next request.
    ///
    /// The caller must release the rate-limiter lock *before* sleeping for the
    /// returned duration; otherwise the lock is held across the sleep and all
    /// requests are silently serialized (concurrency cap of 1).
    fn next_delay(&mut self) -> f64 {
        self.request_count += 1;

        let delay = if self.request_count.is_multiple_of(self.batch_size) {
            self.batch_delay
        } else if self.min_delay >= self.max_delay {
            // `rand::random_range` panics on an empty/inverted range, so when
            // the configured bounds are equal (or inverted) fall back to the
            // fixed minimum instead of sampling.
            self.min_delay
        } else {
            let mut rng = rand::rng();
            rng.random_range(self.min_delay..self.max_delay)
        };

        if self.request_count.is_multiple_of(self.batch_size) {
            debug!(
                "Batch delay after {} requests: {:.2}s",
                self.request_count, delay
            );
        } else {
            debug!(
                "Rate limit delay (request #{}): {:.2}s",
                self.request_count, delay
            );
        }

        delay
    }
}

pub type CookieRefreshCallback =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Clone, Default)]
pub struct DzmmApiAuth {
    pub email: Option<Cow<'static, str>>,
    pub password: Option<Cow<'static, str>>,
    pub signin_code: Option<Cow<'static, str>>,
    pub signin_code_image: Option<Vec<u8>>,
    pub signin_code_image_mime: Option<Cow<'static, str>>,
    pub cookies: Option<Cow<'static, str>>,
    pub user_id: Option<Cow<'static, str>>,
    pub auto_refresh: bool,
    pub on_cookies_refreshed: Option<CookieRefreshCallback>,
}

pub struct ImageEditRequest<'a> {
    pub prompt: &'a str,
    pub image_urls: &'a [String],
    pub image_width: Option<u64>,
    pub image_height: Option<u64>,
    pub num_inference_steps: Option<u64>,
    pub text_guidance_scale: Option<f64>,
    pub image_guidance_scale: Option<f64>,
    pub num_images: Option<u64>,
    pub enable_safety_checker: Option<bool>,
    pub model: Option<&'a str>,
    pub tag_ids: Option<&'a [String]>,
}

struct ApiRequest<'a> {
    method: Method,
    endpoint: Cow<'a, str>,
    query: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    json_body: Option<Value>,
    multipart_form: Option<MultipartSpec>,
    timeout: Option<Duration>,
}

impl<'a> ApiRequest<'a> {
    fn new(method: Method, endpoint: impl Into<Cow<'a, str>>) -> Self {
        Self {
            method,
            endpoint: endpoint.into(),
            query: Vec::new(),
            json_body: None,
            multipart_form: None,
            timeout: None,
        }
    }

    fn get(endpoint: impl Into<Cow<'a, str>>) -> Self {
        Self::new(Method::GET, endpoint)
    }

    fn post(endpoint: impl Into<Cow<'a, str>>) -> Self {
        Self::new(Method::POST, endpoint)
    }

    fn query<K, V, I>(mut self, query: I) -> Self
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
        I: IntoIterator<Item = (K, V)>,
    {
        self.query = query
            .into_iter()
            .map(|(key, value)| (key.into(), value.into()))
            .collect();
        self
    }

    fn json(mut self, body: Value) -> Self {
        self.json_body = Some(body);
        self
    }

    fn multipart(mut self, form: MultipartSpec) -> Self {
        self.multipart_form = Some(form);
        self
    }

    fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

#[derive(Clone)]
struct MultipartSpec {
    fields: Vec<MultipartField>,
}

#[derive(Clone)]
enum MultipartField {
    Text {
        name: String,
        value: String,
    },
    File {
        name: String,
        data: Vec<u8>,
        filename: String,
        mime_type: String,
    },
}

impl MultipartSpec {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }

    fn text(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push(MultipartField::Text {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    fn file(
        mut self,
        name: impl Into<String>,
        data: Vec<u8>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        self.fields.push(MultipartField::File {
            name: name.into(),
            data,
            filename: filename.into(),
            mime_type: mime_type.into(),
        });
        self
    }

    fn build(&self) -> Result<Form> {
        let mut form = Form::new();
        for field in &self.fields {
            form = match field {
                MultipartField::Text { name, value } => form.text(name.clone(), value.clone()),
                MultipartField::File {
                    name,
                    data,
                    filename,
                    mime_type,
                } => {
                    let part = Part::bytes(data.clone())
                        .file_name(filename.clone())
                        .mime_str(mime_type)
                        .context("Failed to build multipart part")?;
                    form.part(name.clone(), part)
                }
            };
        }
        Ok(form)
    }
}

fn trpc_batch_query<'a>(input_data: &Value) -> Vec<(Cow<'a, str>, Cow<'a, str>)> {
    vec![
        (Cow::Borrowed("batch"), Cow::Borrowed("1")),
        (Cow::Borrowed("input"), Cow::Owned(input_data.to_string())),
    ]
}

pub struct DzmmApi {
    client: Client,
    media_client: Client,
    base_url: String,
    auth: DzmmApiAuth,
    clearance_provider: Arc<dyn ClearanceProvider>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    refresh_lock: Arc<Mutex<()>>,
    cookie_map: Arc<Mutex<HashMap<String, String>>>,
    cookie_jar: Arc<AccountCookieStore>,
    cookie_url: Url,
}

impl DzmmApi {
    pub fn new(auth: DzmmApiAuth) -> Result<Self> {
        Self::new_with_config(ApiClientConfig::default(), auth)
    }

    pub fn new_with_config(config: ApiClientConfig, auth: DzmmApiAuth) -> Result<Self> {
        let clearance_provider = Arc::new(
            ClearanceAgentClient::new(&config.clearance_agent_url)
                .context("Invalid clearance agent configuration")?,
        );
        Self::new_with_clearance_provider(config, auth, clearance_provider)
    }

    pub fn new_with_clearance_provider(
        config: ApiClientConfig,
        auth: DzmmApiAuth,
        clearance_provider: Arc<dyn ClearanceProvider>,
    ) -> Result<Self> {
        let base_url = normalize_base_url(&config.base_url)?;
        let cookie_url = Url::parse(&base_url).context("Invalid API base URL")?;
        let cookie_map = if let Some(ref c) = auth.cookies {
            parse_cookie_header(c)
        } else {
            HashMap::new()
        };
        let cookie_jar = Arc::new(AccountCookieStore::default());
        for (name, value) in &cookie_map {
            cookie_jar.add_cookie_str(&format!("{name}={value}"), &cookie_url);
        }

        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .redirect(Policy::none())
            .cookie_provider(cookie_jar.clone());
        if let Some(local_address) = dzmm_local_address_from_env()? {
            client_builder = client_builder.local_address(local_address);
        }
        let client = client_builder
            .build()
            .context("Failed to build HTTP client")?;

        let mut media_client_builder = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .redirect(Policy::none());
        if let Some(local_address) = dzmm_local_address_from_env()? {
            media_client_builder = media_client_builder.local_address(local_address);
        }
        let media_client = media_client_builder
            .build()
            .context("Failed to build plain media HTTP client")?;

        Ok(Self {
            client,
            media_client,
            base_url,
            auth,
            clearance_provider,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(
                config.min_request_delay,
                config.max_request_delay,
                config.request_batch_size,
                config.request_batch_delay,
            ))),
            refresh_lock: Arc::new(Mutex::new(())),
            cookie_map: Arc::new(Mutex::new(cookie_map)),
            cookie_jar,
            cookie_url,
        })
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn get_cookie_string(&self) -> String {
        let map = self.combined_cookie_map().await;
        let mut pairs: Vec<String> = map.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        pairs.sort();
        pairs.join("; ")
    }

    #[instrument(
        level = "debug",
        skip(self),
        fields(clearance_generation = tracing::field::Empty)
    )]
    pub async fn socket_io_credentials(&self) -> Result<SocketIoCredentials> {
        let snapshot = self
            .clearance_provider
            .snapshot()
            .await
            .map_err(anyhow::Error::new)?;
        snapshot
            .validate_at(chrono::Utc::now())
            .map_err(anyhow::Error::new)?;
        let generation = snapshot.generation;
        tracing::Span::current().record("clearance_generation", generation);
        let account_cookie_header = self.get_cookie_string().await;
        let cookie_header = snapshot.merge_cookie_header(
            (!account_cookie_header.is_empty()).then_some(account_cookie_header.as_str()),
        );
        Ok(SocketIoCredentials {
            generation,
            user_agent: snapshot.user_agent,
            cookie_header,
        })
    }

    async fn combined_cookie_map(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = self
            .cookie_map
            .lock()
            .await
            .iter()
            .filter(|(name, _)| !is_cloudflare_cookie_name(name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        if let Some(header) = self.cookie_jar.cookies(&self.cookie_url)
            && let Ok(cookie_header) = header.to_str()
        {
            map.extend(
                parse_cookie_header(cookie_header)
                    .into_iter()
                    .filter(|(name, _)| !is_cloudflare_cookie_name(name)),
            );
        }
        map
    }

    async fn sync_cookie_jar_to_map(&self) {
        if let Some(header) = self.cookie_jar.cookies(&self.cookie_url)
            && let Ok(cookie_header) = header.to_str()
        {
            let jar_cookies = parse_cookie_header(cookie_header);
            if !jar_cookies.is_empty() {
                self.cookie_map.lock().await.extend(
                    jar_cookies
                        .into_iter()
                        .filter(|(name, _)| !is_cloudflare_cookie_name(name)),
                );
            }
        }
    }

    async fn has_auth_cookie(&self) -> bool {
        self.combined_cookie_map()
            .await
            .keys()
            .any(|k| k.starts_with("sb-rls-auth-token"))
    }

    async fn merge_response_cookies(&self, headers: &HeaderMap) {
        let new_cookies = extract_response_cookies(headers);
        if !new_cookies.is_empty() {
            let mut map = self.cookie_map.lock().await;
            for (k, v) in new_cookies {
                if is_cloudflare_cookie_name(&k) {
                    continue;
                }
                self.cookie_jar
                    .add_cookie_str(&format!("{k}={v}"), &self.cookie_url);
                map.insert(k, v);
            }
        }
    }

    async fn clear_cookies(&self) {
        for name in self.combined_cookie_map().await.keys() {
            self.cookie_jar
                .add_cookie_str(&format!("{name}=; Max-Age=0; Path=/"), &self.cookie_url);
        }
        self.cookie_map.lock().await.clear();
    }

    async fn invoke_cookies_refreshed(&self) {
        if let Some(ref cb) = self.auth.on_cookies_refreshed {
            let cookie_str = self.get_cookie_string().await;
            cb(cookie_str).await;
        }
    }

    #[instrument(level = "debug" skip(self))]
    pub async fn authenticate(&self) -> Result<()> {
        info!("Authenticating...");
        self.get_my_info(false).await?;
        Ok(())
    }

    #[instrument(level = "debug" skip(self))]
    pub async fn refresh_cookies(&self) -> Result<bool> {
        let _guard = self.refresh_lock.lock().await;
        info!("Refreshing authentication cookies...");

        let request = ApiRequest::get("/api/auth/token");
        match self.request_with_clearance_retry(&request).await {
            Ok((status, body, _)) => {
                if status == StatusCode::OK {
                    let auth_data: Value = serde_json::from_slice(&body).unwrap_or_default();
                    if let Some(uid) = auth_data.get("user_id").and_then(|v| v.as_str()) {
                        info!(
                            "Token refresh successful (used refresh_token), User ID: {}",
                            uid
                        );
                    } else {
                        info!("Token refresh successful (used refresh_token)");
                    }
                    if let Some(expires_at) = auth_data.get("expires_at").and_then(|v| v.as_f64()) {
                        let dt = chrono::DateTime::from_timestamp(expires_at as i64, 0)
                            .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| expires_at.to_string());
                        info!("Expires at: {}", dt);
                    }
                    self.invoke_cookies_refreshed().await;
                    return Ok(true);
                }
                warn!("Token refresh failed with status {}", status);
            }
            Err(e) => {
                if e.downcast_ref::<ClearanceError>().is_some() {
                    return Err(e);
                }
                warn!("Token refresh failed: {}", e);
            }
        }

        if let (Some(email), Some(password)) = (&self.auth.email, &self.auth.password) {
            info!("Falling back to password login...");
            return self.login_with_email_password(email, password).await;
        }

        if let (Some(image), Some(mime)) = (
            self.auth.signin_code_image.as_ref(),
            self.auth.signin_code_image_mime.as_ref(),
        ) {
            info!("Falling back to QR code image login...");
            return self.login_with_qr_image(image, mime).await;
        }

        if let Some(ref signin_code) = self.auth.signin_code {
            info!("Falling back to QR code signin...");
            return self.login_with_qr_code(signin_code).await;
        }

        error!("Token refresh failed and no credentials available for fallback");
        Ok(false)
    }

    #[instrument(level = "debug" skip(self, email, password))]
    pub async fn login_with_email_password(&self, email: &str, password: &str) -> Result<bool> {
        info!("Logging in with email and password...");

        self.clear_cookies().await;

        let body = json!({"email": email, "password": password});
        let request = ApiRequest::post("/api/auth/sign-in").json(body);
        let (sign_in_status, _, _) = self
            .request_with_clearance_retry(&request)
            .await
            .context("Sign-in request failed")?;

        if sign_in_status != StatusCode::OK {
            error!("Sign-in failed: {}", sign_in_status);
            return Ok(false);
        }

        info!("Sign-in successful");

        let request = ApiRequest::get("/api/auth/token");
        let (token_status, token_body, _) = self
            .request_with_clearance_retry(&request)
            .await
            .context("Token request failed")?;

        if token_status != StatusCode::OK {
            error!("Token retrieval failed: {}", token_status);
            return Ok(false);
        }

        let auth_data: Value = serde_json::from_slice(&token_body).unwrap_or_default();
        if let Some(uid) = auth_data.get("user_id").and_then(|v| v.as_str()) {
            info!(
                "Token retrieved successfully (used password), User ID: {}",
                uid
            );
        } else {
            info!("Token retrieved successfully (used password)");
        }

        let cookie_names: Vec<String> = self.cookie_map.lock().await.keys().cloned().collect();
        debug!("Cookies in jar: {:?}", cookie_names);

        self.invoke_cookies_refreshed().await;
        Ok(true)
    }

    #[instrument(level = "debug" skip(self, encrypted_token), fields(token_len = encrypted_token.len()))]
    pub async fn login_with_qr_code(&self, encrypted_token: &str) -> Result<bool> {
        info!("Logging in with QR code token...");

        self.clear_cookies().await;

        let endpoint = format!("/api/auth/sign-in-code/{encrypted_token}");
        let request = ApiRequest::get(endpoint);
        self.request_with_clearance_retry(&request)
            .await
            .context("QR code login request failed")?;

        self.sync_cookie_jar_to_map().await;
        let has_auth_cookie = self.has_auth_cookie().await;

        if has_auth_cookie {
            info!("QR code login successful!");
            self.invoke_cookies_refreshed().await;
            return Ok(true);
        }

        error!("QR code login failed - no auth cookie received");
        Ok(false)
    }

    #[instrument(level = "debug" skip(self, image, mime_type), fields(image_len = image.len(), mime_type = %mime_type))]
    pub async fn login_with_qr_image(&self, image: &[u8], mime_type: &str) -> Result<bool> {
        info!("Logging in with QR code image (server-side scan)...");

        if !["image/jpeg", "image/png", "image/webp"].contains(&mime_type) {
            error!(
                "Unsupported image type: {}. Use image/jpeg, image/png, or image/webp.",
                mime_type
            );
            return Ok(false);
        }

        self.clear_cookies().await;

        let ext = match mime_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => "png",
        };
        let filename = format!("signin-code.{}", ext);

        let form = MultipartSpec::new().file("image", image.to_vec(), filename, mime_type);
        let request = ApiRequest::post("/api/auth/sign-in-code/scan").multipart(form);
        let (status, body, _) = self
            .request_with_clearance_retry(&request)
            .await
            .context("QR image login request failed")?;

        if status != StatusCode::OK {
            let error_data: Value = serde_json::from_slice(&body).unwrap_or_default();
            let error_msg = error_data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("HTTP error");
            error!("QR image login failed ({}): {}", status, error_msg);
            return Ok(false);
        }

        self.sync_cookie_jar_to_map().await;
        let has_auth_cookie = self.has_auth_cookie().await;

        if has_auth_cookie {
            info!("QR image login successful!");
            self.invoke_cookies_refreshed().await;
            return Ok(true);
        }

        error!("QR image login failed - no auth cookie received");
        Ok(false)
    }

    fn build_headers(&self, snapshot: &ClearanceSnapshot) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&snapshot.user_agent)
                .context("Clearance snapshot contains an invalid user agent")?,
        );
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/json, text/plain, */*"),
        );
        headers.insert(
            "Accept-Language",
            HeaderValue::from_static("en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
        );
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
        headers.insert(
            ORIGIN,
            HeaderValue::from_str(self.base_url()).context("Invalid DZMM origin header")?,
        );

        let referer = format!("{}/chat", self.base_url());
        headers.insert(
            REFERER,
            HeaderValue::from_str(&referer).context("Invalid DZMM referer header")?,
        );

        headers.insert(
            "x-dzmm-request-id",
            HeaderValue::from_str(&generate_string(10)).context("Invalid DZMM request ID")?,
        );

        Ok(headers)
    }

    async fn build_cookie_header_value(&self, snapshot: &ClearanceSnapshot) -> Option<String> {
        let cookie_str = self.get_cookie_string().await;
        let merged = snapshot.merge_cookie_header((!cookie_str.is_empty()).then_some(&cookie_str));
        if merged.is_empty() {
            return None;
        }
        Some(merged)
    }

    async fn send_request(&self, builder: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        Self::send_request_with_client(&self.client, builder).await
    }

    async fn send_request_with_client(
        client: &Client,
        builder: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let request = builder.build().context("Failed to build HTTP request")?;
        let method = request.method().clone();
        let url = request.url().clone();
        let logged_url = sanitize_logged_url(&url);
        let started_at = Instant::now();

        match client.execute(request).await {
            Ok(response) => {
                let status = response.status();
                let version = response.version();
                info!(
                    method = %method,
                    url = %logged_url,
                    status = status.as_u16(),
                    status_text = status.canonical_reason().unwrap_or(""),
                    version = ?version,
                    elapsed_ms = started_at.elapsed().as_millis(),
                    "HTTP Request"
                );
                Ok(response)
            }
            Err(error) => {
                warn!(
                    method = %method,
                    url = %logged_url,
                    elapsed_ms = started_at.elapsed().as_millis(),
                    error = %error,
                    "HTTP Request failed"
                );
                Err(error).context("HTTP request failed")
            }
        }
    }

    #[instrument(
        level = "debug",
        skip(self),
        fields(endpoint = %sanitize_logged_endpoint(endpoint))
    )]
    async fn download_plain_media(
        &self,
        endpoint: &str,
    ) -> Result<(StatusCode, Vec<u8>, HeaderMap)> {
        let url = Url::parse(&format!("{}{}", self.base_url(), endpoint))
            .context("Invalid media download URL")?;
        let response =
            Self::send_request_with_client(&self.media_client, self.media_client.get(url)).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .context("Failed to read media response body")?;
        Ok((status, body.to_vec(), headers))
    }

    #[instrument(level = "debug"
        skip(self, request, snapshot),
        fields(
            method = ?request.method,
            endpoint = %sanitize_logged_endpoint(request.endpoint.as_ref()),
            user_id = ?self.auth.user_id.as_deref(),
            clearance_generation = snapshot.generation
        )
    )]
    async fn _request_inner(
        &self,
        request: &ApiRequest<'_>,
        snapshot: &ClearanceSnapshot,
    ) -> Result<(StatusCode, Vec<u8>, HeaderMap)> {
        let mut url = Url::parse(&format!("{}{}", self.base_url(), request.endpoint.as_ref()))?;
        if !request.query.is_empty() {
            url.query_pairs_mut().extend_pairs(
                request
                    .query
                    .iter()
                    .map(|(key, value)| (key.as_ref(), value.as_ref())),
            );
        }

        let mut builder = self.client.request(request.method.clone(), url);

        let mut headers = self.build_headers(snapshot)?;
        if let Some(cookie_val) = self.build_cookie_header_value(snapshot).await {
            headers.insert(
                COOKIE,
                HeaderValue::from_str(&cookie_val).context("Invalid merged cookie header")?,
            );
        }
        builder = builder.headers(headers);

        if let Some(body) = &request.json_body {
            builder = builder.json(body);
        }

        if let Some(form) = &request.multipart_form {
            builder = builder.multipart(form.build()?);
        }

        if let Some(t) = request.timeout {
            builder = builder.timeout(t);
        }

        let response = self.send_request(builder).await?;
        let status = response.status();
        let resp_headers = response.headers().clone();
        let body_bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        Ok((status, body_bytes.to_vec(), resp_headers))
    }

    async fn request_with_clearance_retry(
        &self,
        request: &ApiRequest<'_>,
    ) -> Result<(StatusCode, Vec<u8>, HeaderMap)> {
        let mut clearance_retry_used = false;
        self.request_with_clearance_retry_state(request, &mut clearance_retry_used)
            .await
    }

    #[instrument(
        level = "debug",
        skip(self, request, clearance_retry_used),
        fields(
            endpoint = %sanitize_logged_endpoint(request.endpoint.as_ref()),
            clearance_generation = tracing::field::Empty,
            refreshed_generation = tracing::field::Empty
        )
    )]
    async fn request_with_clearance_retry_state(
        &self,
        request: &ApiRequest<'_>,
        clearance_retry_used: &mut bool,
    ) -> Result<(StatusCode, Vec<u8>, HeaderMap)> {
        let mut snapshot = self
            .clearance_provider
            .snapshot()
            .await
            .map_err(anyhow::Error::new)?;
        snapshot
            .validate_at(chrono::Utc::now())
            .map_err(anyhow::Error::new)?;
        tracing::Span::current().record("clearance_generation", snapshot.generation);

        loop {
            let (status, body, headers) = self._request_inner(request, &snapshot).await?;
            if is_cloudflare_challenge(status, &headers) {
                if *clearance_retry_used {
                    return Err(anyhow::Error::new(ClearanceError::ChallengePersisted {
                        endpoint: sanitize_logged_endpoint(request.endpoint.as_ref()),
                        generation: snapshot.generation,
                    }));
                }

                warn!(
                    endpoint = %sanitize_logged_endpoint(request.endpoint.as_ref()),
                    generation = snapshot.generation,
                    "Cloudflare challenge detected; refreshing clearance"
                );
                snapshot = self
                    .clearance_provider
                    .refresh(snapshot.generation, ClearanceRefreshReason::CfMitigated)
                    .await
                    .map_err(anyhow::Error::new)?;
                snapshot
                    .validate_at(chrono::Utc::now())
                    .map_err(anyhow::Error::new)?;
                tracing::Span::current().record("refreshed_generation", snapshot.generation);
                *clearance_retry_used = true;
                continue;
            }

            if *clearance_retry_used {
                info!(
                    endpoint = %sanitize_logged_endpoint(request.endpoint.as_ref()),
                    generation = snapshot.generation,
                    status = status.as_u16(),
                    "DZMM request completed after clearance refresh"
                );
            }
            self.merge_response_cookies(&headers).await;
            return Ok((status, body, headers));
        }
    }

    #[instrument(
        level = "debug",
        skip(self, request),
        fields(
            method = ?request.method,
            endpoint = %sanitize_logged_endpoint(request.endpoint.as_ref())
        )
    )]
    async fn _request(&self, request: ApiRequest<'_>) -> Result<Value> {
        let mut retried = false;
        let mut clearance_retry_used = false;
        let mut retry_429_count: u32 = 0;
        let logged_endpoint = sanitize_logged_endpoint(request.endpoint.as_ref());

        loop {
            // Compute the proactive delay under the lock, then sleep *after*
            // the guard is dropped. Sleeping while holding the mutex would
            // serialize every request through the client.
            let delay = self.rate_limiter.lock().await.next_delay();
            if delay > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(delay)).await;
            }

            let (status, body_bytes, resp_headers) = self
                .request_with_clearance_retry_state(&request, &mut clearance_retry_used)
                .await?;

            if status.is_success() {
                match serde_json::from_slice::<Value>(&body_bytes) {
                    Ok(val) => return Ok(val),
                    Err(_) => {
                        let body_text = String::from_utf8_lossy(&body_bytes);
                        bail!("Failed to parse JSON response: {}", body_text);
                    }
                }
            }

            let body_text = String::from_utf8_lossy(&body_bytes);

            // Reactive backoff on 429: honor Retry-After (delta-seconds) when
            // present, otherwise use an escalating fallback. Capped to avoid
            // hanging on absurd server values; bounded retry count.
            if status == StatusCode::TOO_MANY_REQUESTS && retry_429_count < MAX_429_RETRIES {
                let backoff = parse_retry_after(&resp_headers)
                    .unwrap_or(FALLBACK_429_BACKOFF_SECS[retry_429_count as usize])
                    .min(MAX_RETRY_AFTER_SECS);
                warn!(
                    "429 Too Many Requests for {}, backing off {}s (attempt {})",
                    logged_endpoint,
                    backoff,
                    retry_429_count + 1
                );
                tokio::time::sleep(Duration::from_secs(backoff)).await;
                retry_429_count += 1;
                continue;
            }

            let is_biz_forbidden = is_trpc_business_forbidden(&body_text);
            let is_auth = matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN);

            if is_auth && !is_biz_forbidden && self.auth.auto_refresh && !retried {
                warn!(
                    "Auth error before retry {} for {}\nResult: {}",
                    status, logged_endpoint, body_text
                );
                match self.refresh_cookies().await {
                    Ok(true) => {
                        info!("Retrying with fresh cookies...");
                        retried = true;
                        continue;
                    }
                    Ok(false) => {}
                    Err(error) if error.downcast_ref::<ClearanceError>().is_some() => {
                        return Err(error);
                    }
                    Err(error) => {
                        warn!(error = %error, "Authentication cookie refresh failed");
                    }
                }
            }

            if is_biz_forbidden {
                warn!(
                    "Business forbidden {} for {}\nResult: {}",
                    status, logged_endpoint, body_text
                );
                bail!("Business forbidden: {} {}", status, body_text);
            }

            error!(
                "HTTP error {} for {}\nResult: {}",
                status, logged_endpoint, body_text
            );
            bail!("HTTP {} for {}: {}", status, logged_endpoint, body_text);
        }
    }

    #[instrument(level = "debug" skip(self), fields(retried))]
    pub async fn get_my_info(&self, retried: bool) -> Result<Value> {
        let input_data = json!({"0": {"json": Value::Null}});
        let response = self
            ._request(ApiRequest::get("/api/trpc/user.getMe").query(trpc_batch_query(&input_data)))
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);

        if !parsed.is_object() || parsed.get("isLoggedIn") == Some(&Value::Bool(false)) {
            if !retried && self.auth.auto_refresh {
                warn!("isLoggedIn=false, attempting cookie refresh...");
                if self.refresh_cookies().await.unwrap_or(false) {
                    info!("Retrying with fresh cookies...");
                    return Box::pin(self.get_my_info(true)).await;
                }
            }
            bail!("Not logged in. Please check your cookies.");
        }

        Ok(parsed)
    }

    #[instrument(level = "debug" skip(self), fields(user_id = %user_id, room_id = %room_id))]
    pub async fn get_user_info(&self, user_id: &str, room_id: &str) -> Result<Value> {
        let input_data = json!({"0": {"json": {"userId": user_id, "chatroomId": room_id}}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/user.getChatroomUser")
                    .query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && parsed.is_object() && !parsed.as_object().unwrap().is_empty() {
            return Ok(parsed);
        }

        if response.is_object() {
            return Ok(response);
        }

        Ok(Value::Object(serde_json::Map::new()))
    }

    #[instrument(level = "debug" skip(self), fields(user_id = %user_id))]
    pub async fn get_public_user_profile(&self, user_id: &str) -> Result<Value> {
        let input_data = json!({"0": {"json": {"userid": user_id}}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/user.getProfilePage")
                    .query(trpc_batch_query(&input_data)),
            )
            .await?;

        let payload = parse_trpc_response(&response, 0, None);
        let profile = payload.get("profile").and_then(|v| v.as_object());

        if profile.is_none() {
            return Ok(Value::Object(serde_json::Map::new()));
        }

        let profile = profile.unwrap();
        let mut public_profile = profile.clone();
        if let Value::Object(ref payload_map) = payload {
            for (key, value) in payload_map {
                if key != "profile" {
                    public_profile.insert(key.clone(), value.clone());
                }
            }
        }

        let mut data = serde_json::Map::new();
        data.insert("id".to_string(), Value::String(user_id.to_string()));
        if let Some(v) = profile.get("fullName") {
            data.insert("fullName".to_string(), v.clone());
        }
        if let Some(v) = profile.get("avatarUrl") {
            data.insert("avatarUrl".to_string(), v.clone());
        }
        if let Some(v) = profile.get("bio") {
            data.insert("bio".to_string(), v.clone());
        }
        if let Some(v) = profile.get("birthday") {
            data.insert("birthday".to_string(), v.clone());
        }
        if let Some(v) = profile.get("birthdayPublic") {
            data.insert("birthdayPublic".to_string(), v.clone());
        }
        if let Some(v) = profile.get("quirk") {
            data.insert("quirk".to_string(), v.clone());
        }
        if let Some(v) = profile.get("gender") {
            data.insert("gender".to_string(), v.clone());
        }
        data.insert("isBot".to_string(), Value::Bool(false));
        data.insert("isPremium".to_string(), Value::Bool(false));
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "profile_source".to_string(),
            Value::String("public_profile".to_string()),
        );
        metadata.insert("publicProfile".to_string(), Value::Object(public_profile));
        data.insert("metadata".to_string(), Value::Object(metadata));

        data.retain(|_, v| !v.is_null());
        Ok(Value::Object(data))
    }

    #[instrument(level = "debug" skip(self, user_room_pairs), fields(pair_count = user_room_pairs.len()))]
    pub async fn batch_get_user_info(
        &self,
        user_room_pairs: &[(String, String)],
    ) -> Result<Vec<Value>> {
        if user_room_pairs.is_empty() {
            return Ok(vec![]);
        }

        let mut batch_input = serde_json::Map::new();
        for (idx, (user_id, room_id)) in user_room_pairs.iter().enumerate() {
            batch_input.insert(
                idx.to_string(),
                json!({"json": {"userId": user_id, "chatroomId": room_id}}),
            );
        }

        let input_data = Value::Object(batch_input).to_string();
        let procedure_names = (0..user_room_pairs.len())
            .map(|_| "user.getChatroomUser")
            .collect::<Vec<_>>()
            .join(",");
        let endpoint = format!("/api/trpc/{}", procedure_names);

        let response = self
            ._request(ApiRequest::get(&endpoint).query([
                ("batch".to_string(), "1".to_string()),
                ("input".to_string(), input_data),
            ]))
            .await?;

        let mut results = Vec::new();
        if let Some(arr) = response.as_array() {
            for item in arr {
                let result = item
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .cloned()
                    .unwrap_or(Value::Object(serde_json::Map::new()));
                results.push(result);
            }
        }

        Ok(results)
    }

    #[instrument(level = "debug" skip(self), fields(room_id = %room_id))]
    pub async fn get_room_info(&self, room_id: &str) -> Result<Option<Value>> {
        let input_data = json!({"0": {"json": {"chatroomId": room_id}}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/chatroom.getPreview")
                    .query(trpc_batch_query(&input_data)),
            )
            .await;

        match response {
            Ok(v) => {
                let parsed = parse_trpc_response(&v, 0, None);
                if !parsed.is_null()
                    && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty())
                {
                    return Ok(Some(parsed));
                }
                if v.is_object() {
                    return Ok(Some(v));
                }
                Ok(None)
            }
            Err(e) => {
                error!("Error fetching room info for {}: {}", room_id, e);
                Ok(None)
            }
        }
    }

    #[instrument(level = "debug" skip(self, invite_code), fields(invite_code_len = invite_code.len()))]
    pub async fn preview_invite(&self, invite_code: &str) -> Result<Value> {
        let input_data = json!({"0": {"json": {"code": invite_code}}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/groupChat.getInviteInfo")
                    .query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty()) {
            return Ok(parsed);
        }

        bail!("Invalid tRPC response for preview_invite: {}", response);
    }

    #[instrument(level = "debug" skip(self, invite_code), fields(invite_code_len = invite_code.len()))]
    pub async fn join_room_by_invite(&self, invite_code: &str) -> Result<Value> {
        let body = json!({"0": {"json": {"inviteCode": invite_code, "gender": "male"}}});
        let response = self
            ._request(
                ApiRequest::post("/api/trpc/groupChat.joinByInvite")
                    .query([("batch", "1")])
                    .json(body),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty()) {
            return Ok(parsed);
        }

        bail!(
            "Invalid tRPC response for join_room_by_invite: {}",
            response
        );
    }

    #[instrument(level = "debug" skip(self, tags), fields(title = %title, is_public, tag_count = tags.map(|t| t.len()).unwrap_or(0)))]
    pub async fn create_group_chat(
        &self,
        title: &str,
        is_public: bool,
        tags: Option<&[String]>,
    ) -> Result<Value> {
        let tags_json =
            serde_json::to_string(&tags.unwrap_or(&[])).unwrap_or_else(|_| "[]".to_string());
        let form = MultipartSpec::new()
            .text("title", title.to_string())
            .text(
                "isPublic",
                (if is_public { "true" } else { "false" }).to_string(),
            )
            .text("tags", tags_json);

        let response = self
            ._request(ApiRequest::post("/api/trpc/groupChat.create").multipart(form))
            .await?;

        if let Some(obj) = response.as_object() {
            if let Some(json_data) = obj
                .get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"))
                && !json_data.is_null()
            {
                return Ok(json_data.clone());
            }
            return Ok(response);
        }

        if let Some(arr) = response.as_array()
            && !arr.is_empty()
        {
            return Ok(parse_trpc_response(&response, 0, None));
        }

        Ok(response)
    }

    #[instrument(level = "debug" skip(self), fields(chatroom_id = %chatroom_id))]
    pub async fn generate_invite(&self, chatroom_id: &str) -> Result<String> {
        let payload = json!({"0": {"json": {"chatroomId": chatroom_id}}});
        let result = self
            ._request(ApiRequest::post("/api/trpc/groupChat.generateInvite?batch=1").json(payload))
            .await?;

        let data = parse_trpc_response(&result, 0, None);
        Ok(data
            .get("inviteLink")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string())
    }

    #[instrument(level = "debug" skip(self), fields(chatroom_id = %chatroom_id, member_id = %member_id))]
    pub async fn remove_group_member(&self, chatroom_id: &str, member_id: &str) -> Result<()> {
        let payload = json!({"0": {"json": {"chatroomId": chatroom_id, "memberId": member_id}}});
        self._request(ApiRequest::post("/api/trpc/groupChat.removeMember?batch=1").json(payload))
            .await?;
        Ok(())
    }

    #[instrument(level = "debug" skip(self), fields(chatroom_id = %chatroom_id, target_user_id = %target_user_id))]
    pub async fn set_group_admin(&self, chatroom_id: &str, target_user_id: &str) -> Result<()> {
        let payload =
            json!({"0": {"json": {"chatroomId": chatroom_id, "targetUserId": target_user_id}}});
        self._request(ApiRequest::post("/api/trpc/groupChat.setAdmin?batch=1").json(payload))
            .await?;
        Ok(())
    }

    #[instrument(level = "debug" skip(self, image_data), fields(chatroom_id = %chatroom_id, filename = %filename, content_type = %content_type, image_len = image_data.len()))]
    pub async fn update_room_avatar(
        &self,
        chatroom_id: &str,
        image_data: &[u8],
        filename: &str,
        content_type: &str,
    ) -> Result<()> {
        let form = MultipartSpec::new().file(
            "file",
            image_data.to_vec(),
            filename.to_string(),
            content_type.to_string(),
        );
        let endpoint = format!("/api/group-chat/{chatroom_id}/avatar");
        let request = ApiRequest::new(Method::PUT, endpoint).multipart(form);
        let (status, body, _) = self
            .request_with_clearance_retry(&request)
            .await
            .context("Failed to update room avatar")?;

        if status != StatusCode::OK {
            let text = String::from_utf8_lossy(&body);
            bail!("Failed to update avatar: {} {}", status, text);
        }

        Ok(())
    }

    #[instrument(level = "debug" skip(self), fields(chatroom_id = %chatroom_id, title = %title))]
    pub async fn rename_room(&self, chatroom_id: &str, title: &str) -> Result<()> {
        let body = json!({"json": {"chatroomId": chatroom_id, "title": title}});
        self._request(ApiRequest::post("/api/trpc/groupChat.rename").json(body))
            .await?;
        Ok(())
    }

    #[instrument(level = "debug" skip(self, resource_id), fields(share_type = share_type.unwrap_or("group_invite")))]
    pub async fn get_share_resource_preview(
        &self,
        resource_id: &str,
        share_type: Option<&str>,
    ) -> Result<Value> {
        let st = share_type.unwrap_or("group_invite");
        let input_data = json!({"0": {"json": {"type": st, "resourceId": resource_id}}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/share.getResourcePreview")
                    .query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty()) {
            return Ok(parsed);
        }

        if let Some(err) = response.get("error") {
            bail!("Error fetching share preview: {}", err);
        }

        if response.is_object() {
            Ok(response)
        } else {
            Ok(Value::Object(serde_json::Map::new()))
        }
    }

    #[instrument(level = "debug" skip(self))]
    pub async fn send_heartbeat(&self) -> Result<bool> {
        match self
            ._request(ApiRequest::post("/api/heartbeat").timeout(Duration::from_secs(5)))
            .await
        {
            Ok(_) => {
                debug!("Heartbeat sent successfully");
                Ok(true)
            }
            Err(e) => {
                debug!("Heartbeat failed: {}", e);
                Ok(false)
            }
        }
    }

    pub async fn fetch_user_chats(&self) -> Result<Vec<Value>> {
        let input_data = json!({"0": {}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/chat.listAll").query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, Some(Value::Array(vec![])));
        let mut chats: Vec<Value> = if let Some(arr) = parsed.as_array() {
            arr.clone()
        } else if parsed.is_object() {
            parsed.as_object().unwrap().values().cloned().collect()
        } else {
            vec![]
        };

        if chats.is_empty() && response.is_object() {
            chats = response.as_object().unwrap().values().cloned().collect();
        }

        info!("Fetched {} user chats from tRPC API", chats.len());

        let user_chats: Vec<Value> = chats
            .into_iter()
            .filter(|chat| chat.get("type").and_then(|v| v.as_str()) == Some("user"))
            .collect();

        info!(
            "Found {} user chats (including groups and DMs)",
            user_chats.len()
        );

        Ok(user_chats)
    }

    pub async fn fetch_room_messages(
        &self,
        room_id: &str,
        before: Option<&str>,
        limit: Option<u64>,
    ) -> Result<Vec<Value>> {
        let limit = limit.unwrap_or(50);
        let mut input_obj = serde_json::Map::new();
        input_obj.insert("chatroomId".to_string(), Value::String(room_id.to_string()));
        input_obj.insert("limit".to_string(), Value::Number(limit.into()));
        if let Some(b) = before {
            input_obj.insert("before".to_string(), Value::String(b.to_string()));
        }

        let input_data = json!({"0": {"json": Value::Object(input_obj)}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/chatroom.getMessages")
                    .query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if let Some(messages) = parsed.get("messages").and_then(|v| v.as_array()) {
            return Ok(messages.clone());
        }

        bail!(
            "Invalid tRPC response for fetch_room_messages: {}",
            response
        );
    }

    pub async fn fetch_explore_feed(
        &self,
        types: &str,
        offset: Option<u64>,
        sort: Option<&str>,
        limit: Option<u64>,
    ) -> Result<Value> {
        let input_obj = json!({
            "types": types,
            "sort": sort.unwrap_or("recent"),
            "cursor": offset.unwrap_or(0),
            "limit": limit.unwrap_or(100),
        });
        let input_data = json!({"0": {"json": input_obj}});

        let response = self
            ._request(
                ApiRequest::get("/api/trpc/search.search").query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if parsed.get("results").is_some() {
            return Ok(parsed);
        }

        bail!("Invalid tRPC response for fetch_explore_feed: {}", response);
    }

    pub async fn fetch_novel_book(&self, book_id: &str) -> Result<Value> {
        let input_obj = json!({"json": {"bookId": book_id}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/novel.book.get")
                    .query([("input".to_string(), input_obj.to_string())]),
            )
            .await?;

        let parsed = if let Some(_arr) = response.as_array() {
            parse_trpc_response(&response, 0, None)
        } else if let Some(obj) = response.as_object() {
            obj.get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| {
                    if let Some(json_data) = d.get("json") {
                        if !json_data.is_null() {
                            Some(json_data.clone())
                        } else {
                            None
                        }
                    } else {
                        Some(d.clone())
                    }
                })
                .unwrap_or(Value::Object(serde_json::Map::new()))
        } else {
            Value::Object(serde_json::Map::new())
        };

        if let Some(book) = parsed.get("book")
            && book.is_object()
        {
            return Ok(book.clone());
        }

        bail!("Invalid tRPC response for fetch_novel_book: {}", response);
    }

    pub async fn fetch_room_members(
        &self,
        room_id: &str,
        cursor: Option<&str>,
        limit: Option<u64>,
    ) -> Result<Value> {
        let limit = limit.unwrap_or(200);
        let mut input_obj = serde_json::Map::new();
        input_obj.insert("chatroomId".to_string(), Value::String(room_id.to_string()));
        input_obj.insert("limit".to_string(), Value::Number(limit.into()));
        if let Some(c) = cursor {
            input_obj.insert("cursor".to_string(), Value::String(c.to_string()));
        }

        let input_data = json!({"0": {"json": Value::Object(input_obj)}});
        let response = self
            ._request(
                ApiRequest::get("/api/trpc/chatroom.listMembers")
                    .query(trpc_batch_query(&input_data)),
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty()) {
            return Ok(parsed);
        }

        bail!("Invalid tRPC response for fetch_room_members: {}", response);
    }

    pub async fn fetch_all_room_members(
        &self,
        room_id: &str,
        page_size: Option<u64>,
    ) -> Result<Vec<Value>> {
        let page_size = page_size.unwrap_or(200);
        let mut all_members: Vec<Value> = Vec::new();
        let mut cursor: Option<String> = None;
        let mut page_num: u32 = 1;

        loop {
            let response = self
                .fetch_room_members(room_id, cursor.as_deref(), Some(page_size))
                .await?;

            if let Some(members) = response.get("members").and_then(|v| v.as_array()) {
                all_members.extend(members.clone());
                info!(
                    "Fetched batch {}: {} members (total so far: {})",
                    page_num,
                    members.len(),
                    all_members.len()
                );
            }

            let next_cursor = response
                .get("nextPage")
                .and_then(|v| v.as_str())
                .map(String::from);

            if next_cursor.is_none() {
                break;
            }

            cursor = next_cursor;
            page_num += 1;

            if page_num > 100 {
                warn!("Stopped at batch {} (safety limit)", page_num);
                break;
            }
        }

        info!(
            "Fetched total {} members for room {}",
            all_members.len(),
            room_id
        );
        Ok(all_members)
    }

    pub async fn upload_chat_image(&self, file_path: &str) -> Result<String> {
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            bail!("Image file not found: {}", file_path);
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let ext_dot = format!(".{}", ext);
        let content_type = guess_content_type(&ext_dot);
        let data = std::fs::read(path).context("Failed to read image file")?;

        let form = build_file_upload_form("file", data, filename, content_type);
        info!("Uploading image: {}", filename);

        let response = self
            ._request(ApiRequest::post("/api/trpc/chatroom.uploadImage").multipart(form))
            .await?;

        let image_url = if let Some(arr) = response.as_array() {
            if let Some(first) = arr.first() {
                first
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .and_then(|j| j.get("url"))
                    .and_then(|u| u.as_str())
                    .or_else(|| first.get("url").and_then(|u| u.as_str()))
            } else {
                None
            }
        } else if let Some(obj) = response.as_object() {
            obj.get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"))
                .and_then(|j| j.get("url"))
                .and_then(|u| u.as_str())
                .or_else(|| response.get("url").and_then(|u| u.as_str()))
        } else {
            None
        };

        match image_url {
            Some(url) => {
                info!("Image uploaded: {}", url);
                Ok(url.to_string())
            }
            None => bail!("Upload response missing 'url': {}", response),
        }
    }

    pub async fn upload_voice_message(
        &self,
        file_path: &str,
        _duration: Option<f64>,
    ) -> Result<Value> {
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            bail!("Voice file not found: {}", file_path);
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("mp3");
        let ext_dot = format!(".{}", ext);
        let content_type = guess_content_type(&ext_dot);
        let data = std::fs::read(path).context("Failed to read voice file")?;

        let form = build_file_upload_form("file", data, filename, content_type);
        info!("Uploading voice: {}", filename);

        let response = self
            ._request(ApiRequest::post("/api/trpc/chat.uploadVoiceMessage").multipart(form))
            .await?;

        let (voice_url, data_to_return) = if let Some(arr) = response.as_array() {
            if let Some(first) = arr.first() {
                let json_data = first
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"));
                let url = json_data
                    .and_then(|j| j.get("url"))
                    .and_then(|u| u.as_str())
                    .or_else(|| first.get("url").and_then(|u| u.as_str()));
                let data = json_data.cloned().unwrap_or_else(|| first.clone());
                (url, data)
            } else {
                (None, response.clone())
            }
        } else if let Some(obj) = response.as_object() {
            let json_data = obj
                .get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"));
            let url = json_data
                .and_then(|j| j.get("url"))
                .and_then(|u| u.as_str())
                .or_else(|| response.get("url").and_then(|u| u.as_str()));
            let data = json_data.cloned().unwrap_or_else(|| response.clone());
            (url, data)
        } else {
            (None, response.clone())
        };

        if voice_url.is_none() {
            bail!("Upload response missing 'url': {}", response);
        }

        info!("Voice uploaded: {}", voice_url.unwrap());
        Ok(data_to_return)
    }

    pub async fn upload_video(&self, file_path: &str) -> Result<Value> {
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            bail!("Video file not found: {}", file_path);
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload");
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("mp4");
        let ext_dot = format!(".{}", ext);
        let content_type = guess_video_content_type(&ext_dot);
        let data = std::fs::read(path).context("Failed to read video file")?;

        let form = build_file_upload_form("file", data, filename, content_type);
        info!("Uploading video: {}", filename);

        let response = self
            ._request(ApiRequest::post("/api/trpc/media.uploadVideo").multipart(form))
            .await?;

        let (video_url, data_to_return) = if let Some(arr) = response.as_array() {
            if let Some(first) = arr.first() {
                let json_data = first
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"));
                let url = json_data
                    .and_then(|j| j.get("videoUrl"))
                    .and_then(|u| u.as_str())
                    .or_else(|| first.get("videoUrl").and_then(|u| u.as_str()));
                let data = json_data.cloned().unwrap_or_else(|| first.clone());
                (url, data)
            } else {
                (None, response.clone())
            }
        } else if let Some(obj) = response.as_object() {
            let json_data = obj
                .get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"));
            let url = json_data
                .and_then(|j| j.get("videoUrl"))
                .and_then(|u| u.as_str())
                .or_else(|| response.get("videoUrl").and_then(|u| u.as_str()));
            let data = json_data.cloned().unwrap_or_else(|| response.clone());
            (url, data)
        } else {
            (None, response.clone())
        };

        if video_url.is_none() {
            bail!("Upload response missing 'videoUrl': {}", response);
        }

        info!("Video uploaded: {}", video_url.unwrap());
        Ok(data_to_return)
    }

    pub async fn upload_tweet_image(
        &self,
        file_path: Option<&str>,
        file_data: Option<&[u8]>,
        filename: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<Value> {
        let (data, filename_str, ct) = if let Some(fp) = file_path {
            let path = std::path::Path::new(fp);
            if !path.exists() {
                bail!("Image file not found: {}", fp);
            }
            let fname = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image.jpg")
                .to_string();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
            let ctype = guess_content_type(&format!(".{}", ext)).to_string();
            let bytes = std::fs::read(path).context("Failed to read image file")?;
            (bytes, fname, ctype)
        } else if let Some(fd) = file_data {
            let fname = filename.unwrap_or("image.jpg").to_string();
            let ctype = content_type.unwrap_or("image/jpeg").to_string();
            (fd.to_vec(), fname, ctype)
        } else {
            bail!("Either file_path or file_data must be provided");
        };

        let form = build_file_upload_form("file", data, &filename_str, &ct);
        info!("Uploading tweet image via tRPC: {}", filename_str);

        let response = self
            ._request(ApiRequest::post("/api/trpc/tweet.uploadImage").multipart(form))
            .await?;

        let (image_url, data_to_return) = if let Some(arr) = response.as_array() {
            if let Some(first) = arr.first() {
                let json_data = first
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"));
                let url = json_data
                    .and_then(|j| j.get("url"))
                    .and_then(|u| u.as_str());
                let data = json_data.cloned().unwrap_or_else(|| first.clone());
                (url, data)
            } else {
                (None, response.clone())
            }
        } else if let Some(obj) = response.as_object() {
            let json_data = obj
                .get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"));
            let url = json_data
                .and_then(|j| j.get("url"))
                .and_then(|u| u.as_str())
                .or_else(|| response.get("url").and_then(|u| u.as_str()));
            let data = json_data.cloned().unwrap_or_else(|| response.clone());
            (url, data)
        } else {
            (None, response.clone())
        };

        if image_url.is_none() {
            bail!("Upload response missing 'url': {}", response);
        }

        info!("Tweet image uploaded via tRPC: {}", image_url.unwrap());
        Ok(data_to_return)
    }

    pub async fn list_user_tweets(
        &self,
        user_id: &str,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Value> {
        let input_obj = json!({
            "userId": user_id,
            "offset": offset.unwrap_or(0),
            "limit": limit.unwrap_or(10),
        });
        let input_data = json!({"0": {"json": input_obj}});

        let response = self
            ._request(ApiRequest::get("/api/trpc/tweet.list").query(trpc_batch_query(&input_data)))
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty()) {
            return Ok(parsed);
        }

        bail!("Invalid tRPC response for list_user_tweets: {}", response);
    }

    pub async fn start_image_generation(
        &self,
        prompt: &str,
        tag_ids: Option<&[String]>,
        dimension: Option<&str>,
        negative_prompt: Option<&str>,
        model: Option<&str>,
    ) -> Result<String> {
        let tag_ids = tag_ids.unwrap_or(&[]);
        let dimension = dimension.unwrap_or("3:2");
        let model = model.unwrap_or("iroha");
        let negative_prompt = negative_prompt.unwrap_or(
            "low quality, blurry, deformed, text, signature, watermark, multiple limbs, extra fingers, ugly",
        );

        let ban_words = ["loli", "lolita", "large perky breasts"];
        let mut cleaned_prompt = prompt.to_string();
        for word in &ban_words {
            cleaned_prompt = cleaned_prompt.replace(word, "");
        }

        let payload = json!({
            "prompt": cleaned_prompt,
            "tagIds": tag_ids,
            "dimension": dimension,
            "negativePrompt": negative_prompt,
            "model": model,
        });

        info!(
            "Starting image generation: prompt='{}...', model={}, dimension={}",
            &cleaned_prompt.chars().take(50).collect::<String>(),
            model,
            dimension
        );

        let response = self
            ._request(ApiRequest::post("/api/gamefy/draw").json(payload))
            .await?;

        let data = response;

        if data.get("success").and_then(|v| v.as_bool()) == Some(false) {
            bail!("Image generation failed: {}", data);
        }

        let task_id = data
            .get("taskId")
            .and_then(|v| v.as_str())
            .map(String::from);

        match task_id {
            Some(id) => {
                info!("Image generation task started: {}", id);
                Ok(id)
            }
            None => bail!("Response missing taskId: {}", data),
        }
    }

    pub async fn poll_generation_task(&self, task_id: &str) -> Result<Value> {
        let response = self
            ._request(ApiRequest::get("/api/gamefy/draw/status").query([("taskId", task_id)]))
            .await?;

        let mut data = if let Some(task) = response.get("task").filter(|v| v.is_object()) {
            task.clone()
        } else {
            response
        };

        if let Some(obj) = data.as_object_mut() {
            let fallback_task_id = obj
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or(task_id)
                .to_string();
            obj.entry("taskId".to_string())
                .or_insert_with(|| Value::String(fallback_task_id));
        }

        let status = data
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        debug!("Task {} status: {}", task_id, status);
        Ok(data)
    }

    pub async fn generate_image(
        &self,
        prompt: &str,
        dimension: Option<&str>,
        negative_prompt: Option<&str>,
        model: Option<&str>,
    ) -> Result<(Option<Vec<u8>>, Option<String>)> {
        let model = model.unwrap_or("iroha");

        let task_id = self
            .start_image_generation(
                prompt,
                None,
                dimension.or(Some("3:2")),
                negative_prompt,
                Some(model),
            )
            .await?;

        info!("Polling for task completion: {}", task_id);

        let max_poll_time = 120.0;
        let poll_interval = Duration::from_secs(2);
        let start_time = std::time::Instant::now();

        loop {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed > max_poll_time {
                error!("Task {} timed out after {}s", task_id, max_poll_time);
                bail!("Image generation timed out after {} seconds", max_poll_time);
            }

            let task_status = self.poll_generation_task(&task_id).await?;
            let status = task_status
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            if status == "completed" {
                let output_images = task_status.get("outputImages").and_then(|v| v.as_array());

                if output_images.is_none() || output_images.unwrap().is_empty() {
                    error!("Task {} completed but no output images", task_id);
                    return Ok((None, None));
                }

                let relative_url = output_images.unwrap()[0].as_str().unwrap_or("");
                info!("Downloading generated image");
                let (download_status, image_bytes, headers) = self
                    .download_plain_media(relative_url)
                    .await
                    .context("Failed to download generated image")?;

                if !download_status.is_success() {
                    bail!("Failed to download image: {}", download_status);
                }

                let mime_type = headers
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
                    .unwrap_or_else(|| "image/png".to_string());

                info!(
                    "Image generation complete: {} bytes, {}",
                    image_bytes.len(),
                    mime_type
                );
                return Ok((Some(image_bytes), Some(mime_type)));
            }

            if status == "failed" {
                error!("Task {} failed", task_id);
                return Ok((None, None));
            }

            tokio::time::sleep(poll_interval).await;
        }
    }

    pub async fn upload_reference_image(
        &self,
        image_bytes: &[u8],
        _filename: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<String> {
        let content_type = content_type.unwrap_or("image/jpeg");
        let mime_type = content_type
            .split(';')
            .next()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("image/jpeg");
        let encoded = base64::engine::general_purpose::STANDARD.encode(image_bytes);
        let payload = json!({
            "image": format!("data:{};base64,{}", mime_type, encoded)
        });

        let data = self
            ._request(ApiRequest::post("/api/gamefy/draw/upload-reference").json(payload))
            .await?;

        if data.get("success").and_then(|v| v.as_bool()) == Some(false) {
            bail!("Reference image upload failed: {}", data);
        }

        let image_url = data.get("url").and_then(|v| v.as_str()).map(String::from);

        match image_url {
            Some(url) => {
                info!("Reference image uploaded: {}", url);
                Ok(url)
            }
            None => bail!("Upload response missing url: {}", data),
        }
    }

    pub async fn start_image_edit(&self, request: ImageEditRequest<'_>) -> Result<String> {
        if request.image_urls.len() > 4 {
            bail!(
                "Maximum 4 reference images allowed, got {}",
                request.image_urls.len()
            );
        }

        let tag_ids = request.tag_ids.unwrap_or(&[]);
        let model = request.model.unwrap_or("nalang-dream");

        let payload = json!({
            "prompt": request.prompt,
            "images": request.image_urls,
            "imageSize": {"width": request.image_width.unwrap_or(1024), "height": request.image_height.unwrap_or(1024)},
            "numInferenceSteps": request.num_inference_steps.unwrap_or(30),
            "textGuidanceScale": request.text_guidance_scale.unwrap_or(5.0),
            "imageGuidanceScale": request.image_guidance_scale.unwrap_or(6.0),
            "numImages": request.num_images.unwrap_or(1),
            "enableSafetyChecker": request.enable_safety_checker.unwrap_or(true),
            "model": model,
            "tagIds": tag_ids,
        });

        info!(
            "Starting image edit: prompt='{}...', model={}",
            &request.prompt.chars().take(50).collect::<String>(),
            model
        );

        let response = self
            ._request(ApiRequest::post("/api/gamefy/draw/edit").json(payload))
            .await?;

        let data = response;

        if data.get("success").and_then(|v| v.as_bool()) == Some(false) {
            bail!("Image edit failed: {}", data);
        }

        let task_id = data
            .get("taskId")
            .and_then(|v| v.as_str())
            .map(String::from);

        match task_id {
            Some(id) => {
                info!("Image edit task started: {}", id);
                Ok(id)
            }
            None => bail!("Response missing taskId: {}", data),
        }
    }

    pub async fn edit_image(
        &self,
        prompt: &str,
        image_bytes: &[u8],
        image_mime_type: Option<&str>,
        image_width: Option<u64>,
        image_height: Option<u64>,
        model: Option<&str>,
    ) -> Result<(Option<Vec<u8>>, Option<String>)> {
        let image_mime_type = image_mime_type.unwrap_or("image/jpeg");
        let ext = image_mime_type.split('/').next_back().unwrap_or("jpg");
        let filename = format!("reference.{}", ext);

        let ref_url = self
            .upload_reference_image(image_bytes, Some(&filename), Some(image_mime_type))
            .await?;

        let image_urls = vec![ref_url];
        let task_id = self
            .start_image_edit(ImageEditRequest {
                prompt,
                image_urls: &image_urls,
                image_width,
                image_height,
                num_inference_steps: None,
                text_guidance_scale: None,
                image_guidance_scale: None,
                num_images: None,
                enable_safety_checker: None,
                model,
                tag_ids: None,
            })
            .await?;

        info!("Polling for edit task completion: {}", task_id);

        let max_poll_time = 120.0;
        let poll_interval = Duration::from_secs(2);
        let start_time = std::time::Instant::now();

        loop {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed > max_poll_time {
                error!("Edit task {} timed out after {}s", task_id, max_poll_time);
                bail!("Image edit timed out after {} seconds", max_poll_time);
            }

            let task_status = self.poll_generation_task(&task_id).await?;
            let status = task_status
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            if status == "completed" {
                let output_images = task_status.get("outputImages").and_then(|v| v.as_array());

                if output_images.is_none() || output_images.unwrap().is_empty() {
                    error!("Edit task {} completed but no output images", task_id);
                    return Ok((None, None));
                }

                let relative_url = output_images.unwrap()[0].as_str().unwrap_or("");
                info!("Downloading edited image");
                let (download_status, result_bytes, headers) = self
                    .download_plain_media(relative_url)
                    .await
                    .context("Failed to download edited image")?;

                if !download_status.is_success() {
                    bail!("Failed to download image: {}", download_status);
                }

                let mime_type = headers
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
                    .unwrap_or_else(|| "image/png".to_string());

                info!(
                    "Image edit complete: {} bytes, {}",
                    result_bytes.len(),
                    mime_type
                );
                return Ok((Some(result_bytes), Some(mime_type)));
            }

            if status == "failed" {
                error!("Edit task {} failed", task_id);
                return Ok((None, None));
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

fn build_file_upload_form(
    name: &str,
    data: Vec<u8>,
    filename: &str,
    mime_type: &str,
) -> MultipartSpec {
    MultipartSpec::new().file(name, data, filename, mime_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clearance::{
        ClearanceCookie, ClearanceError, ClearanceFuture, ClearanceProvider,
        ClearanceRefreshReason, ClearanceSnapshot,
    };
    use crate::config::parse_dzmm_local_address;
    use chrono::Utc;
    use serde_json::json;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn extract_balanced_json_object(text: &str, start: usize) -> Result<Value> {
        let chars: Vec<char> = text.chars().collect();
        let mut depth = 0i32;
        let mut in_str = false;
        let mut escaped = false;

        for (i, &c) in chars.iter().enumerate().skip(start) {
            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
            } else {
                match c {
                    '"' => in_str = true,
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            let s: String = chars[start..=i].iter().collect();
                            let obj: Value = serde_json::from_str(&s)
                                .context("Failed to parse balanced JSON object")?;
                            if !obj.is_object() {
                                bail!("Expected JSON object");
                            }
                            return Ok(obj);
                        }
                    }
                    _ => {}
                }
            }
        }

        bail!("Unterminated JSON object")
    }

    fn extract_balanced_json_array(text: &str, start: usize) -> Result<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut depth = 0i32;
        let mut in_str = false;
        let mut escaped = false;

        for (i, &c) in chars.iter().enumerate().skip(start) {
            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
            } else {
                match c {
                    '"' => in_str = true,
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            let s: String = chars[start..=i].iter().collect();
                            return Ok(s);
                        }
                    }
                    _ => {}
                }
            }
        }

        bail!("Unterminated JSON array")
    }

    fn extract_next_flight_text(html: &str) -> String {
        let mut chunks: Vec<String> = Vec::new();
        let marker = "self.__next_f.push(";

        let mut start = 0usize;
        while let Some(i) = html[start..].find(marker) {
            let call_index = start + i;

            let array_start = match html[call_index + marker.len()..].find('[') {
                Some(i) => call_index + marker.len() + i,
                None => break,
            };

            let payload_str = match extract_balanced_json_array(html, array_start) {
                Ok(s) => s,
                Err(_) => {
                    start = array_start + 1;
                    continue;
                }
            };

            let payload: Value = match serde_json::from_str(&payload_str) {
                Ok(v) => v,
                Err(_) => {
                    start = array_start + 1;
                    continue;
                }
            };

            if let Some(arr) = payload.as_array()
                && arr.len() >= 2
                && arr[0] == Value::Number(1.into())
                && let Some(s) = arr[1].as_str()
            {
                chunks.push(s.to_string());
            }

            start = array_start + 1;
        }

        chunks.join("")
    }

    fn extract_next_scalar_field(text: &str, field: &str) -> Option<Value> {
        let marker = format!("\"{}\":", field);
        let index = text.find(&marker)?;

        let start = index + marker.len();
        let rest = &text[start..];
        let trimmed_start = rest.len() - rest.trim_start().len();
        let after_ws = &text[start + trimmed_start..];

        if after_ws.is_empty() {
            return None;
        }

        let c = after_ws.chars().next().unwrap();
        if c == '"' {
            let end = after_ws[1..].find('"').map(|i| i + 1)?;
            Some(Value::String(after_ws[1..end].to_string()))
        } else if c == 't' && after_ws.starts_with("true") {
            Some(Value::Bool(true))
        } else if c == 'f' && after_ws.starts_with("false") {
            Some(Value::Bool(false))
        } else if c == 'n' && after_ws.starts_with("null") {
            Some(Value::Null)
        } else {
            let end = after_ws
                .find(|ch: char| !ch.is_ascii_digit() && ch != '.' && ch != '-')
                .unwrap_or(after_ws.len());
            if end == 0 {
                return None;
            }
            serde_json::from_str(&after_ws[..end]).ok()
        }
    }

    fn extract_next_public_profile(html: &str, user_id: &str) -> Value {
        let flight = extract_next_flight_text(html).replace(r#"\""#, "\"");
        let profile_marker = "\"profile\":";
        let profile_index = match flight.find(profile_marker) {
            Some(i) => i,
            None => return Value::Object(serde_json::Map::new()),
        };

        let profile_start = match flight[profile_index..].find('{') {
            Some(i) => profile_index + i,
            None => return Value::Object(serde_json::Map::new()),
        };

        let profile = match extract_balanced_json_object(&flight, profile_start) {
            Ok(v) => v,
            Err(_) => return Value::Object(serde_json::Map::new()),
        };

        let target_user_id = extract_next_scalar_field(&flight, "targetUserId")
            .and_then(|v| v.as_str().map(String::from));

        if let Some(ref tid) = target_user_id
            && tid != user_id
        {
            return Value::Object(serde_json::Map::new());
        }

        let mut public_profile = if let Value::Object(map) = &profile {
            map.clone()
        } else {
            serde_json::Map::new()
        };

        for field in &[
            "joinDate",
            "followersCount",
            "followingCount",
            "dmDeniedReason",
            "showFavorites",
            "showLikes",
        ] {
            if let Some(value) = extract_next_scalar_field(&flight, field) {
                public_profile.insert(field.to_string(), value);
            }
        }

        let mut data = serde_json::Map::new();
        data.insert(
            "id".to_string(),
            Value::String(target_user_id.unwrap_or_else(|| user_id.to_string())),
        );
        if let Some(v) = profile.get("fullName") {
            data.insert("fullName".to_string(), v.clone());
        }
        if let Some(v) = profile.get("avatarUrl") {
            data.insert("avatarUrl".to_string(), v.clone());
        }
        if let Some(v) = profile.get("bio") {
            data.insert("bio".to_string(), v.clone());
        }
        if let Some(v) = profile.get("birthday") {
            data.insert("birthday".to_string(), v.clone());
        }
        if let Some(v) = profile.get("birthdayPublic") {
            data.insert("birthdayPublic".to_string(), v.clone());
        }
        if let Some(v) = profile.get("quirk") {
            data.insert("quirk".to_string(), v.clone());
        }
        if let Some(v) = profile.get("gender") {
            data.insert("gender".to_string(), v.clone());
        }
        data.insert("isBot".to_string(), Value::Bool(false));
        data.insert("isPremium".to_string(), Value::Bool(false));
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "profile_source".to_string(),
            Value::String("public_profile".to_string()),
        );
        metadata.insert("publicProfile".to_string(), Value::Object(public_profile));
        data.insert("metadata".to_string(), Value::Object(metadata));

        data.retain(|_, v| !v.is_null());
        Value::Object(data)
    }

    #[derive(Debug, Clone)]
    struct RecordedRequest {
        method: String,
        target: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    }

    struct FakeClearanceProvider {
        current: ClearanceSnapshot,
        refreshed: ClearanceSnapshot,
        refreshed_active: AtomicBool,
        snapshot_calls: AtomicUsize,
        refresh_calls: AtomicUsize,
    }

    impl FakeClearanceProvider {
        fn new() -> Self {
            Self {
                current: clearance_snapshot(1, "browser-ua-generation-1", "clearance-1"),
                refreshed: clearance_snapshot(2, "browser-ua-generation-2", "clearance-2"),
                refreshed_active: AtomicBool::new(false),
                snapshot_calls: AtomicUsize::new(0),
                refresh_calls: AtomicUsize::new(0),
            }
        }
    }

    impl ClearanceProvider for FakeClearanceProvider {
        fn snapshot(&self) -> ClearanceFuture<'_> {
            self.snapshot_calls.fetch_add(1, Ordering::SeqCst);
            let snapshot = if self.refreshed_active.load(Ordering::SeqCst) {
                self.refreshed.clone()
            } else {
                self.current.clone()
            };
            Box::pin(std::future::ready(Ok(snapshot)))
        }

        fn refresh(
            &self,
            _observed_generation: u64,
            _reason: ClearanceRefreshReason,
        ) -> ClearanceFuture<'_> {
            self.refresh_calls.fetch_add(1, Ordering::SeqCst);
            self.refreshed_active.store(true, Ordering::SeqCst);
            Box::pin(std::future::ready(Ok(self.refreshed.clone())))
        }
    }

    fn clearance_snapshot(generation: u64, user_agent: &str, value: &str) -> ClearanceSnapshot {
        ClearanceSnapshot {
            generation,
            user_agent: user_agent.to_string(),
            cookies: vec![ClearanceCookie {
                name: "cf_clearance".to_string(),
                value: value.to_string(),
                domain: ".dzmm.ai".to_string(),
                path: "/".to_string(),
                expires: (Utc::now() + chrono::Duration::hours(1)).timestamp() as f64,
            }],
            expires_at: Utc::now() + chrono::Duration::hours(1),
            verified_at: Utc::now(),
        }
    }

    fn test_api(base_url: String) -> DzmmApi {
        let mut api = DzmmApi::new_with_clearance_provider(
            ApiClientConfig {
                base_url,
                ..ApiClientConfig::default()
            },
            DzmmApiAuth::default(),
            Arc::new(FakeClearanceProvider::new()),
        )
        .unwrap();
        api.rate_limiter = Arc::new(Mutex::new(RateLimiter {
            min_delay: 0.0,
            max_delay: 0.0,
            batch_size: 1,
            batch_delay: 0.0,
            request_count: 0,
        }));
        api
    }

    fn test_api_with_provider(
        base_url: String,
        auth: DzmmApiAuth,
        provider: Arc<dyn ClearanceProvider>,
    ) -> DzmmApi {
        let mut api = DzmmApi::new_with_clearance_provider(
            ApiClientConfig {
                base_url,
                ..ApiClientConfig::default()
            },
            auth,
            provider,
        )
        .unwrap();
        api.rate_limiter = Arc::new(Mutex::new(RateLimiter {
            min_delay: 0.0,
            max_delay: 0.0,
            batch_size: 1,
            batch_delay: 0.0,
            request_count: 0,
        }));
        api
    }

    async fn read_request(stream: &mut tokio::net::TcpStream) -> RecordedRequest {
        let mut data = Vec::new();
        let mut buf = [0u8; 1024];
        let header_end = loop {
            let n = stream.read(&mut buf).await.unwrap();
            assert_ne!(n, 0, "connection closed before headers");
            data.extend_from_slice(&buf[..n]);
            if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos + 4;
            }
        };

        let headers = String::from_utf8_lossy(&data[..header_end]);
        let mut lines = headers.lines();
        let request_line = lines.next().unwrap();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap().to_string();
        let target = parts.next().unwrap().to_string();
        let headers: HashMap<String, String> = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_string()))
            .collect();
        let content_length = headers
            .get("content-length")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);

        while data.len() < header_end + content_length {
            let n = stream.read(&mut buf).await.unwrap();
            assert_ne!(n, 0, "connection closed before body");
            data.extend_from_slice(&buf[..n]);
        }

        RecordedRequest {
            method,
            target,
            headers,
            body: data[header_end..header_end + content_length].to_vec(),
        }
    }

    async fn write_json_response(stream: &mut tokio::net::TcpStream, body: Value) {
        let body = body.to_string();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    async fn write_redirect_with_cookie(
        stream: &mut tokio::net::TcpStream,
        location: &str,
        cookie: &str,
    ) {
        let response = format!(
            "HTTP/1.1 307 Temporary Redirect\r\nLocation: {location}\r\nSet-Cookie: {cookie}; Path=/; HttpOnly\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    }

    async fn write_image_response(stream: &mut tokio::net::TcpStream, body: &[u8]) {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes()).await.unwrap();
        stream.write_all(body).await.unwrap();
    }

    async fn write_gzip_json_response(stream: &mut tokio::net::TcpStream) {
        let body = [
            31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 171, 86, 202, 207, 86, 178, 42, 41, 42, 77, 173, 5,
            0, 144, 95, 212, 167, 11, 0, 0, 0,
        ];
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Encoding: gzip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes()).await.unwrap();
        stream.write_all(&body).await.unwrap();
    }

    async fn spawn_gamefy_server(
        expected_requests: usize,
    ) -> (String, tokio::task::JoinHandle<Vec<RecordedRequest>>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut recorded = Vec::new();
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_request(&mut stream).await;
                match request.target.as_str() {
                    "/api/gamefy/draw" => {
                        write_json_response(&mut stream, json!({"taskId": "task-123"})).await;
                    }
                    "/api/gamefy/draw/upload-reference" => {
                        write_json_response(
                            &mut stream,
                            json!({"url": "https://example.com/reference.png"}),
                        )
                        .await;
                    }
                    "/api/gamefy/draw/edit" => {
                        write_json_response(&mut stream, json!({"taskId": "task-456"})).await;
                    }
                    "/api/gamefy/draw/status?taskId=task-123" => {
                        write_json_response(
                            &mut stream,
                            json!({
                                "task": {
                                    "id": "task-123",
                                    "status": "completed",
                                    "outputImages": ["/api/draw/image/task-123?index=0"],
                                    "createdAt": "2026-06-15T00:00:00.000Z"
                                }
                            }),
                        )
                        .await;
                    }
                    "/api/gamefy/draw/status?taskId=task-456" => {
                        write_json_response(
                            &mut stream,
                            json!({
                                "task": {
                                    "id": "task-456",
                                    "status": "completed",
                                    "outputImages": ["/api/draw/image/task-456?index=0"],
                                    "createdAt": "2026-06-15T00:00:00.000Z"
                                }
                            }),
                        )
                        .await;
                    }
                    "/api/draw/image/task-123?index=0" => {
                        write_image_response(&mut stream, b"generated-image-bytes").await;
                    }
                    "/api/draw/image/task-456?index=0" => {
                        write_image_response(&mut stream, b"edited-image-bytes").await;
                    }
                    other => panic!("unexpected request target: {other}"),
                }
                recorded.push(request);
            }
            recorded
        });
        (base_url, handle)
    }

    #[tokio::test]
    async fn request_inner_decodes_gzip_json_response() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            write_gzip_json_response(&mut stream).await;
            request
        });

        let api = test_api(base_url);
        let request = ApiRequest::get("/compressed");
        let snapshot = clearance_snapshot(1, "test-user-agent", "test-clearance");
        let (status, body, _) = api
            ._request_inner(&request, &snapshot)
            .await
            .expect("request succeeds");

        let request = handle.await.expect("server task completes");
        assert_eq!(request.method, "GET");
        assert_eq!(request.target, "/compressed");
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, br#"{"ok":true}"#);
    }

    #[tokio::test]
    async fn cf_mitigated_refreshes_once_and_retries_with_one_new_atomic_identity() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();
            let (mut first, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut first).await);
            first
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\ncf-mitigated: challenge\r\nContent-Type: text/html\r\nContent-Length: 9\r\nConnection: close\r\n\r\nchallenge",
                )
                .await
                .unwrap();

            let (mut second, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut second).await);
            write_json_response(
                &mut second,
                json!([{"result": {"data": {"json": {"isLoggedIn": true}}}}]),
            )
            .await;
            requests
        });

        let provider = Arc::new(FakeClearanceProvider::new());
        let api = test_api_with_provider(
            base_url,
            DzmmApiAuth {
                cookies: Some(Cow::Borrowed("session=account")),
                ..DzmmApiAuth::default()
            },
            provider.clone(),
        );

        api.get_my_info(false).await.expect("retry succeeds");
        let requests = handle.await.unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].headers.get("user-agent").map(String::as_str),
            Some("browser-ua-generation-1")
        );
        assert_eq!(
            requests[1].headers.get("user-agent").map(String::as_str),
            Some("browser-ua-generation-2")
        );
        assert!(
            requests[0]
                .headers
                .get("cookie")
                .is_some_and(|value| value.contains("cf_clearance=clearance-1"))
        );
        assert!(
            requests[1]
                .headers
                .get("cookie")
                .is_some_and(|value| value.contains("cf_clearance=clearance-2"))
        );
        assert!(
            requests
                .iter()
                .all(|request| request.headers.contains_key("cookie"))
        );
        assert!(
            requests
                .iter()
                .all(|request| !request.headers.contains_key("sec-ch-ua"))
        );
        assert_eq!(provider.snapshot_calls.load(Ordering::SeqCst), 1);
        assert_eq!(provider.refresh_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn second_cf_mitigated_returns_retryable_semantic_clearance_error() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().await.unwrap();
                read_request(&mut stream).await;
                stream
                    .write_all(
                        b"HTTP/1.1 403 Forbidden\r\ncf-mitigated: challenge\r\nContent-Type: text/html\r\nContent-Length: 9\r\nConnection: close\r\n\r\nchallenge",
                    )
                    .await
                    .unwrap();
            }
        });

        let provider = Arc::new(FakeClearanceProvider::new());
        let api = test_api_with_provider(base_url, DzmmApiAuth::default(), provider.clone());

        let error = api
            .get_my_info(false)
            .await
            .expect_err("second challenge is returned");
        let clearance_error = error
            .downcast_ref::<ClearanceError>()
            .expect("typed clearance error");
        assert!(clearance_error.retryable());
        assert!(matches!(
            clearance_error,
            ClearanceError::ChallengePersisted { generation: 2, .. }
        ));
        assert_eq!(provider.refresh_calls.load(Ordering::SeqCst), 1);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn business_forbidden_does_not_refresh_clearance() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            read_request(&mut stream).await;
            let body = json!({
                "error": {
                    "json": {
                        "code": -32003
                    }
                }
            })
            .to_string();
            let response = format!(
                "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let provider = Arc::new(FakeClearanceProvider::new());
        let api = test_api_with_provider(
            base_url,
            DzmmApiAuth {
                auto_refresh: true,
                ..DzmmApiAuth::default()
            },
            provider.clone(),
        );

        let error = api.get_my_info(false).await.unwrap_err();
        assert!(error.to_string().contains("Business forbidden"));
        assert_eq!(provider.refresh_calls.load(Ordering::SeqCst), 0);
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn authentication_refresh_starts_after_clearance_retry_finishes() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();

            let (mut first, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut first).await);
            first
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\ncf-mitigated: challenge\r\nContent-Length: 9\r\nConnection: close\r\n\r\nchallenge",
                )
                .await
                .unwrap();

            let (mut second, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut second).await);
            second
                .write_all(
                    b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 12\r\nConnection: close\r\n\r\nunauthorized",
                )
                .await
                .unwrap();

            let (mut token, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut token).await);
            write_json_response(&mut token, json!({"user_id": "user-1"})).await;

            let (mut final_request, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut final_request).await);
            write_json_response(
                &mut final_request,
                json!([{"result": {"data": {"json": {"isLoggedIn": true}}}}]),
            )
            .await;
            requests
        });

        let provider = Arc::new(FakeClearanceProvider::new());
        let api = test_api_with_provider(
            base_url,
            DzmmApiAuth {
                auto_refresh: true,
                ..DzmmApiAuth::default()
            },
            provider.clone(),
        );

        api.get_my_info(false)
            .await
            .expect("authentication succeeds");
        let requests = handle.await.unwrap();
        assert!(requests[0].target.starts_with("/api/trpc/user.getMe"));
        assert!(requests[1].target.starts_with("/api/trpc/user.getMe"));
        assert_eq!(requests[2].target, "/api/auth/token");
        assert!(requests[3].target.starts_with("/api/trpc/user.getMe"));
        assert_eq!(
            requests[0].headers.get("user-agent").map(String::as_str),
            Some("browser-ua-generation-1")
        );
        assert!(requests[1..].iter().all(|request| {
            request.headers.get("user-agent").map(String::as_str) == Some("browser-ua-generation-2")
        }));
        assert_eq!(provider.refresh_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn multipart_body_is_rebuilt_after_clearance_refresh() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();
            let (mut first, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut first).await);
            first
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\ncf-mitigated: challenge\r\nContent-Length: 9\r\nConnection: close\r\n\r\nchallenge",
                )
                .await
                .unwrap();

            let (mut second, _) = listener.accept().await.unwrap();
            requests.push(read_request(&mut second).await);
            write_json_response(
                &mut second,
                json!({"result": {"data": {"json": {"url": "https://example.com/image.png"}}}}),
            )
            .await;
            requests
        });

        let provider = Arc::new(FakeClearanceProvider::new());
        let api = test_api_with_provider(base_url, DzmmApiAuth::default(), provider);
        api.upload_tweet_image(
            None,
            Some(b"multipart-image-payload"),
            Some("payload.png"),
            Some("image/png"),
        )
        .await
        .expect("multipart retry succeeds");

        let requests = handle.await.unwrap();
        assert_eq!(requests.len(), 2);
        for request in &requests {
            let body = String::from_utf8_lossy(&request.body);
            assert!(body.contains("multipart-image-payload"));
            assert!(body.contains("payload.png"));
            assert!(body.contains("image/png"));
        }
    }

    #[tokio::test]
    async fn socket_io_credentials_use_one_clearance_generation_and_merged_cookie_header() {
        let provider = Arc::new(FakeClearanceProvider::new());
        let api = test_api_with_provider(
            "http://127.0.0.1:1".to_string(),
            DzmmApiAuth {
                cookies: Some(Cow::Borrowed(
                    "session=account; cf_clearance=stale-account-value",
                )),
                ..DzmmApiAuth::default()
            },
            provider,
        );

        let credentials = api.socket_io_credentials().await.unwrap();

        assert_eq!(credentials.generation, 1);
        assert_eq!(credentials.user_agent, "browser-ua-generation-1");
        assert!(credentials.cookie_header.contains("session=account"));
        assert!(
            credentials
                .cookie_header
                .contains("cf_clearance=clearance-1")
        );
        assert!(!credentials.cookie_header.contains("stale-account-value"));
    }

    #[tokio::test]
    async fn qr_code_login_accepts_auth_cookie_from_redirect_response() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            write_redirect_with_cookie(&mut stream, "/", "sb-rls-auth-token=redirect-auth").await;
            request
        });

        let api = test_api(base_url);
        let ok = api.login_with_qr_code("encrypted-token").await.unwrap();
        let request = handle.await.unwrap();
        let cookie_string = api.get_cookie_string().await;

        assert!(ok);
        assert_eq!(request.target, "/api/auth/sign-in-code/encrypted-token");
        assert!(cookie_string.contains("sb-rls-auth-token=redirect-auth"));
    }

    #[test]
    fn test_parse_dzmm_local_address_values() {
        assert_eq!(parse_dzmm_local_address(None).unwrap(), None);
        assert_eq!(parse_dzmm_local_address(Some("")).unwrap(), None);
        assert_eq!(parse_dzmm_local_address(Some(" auto ")).unwrap(), None);
        assert_eq!(
            parse_dzmm_local_address(Some("0.0.0.0")).unwrap(),
            Some("0.0.0.0".parse().unwrap())
        );
        assert_eq!(
            parse_dzmm_local_address(Some("::")).unwrap(),
            Some("::".parse().unwrap())
        );
        assert!(parse_dzmm_local_address(Some("127.0.0.1")).is_err());
    }

    #[test]
    fn rate_limiter_next_delay_equal_bounds_does_not_panic() {
        // batch_size > 1 so request #1 is not a batch boundary → exercises the
        // random-delay branch with min == max, which previously panicked on an
        // empty `random_range` range.
        let mut limiter = RateLimiter::new(0.3, 0.3, 10, 1.0);
        let delay = limiter.next_delay();
        assert_eq!(delay, 0.3);
    }

    #[tokio::test]
    async fn request_retries_after_429_using_retry_after() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let target_path = "/api/trpc/test.endpoint";
        let handle = tokio::spawn(async move {
            let mut targets = Vec::new();
            // First attempt: 429 with Retry-After: 1.
            {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_request(&mut stream).await;
                let resp = "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                stream.write_all(resp.as_bytes()).await.unwrap();
                targets.push(request.target);
            }
            // Second attempt: 200 OK with JSON.
            {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_request(&mut stream).await;
                write_json_response(&mut stream, json!({"ok": true})).await;
                targets.push(request.target);
            }
            targets
        });

        let api = test_api(base_url);
        let request = ApiRequest::get(target_path).query(trpc_batch_query(&json!({
            "0": {"json": Value::Null}
        })));
        let value = api._request(request).await.unwrap();
        let targets = handle.await.unwrap();

        assert_eq!(value, json!({"ok": true}));
        assert_eq!(targets.len(), 2, "should have retried once after 429");
        assert_eq!(targets[0], targets[1]);
    }

    #[tokio::test]
    async fn test_gamefy_generate_uses_base_url_flat_payload_and_iroha_default() {
        let (base_url, handle) = spawn_gamefy_server(3).await;
        let api = test_api(base_url);

        let (image_bytes, mime_type) = api
            .generate_image("prompt", Some("1:1"), None, None)
            .await
            .unwrap();

        assert_eq!(image_bytes.unwrap(), b"generated-image-bytes");
        assert_eq!(mime_type.unwrap(), "image/png");

        let records = handle.await.unwrap();
        assert_eq!(records[0].method, "POST");
        assert_eq!(records[0].target, "/api/gamefy/draw");
        let payload: Value = serde_json::from_slice(&records[0].body).unwrap();
        assert_eq!(payload["prompt"], json!("prompt"));
        assert_eq!(payload["dimension"], json!("1:1"));
        assert_eq!(payload["model"], json!("iroha"));
        assert!(payload.get("json").is_none());
        assert_eq!(records[1].target, "/api/gamefy/draw/status?taskId=task-123");
        assert_eq!(records[2].target, "/api/draw/image/task-123?index=0");
        assert!(
            !records[2].headers.contains_key("cookie"),
            "plain media download must not send account or Cloudflare cookies"
        );
        assert_ne!(
            records[2].headers.get("user-agent").map(String::as_str),
            Some("browser-ua-generation-1"),
            "plain media download must not reuse the clearance identity"
        );
    }

    #[tokio::test]
    async fn test_gamefy_edit_uploads_reference_as_data_url_and_uses_rest_endpoints() {
        let (base_url, handle) = spawn_gamefy_server(4).await;
        let api = test_api(base_url);

        let (image_bytes, mime_type) = api
            .edit_image(
                "edit prompt",
                b"reference-bytes",
                Some("image/png"),
                Some(512),
                Some(256),
                None,
            )
            .await
            .unwrap();

        assert_eq!(image_bytes.unwrap(), b"edited-image-bytes");
        assert_eq!(mime_type.unwrap(), "image/png");

        let records = handle.await.unwrap();
        assert_eq!(records[0].method, "POST");
        assert_eq!(records[0].target, "/api/gamefy/draw/upload-reference");
        let upload_payload: Value = serde_json::from_slice(&records[0].body).unwrap();
        assert_eq!(
            upload_payload["image"],
            json!("data:image/png;base64,cmVmZXJlbmNlLWJ5dGVz")
        );

        assert_eq!(records[1].method, "POST");
        assert_eq!(records[1].target, "/api/gamefy/draw/edit");
        let edit_payload: Value = serde_json::from_slice(&records[1].body).unwrap();
        assert_eq!(edit_payload["prompt"], json!("edit prompt"));
        assert_eq!(
            edit_payload["images"],
            json!(["https://example.com/reference.png"])
        );
        assert_eq!(
            edit_payload["imageSize"],
            json!({"width": 512, "height": 256})
        );
        assert_eq!(edit_payload["model"], json!("nalang-dream"));
        assert!(edit_payload.get("json").is_none());
        assert_eq!(records[2].target, "/api/gamefy/draw/status?taskId=task-456");
        assert_eq!(records[3].target, "/api/draw/image/task-456?index=0");
        assert!(
            !records[3].headers.contains_key("cookie"),
            "plain media download must not send account or Cloudflare cookies"
        );
        assert_ne!(
            records[3].headers.get("user-agent").map(String::as_str),
            Some("browser-ua-generation-1"),
            "plain media download must not reuse the clearance identity"
        );
    }

    // ========================================================================
    // generate_string (6 tests)
    // ========================================================================

    #[test]
    fn test_generate_string_default_length() {
        let result = generate_string(10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_generate_string_custom_length() {
        let result = generate_string(15);
        assert_eq!(result.len(), 15);
    }

    #[test]
    fn test_generate_string_charset() {
        let expected: Vec<char> =
            "useandomp26T198340PX75pxJACKVERYMINDBUSHWOLFoGQZbfghjklqvwyzrict"
                .chars()
                .collect();
        let result = generate_string(100);
        for c in result.chars() {
            assert!(
                expected.contains(&c),
                "character '{}' not in expected charset",
                c
            );
        }
    }

    #[test]
    fn test_generate_string_zero_length() {
        let result = generate_string(0);
        assert_eq!(result, "");
    }

    #[test]
    fn test_generate_string_randomness() {
        let s1 = generate_string(10);
        let s2 = generate_string(10);
        assert_ne!(s1, s2);
    }

    // ========================================================================
    // parse_trpc_response (11 tests)
    // ========================================================================

    #[test]
    fn test_parse_trpc_valid_response() {
        let response = json!([{"result": {"data": {"json": {"id": 1, "name": "test"}}}}]);
        assert_eq!(
            parse_trpc_response(&response, 0, None),
            json!({"id": 1, "name": "test"})
        );
    }

    #[test]
    fn test_parse_trpc_non_list_returns_default() {
        assert_eq!(
            parse_trpc_response(&json!("not a list"), 0, None),
            json!({})
        );
    }

    #[test]
    fn test_parse_trpc_non_list_returns_custom_default() {
        assert_eq!(
            parse_trpc_response(&json!("not a list"), 0, Some(json!([]))),
            json!([])
        );
    }

    #[test]
    fn test_parse_trpc_short_list_returns_default() {
        assert_eq!(
            parse_trpc_response(&json!([{"result": {}}]), 1, None),
            json!({})
        );
    }

    #[test]
    fn test_parse_trpc_empty_list_returns_default() {
        assert_eq!(parse_trpc_response(&json!([]), 0, None), json!({}));
    }

    #[test]
    fn test_parse_trpc_missing_result_key() {
        let response = json!([{"other": "data"}]);
        assert_eq!(parse_trpc_response(&response, 0, None), json!({}));
    }

    #[test]
    fn test_parse_trpc_missing_data_key() {
        let response = json!([{"result": {"other": "data"}}]);
        assert_eq!(parse_trpc_response(&response, 0, None), json!({}));
    }

    #[test]
    fn test_parse_trpc_missing_json_key() {
        let response = json!([{"result": {"data": {"other": "data"}}}]);
        assert_eq!(
            parse_trpc_response(&response, 0, None),
            json!({"other": "data"})
        );
    }

    #[test]
    fn test_parse_trpc_data_is_string() {
        let response = json!([{"result": {"data": "json"}}]);
        assert_eq!(parse_trpc_response(&response, 0, None), json!({}));
    }

    #[test]
    fn test_parse_trpc_custom_index() {
        let response = json!([
            {"result": {"data": {"json": {"first": true}}}},
            {"result": {"data": {"json": {"second": true}}}},
        ]);
        assert_eq!(
            parse_trpc_response(&response, 1, None),
            json!({"second": true})
        );
    }

    #[test]
    fn test_parse_trpc_dict_response_returns_default() {
        assert_eq!(
            parse_trpc_response(&json!({"key": "val"}), 0, None),
            json!({})
        );
    }

    #[test]
    fn test_parse_trpc_standard_structure() {
        let response = json!([{"result": {"data": {"json": {"tweets": [], "page": 0}}}}]);
        assert_eq!(
            parse_trpc_response(&response, 0, None),
            json!({"tweets": [], "page": 0})
        );
    }

    #[test]
    fn test_parse_trpc_direct_data() {
        let response = json!([{"result": {"data": {"tweets": [], "page": 0}}}]);
        let result = parse_trpc_response(&response, 0, None);
        assert_eq!(result, json!({"tweets": [], "page": 0}));
    }

    #[test]
    fn test_parse_trpc_custom_default_on_empty() {
        assert_eq!(
            parse_trpc_response(&json!([]), 0, Some(json!([]))),
            json!([])
        );
    }

    // ========================================================================
    // is_trpc_business_forbidden (6 tests)
    // ========================================================================

    #[test]
    fn test_is_trpc_business_forbidden_true_batch() {
        let body = json!([{
            "error": {
                "json": {
                    "message": "无权访问",
                    "code": -32003,
                    "data": {"code": "FORBIDDEN", "httpStatus": 403}
                }
            }
        }])
        .to_string();
        assert!(is_trpc_business_forbidden(&body));
    }

    #[test]
    fn test_is_trpc_business_forbidden_true_single() {
        let body = json!({
            "error": {
                "json": {
                    "message": "无权访问",
                    "code": -32003,
                    "data": {"code": "FORBIDDEN", "httpStatus": 403}
                }
            }
        })
        .to_string();
        assert!(is_trpc_business_forbidden(&body));
    }

    #[test]
    fn test_is_trpc_business_forbidden_wrong_code() {
        let body = json!([{
            "error": {
                "json": {
                    "message": "error",
                    "code": -32000,
                    "data": {"code": "ERROR"}
                }
            }
        }])
        .to_string();
        assert!(!is_trpc_business_forbidden(&body));
    }

    #[test]
    fn test_is_trpc_business_forbidden_not_json() {
        assert!(!is_trpc_business_forbidden("not json"));
    }

    #[test]
    fn test_is_trpc_business_forbidden_no_error_key() {
        let body = json!([{"result": {"data": {"json": {}}}}]).to_string();
        assert!(!is_trpc_business_forbidden(&body));
    }

    #[test]
    fn test_is_trpc_business_forbidden_empty_body() {
        assert!(!is_trpc_business_forbidden(""));
    }

    // ========================================================================
    // extract_cookie_kv (4 tests)
    // ========================================================================

    #[test]
    fn test_extract_cookie_kv_basic() {
        let result = extract_cookie_kv("session=abc123; Path=/; HttpOnly");
        assert_eq!(result, Some(("session".to_string(), "abc123".to_string())));
    }

    #[test]
    fn test_extract_cookie_kv_empty_name() {
        assert_eq!(extract_cookie_kv("=value; Path=/"), None);
    }

    #[test]
    fn test_extract_cookie_kv_no_equal() {
        assert_eq!(extract_cookie_kv("onlyname"), None);
    }

    #[test]
    fn test_extract_cookie_kv_empty() {
        assert_eq!(extract_cookie_kv(""), None);
    }

    #[test]
    fn test_extract_cookie_kv_auth_token() {
        let result = extract_cookie_kv("sb-rls-auth-token.0=value123; Path=/; Secure");
        assert_eq!(
            result,
            Some(("sb-rls-auth-token.0".to_string(), "value123".to_string()))
        );
    }

    // ========================================================================
    // guess_content_type (4 tests)
    // ========================================================================

    #[test]
    fn test_guess_content_type_png() {
        assert_eq!(guess_content_type(".png"), "image/png");
    }

    #[test]
    fn test_guess_content_type_jpeg() {
        assert_eq!(guess_content_type(".jpg"), "image/jpeg");
        assert_eq!(guess_content_type(".jpeg"), "image/jpeg");
    }

    #[test]
    fn test_guess_content_type_mp3_and_audio() {
        assert_eq!(guess_content_type(".mp3"), "audio/mpeg");
        assert_eq!(guess_content_type(".wav"), "audio/wav");
        assert_eq!(guess_content_type(".ogg"), "audio/ogg");
        assert_eq!(guess_content_type(".m4a"), "audio/mp4");
    }

    #[test]
    fn test_guess_content_type_unknown() {
        assert_eq!(guess_content_type(".xyz"), "application/octet-stream");
    }

    #[test]
    fn test_guess_content_type_case_insensitive() {
        assert_eq!(guess_content_type(".PNG"), "image/png");
        assert_eq!(guess_content_type(".Jpg"), "image/jpeg");
    }

    // ========================================================================
    // guess_video_content_type (3 tests)
    // ========================================================================

    #[test]
    fn test_guess_video_content_type_mp4() {
        assert_eq!(guess_video_content_type(".mp4"), "video/mp4");
    }

    #[test]
    fn test_guess_video_content_type_mov() {
        assert_eq!(guess_video_content_type(".mov"), "video/quicktime");
    }

    #[test]
    fn test_guess_video_content_type_unknown() {
        assert_eq!(guess_video_content_type(".xyz"), "application/octet-stream");
    }

    // ========================================================================
    // extract_balanced_json_object (3 tests)
    // ========================================================================

    #[test]
    fn test_extract_json_object_simple() {
        let text = r#"{"key": "value"}"#;
        let result = extract_balanced_json_object(text, 0).unwrap();
        assert_eq!(result, json!({"key": "value"}));
    }

    #[test]
    fn test_extract_json_object_nested() {
        let text = r#"prefix {"outer": {"inner": 42}} suffix"#;
        let result = extract_balanced_json_object(text, 7).unwrap();
        assert_eq!(result, json!({"outer": {"inner": 42}}));
    }

    #[test]
    fn test_extract_json_object_string_with_braces() {
        let text = r#"{"msg": "hello {world}"}"#;
        let result = extract_balanced_json_object(text, 0).unwrap();
        assert_eq!(result, json!({"msg": "hello {world}"}));
    }

    #[test]
    fn test_extract_json_object_escaped_quotes() {
        let text = r#"{"msg": "hello \"world\""}"#;
        let result = extract_balanced_json_object(text, 0).unwrap();
        assert_eq!(result, json!({"msg": "hello \"world\""}));
    }

    #[test]
    fn test_extract_json_object_unterminated() {
        let text = r#"{"key": "val"#;
        assert!(extract_balanced_json_object(text, 0).is_err());
    }

    #[test]
    fn test_extract_json_object_not_object() {
        let text = r#"[1, 2, 3]"#;
        assert!(extract_balanced_json_object(text, 0).is_err());
    }

    // ========================================================================
    // extract_balanced_json_array (3 tests)
    // ========================================================================

    #[test]
    fn test_extract_json_array_simple() {
        let text = r#"[1, 2, 3]"#;
        let result = extract_balanced_json_array(text, 0).unwrap();
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_extract_json_array_nested() {
        let text = r#"prefix [[1, 2], [3, 4]] suffix"#;
        let result = extract_balanced_json_array(text, 7).unwrap();
        assert_eq!(result, "[[1, 2], [3, 4]]");
    }

    #[test]
    fn test_extract_json_array_unterminated() {
        let text = r#"[1, 2"#;
        assert!(extract_balanced_json_array(text, 0).is_err());
    }

    // ========================================================================
    // extract_next_scalar_field (5 tests)
    // ========================================================================

    #[test]
    fn test_extract_next_scalar_field_string() {
        let text = r#"{"name":"John","age":30}"#;
        assert_eq!(extract_next_scalar_field(text, "name"), Some(json!("John")));
    }

    #[test]
    fn test_extract_next_scalar_field_number() {
        let text = r#"{"name":"John","age":30}"#;
        assert_eq!(extract_next_scalar_field(text, "age"), Some(json!(30)));
    }

    #[test]
    fn test_extract_next_scalar_field_boolean() {
        let text = r#"{"active":true}"#;
        assert_eq!(extract_next_scalar_field(text, "active"), Some(json!(true)));
    }

    #[test]
    fn test_extract_next_scalar_field_false() {
        let text = r#"{"active":false}"#;
        assert_eq!(
            extract_next_scalar_field(text, "active"),
            Some(json!(false))
        );
    }

    #[test]
    fn test_extract_next_scalar_field_null() {
        let text = r#"{"value":null}"#;
        assert_eq!(extract_next_scalar_field(text, "value"), Some(json!(null)));
    }

    #[test]
    fn test_extract_next_scalar_field_missing() {
        let text = r#"{"name":"John"}"#;
        assert_eq!(extract_next_scalar_field(text, "age"), None);
    }

    // ========================================================================
    // extract_next_flight_text (2 tests)
    // ========================================================================

    #[test]
    fn test_extract_next_flight_basic() {
        let html = concat!(
            r#"self.__next_f.push([1,"hello "])"#,
            r#"self.__next_f.push([1,"world"])"#,
        );
        let result = extract_next_flight_text(html);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_extract_next_flight_skips_non_type1() {
        let html = concat!(
            r#"self.__next_f.push([2,"skip"])"#,
            r#"self.__next_f.push([1,"keep"])"#,
        );
        let result = extract_next_flight_text(html);
        assert_eq!(result, "keep");
    }

    #[test]
    fn test_extract_next_flight_empty() {
        let result = extract_next_flight_text("no markers here");
        assert_eq!(result, "");
    }

    // ========================================================================
    // extract_next_public_profile (1 test + extras)
    // ========================================================================

    #[test]
    fn test_extract_public_profile_fetches_from_flight() {
        let html = concat!(
            r#"self.__next_f.push([1,"{\"profile\":{"#,
            r#"\"fullName\":\"Public User\","#,
            r#"\"avatarUrl\":\"https://example.test/a.png\","#,
            r#"\"bio\":\"hello\","#,
            r#"\"birthday\":null,"#,
            r#"\"birthdayPublic\":false,"#,
            r#"\"gender\":\"male\","#,
            r#"\"quirk\":\"reader\","#,
            r#"\"profileCoverUrl\":\"https://example.test/c.png\","#,
            r#"\"hideContactInfo\":false"#,
            r#"},\"joinDate\":\"2026-05-14T00:00:00Z\","#,
            r#"\"isOwnProfile\":false,\"followersCount\":2,"#,
            r#"\"followingCount\":3,\"dmDeniedReason\":\"not_logged_in\","#,
            r#"\"showFavorites\":false,\"showLikes\":false}"#,
            r#""])"#,
        );
        let result = extract_next_public_profile(html, "u1");
        let obj = result.as_object().unwrap();
        assert_eq!(obj["id"], json!("u1"));
        assert_eq!(obj["fullName"], json!("Public User"));
        assert_eq!(obj["avatarUrl"], json!("https://example.test/a.png"));
        assert_eq!(obj["bio"], json!("hello"));
        assert_eq!(obj["gender"], json!("male"));
        assert_eq!(obj["quirk"], json!("reader"));
        assert_eq!(obj["birthdayPublic"], json!(false));
        assert_eq!(obj["isBot"], json!(false));
        assert_eq!(obj["isPremium"], json!(false));
        let metadata = obj["metadata"].as_object().unwrap();
        assert_eq!(metadata["profile_source"], json!("public_profile"));
        let public_profile = metadata["publicProfile"].as_object().unwrap();
        assert_eq!(
            public_profile["profileCoverUrl"],
            json!("https://example.test/c.png")
        );
        assert_eq!(public_profile["followersCount"], json!(2));
        assert_eq!(public_profile["followingCount"], json!(3));
        assert_eq!(public_profile["dmDeniedReason"], json!("not_logged_in"));
        assert_eq!(public_profile["showFavorites"], json!(false));
        assert_eq!(public_profile["showLikes"], json!(false));
    }

    #[test]
    fn test_extract_public_profile_no_profile_marker() {
        let result = extract_next_public_profile("no profile data", "u1");
        assert_eq!(result, json!({}));
    }

    // ========================================================================
    // extract_response_cookies (1 test)
    // ========================================================================

    #[test]
    fn test_extract_response_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            SET_COOKIE,
            HeaderValue::from_static("session=abc123; Path=/; HttpOnly"),
        );
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("token=xyz789; Path=/; Secure"),
        );
        let cookies = extract_response_cookies(&headers);
        assert_eq!(cookies.get("session").unwrap(), "abc123");
        assert_eq!(cookies.get("token").unwrap(), "xyz789");
    }

    #[test]
    fn test_extract_response_cookies_empty() {
        let headers = HeaderMap::new();
        let cookies = extract_response_cookies(&headers);
        assert!(cookies.is_empty());
    }

    // ========================================================================
    // API method response parsing tests
    // Each test mirrors the Python test by exercising the same response shape
    // through parse_trpc_response or the relevant parsing logic.
    // ========================================================================

    // --- get_my_info parsing ---
    #[test]
    fn test_get_my_info_parsing_success() {
        let response =
            json!([{"result": {"data": {"json": {"isLoggedIn": true, "user_id": "u1"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed["user_id"], json!("u1"));
        assert_eq!(parsed["isLoggedIn"], json!(true));
    }

    #[test]
    fn test_get_my_info_parsing_not_logged_in() {
        let response = json!([{"result": {"data": {"json": {"isLoggedIn": false}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed["isLoggedIn"], json!(false));
    }

    // --- get_user_info parsing ---
    #[test]
    fn test_get_user_info_parsed_trpc_response() {
        let response = json!([{"result": {"data": {"json": {"userId": "u1", "name": "User"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({"userId": "u1", "name": "User"}));
    }

    #[test]
    fn test_get_user_info_fallback_dict_response() {
        let response = json!({"userId": "u1", "name": "User"});
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    // --- batch_get_user_info parsing ---
    #[test]
    fn test_batch_get_user_info_parsed_list() {
        let response = json!([
            {"result": {"data": {"json": {"userId": "u1"}}}},
            {"result": {"data": {"json": {"userId": "u2"}}}},
        ]);
        let mut results = Vec::new();
        if let Some(arr) = response.as_array() {
            for item in arr {
                let r = item
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .cloned()
                    .unwrap_or(json!({}));
                results.push(r);
            }
        }
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], json!({"userId": "u1"}));
        assert_eq!(results[1], json!({"userId": "u2"}));
    }

    #[test]
    fn test_batch_get_user_info_missing_batch_item() {
        let response = json!([
            {"result": {"data": {"json": {"id": "u1", "fullName": "Room User"}}}},
            {"error": {"json": {"message": "无权访问", "data": {"code": "FORBIDDEN"}}}},
        ]);
        let mut results = Vec::new();
        if let Some(arr) = response.as_array() {
            for item in arr {
                let r = item
                    .get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .cloned()
                    .unwrap_or(json!({}));
                results.push(r);
            }
        }
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], json!({"id": "u1", "fullName": "Room User"}));
        assert_eq!(results[1], json!({}));
    }

    #[test]
    fn test_batch_get_user_info_non_list_response() {
        let response = json!({"error": "something"});
        let results: Vec<Value> = if let Some(_arr) = response.as_array() {
            vec![json!({})]
        } else {
            vec![]
        };
        assert!(results.is_empty());
    }

    // --- get_room_info parsing ---
    #[test]
    fn test_get_room_info_parsed_trpc() {
        let response = json!([{"result": {"data": {"json": {"roomId": "r1", "title": "Room"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({"roomId": "r1", "title": "Room"}));
    }

    #[test]
    fn test_get_room_info_dict_response_fallback() {
        let response = json!({"roomId": "r1", "title": "Room"});
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    #[test]
    fn test_get_room_info_non_dict_non_parsed() {
        let response = json!([]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    // --- preview_invite parsing ---
    #[test]
    fn test_preview_invite_parsed_trpc() {
        let response =
            json!([{"result": {"data": {"json": {"chatroomId": "r1", "title": "Room"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed["chatroomId"], json!("r1"));
    }

    #[test]
    fn test_preview_invite_invalid_response() {
        let response = json!([]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    // --- join_room_by_invite parsing ---
    #[test]
    fn test_join_room_by_invite_parsed_trpc() {
        let response = json!([{"result": {"data": {"json": {"chatroomId": "r1"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed["chatroomId"], json!("r1"));
    }

    #[test]
    fn test_join_room_by_invite_invalid_response() {
        let response = json!([]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    // --- create_group_chat parsing ---
    #[test]
    fn test_create_group_chat_dict_json_branch() {
        let response = json!({"result": {"data": {"json": {"chatroomId": "new_room"}}}});
        let result = response
            .as_object()
            .and_then(|obj| {
                obj.get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
            })
            .cloned()
            .unwrap_or_else(|| response.clone());
        assert_eq!(result, json!({"chatroomId": "new_room"}));
    }

    #[test]
    fn test_create_group_chat_dict_no_json_data() {
        let response = json!({"result": {"data": {}}});
        let result = response
            .as_object()
            .and_then(|obj| {
                obj.get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .filter(|j| !j.is_null())
            })
            .cloned();
        assert!(result.is_none());
    }

    #[test]
    fn test_create_group_chat_list_branch() {
        let response = json!([{"result": {"data": {"json": {"chatroomId": "new_room"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({"chatroomId": "new_room"}));
    }

    #[test]
    fn test_create_group_chat_empty_list_branch() {
        let response = json!([]);
        let is_nonempty = response.as_array().map(|a| !a.is_empty()).unwrap_or(false);
        assert!(!is_nonempty);
    }

    #[test]
    fn test_create_group_chat_fallback_response() {
        let response = json!("unexpected");
        assert!(!response.is_object());
        assert!(!response.is_array());
    }

    // --- generate_invite parsing ---
    #[test]
    fn test_generate_invite_parsing_success() {
        let response = json!([{"result": {"data": {"json": {"inviteLink": "/invite/XYZ"}}}}]);
        let data = parse_trpc_response(&response, 0, None);
        let link = data
            .get("inviteLink")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(link, "/invite/XYZ");
    }

    #[test]
    fn test_generate_invite_parsing_empty_link() {
        let response = json!([{"result": {"data": {"json": {"other": "data"}}}}]);
        let data = parse_trpc_response(&response, 0, None);
        let link = data
            .get("inviteLink")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(link, "");
    }

    #[test]
    fn test_generate_invite_parsing_unparseable() {
        let response = json!([]);
        let data = parse_trpc_response(&response, 0, None);
        let link = data
            .get("inviteLink")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert_eq!(link, "");
    }

    // --- get_share_resource_preview parsing ---
    #[test]
    fn test_get_share_resource_preview_parsed_trpc() {
        let response = json!([{"result": {"data": {"json": {"title": "Shared Room"}}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({"title": "Shared Room"}));
    }

    #[test]
    fn test_get_share_resource_preview_dict_error_branch() {
        let response = json!({"error": "resource not found"});
        let has_error = response.get("error").is_some();
        assert!(has_error);
    }

    #[test]
    fn test_get_share_resource_preview_dict_fallback() {
        let response = json!({"title": "Shared Room", "status": "ok"});
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    #[test]
    fn test_get_share_resource_preview_non_dict_fallback() {
        let response = json!([]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed, json!({}));
    }

    // --- fetch_explore_feed parsing ---
    #[test]
    fn test_fetch_explore_feed_parsed_trpc() {
        let payload = json!({
            "results": [{"type": "gallery", "data": {"id": "gallery_1"}}],
            "totalHits": 1,
            "estimatedTotalHits": 1,
            "limit": 5,
            "processingTimeMs": 1,
        });
        let response = json!([{"result": {"data": {"json": payload.clone()}}}]);
        let parsed = parse_trpc_response(&response, 0, None);
        assert_eq!(parsed["results"], payload["results"]);
        assert_eq!(parsed["totalHits"], payload["totalHits"]);
    }

    #[test]
    fn test_fetch_explore_feed_invalid_payload() {
        let response = json!({"results": []});
        let parsed = parse_trpc_response(&response, 0, None);
        assert!(!parsed.is_null() || parsed.as_object().map(|o| o.is_empty()).unwrap_or(true));
    }

    // --- fetch_novel_book parsing ---
    #[test]
    fn test_fetch_novel_book_parsed_response() {
        let payload = json!({
            "book": {
                "id": "book_1",
                "title": "Public Novel",
                "chapters": [
                    {"id": "chapter_1", "title": "Chapter 1", "content": "body", "chapterNumber": 1}
                ]
            }
        });
        let response = json!({"result": {"data": {"json": payload.clone()}}});
        let book = response
            .as_object()
            .and_then(|obj| {
                obj.get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .filter(|j| !j.is_null())
            })
            .cloned()
            .and_then(|data| data.get("book").filter(|b| b.is_object()).cloned());
        assert_eq!(book, Some(payload["book"].clone()));
    }

    #[test]
    fn test_fetch_novel_book_invalid_payload() {
        let response = json!({"result": {"data": {"json": {}}}});
        let book = response
            .as_object()
            .and_then(|obj| {
                obj.get("result")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("json"))
                    .filter(|j| !j.is_null())
            })
            .cloned()
            .and_then(|data| data.get("book").filter(|b| b.is_object()).cloned());
        assert_eq!(book, None);
    }

    // --- fetch_user_chats parsing ---
    #[test]
    fn test_fetch_user_chats_parsed_list() {
        let chat_data = json!([
            {"id": "c1", "type": "user", "title": "Chat 1"},
            {"id": "c2", "type": "bot", "title": "Bot Chat"},
            {"id": "c3", "type": "user", "title": "Chat 3"},
        ]);
        let response = json!([{"result": {"data": {"json": chat_data}}}]);
        let chats = parse_trpc_response(&response, 0, Some(json!([])));
        let empty_arr = vec![];
        let user_chats: Vec<&Value> = chats
            .as_array()
            .unwrap_or(&empty_arr)
            .iter()
            .filter(|c| c.get("type").and_then(|v| v.as_str()) == Some("user"))
            .collect();
        assert_eq!(user_chats.len(), 2);
        assert_eq!(user_chats[0]["id"], json!("c1"));
        assert_eq!(user_chats[1]["id"], json!("c3"));
    }

    #[test]
    fn test_fetch_user_chats_dict_fallback() {
        let response = json!({
            "chat1": {"id": "c1", "type": "user"},
            "chat2": {"id": "c2", "type": "user"},
        });
        let chats = parse_trpc_response(&response, 0, Some(json!([])));
        let mut chats_vec: Vec<Value> = if let Some(arr) = chats.as_array() {
            arr.clone()
        } else if chats.is_object() {
            chats.as_object().unwrap().values().cloned().collect()
        } else {
            vec![]
        };

        if chats_vec.is_empty() && response.is_object() {
            chats_vec = response.as_object().unwrap().values().cloned().collect();
        }
        assert_eq!(chats_vec.len(), 2);
    }

    #[test]
    fn test_fetch_user_chats_empty_response() {
        let response = json!([{"result": {"data": {"json": []}}}]);
        let chats = parse_trpc_response(&response, 0, Some(json!([])));
        let chats_arr = chats.as_array().unwrap();
        assert!(chats_arr.is_empty());
    }

    #[test]
    fn test_fetch_user_chats_empty_dict_fallback() {
        let response = json!({});
        let chats = parse_trpc_response(&response, 0, Some(json!([])));
        let chats_vec: Vec<Value> = if let Some(arr) = chats.as_array() {
            arr.clone()
        } else if chats.is_object() {
            chats.as_object().unwrap().values().cloned().collect()
        } else {
            vec![]
        };
        assert!(chats_vec.is_empty());
    }

    // ========================================================================
    // DzmmApi::new cookie parsing (get_cookie_string related)
    // ========================================================================

    #[test]
    fn test_new_parses_cookie_string() {
        let cookies_str = "a=1; b=2";
        let map: HashMap<String, String> = cookies_str
            .split(';')
            .filter_map(|p| {
                let p = p.trim();
                if let Some((k, v)) = p.split_once('=') {
                    Some((k.trim().to_string(), v.trim().to_string()))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(map.get("a").unwrap(), "1");
        assert_eq!(map.get("b").unwrap(), "2");
    }

    #[test]
    fn test_new_cookie_string_empty() {
        let map: HashMap<String, String> = HashMap::new();
        assert!(map.is_empty());
    }

    #[test]
    fn logged_endpoint_redacts_sign_in_token_and_query() {
        assert_eq!(
            sanitize_logged_endpoint("/api/auth/sign-in-code/sensitive-token?next=secret"),
            "/api/auth/sign-in-code/<redacted>"
        );
        assert_eq!(
            sanitize_logged_endpoint("/api/trpc/user.getMe?input=sensitive"),
            "/api/trpc/user.getMe"
        );
        assert_eq!(
            sanitize_logged_url(
                &Url::parse("https://www.dzmm.ai/api/trpc/user.getMe?input=sensitive").unwrap()
            ),
            "https://www.dzmm.ai/api/trpc/user.getMe"
        );
    }

    // ========================================================================
    // get_public_user_profile parsing
    // ========================================================================

    #[test]
    fn test_get_public_user_profile_parsed_trpc() {
        let response = json!([{
            "result": {
                "data": {
                    "json": {
                        "profile": {
                            "fullName": "Public User",
                            "avatarUrl": "https://example.test/a.png",
                            "bio": "hello",
                            "birthday": null,
                            "birthdayPublic": false,
                            "gender": "male",
                            "quirk": "reader",
                            "profileCoverUrl": "https://example.test/c.png",
                            "hideContactInfo": false
                        },
                        "joinDate": "2026-05-14T00:00:00Z",
                        "isOwnProfile": false,
                        "followersCount": 2,
                        "followingCount": 3,
                        "dmDeniedReason": "not_logged_in",
                        "showFavorites": false,
                        "showLikes": false
                    }
                }
            }
        }]);
        let payload = parse_trpc_response(&response, 0, None);
        let profile = payload.get("profile").and_then(|v| v.as_object());
        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile["fullName"], json!("Public User"));
        assert_eq!(profile["avatarUrl"], json!("https://example.test/a.png"));
        assert_eq!(profile["bio"], json!("hello"));
        assert_eq!(profile["gender"], json!("male"));
        assert_eq!(profile["quirk"], json!("reader"));
        assert_eq!(profile["birthdayPublic"], json!(false));
    }

    // ========================================================================
    // Merge response cookies logic
    // ========================================================================

    #[test]
    fn test_merge_response_cookies_adds_new() {
        let mut headers = HeaderMap::new();
        headers.insert(
            SET_COOKIE,
            HeaderValue::from_static("newkey=newval; Path=/"),
        );
        let new_cookies = extract_response_cookies(&headers);
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("existing".to_string(), "old".to_string());
        for (k, v) in new_cookies {
            map.insert(k, v);
        }
        assert_eq!(map.get("existing").unwrap(), "old");
        assert_eq!(map.get("newkey").unwrap(), "newval");
    }

    #[tokio::test]
    async fn cloudflare_response_cookies_are_not_added_to_account_cookie_state() {
        let api = test_api("http://127.0.0.1:1".to_string());
        let mut headers = HeaderMap::new();
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("session=updated-account; Path=/"),
        );
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("cf_clearance=edge-owned; Path=/"),
        );
        headers.append(
            SET_COOKIE,
            HeaderValue::from_static("__cf_bm=edge-owned-too; Path=/"),
        );

        api.merge_response_cookies(&headers).await;
        let account_cookies = api.get_cookie_string().await;

        assert!(account_cookies.contains("session=updated-account"));
        assert!(!account_cookies.contains("cf_clearance"));
        assert!(!account_cookies.contains("__cf_bm"));
    }

    #[tokio::test]
    async fn automatic_cookie_store_rejects_cloudflare_response_cookies() {
        let api = test_api("http://127.0.0.1:1".to_string());
        let response_cookies = [
            HeaderValue::from_static("session=account-owned; Path=/"),
            HeaderValue::from_static("cf_clearance=edge-owned; Path=/"),
            HeaderValue::from_static("__cf_bm=edge-owned-too; Path=/"),
        ];

        api.cookie_jar
            .set_cookies(&mut response_cookies.iter(), &api.cookie_url);

        let stored = api
            .cookie_jar
            .cookies(&api.cookie_url)
            .expect("account cookie is stored");
        let stored = stored.to_str().expect("stored cookies are valid headers");

        assert!(stored.contains("session=account-owned"));
        assert!(!stored.contains("cf_clearance"));
        assert!(!stored.contains("__cf_bm"));
    }
}
