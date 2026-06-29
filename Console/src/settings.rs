//! Console(GUI フロントエンド)の設定の永続化。
//!
//! 保存先は `platform::config_path()`(= Console 用データディレクトリ内の config.toml)。
//! 持つのは「接続先サービスのアドレス」だけ。受信ログ本体の保存は
//! サービス側の責務なので、ここでは扱わない。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 接続先サービスの TCP 配信アドレス(host:port)。
    /// 既定はサービスの既定 `stream_addr` と同じ 127.0.0.1:5141。
    pub server_addr: String,
    /// 接続先サービスの制御アドレス(host:port)。設定の取得/変更に使う。
    /// 既定はサービスの既定 `control_addr` と同じ 127.0.0.1:5142。
    pub control_addr: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:5141".to_string(),
            control_addr: "127.0.0.1:5142".to_string(),
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
