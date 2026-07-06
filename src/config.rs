use std::path::PathBuf;

use serde::Deserialize;

const DEFAULT_FORMAT: &str = "{index}. {title}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: DEFAULT_FORMAT.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    format: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let Some(config_dir) = std::env::var_os("HERDR_PLUGIN_CONFIG_DIR") else {
            return Ok(Self::default());
        };

        Self::load_from_dir(PathBuf::from(config_dir))
    }

    pub fn load_from_dir(config_dir: PathBuf) -> Result<Self, ConfigError> {
        let path = config_dir.join("config.toml");
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path).map_err(ConfigError::Io)?;
        let file: ConfigFile = toml::from_str(&contents).map_err(ConfigError::Toml)?;

        Ok(Self {
            format: file.format.unwrap_or_else(|| Self::default().format),
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "failed to read config: {error}"),
            Self::Toml(error) => write!(f, "failed to parse config: {error}"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn temp_config_dir(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("herdr-tab-title-{name}-{nonce}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn uses_default_format_when_config_file_is_missing() {
        let dir = temp_config_dir("missing-config");

        let config = Config::load_from_dir(dir).unwrap();

        assert_eq!(config.format, DEFAULT_FORMAT);
    }

    #[test]
    fn uses_default_format_when_config_omits_format() {
        let dir = temp_config_dir("missing-format");
        std::fs::write(dir.join("config.toml"), "# no format configured\n").unwrap();

        let config = Config::load_from_dir(dir).unwrap();

        assert_eq!(config.format, DEFAULT_FORMAT);
    }
}
