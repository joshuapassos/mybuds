use std::path::PathBuf;

use serde::Deserialize;

/// Application configuration stored as TOML.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Selected device Bluetooth address.
    pub device_address: Option<String>,
    /// Selected device name.
    pub device_name: Option<String>,
}

impl AppConfig {
    /// Config file path: ~/.config/mybuds/config.toml
    pub fn path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mybuds");
        config_dir.join("config.toml")
    }

    /// Load config from disk, or return defaults.
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => tracing::warn!("Failed to parse config: {}", e),
                },
                Err(e) => tracing::warn!("Failed to read config: {}", e),
            }
        }
        Self::default()
    }
}
