use anyhow::{Context, Result};
use lilium_api_client::config::ApiClientConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub worker: WorkerConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub queue_size: usize,
    pub batch_size: usize,
    pub buffer_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub websocket_url: String,
    pub reconnect_delay_ms: u64,
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

fn env_string(name: &str, default: String) -> String {
    std::env::var(name).unwrap_or(default)
}

fn env_required_string(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("required env var '{name}' is missing"))
}

fn env_usize(name: &str, default: usize) -> Result<usize> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<usize>()
            .with_context(|| format!("failed to parse {name} as usize")),
        Err(_) => Ok(default),
    }
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
            worker: WorkerConfig {
                queue_size: env_usize("SPIDER_QUEUE_SIZE", default_queue_size())?,
                batch_size: env_usize("SPIDER_BATCH_SIZE", 100)?,
                buffer_dir: env_path("SPIDER_BUFFER_DIR", default_buffer_dir()),
                runtime_dir: env_path("SPIDER_RUNTIME_DIR", default_runtime_dir()),
                websocket_url: env_string("SPIDER_WEBSOCKET_URL", default_ws_url()),
                reconnect_delay_ms: env_u64(
                    "SPIDER_RECONNECT_DELAY_MS",
                    default_reconnect_delay_ms(),
                )?,
            },
        })
    }
}

impl From<DatabaseConfig> for lilium_database::DatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DatabaseConfig::from_url(value.url, value.max_connections)
    }
}
