use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A folder to index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderEntry {
    pub path: PathBuf,
    #[serde(default = "default_true")]
    pub recursive: bool,
    #[serde(default)]
    pub extensions: Vec<String>,
}

/// General configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
}

/// Top-level application config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub folders: Vec<FolderEntry>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: default_max_file_size(),
        }
    }
}

/// Returns the config file path, respecting XDG and env overrides.
pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("SOTIS_CONFIG") {
        return PathBuf::from(p);
    }
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("sotis");
    config_dir.join("config.toml")
}

/// Returns the data directory path for the search index.
pub fn data_dir() -> PathBuf {
    if let Ok(p) = std::env::var("SOTIS_DATA") {
        return PathBuf::from(p);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join("sotis")
}

fn default_true() -> bool {
    true
}

fn default_max_file_size() -> u64 {
    50
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert_eq!(config.general.max_file_size_mb, 50);
        assert!(config.folders.is_empty());
    }
}
