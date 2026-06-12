use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub worker: WorkerConfig,
    pub processor: ProcessorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessorConfig {
    #[serde(default = "default_polling_interval")]
    pub polling_interval_secs: u64,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_pool_size() -> u32 { 5 }
fn default_queue_size() -> usize { 5000 }
fn default_batch_size() -> usize { 100 }
fn default_polling_interval() -> u64 { 5 }

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
