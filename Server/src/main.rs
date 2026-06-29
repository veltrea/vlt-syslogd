mod parser;
mod config;
mod platform;

use std::error::Error;
use std::panic;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UdpSocket};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

// --- Windows サービス連携（Windows ターゲットでのみコンパイル）---
#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::sync::mpsc;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

#[cfg(windows)]
const SERVICE_NAME: &str = "vlt-syslogd-srv";
#[cfg(windows)]
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

#[cfg(windows)]
define_windows_service!(ffi_service_main, syslog_service_main);

fn main() -> Result<(), Box<dyn Error>> {
    // 1. 設定の読み込み
    let config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            config::Config::default()
        }
    };

    // 2. ロガーの初期化（プロセスの最優先事項）
    //    LoggerHandle は WriteMode::Async のバックグラウンド書き込みスレッドを生かし続ける。
    //    早期に drop すると以降のログ出力が "Send" エラーで全て失われるため、
    //    main の終わりまで保持する。
    let _logger_handle = match init_logger(&config) {
        Ok(handle) => Some(handle),
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            None
        }
    };

    // 3. パニックハンドリングの設定
    setup_panic_hook();

    // 開発/デバッグ用：引数があればコマンドとして処理（全プラットフォーム共通）
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "run" => {
                log::info!("Running as console app mode...");
                let rt = Runtime::new()?;
                rt.block_on(run_syslog_server(config))?;
                return Ok(());
            }
            _ => {
                println!("Usage: vlt-syslogd-srv [run]");
                #[cfg(windows)]
                println!("Wait for Windows Service Manager if no args.");
                #[cfg(not(windows))]
                println!("With no args, runs as a foreground console daemon (macOS / Linux).");
            }
        }
    }

    // 引数なしの既定動作はプラットフォームで分岐する
    #[cfg(windows)]
    {
        // Windows: サービスディスパッチャに制御を渡す
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    }
    #[cfg(not(windows))]
    {
        // macOS / Linux: フォアグラウンドのコンソール常駐サーバとして動作
        // （launchd / systemd から起動する想定。Ctrl+C で停止）
        log::info!("Running as console daemon (non-Windows)...");
        println!("vlt-syslogd-srv: console daemon mode. Press Ctrl+C to stop.");
        let rt = Runtime::new()?;
        rt.block_on(run_syslog_server(config))?;
    }

    Ok(())
}

#[cfg(windows)]
fn syslog_service_main(_arguments: Vec<OsString>) {
    // サービスのメインエントリーポイントでも再初期化を試みる。
    // LoggerHandle は run_service の実行が終わるまで保持する（早期 drop で async ログが失われる）。
    let config = config::load_config().unwrap_or_default();
    let _logger_handle = init_logger(&config).ok();

    if let Err(e) = run_service(config) {
        log::error!("Service runtime error: {}", e);
    }
}

/// パニック時に詳細をログに残し、サービスマネージャに異常終了を伝える
fn setup_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            &s[..]
        } else {
            "Box<Any>"
        };
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_else(|| "unknown".to_string());
        log::error!("PANIC occurred at {}: {}", location, msg);
    }));
}

/// ロギング初期化。返す LoggerHandle は呼び出し側がプログラム終了まで保持すること。
fn init_logger(
    config: &config::Config,
) -> Result<flexi_logger::LoggerHandle, Box<dyn Error>> {
    let log_dir = config::get_log_dir();

    // ディレクトリ作成
    if !log_dir.exists() {
        let _ = std::fs::create_dir_all(&log_dir);
    }

    let handle = flexi_logger::Logger::try_with_str(&config.logging.level)?
        .log_to_file(
            flexi_logger::FileSpec::default()
                .directory(log_dir)
                .basename("vlt-syslogd-srv")
                .suffix("log"),
        )
        .write_mode(flexi_logger::WriteMode::Async) // 非同期書き込みでパフォーマンス向上
        .format(flexi_logger::opt_format)
        .rotate(
            flexi_logger::Criterion::Size(config.logging.max_size_mb * 1024 * 1024),
            flexi_logger::Naming::Numbers,
            flexi_logger::Cleanup::KeepLogFiles(config.logging.keep_files),
        )
        .start()?;

    Ok(handle)
}

#[cfg(windows)]
fn run_service(config: config::Config) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                log::info!("Received stop control from Service Manager");
                let _ = tx.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // 非同期ランタイムの起動
    let rt = Runtime::new()?;
    rt.block_on(async {
        tokio::select! {
            result = run_syslog_server(config) => {
                if let Err(e) = result {
                    log::error!("Syslog server loop error: {}", e);
                }
            },
            _ = tokio::task::spawn_blocking(move || rx.recv()) => {
                log::info!("Service stopping gracefully...");
            }
        }
    });

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

async fn run_syslog_server(config: config::Config) -> Result<(), Box<dyn Error>> {
    let addr = config.server.bind_addr.clone();
    let socket = UdpSocket::bind(&addr).await?;

    log::info!("vlt-syslogd-srv engine started on {}", addr);

    // GUI フロントエンドへ受信ログを配信する broadcast チャネル。
    // 購読者(接続中の GUI)がいなければ送信は黙って捨てられる。
    // これによりサービス本体は GUI の有無に一切依存せず動き続ける。
    let (stream_tx, _) = broadcast::channel::<String>(1024);

    // TCP 配信タスクを起動する。listen に失敗しても(ポート使用中など)
    // サービス本体(UDP 受信 + ファイルログ)は止めない。配信だけが無効になる。
    {
        let stream_addr = config.server.stream_addr.clone();
        let stream_tx = stream_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = run_stream_server(&stream_addr, stream_tx).await {
                log::error!("Stream listener on {} terminated: {}", stream_addr, e);
            }
        });
    }

    // 設定の取得/変更を受け付ける制御サーバを起動する。listen に失敗しても
    // サービス本体(UDP 受信 + 配信)は止めない。制御だけが無効になる。
    {
        let control_addr = config.server.control_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = run_control_server(&control_addr).await {
                log::error!("Control listener on {} terminated: {}", control_addr, e);
            }
        });
    }

    let mut buf = [0u8; 8192];
    loop {
        let (size, src) = socket.recv_from(&mut buf).await?;
        let raw_msg = &buf[..size];

        let parsed = parser::parse_syslog(raw_msg);

        // サービス版：全受信メッセージをINFOレベルで記録
        log::info!(
            "[{:?}] [src:{}] [enc:{}] {}",
            parsed.severity, src, parsed.encoding, parsed.content
        );

        // GUI フロントエンドへ JSON Lines(1メッセージ=1行 JSON)で配信する。
        // 購読者がいなければ send は Err になるが、その場合は捨ててよい。
        if let Ok(json) = serde_json::to_string(&parsed) {
            let _ = stream_tx.send(json);
        }
    }
}

/// GUI フロントエンド向けの TCP 配信サーバ(JSON Lines)。
///
/// ループバック(既定 127.0.0.1:5141)で listen し、接続してきた各 GUI クライアントへ
/// broadcast チャネルの JSON 行を流す。接続ごとに独立したタスクで購読する。
/// 外部公開しない前提なので認証は持たない(bind を 0.0.0.0 等にする運用は想定しない)。
async fn run_stream_server(
    addr: &str,
    stream_tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("vlt-syslogd-srv stream listener started on {}", addr);

    loop {
        let (mut socket, peer) = listener.accept().await?;
        log::info!("GUI client connected from {}", peer);
        let mut rx = stream_tx.subscribe();

        // 接続クライアントごとに購読タスクを分離する。
        // 1 クライアントの切断・遅延が他クライアントや本体に波及しないようにする。
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(line) => {
                        // JSON 行 + 改行。書き込み失敗は切断とみなしてタスク終了。
                        if socket.write_all(line.as_bytes()).await.is_err()
                            || socket.write_all(b"\n").await.is_err()
                        {
                            break;
                        }
                    }
                    // 受信が追いつかず取りこぼした場合。最新を優先してスキップ継続。
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        log::warn!("GUI client {} lagged; skipped {} messages", peer, n);
                        continue;
                    }
                    // 送信側(サービス本体)が終了した場合。
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            log::info!("GUI client {} disconnected", peer);
        });
    }
}

/// Console(GUI フロントエンド)からの設定取得/変更を受け付ける制御サーバ。
///
/// ループバック(既定 127.0.0.1:5142)で listen し、接続ごとに 1 行 JSON のリクエストを
/// 受けて 1 行 JSON のレスポンスを返す(行区切り JSON / JSONL。Content-Length は付けない)。
/// stream と同じく外部公開しない前提なので認証は持たない。
///
/// set_config は config.toml を書き換えるのみで、反映はサービス再起動で行う方針
/// (動作中プロセスのホットリロードはしない)。レスポンスで restart_required を返す。
async fn run_control_server(addr: &str) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("vlt-syslogd-srv control listener started on {}", addr);

    loop {
        let (socket, peer) = listener.accept().await?;
        tokio::spawn(async move {
            let mut reader = BufReader::new(socket);
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => return, // 何も送られてこなかった。
                Ok(_) => {}
                Err(e) => {
                    log::warn!("control read error from {}: {}", peer, e);
                    return;
                }
            }
            let response = handle_control(&line);
            let mut socket = reader.into_inner();
            if socket.write_all(response.as_bytes()).await.is_err()
                || socket.write_all(b"\n").await.is_err()
            {
                log::warn!("control write failed to {}", peer);
            }
        });
    }
}

/// 制御リクエスト 1 行を処理してレスポンス JSON(1 行ぶん)を返す。
fn handle_control(line: &str) -> String {
    let err = |msg: String| serde_json::json!({ "ok": false, "error": msg }).to_string();

    let value: serde_json::Value = match serde_json::from_str(line.trim()) {
        Ok(v) => v,
        Err(e) => return err(format!("invalid json: {e}")),
    };

    match value.get("cmd").and_then(|c| c.as_str()) {
        Some("get_config") => match config::load_config() {
            Ok(cfg) => serde_json::json!({ "ok": true, "config": cfg }).to_string(),
            Err(e) => err(format!("failed to load config: {e}")),
        },
        Some("set_config") => {
            let cfg_value = match value.get("config") {
                Some(v) => v.clone(),
                None => return err("missing 'config' field".to_string()),
            };
            let cfg: config::Config = match serde_json::from_value(cfg_value) {
                Ok(c) => c,
                Err(e) => return err(format!("invalid config: {e}")),
            };
            match config::save_config(&cfg) {
                Ok(()) => {
                    log::info!("config updated via control port (restart required to apply)");
                    serde_json::json!({ "ok": true, "restart_required": true }).to_string()
                }
                Err(e) => err(format!("failed to save config: {e}")),
            }
        }
        other => err(format!("unknown cmd: {:?}", other)),
    }
}
