mod caddy;
mod certs;
mod hosts;
mod logs;
pub mod paths;
mod server;
pub mod tunnel;
mod xampp;

use tracing::info;

#[cfg(unix)]
fn verify_privileged() {
    let is_root = unsafe { libc::geteuid() == 0 };
    if !is_root {
        eprintln!("localdomain-daemon must be run as root");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn verify_privileged() {
    // Quick check: `net session` fails if not running as Administrator
    let is_admin = std::process::Command::new("net")
        .args(["session"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !is_admin {
        eprintln!("localdomain-daemon must be run as Administrator");
        std::process::exit(1);
    }
}

/// Core daemon logic — shared between direct execution and Windows Service mode.
/// `as_service`: true when running as a Windows Service (skips privilege check, logs to file).
async fn run_daemon(as_service: bool) -> anyhow::Result<()> {
    // Set up logging — file-based for Windows Service mode, stdout otherwise
    #[cfg(target_os = "windows")]
    {
        if as_service {
            std::fs::create_dir_all(paths::LOGS_DIR).ok();
            if let Ok(log_file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("{}\\daemon.log", paths::LOGS_DIR))
            {
                tracing_subscriber::fmt()
                    .with_env_filter(
                        tracing_subscriber::EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| "info".into()),
                    )
                    .with_writer(std::sync::Mutex::new(log_file))
                    .with_ansi(false)
                    .init();
            } else {
                tracing_subscriber::fmt()
                    .with_env_filter(
                        tracing_subscriber::EnvFilter::try_from_default_env()
                            .unwrap_or_else(|_| "info".into()),
                    )
                    .init();
            }
        } else {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "info".into()),
                )
                .init();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "info".into()),
            )
            .init();
    }

    // Skip privilege check when running as a Windows Service — services
    // run as SYSTEM which is inherently privileged, and `net session` is
    // unreliable in service context.
    if !as_service {
        verify_privileged();
    }

    // Ensure data directories exist
    std::fs::create_dir_all(paths::CERTS_DIR)?;
    std::fs::create_dir_all(paths::CADDY_DIR)?;
    std::fs::create_dir_all(paths::LOGS_DIR)?;
    std::fs::create_dir_all(paths::TUNNEL_DIR)?;

    #[cfg(target_os = "windows")]
    {
        // Also ensure the bin directory exists on Windows
        std::fs::create_dir_all("C:\\ProgramData\\LocalDomain\\bin").ok();
    }

    info!("localdomain-daemon starting");

    server::run_server().await
}

// ---- macOS / Linux: just run directly ----

#[cfg(unix)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_daemon(false).await
}

// ---- Windows: support both --console mode and Windows Service mode ----

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    // If --console flag is passed, run in console mode (for development/debugging)
    if std::env::args().any(|a| a == "--console") {
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(run_daemon(false));
    }

    // Otherwise, start as a Windows Service
    windows_service_main()
}

#[cfg(windows)]
fn windows_service_main() -> anyhow::Result<()> {
    use windows_service::service_dispatcher;

    service_dispatcher::start("localdomain-daemon", ffi_service_main)
        .map_err(|e| anyhow::anyhow!("Failed to start service dispatcher: {}", e))?;
    Ok(())
}

// The define_windows_service! macro creates the FFI wrapper that converts
// the raw Windows service entry point to a Rust-friendly signature.
#[cfg(windows)]
windows_service::define_windows_service!(ffi_service_main, service_main);

#[cfg(windows)]
fn service_main(_arguments: Vec<std::ffi::OsString>) {
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};

    let shutdown_tx: std::sync::Arc<tokio::sync::Notify> =
        std::sync::Arc::new(tokio::sync::Notify::new());
    let shutdown_rx = shutdown_tx.clone();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                shutdown_tx.notify_one();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = match service_control_handler::register("localdomain-daemon", event_handler)
    {
        Ok(h) => h,
        Err(_) => return,
    };

    // Report start-pending first so SCM doesn't time out while we bootstrap.
    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 1,
        wait_hint: std::time::Duration::from_secs(15),
        process_id: None,
    });

    // Build runtime before reporting Running.
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => {
            let _ = status_handle.set_service_status(ServiceStatus {
                service_type: ServiceType::OWN_PROCESS,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(1),
                checkpoint: 0,
                wait_hint: std::time::Duration::default(),
                process_id: None,
            });
            return;
        }
    };

    // Report running
    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    });

    let clean_stop = rt.block_on(async {
        let (daemon_done_tx, mut daemon_done_rx) =
            tokio::sync::oneshot::channel::<anyhow::Result<()>>();

        // Spawn the daemon server (as_service=true: skip privilege check, log to file)
        let server_handle = tokio::spawn(async move {
            let _ = daemon_done_tx.send(run_daemon(true).await);
        });

        tokio::select! {
            // Normal service stop/shutdown from SCM
            _ = shutdown_rx.notified() => {
                server_handle.abort();
                true
            }
            // Daemon crashed/exited unexpectedly
            daemon_result = &mut daemon_done_rx => {
                if let Ok(Err(e)) = daemon_result {
                    tracing::error!("Daemon error: {}", e);
                }
                false
            }
        }
    });

    let exit_code = if clean_stop { 0 } else { 1 };

    // Report final stopped state
    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(exit_code),
        checkpoint: 0,
        wait_hint: std::time::Duration::default(),
        process_id: None,
    });
}
