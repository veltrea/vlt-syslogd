//! ユーザー設定(待ち受けポート / ログ保存先)の永続化。
//!
//! 保存先は `platform::config_path()`(= データディレクトリ内の config.toml)で、
//! ログ本体や Server 版と置き場の思想を揃えている。TOML 形式。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 待ち受けポート(既定 514)。
    pub bind_port: u16,
    /// ログ保存先の上書き。空文字なら platform 既定(`platform::log_dir()`)を使う。
    pub log_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bind_port: 514,
            log_dir: String::new(),
        }
    }
}

/// 設定を読み込む。ファイルが無い/壊れている場合は既定値。
pub fn load() -> Settings {
    let path = crate::platform::config_path();
    match std::fs::read_to_string(&path) {
        Ok(s) => toml::from_str(&s).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

/// 設定を config.toml に書き出す(親ディレクトリが無ければ作成)。
pub fn save(s: &Settings) -> std::io::Result<()> {
    let path = crate::platform::config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(s).map_err(std::io::Error::other)?;
    std::fs::write(path, body)
}

/// 設定の上書きを考慮した実効ログディレクトリ。
/// log_dir が指定されていればそれを「ログ置き場そのもの」として使う(下に logs を作らない)。
pub fn effective_log_dir(s: &Settings) -> PathBuf {
    if s.log_dir.trim().is_empty() {
        crate::platform::log_dir()
    } else {
        PathBuf::from(s.log_dir.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_log_dir_falls_back_to_platform() {
        let s = Settings::default();
        assert_eq!(effective_log_dir(&s), crate::platform::log_dir());
    }

    #[test]
    fn explicit_log_dir_is_used_verbatim() {
        let s = Settings {
            bind_port: 5514,
            log_dir: "/tmp/my-logs".to_string(),
        };
        assert_eq!(effective_log_dir(&s), PathBuf::from("/tmp/my-logs"));
    }
}
