use anyhow::{Context, Result};
use lilium_api_client::config::ApiClientConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub notification: NotificationConfig,
    pub spider: SpiderConfig,
    pub processor: ProcessorConfig,
    pub cli: CliConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct NotificationConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct SpiderConfig {
    pub queue_size: usize,
    pub batch_size: usize,
    pub buffer_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub websocket_url: String,
    pub reconnect_delay_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub polling_interval_secs: u64,
    pub batch_size: usize,
}

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub data_path: String,
}

fn default_pool_size() -> u32 {
    5
}

fn default_queue_size() -> usize {
    5_000
}

fn default_buffer_dir() -> PathBuf {
    PathBuf::from("data/event/buffer")
}

fn default_runtime_dir() -> PathBuf {
    PathBuf::from("runtime/spider")
}

fn default_ws_url() -> String {
    ApiClientConfig::default().ws_url
}

fn default_reconnect_delay_ms() -> u64 {
    5_000
}

fn default_polling_interval() -> u64 {
    5
}

fn default_batch_size() -> usize {
    100
}

fn default_data_path() -> &'static str {
    "./data"
}

fn env_string(name: &str, default: String) -> String {
    std::env::var(name).unwrap_or(default)
}

fn env_required_string(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("required env var '{name}' is missing"))
}

fn env_optional_string(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn fallback_string(
    primary: Option<&str>,
    fallback: Option<&str>,
    primary_name: &str,
    fallback_name: &str,
) -> Result<String> {
    primary
        .map(str::to_owned)
        .or_else(|| fallback.map(str::to_owned))
        .with_context(|| {
            format!("required env vars '{primary_name}' or '{fallback_name}' are missing")
        })
}

fn env_fallback_string(primary: &str, fallback: &str) -> Result<String> {
    fallback_string(
        std::env::var(primary).ok().as_deref(),
        std::env::var(fallback).ok().as_deref(),
        primary,
        fallback,
    )
}

fn env_u32(name: &str, default: u32) -> Result<u32> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<u32>()
            .with_context(|| format!("failed to parse {name} as u32")),
        Err(_) => Ok(default),
    }
}

fn env_u64(name: &str, default: u64) -> Result<u64> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .with_context(|| format!("failed to parse {name} as u64")),
        Err(_) => Ok(default),
    }
}

fn env_usize(name: &str, default: usize) -> Result<usize> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .with_context(|| format!("failed to parse {name} as usize")),
        Err(_) => Ok(default),
    }
}

fn env_path(name: &str, default: PathBuf) -> PathBuf {
    std::env::var(name).map(PathBuf::from).unwrap_or(default)
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            database: DatabaseConfig {
                url: env_required_string("DATABASE_URL")?,
                max_connections: env_u32("DATABASE_POOL_SIZE", default_pool_size())?,
            },
            notification: NotificationConfig {
                url: env_fallback_string("DATABASE_NOTIFICATION_URL", "DATABASE_URL")?,
            },
            spider: SpiderConfig {
                queue_size: env_usize("SPIDER_QUEUE_SIZE", default_queue_size())?,
                batch_size: env_usize("SPIDER_BATCH_SIZE", default_batch_size())?,
                buffer_dir: env_path("SPIDER_BUFFER_DIR", default_buffer_dir()),
                runtime_dir: env_path("SPIDER_RUNTIME_DIR", default_runtime_dir()),
                websocket_url: env_string("SPIDER_WEBSOCKET_URL", default_ws_url()),
                reconnect_delay_ms: env_u64(
                    "SPIDER_RECONNECT_DELAY_MS",
                    default_reconnect_delay_ms(),
                )?,
            },
            processor: ProcessorConfig {
                polling_interval_secs: env_u64(
                    "EVENT_PROCESSOR_POLLING_INTERVAL_SECS",
                    default_polling_interval(),
                )?,
                batch_size: env_usize("EVENT_PROCESSOR_BATCH_SIZE", default_batch_size())?,
            },
            cli: CliConfig {
                data_path: env_optional_string("DATA_PATH", default_data_path()),
            },
        })
    }
}

impl From<DatabaseConfig> for lilium_database::DatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DatabaseConfig::from_url(value.url, value.max_connections)
    }
}

impl From<DatabaseConfig> for lilium_database::DedicatedDatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DedicatedDatabaseConfig::from_url(value.url)
    }
}

impl From<NotificationConfig> for lilium_database::NotificationDatabaseConfig {
    fn from(value: NotificationConfig) -> Self {
        lilium_database::NotificationDatabaseConfig::from_url(value.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_string_prefers_primary_value() {
        let value =
            fallback_string(Some("primary"), Some("fallback"), "PRIMARY", "FALLBACK").unwrap();
        assert_eq!(value, "primary");
    }

    #[test]
    fn fallback_string_uses_fallback_value() {
        let value = fallback_string(None, Some("fallback"), "PRIMARY", "FALLBACK").unwrap();
        assert_eq!(value, "fallback");
    }

    #[test]
    fn notification_config_converts_url() {
        let config = NotificationConfig {
            url: "postgresql://notify".into(),
        };
        let db_config: lilium_database::NotificationDatabaseConfig = config.into();
        assert_eq!(db_config.normalized_url(), "postgres://notify");
    }
}
