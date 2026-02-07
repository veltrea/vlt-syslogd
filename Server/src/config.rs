use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub max_size_mb: u64,
    pub keep_files: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind_addr: "0.0.0.0:514".to_string(),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                max_size_mb: 10,
                keep_files: 7,
            },
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let config_path = get_config_path();

    if !config_path.exists() {
        let default_config = Config::default();
        let toml_str = toml::to_string_pretty(&default_config)?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&config_path, toml_str)?;
        return Ok(default_config);
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}

pub fn get_config_path() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\ProgramData\vlt-syslogd\config.toml")
    } else {
        PathBuf::from("config.toml")
    }
}

pub fn get_log_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\ProgramData\vlt-syslogd\logs")
    } else {
        PathBuf::from("logs")
    }
}
