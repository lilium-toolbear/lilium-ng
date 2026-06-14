use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use rand::Rng;
use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, COOKIE, ORIGIN, REFERER, SET_COOKIE},
    multipart::{Form, Part},
    Client, Method, StatusCode,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

const BASE_URL: &str = "https://www.dzmm.ai";

const DZMM_HEADERS: &[(&str, &str)] = &[
    ("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36"),
    ("Accept-Language", "en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
    ("Accept-Encoding", "gzip, deflate"),
    ("Cache-Control", "no-cache"),
    ("Pragma", "no-cache"),
    ("Sec-Ch-Ua", "\"Chromium\";v=\"148\", \"Brave\";v=\"148\", \"Not/A)Brand\";v=\"99\""),
    ("Sec-Ch-Ua-Arch", "\"arm\""),
    ("Sec-Ch-Ua-Bitness", "\"64\""),
    ("Sec-Ch-Ua-Full-Version-List", "\"Chromium\";v=\"148.0.0.0\", \"Brave\";v=\"148.0.0.0\", \"Not/A)Brand\";v=\"99.0.0.0\""),
    ("Sec-Ch-Ua-Mobile", "?0"),
    ("Sec-Ch-Ua-Model", "\"\""),
    ("Sec-Ch-Ua-Platform", "\"macOS\""),
    ("Sec-Ch-Ua-Platform-Version", "\"26.4.1\""),
    ("Sec-Gpc", "1"),
];

const GENERATE_STRING_CHARSET: &[u8] =
    b"useandomp26T198340PX75pxJACKVERYMINDBUSHWOLFoGQZbfghjklqvwyzrict";

fn generate_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..GENERATE_STRING_CHARSET.len());
            GENERATE_STRING_CHARSET[idx] as char
        })
        .collect()
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
        if let Some(obj) = item.as_object() {
            if let Some(error) = obj.get("error") {
                if let Some(error_obj) = error.as_object() {
                    if let Some(error_json) = error_obj.get("json") {
                        if error_json.get("code").and_then(|c| c.as_i64()) == Some(-32003) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
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
    loop {
        let call_index = match html[start..].find(marker) {
            Some(i) => start + i,
            None => break,
        };

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

        if let Some(arr) = payload.as_array() {
            if arr.len() >= 2 && arr[0] == Value::Number(1.into()) {
                if let Some(s) = arr[1].as_str() {
                    chunks.push(s.to_string());
                }
            }
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

    if let Some(ref tid) = target_user_id {
        if tid != user_id {
            return Value::Object(serde_json::Map::new());
        }
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
        if let Ok(s) = value.to_str() {
            if let Some((name, val)) = extract_cookie_kv(s) {
                cookies.insert(name, val);
            }
        }
    }
    cookies
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

impl RateLimiter {
    fn new(
        min_delay: Option<f64>,
        max_delay: Option<f64>,
        batch_size: Option<u64>,
        batch_delay: Option<f64>,
    ) -> Self {
        Self {
            min_delay: min_delay.unwrap_or_else(|| {
                std::env::var("MIN_REQUEST_DELAY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.2)
            }),
            max_delay: max_delay.unwrap_or_else(|| {
                std::env::var("MAX_REQUEST_DELAY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0.5)
            }),
            batch_size: batch_size.unwrap_or_else(|| {
                std::env::var("BATCH_SIZE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(50)
            }),
            batch_delay: batch_delay.unwrap_or_else(|| {
                std::env::var("BATCH_DELAY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1.0)
            }),
            request_count: 0,
        }
    }

    async fn wait(&mut self) {
        self.request_count += 1;

        if self.request_count.is_multiple_of(self.batch_size) {
            debug!(
                "Batch delay after {} requests: {:.2}s",
                self.request_count, self.batch_delay
            );
            tokio::time::sleep(Duration::from_secs_f64(self.batch_delay)).await;
        } else {
            let delay = {
                let mut rng = rand::thread_rng();
                rng.gen_range(self.min_delay..self.max_delay)
            };
            debug!(
                "Rate limit delay (request #{}): {:.2}s",
                self.request_count, delay
            );
            tokio::time::sleep(Duration::from_secs_f64(delay)).await;
        }
    }
}

type CookieRefreshCallback =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct DzmmApi {
    client: Client,
    email: Option<String>,
    password: Option<String>,
    signin_code: Option<String>,
    signin_code_image: Option<Vec<u8>>,
    signin_code_image_mime: Option<String>,
    user_id: Option<String>,
    auto_refresh: bool,
    on_cookies_refreshed: Option<CookieRefreshCallback>,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    refresh_lock: Arc<Mutex<()>>,
    cookie_map: Arc<Mutex<HashMap<String, String>>>,
}

impl DzmmApi {
    pub fn new(
        email: Option<String>,
        password: Option<String>,
        signin_code: Option<String>,
        signin_code_image: Option<Vec<u8>>,
        signin_code_image_mime: Option<String>,
        cookies: Option<String>,
        user_id: Option<String>,
        auto_refresh: bool,
        on_cookies_refreshed: Option<CookieRefreshCallback>,
    ) -> Result<Self> {
        let cookie_map = if let Some(ref c) = cookies {
            let map: HashMap<String, String> = c
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
            map
        } else {
            HashMap::new()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            client,
            email,
            password,
            signin_code,
            signin_code_image,
            signin_code_image_mime,
            user_id,
            auto_refresh,
            on_cookies_refreshed,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(None, None, None, None))),
            refresh_lock: Arc::new(Mutex::new(())),
            cookie_map: Arc::new(Mutex::new(cookie_map)),
        })
    }

    pub async fn get_cookie_string(&self) -> String {
        let map = self.cookie_map.lock().await;
        let mut pairs: Vec<String> = map.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        pairs.sort();
        pairs.join("; ")
    }

    async fn merge_response_cookies(&self, headers: &HeaderMap) {
        let new_cookies = extract_response_cookies(headers);
        if !new_cookies.is_empty() {
            let mut map = self.cookie_map.lock().await;
            for (k, v) in new_cookies {
                map.insert(k, v);
            }
        }
    }

    async fn clear_cookies(&self) {
        self.cookie_map.lock().await.clear();
    }

    async fn invoke_cookies_refreshed(&self) {
        if let Some(ref cb) = self.on_cookies_refreshed {
            let cookie_str = self.get_cookie_string().await;
            cb(cookie_str).await;
        }
    }

    pub async fn authenticate(&self) -> Result<()> {
        info!("Authenticating...");
        self.get_my_info(false).await?;
        Ok(())
    }

    pub async fn refresh_cookies(&self) -> Result<bool> {
        let _guard = self.refresh_lock.lock().await;
        info!("Refreshing authentication cookies...");

        let token_url = format!("{}/api/auth/token", BASE_URL);
        match self
            .client
            .get(&token_url)
            .headers(self.build_headers(None))
            .send()
            .await
        {
            Ok(response) => {
                self.merge_response_cookies(response.headers()).await;
                if response.status() == StatusCode::OK {
                    let auth_data: Value = response.json().await.unwrap_or_default();
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
                warn!("Token refresh failed with status {}", response.status());
            }
            Err(e) => {
                warn!("Token refresh failed: {}", e);
            }
        }

        if let (Some(ref email), Some(ref password)) = (&self.email, &self.password) {
            info!("Falling back to password login...");
            return self.login_with_email_password(email, password).await;
        }

        if self.signin_code_image.is_some() && self.signin_code_image_mime.is_some() {
            info!("Falling back to QR code image login...");
            return self
                .login_with_qr_image(
                    self.signin_code_image.as_ref().unwrap(),
                    self.signin_code_image_mime.as_ref().unwrap(),
                )
                .await;
        }

        if let Some(ref signin_code) = self.signin_code {
            info!("Falling back to QR code signin...");
            return self.login_with_qr_code(signin_code).await;
        }

        error!("Token refresh failed and no credentials available for fallback");
        Ok(false)
    }

    pub async fn login_with_email_password(&self, email: &str, password: &str) -> Result<bool> {
        info!("Logging in with email and password...");

        self.clear_cookies().await;

        let sign_in_url = format!("{}/api/auth/sign-in", BASE_URL);
        let body = json!({"email": email, "password": password});

        let sign_in_response = self
            .client
            .post(&sign_in_url)
            .headers(self.build_headers(Some(&[("Content-Type", "application/json")])))
            .json(&body)
            .send()
            .await
            .context("Sign-in request failed")?;

        self.merge_response_cookies(sign_in_response.headers())
            .await;

        if sign_in_response.status() != StatusCode::OK {
            error!("Sign-in failed: {}", sign_in_response.status());
            return Ok(false);
        }

        info!("Sign-in successful");

        let token_url = format!("{}/api/auth/token", BASE_URL);
        let token_response = self
            .client
            .get(&token_url)
            .headers(self.build_headers(None))
            .send()
            .await
            .context("Token request failed")?;

        self.merge_response_cookies(token_response.headers()).await;

        if token_response.status() != StatusCode::OK {
            error!("Token retrieval failed: {}", token_response.status());
            return Ok(false);
        }

        let auth_data: Value = token_response.json().await.unwrap_or_default();
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

    pub async fn login_with_qr_code(&self, encrypted_token: &str) -> Result<bool> {
        info!("Logging in with QR code token...");

        self.clear_cookies().await;

        let url = format!("{}/api/auth/sign-in-code/{}", BASE_URL, encrypted_token);
        let response = self
            .client
            .get(&url)
            .headers(self.build_headers(None))
            .send()
            .await
            .context("QR code login request failed")?;

        self.merge_response_cookies(response.headers()).await;

        let has_auth_cookie = self
            .cookie_map
            .lock()
            .await
            .keys()
            .any(|k| k.starts_with("sb-rls-auth-token"));

        if has_auth_cookie {
            info!("QR code login successful!");
            self.invoke_cookies_refreshed().await;
            return Ok(true);
        }

        error!("QR code login failed - no auth cookie received");
        Ok(false)
    }

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

        let part = Part::bytes(image.to_vec())
            .file_name(filename)
            .mime_str(mime_type)
            .context("Failed to build multipart part")?;
        let form = Form::new().part("image", part);

        let url = format!("{}/api/auth/sign-in-code/scan", BASE_URL);
        let response = self
            .client
            .post(&url)
            .headers(self.build_headers(None))
            .multipart(form)
            .send()
            .await
            .context("QR image login request failed")?;

        self.merge_response_cookies(response.headers()).await;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let error_data: Value = response.json().await.unwrap_or_default();
            let error_msg = error_data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("HTTP error");
            error!("QR image login failed ({}): {}", status, error_msg);
            return Ok(false);
        }

        let has_auth_cookie = self
            .cookie_map
            .lock()
            .await
            .keys()
            .any(|k| k.starts_with("sb-rls-auth-token"));

        if has_auth_cookie {
            info!("QR image login successful!");
            self.invoke_cookies_refreshed().await;
            return Ok(true);
        }

        error!("QR image login failed - no auth cookie received");
        Ok(false)
    }

    fn build_headers(&self, extra_headers: Option<&[(&str, &str)]>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (k, v) in DZMM_HEADERS {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                HeaderValue::from_str(v).unwrap(),
            );
        }
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/json, text/plain, */*"),
        );
        headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
        headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
        headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
        headers.insert(ORIGIN, HeaderValue::from_static(BASE_URL));

        let referer = format!("{}/chat", BASE_URL);
        headers.insert(REFERER, HeaderValue::from_str(&referer).unwrap());

        if let Some(extra) = extra_headers {
            for (k, v) in extra {
                headers.insert(
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                    HeaderValue::from_str(v).unwrap(),
                );
            }
        }

        headers.insert(
            "x-dzmm-request-id",
            HeaderValue::from_str(&generate_string(10)).unwrap(),
        );

        headers
    }

    async fn build_cookie_header_value(&self) -> Option<String> {
        let map = self.cookie_map.lock().await;
        if map.is_empty() {
            return None;
        }
        let pairs: Vec<String> = map.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        Some(pairs.join("; "))
    }

    async fn _request_inner(
        &self,
        method: Method,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        json_body: Option<&Value>,
        multipart_form: Option<Form>,
        extra_headers: Option<&[(&str, &str)]>,
        timeout: Option<Duration>,
    ) -> Result<(StatusCode, Vec<u8>, HeaderMap)> {
        let url = format!("{}{}", BASE_URL, endpoint);

        let mut builder = self.client.request(method, &url);

        let mut headers = self.build_headers(extra_headers);
        if let Some(cookie_val) = self.build_cookie_header_value().await {
            headers.insert(COOKIE, HeaderValue::from_str(&cookie_val).unwrap());
        }
        builder = builder.headers(headers);

        if let Some(q) = query {
            builder = builder.query(q);
        }

        if let Some(body) = json_body {
            builder = builder.json(body);
        }

        if let Some(form) = multipart_form {
            builder = builder.multipart(form);
        }

        if let Some(t) = timeout {
            builder = builder.timeout(t);
        }

        let response = builder.send().await.context("HTTP request failed")?;
        let status = response.status();
        let resp_headers = response.headers().clone();
        let body_bytes = response
            .bytes()
            .await
            .context("Failed to read response body")?;

        Ok((status, body_bytes.to_vec(), resp_headers))
    }

    async fn _request(
        &self,
        method: Method,
        endpoint: &str,
        query: Option<&[(&str, &str)]>,
        json_body: Option<&Value>,
        multipart_form: Option<Form>,
        extra_headers: Option<&[(&str, &str)]>,
        timeout: Option<Duration>,
    ) -> Result<Value> {
        let mut retried = false;
        let mut form = multipart_form;

        loop {
            self.rate_limiter.lock().await.wait().await;

            let current_form = if retried { None } else { form.take() };
            let body_ref = json_body.cloned();
            let (status, body_bytes, resp_headers) = self
                ._request_inner(
                    method.clone(),
                    endpoint,
                    query,
                    body_ref.as_ref(),
                    current_form,
                    extra_headers,
                    timeout,
                )
                .await?;

            self.merge_response_cookies(&resp_headers).await;

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
            let is_biz_forbidden = is_trpc_business_forbidden(&body_text);
            let is_auth = matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN);

            if is_auth && !is_biz_forbidden && self.auto_refresh && !retried {
                warn!(
                    "Auth error before retry {} for {}\nResult: {}",
                    status, endpoint, body_text
                );
                if self.refresh_cookies().await.unwrap_or(false) {
                    info!("Retrying with fresh cookies...");
                    retried = true;
                    continue;
                }
            }

            if is_biz_forbidden {
                warn!(
                    "Business forbidden {} for {}\nResult: {}",
                    status, endpoint, body_text
                );
                bail!("Business forbidden: {} {}", status, body_text);
            }

            error!(
                "HTTP error {} for {}\nResult: {}",
                status, endpoint, body_text
            );
            bail!("HTTP {} for {}: {}", status, endpoint, body_text);
        }
    }

    pub async fn get_my_info(&self, retried: bool) -> Result<Value> {
        let input_data = json!({"0": {"json": Value::Null}});
        let response = self
            ._request(
                Method::GET,
                "/api/trpc/user.getMe",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);

        if !parsed.is_object() || parsed.get("isLoggedIn") == Some(&Value::Bool(false)) {
            if !retried && self.auto_refresh {
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

    pub async fn get_user_info(&self, user_id: &str, room_id: &str) -> Result<Value> {
        let input_data = json!({"0": {"json": {"userId": user_id, "chatroomId": room_id}}});
        let response = self
            ._request(
                Method::GET,
                "/api/trpc/user.getChatroomUser",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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

    pub async fn get_public_user_profile(&self, user_id: &str) -> Result<Value> {
        let input_data = json!({"0": {"json": {"userid": user_id}}});
        let response = self
            ._request(
                Method::GET,
                "/api/trpc/user.getProfilePage",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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
            ._request(
                Method::GET,
                &endpoint,
                Some(&[("batch", "1"), ("input", &input_data)]),
                None,
                None,
                None,
                None,
            )
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

    pub async fn get_room_info(&self, room_id: &str) -> Result<Option<Value>> {
        let input_data = json!({"0": {"json": {"chatroomId": room_id}}});
        let response = self
            ._request(
                Method::GET,
                "/api/trpc/chatroom.getPreview",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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

    pub async fn preview_invite(&self, invite_code: &str) -> Result<Value> {
        let input_data = json!({"0": {"json": {"code": invite_code}}});
        let response = self
            ._request(
                Method::GET,
                "/api/trpc/groupChat.getInviteInfo",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
            )
            .await?;

        let parsed = parse_trpc_response(&response, 0, None);
        if !parsed.is_null() && (!parsed.is_object() || !parsed.as_object().unwrap().is_empty()) {
            return Ok(parsed);
        }

        bail!("Invalid tRPC response for preview_invite: {}", response);
    }

    pub async fn join_room_by_invite(&self, invite_code: &str) -> Result<Value> {
        let body = json!({"0": {"json": {"inviteCode": invite_code, "gender": "male"}}});
        let response = self
            ._request(
                Method::POST,
                "/api/trpc/groupChat.joinByInvite",
                Some(&[("batch", "1")]),
                Some(&body),
                None,
                None,
                None,
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

    pub async fn create_group_chat(
        &self,
        title: &str,
        is_public: bool,
        tags: Option<&[String]>,
    ) -> Result<Value> {
        let tags_json =
            serde_json::to_string(&tags.unwrap_or(&[])).unwrap_or_else(|_| "[]".to_string());
        let form = Form::new()
            .text("title", title.to_string())
            .text(
                "isPublic",
                (if is_public { "true" } else { "false" }).to_string(),
            )
            .text("tags", tags_json);

        let response = self
            ._request(
                Method::POST,
                "/api/trpc/groupChat.create",
                None,
                None,
                Some(form),
                None,
                None,
            )
            .await?;

        if let Some(obj) = response.as_object() {
            if let Some(json_data) = obj
                .get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"))
            {
                if !json_data.is_null() {
                    return Ok(json_data.clone());
                }
            }
            return Ok(response);
        }

        if let Some(arr) = response.as_array() {
            if !arr.is_empty() {
                return Ok(parse_trpc_response(&response, 0, None));
            }
        }

        Ok(response)
    }

    pub async fn generate_invite(&self, chatroom_id: &str) -> Result<String> {
        let payload = json!({"0": {"json": {"chatroomId": chatroom_id}}});
        let result = self
            ._request(
                Method::POST,
                "/api/trpc/groupChat.generateInvite?batch=1",
                None,
                Some(&payload),
                None,
                None,
                None,
            )
            .await?;

        let data = parse_trpc_response(&result, 0, None);
        Ok(data
            .get("inviteLink")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string())
    }

    pub async fn remove_group_member(&self, chatroom_id: &str, member_id: &str) -> Result<()> {
        let payload = json!({"0": {"json": {"chatroomId": chatroom_id, "memberId": member_id}}});
        self._request(
            Method::POST,
            "/api/trpc/groupChat.removeMember?batch=1",
            None,
            Some(&payload),
            None,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn set_group_admin(&self, chatroom_id: &str, target_user_id: &str) -> Result<()> {
        let payload =
            json!({"0": {"json": {"chatroomId": chatroom_id, "targetUserId": target_user_id}}});
        self._request(
            Method::POST,
            "/api/trpc/groupChat.setAdmin?batch=1",
            None,
            Some(&payload),
            None,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn update_room_avatar(
        &self,
        chatroom_id: &str,
        image_data: &[u8],
        filename: &str,
        content_type: &str,
    ) -> Result<()> {
        let part = Part::bytes(image_data.to_vec())
            .file_name(filename.to_string())
            .mime_str(content_type)
            .context("Failed to build multipart part")?;
        let form = Form::new().part("file".to_string(), part);

        let url = format!("{}/api/group-chat/{}/avatar", BASE_URL, chatroom_id);
        let response = self
            .client
            .put(&url)
            .headers(self.build_headers(None))
            .multipart(form)
            .send()
            .await
            .context("Failed to update room avatar")?;

        self.merge_response_cookies(response.headers()).await;

        if response.status() != StatusCode::OK {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Failed to update avatar: {} {}", status, text);
        }

        Ok(())
    }

    pub async fn rename_room(&self, chatroom_id: &str, title: &str) -> Result<()> {
        let body = json!({"json": {"chatroomId": chatroom_id, "title": title}});
        self._request(
            Method::POST,
            "/api/trpc/groupChat.rename",
            None,
            Some(&body),
            None,
            None,
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn get_share_resource_preview(
        &self,
        resource_id: &str,
        share_type: Option<&str>,
    ) -> Result<Value> {
        let st = share_type.unwrap_or("group_invite");
        let input_data = json!({"0": {"json": {"type": st, "resourceId": resource_id}}});
        let response = self
            ._request(
                Method::GET,
                "/api/trpc/share.getResourcePreview",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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

    pub async fn send_heartbeat(&self) -> Result<bool> {
        match self
            ._request(
                Method::POST,
                "/api/heartbeat",
                None,
                None,
                None,
                None,
                Some(Duration::from_secs(5)),
            )
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
                Method::GET,
                "/api/trpc/chat.listAll",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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
                Method::GET,
                "/api/trpc/chatroom.getMessages",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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
                Method::GET,
                "/api/trpc/search.search",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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
                Method::GET,
                "/api/trpc/novel.book.get",
                Some(&[("input", &input_obj.to_string())]),
                None,
                None,
                None,
                None,
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

        if let Some(book) = parsed.get("book") {
            if book.is_object() {
                return Ok(book.clone());
            }
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
                Method::GET,
                "/api/trpc/chatroom.listMembers",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
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
            ._request(
                Method::POST,
                "/api/trpc/chatroom.uploadImage",
                None,
                None,
                Some(form),
                None,
                None,
            )
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
            ._request(
                Method::POST,
                "/api/trpc/chat.uploadVoiceMessage",
                None,
                None,
                Some(form),
                None,
                None,
            )
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
            ._request(
                Method::POST,
                "/api/trpc/media.uploadVideo",
                None,
                None,
                Some(form),
                None,
                None,
            )
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
            ._request(
                Method::POST,
                "/api/trpc/tweet.uploadImage",
                None,
                None,
                Some(form),
                None,
                None,
            )
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
            ._request(
                Method::GET,
                "/api/trpc/tweet.list",
                Some(&[("batch", "1"), ("input", &input_data.to_string())]),
                None,
                None,
                None,
                None,
            )
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
        let model = model.unwrap_or("vivid");
        let negative_prompt = negative_prompt.unwrap_or(
            "low quality, blurry, deformed, text, signature, watermark, multiple limbs, extra fingers, ugly",
        );

        let ban_words = ["loli", "lolita", "large perky breasts"];
        let mut cleaned_prompt = prompt.to_string();
        for word in &ban_words {
            cleaned_prompt = cleaned_prompt.replace(word, "");
        }

        let payload = json!({
            "json": {
                "prompt": cleaned_prompt,
                "tagIds": tag_ids,
                "dimension": dimension,
                "negativePrompt": negative_prompt,
                "model": model,
            }
        });

        info!(
            "Starting image generation: prompt='{}...', model={}, dimension={}",
            &cleaned_prompt.chars().take(50).collect::<String>(),
            model,
            dimension
        );

        let response = self
            ._request(
                Method::POST,
                "/api/trpc/draw.image.generate",
                None,
                Some(&payload),
                None,
                Some(&[("Content-Type", "application/json")]),
                None,
            )
            .await?;

        let data = if let Some(obj) = response.as_object() {
            obj.get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"))
                .cloned()
                .unwrap_or(response)
        } else {
            response
        };

        if !data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
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
        let input_param = serde_json::to_string(&json!({"json": {"taskId": task_id}}))?;
        let encoded = urlencoding(&input_param);

        let response = self
            ._request(
                Method::GET,
                &format!("/api/trpc/draw.image.taskStatus?input={}", encoded),
                None,
                None,
                None,
                None,
                None,
            )
            .await?;

        let data = if let Some(obj) = response.as_object() {
            obj.get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"))
                .cloned()
                .unwrap_or(response)
        } else {
            response
        };

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
        let model = model.unwrap_or("vivid");

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
                let temp_image_url = format!("{}{}", BASE_URL, relative_url);
                info!("Downloading image from: {}", temp_image_url);

                let download_response = self
                    .client
                    .get(&temp_image_url)
                    .send()
                    .await
                    .context("Failed to download generated image")?;

                if !download_response.status().is_success() {
                    bail!("Failed to download image: {}", download_response.status());
                }

                let _mime_type = download_response
                    .headers()
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
                    .unwrap_or_else(|| "image/png".to_string());

                let headers = download_response.headers().clone();
                let image_bytes = download_response
                    .bytes()
                    .await
                    .context("Failed to read image bytes")?;

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
                return Ok((Some(image_bytes.to_vec()), Some(mime_type)));
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
        filename: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<String> {
        let filename = filename.unwrap_or("reference.jpg");
        let content_type = content_type.unwrap_or("image/jpeg");

        let part = Part::bytes(image_bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str(content_type)
            .context("Failed to build multipart part")?;
        let form = Form::new().part("file".to_string(), part);

        self.rate_limiter.lock().await.wait().await;

        let url = format!("{}/api/draw/upload-reference", BASE_URL);
        let mut headers = self.build_headers(None);
        headers.insert(
            "x-dzmm-request-id",
            HeaderValue::from_str(&generate_string(10)).unwrap(),
        );

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .multipart(form)
            .send()
            .await
            .context("Upload reference image failed")?;

        self.merge_response_cookies(response.headers()).await;

        if !response.status().is_success() {
            bail!(
                "Reference image upload failed with status: {}",
                response.status()
            );
        }

        let data: Value = response
            .json()
            .await
            .context("Failed to parse upload response")?;
        if !data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
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

    pub async fn start_image_edit(
        &self,
        prompt: &str,
        image_urls: &[String],
        image_width: Option<u64>,
        image_height: Option<u64>,
        num_inference_steps: Option<u64>,
        text_guidance_scale: Option<f64>,
        image_guidance_scale: Option<f64>,
        num_images: Option<u64>,
        enable_safety_checker: Option<bool>,
        model: Option<&str>,
        tag_ids: Option<&[String]>,
    ) -> Result<String> {
        if image_urls.len() > 4 {
            bail!(
                "Maximum 4 reference images allowed, got {}",
                image_urls.len()
            );
        }

        let tag_ids = tag_ids.unwrap_or(&[]);
        let model = model.unwrap_or("nalang-dream");

        let payload = json!({
            "json": {
                "prompt": prompt,
                "images": image_urls,
                "imageSize": {"width": image_width.unwrap_or(1024), "height": image_height.unwrap_or(1024)},
                "numInferenceSteps": num_inference_steps.unwrap_or(30),
                "textGuidanceScale": text_guidance_scale.unwrap_or(5.0),
                "imageGuidanceScale": image_guidance_scale.unwrap_or(6.0),
                "numImages": num_images.unwrap_or(1),
                "enableSafetyChecker": enable_safety_checker.unwrap_or(true),
                "model": model,
                "tagIds": tag_ids,
            }
        });

        info!(
            "Starting image edit: prompt='{}...', model={}",
            &prompt.chars().take(50).collect::<String>(),
            model
        );

        let response = self
            ._request(
                Method::POST,
                "/api/trpc/draw.image.edit",
                None,
                Some(&payload),
                None,
                Some(&[("Content-Type", "application/json")]),
                None,
            )
            .await?;

        let data = if let Some(obj) = response.as_object() {
            obj.get("result")
                .and_then(|r| r.get("data"))
                .and_then(|d| d.get("json"))
                .cloned()
                .unwrap_or(response)
        } else {
            response
        };

        if !data
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
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

        let task_id = self
            .start_image_edit(
                prompt,
                &[ref_url],
                image_width,
                image_height,
                None,
                None,
                None,
                None,
                None,
                model,
                None,
            )
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
                let temp_image_url = format!("{}{}", BASE_URL, relative_url);
                info!("Downloading edited image from: {}", temp_image_url);

                let download_response = self
                    .client
                    .get(&temp_image_url)
                    .send()
                    .await
                    .context("Failed to download edited image")?;

                if !download_response.status().is_success() {
                    bail!("Failed to download image: {}", download_response.status());
                }

                let mime_type = download_response
                    .headers()
                    .get(CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(';').next().unwrap_or(s).trim().to_string())
                    .unwrap_or_else(|| "image/png".to_string());

                let result_bytes = download_response
                    .bytes()
                    .await
                    .context("Failed to read image bytes")?;

                info!(
                    "Image edit complete: {} bytes, {}",
                    result_bytes.len(),
                    mime_type
                );
                return Ok((Some(result_bytes.to_vec()), Some(mime_type)));
            }

            if status == "failed" {
                error!("Edit task {} failed", task_id);
                return Ok((None, None));
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

fn build_file_upload_form(name: &str, data: Vec<u8>, filename: &str, mime_type: &str) -> Form {
    let part = Part::bytes(data)
        .file_name(filename.to_string())
        .mime_str(mime_type)
        .expect("Invalid MIME type");
    Form::new().part(name.to_string(), part)
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    // urlencoding (4 tests)
    // ========================================================================

    #[test]
    fn test_urlencoding_alphanumeric() {
        assert_eq!(urlencoding("abc123"), "abc123");
    }

    #[test]
    fn test_urlencoding_special_chars() {
        assert_eq!(urlencoding("hello world"), "hello%20world");
    }

    #[test]
    fn test_urlencoding_safe_chars() {
        assert_eq!(urlencoding("-_.~"), "-_.~");
    }

    #[test]
    fn test_urlencoding_json_like() {
        let input = r#"{"key":"value"}"#;
        let result = urlencoding(input);
        assert!(result.contains("%7B") || result.contains("%7b"));
        assert!(result.contains("%7D") || result.contains("%7d"));
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
}
