mod parser;
mod config;

use std::error::Error;
use std::ffi::OsString;
use std::panic;
use std::sync::mpsc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::runtime::Runtime;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

const SERVICE_NAME: &str = "vlt-syslog-srv";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

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
    if let Err(e) = init_logger(&config) {
        eprintln!("Failed to initialize logger: {}", e);
    }

    // 3. パニックハンドリングの設定
    setup_panic_hook();

    // 開発/デバッグ用：引数があればコマンドとして処理
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
                println!("Wait for Windows Service Manager if no args.");
            }
        }
    }

    // Windows サービスディスパッチャの起動
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

fn syslog_service_main(_arguments: Vec<OsString>) {
    // サービスのメインエントリーポイントでも再初期化を試みる
    let config = config::load_config().unwrap_or_default();
    let _ = init_logger(&config);
    
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

/// Windows サービスとしてのロギング初期化
fn init_logger(config: &config::Config) -> Result<(), Box<dyn Error>> {
    let log_dir = config::get_log_dir();

    // ディレクトリ作成
    if !log_dir.exists() {
        let _ = std::fs::create_dir_all(&log_dir);
    }

    flexi_logger::Logger::try_with_str(&config.logging.level)?
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

    Ok(())
}

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
