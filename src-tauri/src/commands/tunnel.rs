use crate::db::models;
use crate::error::AppError;
use crate::state::AppState;
use localdomain_shared::protocol::{
    EnsureCloudflaredResult, ListTunnelsResult, StartTunnelParams, StartTunnelResult, TunnelType,
    TunnelStatusResult,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct StartTunnelRequest {
    pub domain_id: String,
    pub tunnel_type: TunnelType,
}

#[tauri::command]
pub fn start_tunnel(
    state: State<AppState>,
    request: StartTunnelRequest,
) -> Result<StartTunnelResult, AppError> {
    let (domain_name, target_port) = {
        let conn = state.db.lock().unwrap();
        let domain = models::get_domain(&conn, &request.domain_id)?
            .ok_or_else(|| AppError::Validation("Domain not found".to_string()))?;

        let mut port = domain.target_port;

        // For XAMPP domains, resolve the port from XAMPP config if not explicitly set.
        // This matches the logic in sync_state_to_daemon() which also resolves the
        // XAMPP Apache port at runtime.
        if port <= 0 && domain.domain_type == "xampp" {
            let xampp_path = models::get_setting(&conn, "xampp_path")
                .ok()
                .flatten()
                .unwrap_or_default();
            if !xampp_path.is_empty() {
                let (http_port, _) = crate::xampp::get_xampp_ports(&xampp_path);
                port = http_port as i32;
            }
        }

        (domain.name, port)
    };

    if target_port <= 0 {
        return Err(AppError::Validation(
            "Domain must have a target port to create a tunnel. For XAMPP domains, ensure XAMPP path is configured in Settings.".to_string(),
        ));
    }

    let params = StartTunnelParams {
        domain: domain_name.clone(),
        local_port: target_port as u16,
        tunnel_type: request.tunnel_type,
    };

    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Err(AppError::Daemon("Daemon is not running".to_string()));
    }

    let result = client
        .start_tunnel(params)
        .map_err(|e| AppError::Daemon(e.to_string()))?;

    // Audit log
    let conn = state.db.lock().unwrap();
    models::insert_audit_log(
        &conn,
        "tunnel_started",
        Some(&request.domain_id),
        Some(&format!("{} -> {}", domain_name, result.public_url)),
    )
    .ok();

    Ok(result)
}

#[tauri::command]
pub fn stop_tunnel(state: State<AppState>, domain_id: String) -> Result<(), AppError> {
    let domain_name = {
        let conn = state.db.lock().unwrap();
        let domain = models::get_domain(&conn, &domain_id)?
            .ok_or_else(|| AppError::Validation("Domain not found".to_string()))?;
        domain.name
    };

    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Err(AppError::Daemon("Daemon is not running".to_string()));
    }

    client
        .stop_tunnel(&domain_name)
        .map_err(|e| AppError::Daemon(e.to_string()))?;

    let conn = state.db.lock().unwrap();
    models::insert_audit_log(&conn, "tunnel_stopped", Some(&domain_id), Some(&domain_name)).ok();

    Ok(())
}

#[tauri::command]
pub fn get_tunnel_status(
    state: State<AppState>,
    domain_id: String,
) -> Result<TunnelStatusResult, AppError> {
    let domain_name = {
        let conn = state.db.lock().unwrap();
        let domain = models::get_domain(&conn, &domain_id)?
            .ok_or_else(|| AppError::Validation("Domain not found".to_string()))?;
        domain.name
    };

    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Ok(TunnelStatusResult {
            active: false,
            public_url: None,
            tunnel_type: None,
            error: Some("Daemon is not running".to_string()),
        });
    }

    client
        .tunnel_status(&domain_name)
        .map_err(|e| AppError::Daemon(e.to_string()))
}

#[tauri::command]
pub fn list_tunnels(state: State<AppState>) -> Result<ListTunnelsResult, AppError> {
    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Ok(ListTunnelsResult {
            tunnels: Vec::new(),
        });
    }

    client
        .list_tunnels()
        .map_err(|e| AppError::Daemon(e.to_string()))
}

#[tauri::command]
pub fn ensure_cloudflared(
    state: State<AppState>,
) -> Result<EnsureCloudflaredResult, AppError> {
    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Err(AppError::Daemon("Daemon is not running".to_string()));
    }

    client
        .ensure_cloudflared()
        .map_err(|e| AppError::Daemon(e.to_string()))
}

#[tauri::command]
pub fn save_tunnel_config(
    state: State<AppState>,
    domain_id: String,
    subdomain: String,
    domain: String,
) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    models::save_tunnel_config(&conn, &domain_id, &subdomain, &domain)?;
    Ok(())
}

// ---- Cloudflare automated auth flow ----

#[derive(Debug, Serialize)]
pub struct CloudflareLoginStatus {
    pub logged_in: bool,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareSetupRequest {
    pub subdomain: String,
    pub domain: String,
}

#[derive(Debug, Serialize)]
pub struct CloudflareSetupResult {
    pub tunnel_name: String,
    pub tunnel_id: String,
    pub token: String,
    pub credentials_json: String,
    pub public_url: String,
}

#[tauri::command]
pub fn cloudflare_check_login() -> Result<CloudflareLoginStatus, AppError> {
    let home = dirs::home_dir().ok_or_else(|| AppError::Other("Cannot find home directory".into()))?;
    let cert_path = home.join(".cloudflared").join("cert.pem");
    Ok(CloudflareLoginStatus {
        logged_in: cert_path.exists(),
    })
}

#[tauri::command]
pub async fn cloudflare_login(state: State<'_, AppState>) -> Result<CloudflareLoginStatus, AppError> {
    let cloudflared_path = {
        let client = state.daemon_client.lock().unwrap();
        if !client.is_daemon_running() {
            return Err(AppError::Daemon("Daemon is not running".into()));
        }
        let result = client.ensure_cloudflared().map_err(|e| AppError::Daemon(e.to_string()))?;
        if !result.installed {
            return Err(AppError::Other("cloudflared is not installed".into()));
        }
        result.path
    };

    let path = cloudflared_path.clone();
    let output = tokio::task::spawn_blocking(move || {
        Command::new(&path)
            .args(["tunnel", "login"])
            .output()
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {}", e)))?
    .map_err(|e| AppError::Other(format!("Failed to run cloudflared login: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!("cloudflared login failed: {}", stderr)));
    }

    let home = dirs::home_dir().ok_or_else(|| AppError::Other("Cannot find home directory".into()))?;
    let cert_path = home.join(".cloudflared").join("cert.pem");
    Ok(CloudflareLoginStatus {
        logged_in: cert_path.exists(),
    })
}

/// Try to extract the tunnel UUID from `cloudflared tunnel create` output.
/// Falls back to `cloudflared tunnel list -o json` if parsing fails.
fn extract_tunnel_id(stdout: &str, tunnel_name: &str, cloudflared_path: &str) -> Result<String, AppError> {
    // Output format: "Created tunnel <name> with id <uuid>"
    for word in stdout.split_whitespace().collect::<Vec<_>>().windows(2) {
        if word[0] == "id" {
            let id = word[1].trim();
            if id.len() >= 36 {
                return Ok(id.to_string());
            }
        }
    }

    // Fallback: list tunnels as JSON
    let output = Command::new(cloudflared_path)
        .args(["tunnel", "list", "-o", "json"])
        .output()
        .map_err(|e| AppError::Other(format!("Failed to list tunnels: {}", e)))?;

    // Try both stdout and stderr (cloudflared varies by version)
    let list_stdout = String::from_utf8_lossy(&output.stdout);
    let list_stderr = String::from_utf8_lossy(&output.stderr);
    for json_src in [list_stdout.as_ref(), list_stderr.as_ref()] {
        if let Ok(tunnels) = serde_json::from_str::<Vec<serde_json::Value>>(json_src) {
            for t in &tunnels {
                if let (Some(name), Some(id)) = (t["name"].as_str(), t["id"].as_str()) {
                    if name == tunnel_name {
                        return Ok(id.to_string());
                    }
                }
            }
        }
    }

    Err(AppError::Other(format!(
        "Could not find tunnel ID for '{}'. stdout: {} stderr: {}",
        tunnel_name, list_stdout, list_stderr
    )))
}

#[tauri::command]
pub async fn cloudflare_setup_tunnel(
    state: State<'_, AppState>,
    request: CloudflareSetupRequest,
) -> Result<CloudflareSetupResult, AppError> {
    let cloudflared_path = {
        let client = state.daemon_client.lock().unwrap();
        if !client.is_daemon_running() {
            return Err(AppError::Daemon("Daemon is not running".into()));
        }
        let result = client.ensure_cloudflared().map_err(|e| AppError::Daemon(e.to_string()))?;
        if !result.installed {
            return Err(AppError::Other("cloudflared is not installed".into()));
        }
        result.path
    };

    let tunnel_name = format!("localdomain-{}-{}", request.subdomain, request.domain);
    let hostname = format!("{}.{}", request.subdomain, request.domain);
    let cf_path = cloudflared_path.clone();
    let tn = tunnel_name.clone();

    // Step 1: Create tunnel (or reuse existing)
    let (tunnel_id, cf_path) = tokio::task::spawn_blocking(move || {
        // First try: clean up any stale connector state
        let _ = Command::new(&cf_path)
            .args(["tunnel", "cleanup", &tn])
            .output();

        let output = Command::new(&cf_path)
            .args(["tunnel", "create", &tn])
            .output()
            .map_err(|e| AppError::Other(format!("Failed to create tunnel: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{} {}", stdout, stderr);

        if output.status.success() {
            let id = extract_tunnel_id(&combined, &tn, &cf_path)?;
            Ok::<(String, String), AppError>((id, cf_path))
        } else if combined.contains("already exists") {
            // Tunnel already exists, find it
            let id = extract_tunnel_id("", &tn, &cf_path)?;
            Ok((id, cf_path))
        } else {
            Err(AppError::Other(format!("Failed to create tunnel: {}", stderr)))
        }
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {}", e)))??;

    // Step 2: Route DNS
    let cf_path2 = cf_path.clone();
    let tid = tunnel_id.clone();
    let hn = hostname.clone();
    tokio::task::spawn_blocking(move || {
        let output = Command::new(&cf_path2)
            .args(["tunnel", "route", "dns", &tid, &hn])
            .output()
            .map_err(|e| AppError::Other(format!("Failed to route DNS: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore "already exists" errors
            if !stderr.contains("already exists") {
                return Err(AppError::Other(format!("Failed to route DNS: {}", stderr)));
            }
        }
        Ok::<(), AppError>(())
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {}", e)))??;

    // Step 3: Get tunnel token
    let cf_path3 = cf_path.clone();
    let tn2 = tunnel_name.clone();
    let token = tokio::task::spawn_blocking(move || {
        let output = Command::new(&cf_path3)
            .args(["tunnel", "token", &tn2])
            .output()
            .map_err(|e| AppError::Other(format!("Failed to get tunnel token: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Other(format!("Failed to get tunnel token: {}", stderr)));
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if token.is_empty() {
            // Some versions output to stderr
            let token_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if !token_stderr.is_empty() && !token_stderr.contains("ERR") {
                return Ok::<String, AppError>(token_stderr);
            }
            return Err(AppError::Other("Empty token returned".into()));
        }
        Ok(token)
    })
    .await
    .map_err(|e| AppError::Other(format!("Task join error: {}", e)))??;

    // Step 4: Read credentials file (~/.cloudflared/<UUID>.json)
    let credentials_json = {
        let home = dirs::home_dir().ok_or_else(|| AppError::Other("Cannot find home directory".into()))?;
        let creds_path = home.join(".cloudflared").join(format!("{}.json", tunnel_id));
        std::fs::read_to_string(&creds_path).unwrap_or_default()
    };

    // Audit log
    {
        let conn = state.db.lock().unwrap();
        models::insert_audit_log(
            &conn,
            "cloudflare_tunnel_created",
            None,
            Some(&format!("{} -> https://{}", tunnel_name, hostname)),
        )
        .ok();
    }

    Ok(CloudflareSetupResult {
        tunnel_name,
        tunnel_id: tunnel_id.clone(),
        token,
        credentials_json,
        public_url: format!("https://{}", hostname),
    })
}
