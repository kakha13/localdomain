use anyhow::Result;
use localdomain_shared::protocol::*;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{error, info};

use crate::caddy;
use crate::certs;
use crate::hosts;
use crate::logs;
use crate::paths;
use crate::tunnel;
use crate::xampp;

// --- Platform-specific server entry points ---

#[cfg(unix)]
pub async fn run_server() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use tokio::net::UnixListener;

    // Remove stale socket
    let _ = std::fs::remove_file(paths::SOCKET_PATH);

    let listener = UnixListener::bind(paths::SOCKET_PATH)?;

    // Set socket permissions so unprivileged app can connect
    std::fs::set_permissions(paths::SOCKET_PATH, std::fs::Permissions::from_mode(0o666))?;

    info!("Daemon listening on {}", paths::SOCKET_PATH);

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(async move {
                    let (reader, writer) = stream.into_split();
                    if let Err(e) = handle_connection(reader, writer).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

#[cfg(windows)]
pub async fn run_server() -> Result<()> {
    info!("Daemon listening on {}", paths::PIPE_NAME);

    // Create first pipe instance with permissive security (allows non-admin users)
    let mut server = create_pipe_instance(true)?;

    loop {
        server.connect().await?;

        let connected = server;
        server = create_pipe_instance(false)?;

        tokio::spawn(async move {
            let (reader, writer) = tokio::io::split(connected);
            if let Err(e) = handle_connection(reader, writer).await {
                error!("Connection error: {}", e);
            }
        });
    }
}

/// Create a named pipe server instance with a NULL DACL security descriptor.
/// This is the Windows equivalent of `chmod 0o666` on the Unix socket —
/// it allows unprivileged user processes to connect to the SYSTEM-owned pipe.
#[cfg(windows)]
fn create_pipe_instance(first: bool) -> Result<tokio::net::windows::named_pipe::NamedPipeServer> {
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::Security::{
        InitializeSecurityDescriptor, SetSecurityDescriptorDacl, SECURITY_ATTRIBUTES,
        SECURITY_DESCRIPTOR,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::*;

    // Build a NULL DACL security descriptor (grants access to everyone)
    let mut sd = std::mem::MaybeUninit::<SECURITY_DESCRIPTOR>::zeroed();
    unsafe {
        InitializeSecurityDescriptor(sd.as_mut_ptr() as *mut _, 1);
        SetSecurityDescriptorDacl(sd.as_mut_ptr() as *mut _, 1, std::ptr::null(), 0);
    }

    let mut sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd.as_mut_ptr() as *mut _,
        bInheritHandle: 0,
    };

    let pipe_name: Vec<u16> = paths::PIPE_NAME.encode_utf16().chain(Some(0)).collect();

    let mut open_mode = PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED;
    if first {
        open_mode |= FILE_FLAG_FIRST_PIPE_INSTANCE;
    }

    let handle = unsafe {
        CreateNamedPipeW(
            pipe_name.as_ptr(),
            open_mode,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            0,
            &mut sa,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error().into());
    }

    // Safety: handle is valid and overlapped-mode
    unsafe {
        #[allow(unused_imports)]
        use std::os::windows::io::FromRawHandle;
        Ok(tokio::net::windows::named_pipe::NamedPipeServer::from_raw_handle(handle as _)?)
    }
}

// --- Platform-agnostic connection handler ---

async fn handle_connection<R, W>(reader: R, mut writer: W) -> Result<()>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => dispatch(request).await,
            Err(e) => JsonRpcResponse::error(0, PARSE_ERROR, format!("Parse error: {}", e)),
        };

        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        line.clear();
    }

    Ok(())
}

async fn dispatch(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id;
    match request.method.as_str() {
        "ping" => JsonRpcResponse::success(id, serde_json::json!("pong")),

        "status" => {
            let caddy_running = caddy::process::is_caddy_running();
            let ca_installed = certs::ca::ca_exists();
            let ca_trusted = certs::trust::verify_ca_trust();
            // Check XAMPP/Apache status using detected or default path
            let xampp_detect = xampp::detect::detect_xampp();
            let xampp_running = xampp_detect
                .path
                .as_deref()
                .map(|p| xampp::process::is_apache_running(p))
                .unwrap_or(false);
            JsonRpcResponse::success(
                id,
                serde_json::json!(StatusResult {
                    daemon_running: true,
                    caddy_running,
                    ca_installed,
                    ca_trusted,
                    xampp_running,
                }),
            )
        }

        "sync_hosts" => match serde_json::from_value::<SyncHostsParams>(request.params) {
            Ok(params) => match hosts::sync_hosts(&params.entries) {
                Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "sync_caddy_config" => {
            match serde_json::from_value::<SyncCaddyConfigParams>(request.params) {
                Ok(params) => match caddy::config::generate_caddyfile(
                    &params.domains,
                    params.http_port,
                    params.https_port,
                ) {
                    Ok(()) => match caddy::process::reload_caddy() {
                        Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
                        Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
                    },
                    Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
                },
                Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
            }
        }

        "start_caddy" => match caddy::process::start_caddy() {
            Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "stop_caddy" => match caddy::process::stop_caddy() {
            Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "generate_ca" => match certs::ca::generate_ca() {
            Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "generate_cert" => match serde_json::from_value::<GenerateCertParams>(request.params) {
            Ok(params) => match certs::domain::generate_domain_cert(&params.domain) {
                Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "install_ca_trust" => match certs::trust::install_ca_trust() {
            Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "remove_ca_trust" => match certs::trust::remove_ca_trust() {
            Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "get_access_log" => match serde_json::from_value::<GetAccessLogParams>(request.params) {
            Ok(params) => {
                let limit = params.limit.unwrap_or(100);
                match logs::read_access_log(&params.domain, limit) {
                    Ok(entries) => {
                        JsonRpcResponse::success(id, serde_json::to_value(entries).unwrap())
                    }
                    Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
                }
            }
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "clear_access_log" => {
            match serde_json::from_value::<ClearAccessLogParams>(request.params) {
                Ok(params) => match logs::clear_access_log(&params.domain) {
                    Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
                    Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
                },
                Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
            }
        }

        "start_tunnel" => match serde_json::from_value::<StartTunnelParams>(request.params) {
            Ok(params) => match tunnel::manager::start_tunnel(params) {
                Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "stop_tunnel" => match serde_json::from_value::<StopTunnelParams>(request.params) {
            Ok(params) => match tunnel::manager::stop_tunnel(params) {
                Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "tunnel_status" => match serde_json::from_value::<TunnelStatusParams>(request.params) {
            Ok(params) => match tunnel::manager::tunnel_status(params) {
                Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "list_tunnels" => match tunnel::manager::list_tunnels() {
            Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "ensure_cloudflared" => match tunnel::download::ensure_cloudflared() {
            Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "stop_all_tunnels" => match tunnel::manager::stop_all_tunnels() {
            Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
            Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
        },

        "detect_xampp" => {
            let result = xampp::detect::detect_xampp();
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }

        "sync_xampp_config" => {
            match serde_json::from_value::<SyncXamppConfigParams>(request.params) {
                Ok(params) => {
                    // Write config
                    match xampp::config::sync_vhosts_config(&params.vhosts, &params.xampp_path) {
                        Ok(()) => {
                            // Test config before restarting
                            match xampp::process::test_apache_config(&params.xampp_path) {
                                Ok(()) => {
                                    // Restart Apache
                                    match xampp::process::restart_apache(&params.xampp_path) {
                                        Ok(()) => JsonRpcResponse::success(
                                            id,
                                            serde_json::json!(null),
                                        ),
                                        Err(e) => {
                                            // Rollback on restart failure
                                            xampp::config::rollback_vhosts(&params.xampp_path).ok();
                                            JsonRpcResponse::error(
                                                id,
                                                INTERNAL_ERROR,
                                                format!("Apache restart failed: {}", e),
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Rollback on config test failure
                                    xampp::config::rollback_vhosts(&params.xampp_path).ok();
                                    JsonRpcResponse::error(
                                        id,
                                        INTERNAL_ERROR,
                                        format!("Apache config test failed: {}", e),
                                    )
                                }
                            }
                        }
                        Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
                    }
                }
                Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
            }
        }

        "start_apache" => match serde_json::from_value::<XamppActionParams>(request.params) {
            Ok(params) => match xampp::process::start_apache(&params.xampp_path) {
                Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        "stop_apache" => match serde_json::from_value::<XamppActionParams>(request.params) {
            Ok(params) => match xampp::process::stop_apache(&params.xampp_path) {
                Ok(()) => JsonRpcResponse::success(id, serde_json::json!(null)),
                Err(e) => JsonRpcResponse::error(id, INTERNAL_ERROR, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(id, INVALID_PARAMS, e.to_string()),
        },

        _ => JsonRpcResponse::error(id, METHOD_NOT_FOUND, "Method not found".to_string()),
    }
}
