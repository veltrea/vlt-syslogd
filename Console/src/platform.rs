//! GUI フロントエンド(Console)の OS ごとのファイル保存先と、
//! ファイルマネージャ連携をここ1枚に集約する。
//!
//! Console は「自分の設定(config.toml)」だけを持つ表示専用アプリで、
//! 受信ログ本体は常駐サービス(Server 版)が `C:\ProgramData\vlt-syslogd` 等に残す。
//! ここで返すのは Console 自身の設定置き場(ユーザーデータ領域)であり、
//! サービスのデータ置き場とは別物(衝突しないようアプリ名を分ける)。
//!
//! どの OS でも、環境変数 `VLT_SYSLOGD_CONSOLE_DATA_DIR` があればそれを最優先する
//! (開発時のローカル実行や任意配置に使う)。

use std::path::PathBuf;

const APP: &str = "vlt-syslogd-console";

/// Console 自身の設定の保存先ルートディレクトリ。
pub fn data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("VLT_SYSLOGD_CONSOLE_DATA_DIR")
        && !custom.is_empty()
    {
        return PathBuf::from(custom);
    }
    app_data_dir()
}

/// 設定ファイルのパス(`<data_dir>/config.toml`)。
pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// OS 標準のユーザーデータ領域(`<...>/vlt-syslogd-console`)。
///
/// Console はユーザー権限で動く GUI なので、書き込みに管理者が要る ProgramData ではなく
/// 各 OS のユーザーデータ領域に置く。
fn app_data_dir() -> PathBuf {
    if cfg!(windows) {
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(base).join(APP)
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join("Library/Application Support")
            .join(APP)
    } else {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME")
            && !xdg.is_empty()
        {
            return PathBuf::from(xdg).join(APP);
        }
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local/share").join(APP)
    }
}

/// 指定ディレクトリを OS 標準のファイルマネージャ(Finder / エクスプローラー等)で開く。
///   - macOS : `open <dir>`
///   - Windows: `explorer <dir>`
///   - Linux : `xdg-open <dir>`
pub fn open_in_file_manager(dir: &std::path::Path) -> std::io::Result<()> {
    let _ = std::fs::create_dir_all(dir);

    let program = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(windows) {
        "explorer"
    } else {
        "xdg-open"
    };

    // explorer は対象が存在しても終了コード 1 を返すことがあるため、
    // spawn できたか(コマンドの有無)だけを成否の基準にする。
    std::process::Command::new(program).arg(dir).spawn().map(|_| ())
}
