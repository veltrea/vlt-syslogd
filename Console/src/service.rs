//! 常駐サービス(Server 版 = vlt-syslogd-srv)の状態監視と制御を OS ごとに抽象化する。
//!
//! - 状態取得(`status`)は基本的に管理者権限なしで行える(読み取りのみ)。
//! - 開始/停止/再起動はサービスマネージャの操作なので **管理者権限が要る**。
//!   各 OS の作法で権限昇格を伴って実行する:
//!     - Windows : `sc.exe` を PowerShell の `Start-Process -Verb RunAs`(UAC)経由で実行
//!     - macOS   : `launchctl` を osascript の「管理者として実行」経由で実行
//!     - Linux   : `systemctl`(pkexec があればそれ経由で昇格)
//!
//! サービス識別子は各 OS のインストール手順に合わせる:
//!   - Windows サービス名 : `vlt-syslogd-srv`(Server/src/main.rs の SERVICE_NAME)
//!   - macOS launchd ラベル: `com.veltrea.vlt-syslogd-srv`(install-macos.sh)
//!   - Linux systemd ユニット: `vlt-syslogd-srv.service`(慣例。自動化スクリプトは未提供)

use std::process::Command;

/// サービスの稼働状態。GUI のステータス表示に使う。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceStatus {
    /// 稼働中。
    Running,
    /// インストール済みだが停止中。
    Stopped,
    /// サービスが未インストール(登録されていない)。
    NotInstalled,
    /// 判定できなかった(コマンド失敗・権限不足など)。中身は理由。
    Unknown(String),
}

impl ServiceStatus {
    /// GUI 表示用の短いラベルとアイコン。
    pub fn label(&self) -> String {
        match self {
            ServiceStatus::Running => "🟢 稼働中".to_string(),
            ServiceStatus::Stopped => "⚪ 停止中".to_string(),
            ServiceStatus::NotInstalled => "❌ 未インストール".to_string(),
            ServiceStatus::Unknown(why) => format!("❓ 不明 ({})", why),
        }
    }
}

#[cfg(windows)]
const WIN_SERVICE_NAME: &str = "vlt-syslogd-srv";
#[cfg(target_os = "macos")]
const MAC_PLIST: &str = "/Library/LaunchDaemons/com.veltrea.vlt-syslogd-srv.plist";
#[cfg(target_os = "macos")]
const MAC_LABEL: &str = "com.veltrea.vlt-syslogd-srv";
#[cfg(all(unix, not(target_os = "macos")))]
const LINUX_UNIT: &str = "vlt-syslogd-srv.service";

// ============================ Windows ============================
#[cfg(windows)]
pub fn status() -> ServiceStatus {
    // `sc query` は一般ユーザーでも実行できる。
    let out = match Command::new("sc").args(["query", WIN_SERVICE_NAME]).output() {
        Ok(o) => o,
        Err(e) => return ServiceStatus::Unknown(e.to_string()),
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    // 1060 = サービスが存在しない。
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        if stdout.contains("1060") || stderr.contains("1060") {
            return ServiceStatus::NotInstalled;
        }
        return ServiceStatus::Unknown(stderr.trim().to_string());
    }
    if stdout.contains("RUNNING") {
        ServiceStatus::Running
    } else if stdout.contains("STOPPED") || stdout.contains("STOP_PENDING") {
        ServiceStatus::Stopped
    } else {
        ServiceStatus::Unknown("状態をパースできませんでした".to_string())
    }
}

#[cfg(windows)]
fn run_elevated_sc(verb: &str) -> Result<(), String> {
    // sc.exe を UAC 昇格して実行する。管理者でなくても UAC 承認で操作できる。
    // Start-Process -Verb RunAs は新プロセスを起こすため、成否は status() の再ポーリングで確認する。
    let ps_cmd = format!(
        "Start-Process -FilePath 'sc.exe' -ArgumentList '{}','{}' -Verb RunAs",
        verb, WIN_SERVICE_NAME
    );
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(windows)]
pub fn start() -> Result<(), String> {
    ensure_installed()?;
    run_elevated_sc("start")
}
#[cfg(windows)]
pub fn stop() -> Result<(), String> {
    ensure_installed()?;
    run_elevated_sc("stop")
}

// ============================ macOS ============================
#[cfg(target_os = "macos")]
pub fn status() -> ServiceStatus {
    // plist が無ければ未インストール扱い。
    if !std::path::Path::new(MAC_PLIST).exists() {
        return ServiceStatus::NotInstalled;
    }
    // `launchctl list <label>` は読み取り。ロード済みなら 0、未ロードなら非 0。
    let out = match Command::new("launchctl").args(["list", MAC_LABEL]).output() {
        Ok(o) => o,
        Err(e) => return ServiceStatus::Unknown(e.to_string()),
    };
    if !out.status.success() {
        // plist はあるがロードされていない = 停止中。
        return ServiceStatus::Stopped;
    }
    // 出力に "PID" = N があり 0 でなければ稼働中。"PID" 行が無ければロード済みだが非実行。
    let stdout = String::from_utf8_lossy(&out.stdout);
    let has_pid = stdout
        .lines()
        .any(|l| l.contains("\"PID\"") && !l.contains("= 0;"));
    if has_pid {
        ServiceStatus::Running
    } else {
        ServiceStatus::Stopped
    }
}

#[cfg(target_os = "macos")]
fn run_admin_osascript(shell_cmd: &str) -> Result<(), String> {
    // osascript で「管理者として実行」(GUI のパスワードプロンプト)。
    let script = format!(
        "do shell script \"{}\" with administrator privileges",
        shell_cmd.replace('\\', "\\\\").replace('"', "\\\"")
    );
    let out = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(target_os = "macos")]
pub fn start() -> Result<(), String> {
    ensure_installed()?;
    // 旧 `launchctl load` は非存在/失敗でも exit 0 を返すため使わない。
    // install-macos.sh と同じ `bootstrap`(正しい終了コードを返す)を使う。
    run_admin_osascript(&format!("launchctl bootstrap system {}", MAC_PLIST))
}
#[cfg(target_os = "macos")]
pub fn stop() -> Result<(), String> {
    ensure_installed()?;
    // 旧 `launchctl unload` 同様に `bootout` を使う(uninstall-macos.sh と同じ)。
    run_admin_osascript(&format!("launchctl bootout system/{}", MAC_LABEL))
}

// ============================ Linux ============================
#[cfg(all(unix, not(target_os = "macos")))]
pub fn status() -> ServiceStatus {
    // `systemctl is-active` は読み取り。active / inactive / failed / unknown を返す。
    let out = match Command::new("systemctl")
        .args(["is-active", LINUX_UNIT])
        .output()
    {
        Ok(o) => o,
        Err(e) => return ServiceStatus::Unknown(e.to_string()),
    };
    let s = String::from_utf8_lossy(&out.stdout);
    match s.trim() {
        "active" => ServiceStatus::Running,
        "inactive" | "deactivating" => ServiceStatus::Stopped,
        "failed" => ServiceStatus::Stopped,
        other => {
            // unit が無いと "unknown" や空が返る。is-enabled でインストール有無を補足判定。
            let enabled = Command::new("systemctl")
                .args(["is-enabled", LINUX_UNIT])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if enabled {
                ServiceStatus::Stopped
            } else if other.is_empty() || other == "unknown" {
                ServiceStatus::NotInstalled
            } else {
                ServiceStatus::Unknown(other.to_string())
            }
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn run_elevated_systemctl(verb: &str) -> Result<(), String> {
    // pkexec があれば GUI で昇格。無ければ sudo を試す(端末が無いと失敗しうる)。
    let elevator = if Command::new("which")
        .arg("pkexec")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        "pkexec"
    } else {
        "sudo"
    };
    let out = Command::new(elevator)
        .args(["systemctl", verb, LINUX_UNIT])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn start() -> Result<(), String> {
    ensure_installed()?;
    run_elevated_systemctl("start")
}
#[cfg(all(unix, not(target_os = "macos")))]
pub fn stop() -> Result<(), String> {
    ensure_installed()?;
    run_elevated_systemctl("stop")
}

// ============================ 共通 ============================

/// 開始/停止/再起動の前提チェック。サービスが未インストールなら、
/// 権限昇格(認証ダイアログ)を出す **前に** エラーで弾く。
///
/// これを入れないと、未インストールでも `launchctl`/`sc`/`systemctl` を昇格実行してしまい、
/// ・無駄に管理者認証ダイアログが出る
/// ・macOS の旧 `launchctl load/unload` は非存在 plist でも exit 0 を返すため、
///   「成功」と誤判定する(偽の成功表示)
/// という不具合になる。
fn ensure_installed() -> Result<(), String> {
    if status() == ServiceStatus::NotInstalled {
        return Err("サービスが未インストールです。先にインストールしてください。".to_string());
    }
    Ok(())
}

/// 再起動 = 停止してから開始。
///
/// - 未インストールなら認証を出す前にエラーを返す。
/// - 「停止済み」からの再起動では stop が失敗扱いになりうる(既にロードされていない等)。
///   再起動の成否は最終的な start で判断するため、stop の失敗は無視する。
/// - OS によっては stop が時間差で効くため、呼び出し側は status() の再ポーリングでも反映を確認すること。
pub fn restart() -> Result<(), String> {
    ensure_installed()?;
    let _ = stop();
    start()
}
