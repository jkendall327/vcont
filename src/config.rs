use serde::Deserialize;
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Deserialize, Debug)]
pub struct ScheduleItem {
    pub time: String,
    pub volume: u8,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub ramp_duration_seconds: u64,
    pub schedule: Vec<ScheduleItem>,
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<AppConfig, ConfigError> {
    let contents = tokio::fs::read_to_string(path).await?;
    let config: AppConfig = toml::from_str(&contents)?;
    Ok(config)
}
