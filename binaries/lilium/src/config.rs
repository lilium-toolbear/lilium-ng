// Python parity source: dzmm_archive@0efb507c6126a2638d3d38aca4018a804431291e cli/send_command.py
// Config is environment-driven, mirroring the Python `.env` + `setup_logging` bootstrap.
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub notification: NotificationConfig,
    /// Local data directory for media downloads (used by explore/history CLIs).
    #[allow(dead_code)]
    pub data_path: String,
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

fn default_pool_size() -> u32 {
    5
}

fn default_data_path() -> &'static str {
    "./data"
}

fn env_u32(name: &str, default: u32) -> Result<u32> {
    match std::env::var(name) {
        Ok(value) => value
            .parse::<u32>()
            .with_context(|| format!("failed to parse {name} as u32")),
        Err(_) => Ok(default),
    }
}

fn env_required_string(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("required env var '{name}' is missing"))
}

fn env_fallback_string(primary: &str, fallback: &str) -> Result<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(fallback).ok())
        .with_context(|| format!("required env vars '{primary}' or '{fallback}' are missing"))
}

fn env_optional_string(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
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
            data_path: env_optional_string("DATA_PATH", default_data_path()),
        })
    }
}

impl From<DatabaseConfig> for lilium_database::DatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DatabaseConfig::from_url(value.url, value.max_connections)
    }
}

impl From<NotificationConfig> for lilium_database::NotificationDatabaseConfig {
    fn from(value: NotificationConfig) -> Self {
        lilium_database::NotificationDatabaseConfig::from_url(value.url)
    }
}
