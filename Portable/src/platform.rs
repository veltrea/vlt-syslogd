//! OS ごと・配布形態ごとのファイル保存先をここ1枚に集約する。
//!
//! このファイルだけを見れば「データ(logs / config)がどこに置かれるか」が
//! 全部分かる、という状態を保つこと(散らかった `cfg!` を増やさない)。
//!
//! 配布形態は cargo の feature `portable` で切り替える:
//!   - `--features portable` あり … 実行ファイルの隣に書く(USB 持ち歩き・ゼロフットプリント)
//!   - feature なし(既定)        … OS 標準のユーザーデータ領域に書く(普通のアプリ)
//!
//! どちらの形態でも、環境変数 `VLT_SYSLOGD_DATA_DIR` があればそれを最優先する。

use std::path::PathBuf;

/// データ(logs / config.toml)の保存先ルートディレクトリ。
///
/// 優先順:
///   1. 環境変数 `VLT_SYSLOGD_DATA_DIR`(Portable/App どちらでも効く逃げ道)
///   2. feature `portable` あり → 実行ファイルの隣
///   3. feature `portable` なし → OS 標準のユーザーデータ領域
pub fn data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("VLT_SYSLOGD_DATA_DIR")
        && !custom.is_empty()
    {
        return PathBuf::from(custom);
    }

    if cfg!(feature = "portable") {
        exe_sibling_dir()
    } else {
        app_data_dir()
    }
}

/// ログ保存ディレクトリ(`<data_dir>/logs`)。
pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

/// 設定ファイルのパス(`<data_dir>/config.toml`)。
/// 現状 Portable では未使用だが、Server と設定の置き場を揃えるため用意しておく。
#[allow(dead_code)]
pub fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

/// 実行ファイル自身の隣のディレクトリ。
///
/// macOS の `.app` では実行ファイルが `Foo.app/Contents/MacOS/<bin>` に居るため、
/// バンドルの中ではなく「`.app` の隣」(= `.app` を含むフォルダ)を返す。
/// バンドル内に書くと署名が壊れ、App Translocation 下では読み取り専用なので避ける。
fn exe_sibling_dir() -> PathBuf {
    let Ok(exe) = std::env::current_exe() else {
        return PathBuf::from(".");
    };
    let Some(dir) = exe.parent() else {
        return PathBuf::from(".");
    };

    if cfg!(target_os = "macos") && dir.ends_with("Contents/MacOS") {
        // dir = .../Foo.app/Contents/MacOS
        //   ancestors: 0=MacOS, 1=Contents, 2=Foo.app, 3=.app の親
        if let Some(parent_of_app) = dir.ancestors().nth(3) {
            return parent_of_app.to_path_buf();
        }
    }

    dir.to_path_buf()
}

/// OS 標準のユーザーデータ領域(`<...>/vlt-syslogd`)。
fn app_data_dir() -> PathBuf {
    const APP: &str = "vlt-syslogd";

    if cfg!(windows) {
        // GUI/App はユーザー権限で動くので、書き込みに管理者が要る ProgramData ではなく
        // %APPDATA%(Roaming)を使う。
        let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(base).join(APP)
    } else if cfg!(target_os = "macos") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join("Library/Application Support")
            .join(APP)
    } else {
        // Linux: XDG_DATA_HOME があればそれ、無ければ ~/.local/share
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
///
/// メニューの「ログフォルダを開く」から使う。OS ごとに起動コマンドが異なるため
/// ここで吸収する:
///   - macOS : `open <dir>`(Finder)
///   - Windows: `explorer <dir>`(エクスプローラー)
///   - Linux : `xdg-open <dir>`(既定のファイルマネージャ)
///
/// 開けなくても致命的ではないので、失敗は呼び出し側で握りつぶせるよう Result で返す。
pub fn open_in_file_manager(dir: &std::path::Path) -> std::io::Result<()> {
    // 開く前にディレクトリを作っておく(初回起動直後でまだ無いことがある)。
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 環境変数を触るテストはプロセス全体で共有する env をいじるため、
    // 並列実行だと取り合いになる。Mutex で直列化する。
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// 環境変数 VLT_SYSLOGD_DATA_DIR は配布形態に関わらず最優先される。
    #[test]
    fn env_override_wins() {
        let _guard = ENV_LOCK.lock().unwrap();
        // edition 2024 では env の変更は unsafe(他スレッドとの競合が UB になりうる)
        unsafe { std::env::set_var("VLT_SYSLOGD_DATA_DIR", "/tmp/vlt-test-dir") };
        assert_eq!(data_dir(), PathBuf::from("/tmp/vlt-test-dir"));
        assert_eq!(log_dir(), PathBuf::from("/tmp/vlt-test-dir/logs"));
        unsafe { std::env::remove_var("VLT_SYSLOGD_DATA_DIR") };
    }

    /// 既定(feature なし = App 版)では OS 標準のユーザーデータ領域に落ちる。
    #[test]
    #[cfg(not(feature = "portable"))]
    fn app_build_uses_user_data_dir() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("VLT_SYSLOGD_DATA_DIR") };
        let dir = data_dir();
        if cfg!(target_os = "macos") {
            assert!(dir.ends_with("Library/Application Support/vlt-syslogd"));
        } else if cfg!(windows) {
            assert!(dir.ends_with("vlt-syslogd"));
        } else {
            assert!(dir.ends_with("vlt-syslogd"));
        }
    }

    /// Portable 版では実行ファイルの隣(= current_exe の親)を基準にする。
    #[test]
    #[cfg(feature = "portable")]
    fn portable_build_uses_exe_sibling() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe { std::env::remove_var("VLT_SYSLOGD_DATA_DIR") };
        // テストバイナリの隣を指すはずで、少なくとも絶対パスになる
        assert!(data_dir().is_absolute());
    }
}
