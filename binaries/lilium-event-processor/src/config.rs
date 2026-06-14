use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub processor: ProcessorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessorConfig {
    #[serde(default = "default_polling_interval")]
    pub polling_interval_secs: u64,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_polling_interval() -> u64 {
    5
}
fn default_batch_size() -> usize {
    100
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
