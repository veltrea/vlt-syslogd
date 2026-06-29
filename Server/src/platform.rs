//! OS ごとのデータ保存先をここ1枚に集約する(常駐デーモン向け)。
//!
//! Server は root / サービスマネージャ(launchd・systemd・Windows サービス)から
//! 起動される常駐プロセスなので、データ置き場は「OS 標準のシステム領域」に固定する。
//! 旧実装はカレントディレクトリ相対(`config.toml` / `logs`)で、起動方法によって
//! 場所がぶれていた。それを解消し、起動 CWD に依存しないようにする。
//!
//! どの OS でも、環境変数 `VLT_SYSLOGD_DATA_DIR` があればそれを最優先する
//! (Portable 版と同じキー。開発時のローカル実行や任意配置に使う)。

use std::path::PathBuf;

/// データ(config.toml / logs)の保存先ルートディレクトリ。
///
/// 既定のシステム領域:
///   - Windows : `C:\ProgramData\vlt-syslogd`
///   - macOS   : `/usr/local/var/vlt-syslogd`(install-macos.sh の DATA_DIR と一致)
///   - Linux   : `/var/lib/vlt-syslogd`(FHS の可変状態データ置き場)
pub fn data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("VLT_SYSLOGD_DATA_DIR")
        && !custom.is_empty()
    {
        return PathBuf::from(custom);
    }

    if cfg!(windows) {
        PathBuf::from(r"C:\ProgramData\vlt-syslogd")
    } else if cfg!(target_os = "macos") {
        PathBuf::from("/usr/local/var/vlt-syslogd")
    } else {
        PathBuf::from("/var/lib/vlt-syslogd")
    }
}

/// 設定ファイルのパス(`<data_dir>/config.toml`)。
pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// ログ保存ディレクトリ(`<data_dir>/logs`)。
pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 環境変数はプロセス全体で共有されるため、並列テストで取り合わないよう直列化する。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn env_override_wins() {
        let _guard = ENV_LOCK.lock().unwrap();
        // edition 2024 では env 変更は unsafe
        unsafe { std::env::set_var("VLT_SYSLOGD_DATA_DIR", "/tmp/vlt-srv-test") };
        assert_eq!(data_dir(), PathBuf::from("/tmp/vlt-srv-test"));
        assert_eq!(config_path(), PathBuf::from("/tmp/vlt-srv-test/config.toml"));
        assert_eq!(log_dir(), PathBuf::from("/tmp/vlt-srv-test/logs"));
        unsafe { std::env::remove_var("VLT_SYSLOGD_DATA_DIR") };
    }

    #[test]
    fn default_is_absolute_system_path() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("VLT_SYSLOGD_DATA_DIR") };
        let dir = data_dir();
        assert!(dir.is_absolute());
        assert!(dir.ends_with("vlt-syslogd"));
    }
}
