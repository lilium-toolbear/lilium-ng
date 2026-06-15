use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub processor: ProcessorConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub polling_interval_secs: u64,
    pub batch_size: usize,
}

fn default_pool_size() -> u32 {
    5
}

fn default_polling_interval() -> u64 {
    5
}

fn default_batch_size() -> usize {
    100
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

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Self {
            database: DatabaseConfig {
                url: env_required_string("DATABASE_URL")?,
                max_connections: env_u32("DATABASE_POOL_SIZE", default_pool_size())?,
            },
            processor: ProcessorConfig {
                polling_interval_secs: env_u64(
                    "EVENT_PROCESSOR_POLLING_INTERVAL_SECS",
                    default_polling_interval(),
                )?,
                batch_size: env_usize("EVENT_PROCESSOR_BATCH_SIZE", default_batch_size())?,
            },
        })
    }
}

impl From<DatabaseConfig> for lilium_database::DatabaseConfig {
    fn from(value: DatabaseConfig) -> Self {
        lilium_database::DatabaseConfig::from_url(value.url, value.max_connections)
    }
}
