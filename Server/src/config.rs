use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    /// GUI フロントエンドへ受信ログを配信する TCP アドレス(JSON Lines)。
    /// 既定はループバック限定(127.0.0.1)で外部には一切公開しない。
    /// 既存 config.toml(stream_addr 無し)との互換のため serde default で補う。
    #[serde(default = "default_stream_addr")]
    pub stream_addr: String,
    /// GUI フロントエンド(Console)からの設定取得/変更を受け付ける制御 TCP アドレス。
    /// 既定はループバック限定(127.0.0.1)で外部には一切公開しない。
    /// 既存 config.toml(control_addr 無し)との互換のため serde default で補う。
    #[serde(default = "default_control_addr")]
    pub control_addr: String,
}

/// stream_addr の既定値。ループバックの 5141 番。
fn default_stream_addr() -> String {
    "127.0.0.1:5141".to_string()
}

/// control_addr の既定値。ループバックの 5142 番。
fn default_control_addr() -> String {
    "127.0.0.1:5142".to_string()
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
                stream_addr: default_stream_addr(),
                control_addr: default_control_addr(),
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

/// 設定を config.toml に書き出す(親ディレクトリが無ければ作成)。
/// Console の制御ポートからの set_config で使う。
pub fn save_config(config: &Config) -> Result<(), Box<dyn Error>> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let toml_str = toml::to_string_pretty(config)?;
    fs::write(&config_path, toml_str)?;
    Ok(())
}

// 保存先の決定は platform モジュールに一元化した(CWD 相対をやめ、起動方法に依存しない)。
pub fn get_config_path() -> PathBuf {
    crate::platform::config_path()
}

pub fn get_log_dir() -> PathBuf {
    crate::platform::log_dir()
}
