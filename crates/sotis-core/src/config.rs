use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// A folder to index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderEntry {
    pub path: PathBuf,
    #[serde(default = "default_true")]
    pub recursive: bool,
    #[serde(default)]
    pub extensions: Vec<String>,
}

/// General configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    #[serde(default)]
    pub ocr_enabled: bool,
    #[serde(default)]
    pub tessdata_path: Option<String>,
}

/// Top-level application config.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
            ocr_enabled: false,
            tessdata_path: None,
        }
    }
}

impl Config {
    /// Load the app configuration, creating a default file if needed.
    pub fn load() -> Result<Self> {
        Self::load_from_path(&config_path())
    }

    /// Load config from the provided path, creating a default file on first run.
    pub fn load_from_path(path: &Path) -> Result<Self> {
        if path.is_file() {
            let contents = fs::read_to_string(path).map_err(|source| Error::ConfigIo {
                path: path.to_path_buf(),
                source,
            })?;
            return toml::from_str(&contents).map_err(|source| Error::ConfigParse {
                path: path.to_path_buf(),
                source,
            });
        }

        let default_config = Self::default();
        default_config.save_to_path(path)?;
        Ok(default_config)
    }

    /// Save the app configuration to the resolved default path.
    pub fn save(&self) -> Result<()> {
        self.save_to_path(&config_path())
    }

    /// Save config to the provided path.
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| Error::ConfigIo {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(self).map_err(Error::ConfigSerialize)?;
        fs::write(path, serialized).map_err(|source| Error::ConfigIo {
            path: path.to_path_buf(),
            source,
        })
    }
}

/// Returns the config file path, respecting XDG and env overrides.
pub fn config_path() -> PathBuf {
    resolve_config_path_from_values(
        std::env::var_os("SOTIS_CONFIG").map(PathBuf::from),
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

/// Returns the data directory path for the search index.
pub fn data_dir() -> PathBuf {
    resolve_data_dir_from_values(
        std::env::var_os("SOTIS_DATA").map(PathBuf::from),
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )
}

fn default_true() -> bool {
    true
}

fn default_max_file_size() -> u64 {
    50
}

fn resolve_config_path_from_values(
    sotis_config: Option<PathBuf>,
    xdg_config_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = sotis_config {
        return path;
    }

    if let Some(path) = xdg_config_home {
        return path.join("sotis").join("config.toml");
    }

    if let Some(path) = home {
        return path.join(".config").join("sotis").join("config.toml");
    }

    PathBuf::from(".config").join("sotis").join("config.toml")
}

fn resolve_data_dir_from_values(
    sotis_data: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = sotis_data {
        return path;
    }

    if let Some(path) = xdg_data_home {
        return path.join("sotis");
    }

    if let Some(path) = home {
        return path.join(".local").join("share").join("sotis");
    }

    PathBuf::from(".local").join("share").join("sotis")
}

#[cfg(test)]
mod tests {
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert_eq!(config.general.max_file_size_mb, 50);
        assert!(!config.general.ocr_enabled);
        assert!(config.general.tessdata_path.is_none());
        assert!(config.folders.is_empty());
    }

    #[test]
    fn config_path_prefers_override_env_var() {
        let path = resolve_config_path_from_values(
            Some(PathBuf::from("/tmp/custom.toml")),
            Some(PathBuf::from("/tmp/xdg-config")),
            Some(PathBuf::from("/tmp/home")),
        );
        assert_eq!(path, PathBuf::from("/tmp/custom.toml"));
    }

    #[test]
    fn config_path_uses_xdg_then_home_fallback() {
        let xdg_path = resolve_config_path_from_values(
            None,
            Some(PathBuf::from("/tmp/xdg-config")),
            Some(PathBuf::from("/tmp/home")),
        );
        assert_eq!(xdg_path, PathBuf::from("/tmp/xdg-config/sotis/config.toml"));

        let home_fallback =
            resolve_config_path_from_values(None, None, Some(PathBuf::from("/tmp/home")));
        assert_eq!(
            home_fallback,
            PathBuf::from("/tmp/home/.config/sotis/config.toml")
        );
    }

    #[test]
    fn data_dir_prefers_override_and_falls_back_to_home() {
        let override_path = resolve_data_dir_from_values(
            Some(PathBuf::from("/tmp/sotis-data")),
            Some(PathBuf::from("/tmp/xdg-data")),
            Some(PathBuf::from("/tmp/home")),
        );
        assert_eq!(override_path, PathBuf::from("/tmp/sotis-data"));

        let xdg_path = resolve_data_dir_from_values(
            None,
            Some(PathBuf::from("/tmp/xdg-data")),
            Some(PathBuf::from("/tmp/home")),
        );
        assert_eq!(xdg_path, PathBuf::from("/tmp/xdg-data/sotis"));

        let home_fallback =
            resolve_data_dir_from_values(None, None, Some(PathBuf::from("/tmp/home")));
        assert_eq!(home_fallback, PathBuf::from("/tmp/home/.local/share/sotis"));
    }

    #[test]
    fn load_from_path_creates_default_config_on_first_run() {
        let tmp_dir = unique_temp_dir();
        let config_file = tmp_dir.join("sotis").join("config.toml");

        let loaded = Config::load_from_path(&config_file).expect("load should succeed");
        assert_eq!(loaded, Config::default());
        assert!(config_file.exists());

        cleanup_temp_dir(&tmp_dir);
    }

    #[test]
    fn save_and_load_round_trip() {
        let tmp_dir = unique_temp_dir();
        let config_file = tmp_dir.join("sotis").join("config.toml");

        let config = Config {
            general: GeneralConfig {
                max_file_size_mb: 128,
                ocr_enabled: true,
                tessdata_path: Some("/tmp/tessdata".to_string()),
            },
            folders: vec![FolderEntry {
                path: PathBuf::from("/tmp/projects"),
                recursive: false,
                extensions: vec![".rs".to_string(), ".md".to_string()],
            }],
        };

        config
            .save_to_path(&config_file)
            .expect("save should succeed");
        let loaded = Config::load_from_path(&config_file).expect("reload should succeed");
        assert_eq!(loaded, config);

        cleanup_temp_dir(&tmp_dir);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("sotis-config-tests-{}-{}", process::id(), nanos))
    }

    fn cleanup_temp_dir(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
