use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Application configuration stored as TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Selected device Bluetooth address.
    pub device_address: Option<String>,
    /// Selected device name.
    pub device_name: Option<String>,
    /// Auto-connect on startup.
    #[serde(default = "default_true")]
    pub auto_connect: bool,
    /// Start minimized to tray.
    #[serde(default)]
    pub start_minimized: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_address: None,
            device_name: None,
            auto_connect: true,
            start_minimized: false,
        }
    }
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

    /// Save config to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        tracing::info!("Config saved to {}", path.display());
        Ok(())
    }
}
