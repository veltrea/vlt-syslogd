mod parser;

use std::error::Error;
use std::ffi::OsString;
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "run" => {
                println!("Running as console app...");
                let rt = Runtime::new()?;
                rt.block_on(run_syslog_server())?;
                return Ok(());
            }
            _ => {
                println!("Usage: vlt-syslog-srv [run]");
                println!("Wait for Windows Service Manager if no args.");
            }
        }
    }

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

fn syslog_service_main(_arguments: Vec<OsString>) {
    if let Err(_e) = run_service() {
    }
}

fn run_service() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                tx.send(()).unwrap();
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

    let rt = Runtime::new()?;
    rt.block_on(async {
        tokio::select! {
            _ = run_syslog_server() => {},
            _ = tokio::task::spawn_blocking(move || rx.recv()) => {
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

async fn run_syslog_server() -> Result<(), Box<dyn Error>> {
    let addr = "0.0.0.0:514";
    let socket = UdpSocket::bind(addr).await?;

    println!("vlt-syslog-srv engine started on {}", addr);

    let mut buf = [0u8; 8192];
    loop {
        let (size, src) = socket.recv_from(&mut buf).await?;
        let raw_msg = &buf[..size];
        
        let parsed = parser::parse_syslog(raw_msg);
        
        let log_line = format!(
            "[{}] [{:?}] [src:{}] [enc:{}] {}",
            parsed.timestamp, parsed.severity, src, parsed.encoding, parsed.content
        );
        println!("{}", log_line);
    }
}
