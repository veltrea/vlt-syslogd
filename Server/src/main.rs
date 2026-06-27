mod parser;
mod config;

use std::error::Error;
use std::panic;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;

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
const SERVICE_NAME: &str = "vlt-syslog-srv";
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
                println!("Usage: vlt-syslog-srv [run]");
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
        println!("vlt-syslog-srv: console daemon mode. Press Ctrl+C to stop.");
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
                .basename("vlt-syslog-srv")
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
    let addr = &config.server.bind_addr;
    let socket = UdpSocket::bind(addr).await?;

    log::info!("vlt-syslog-srv engine started on {}", addr);

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
    }
}
