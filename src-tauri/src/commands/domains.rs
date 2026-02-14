use crate::db::models::{self, CreateDomainRequest, Domain, UpdateDomainRequest};
use crate::error::AppError;
use crate::state::AppState;
use crate::tray;
use crate::xampp;
use localdomain_shared::domain::{
    validate_document_root, validate_domain_name, validate_port, CaddyDomainConfig, HostsEntry,
    XamppVhostConfig,
};
use tauri::{AppHandle, Manager, State};

fn get_port_settings(state: &AppState) -> (u16, u16) {
    let conn = state.db.lock().unwrap();
    let http_port = models::get_setting(&conn, "http_port")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);
    let https_port = models::get_setting(&conn, "https_port")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(443);
    (http_port, https_port)
}

pub fn sync_state_to_daemon(state: &AppState) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    let domains = models::list_domains(&conn).map_err(AppError::Database)?;
    drop(conn);

    // Hosts: ALL domains (enabled + disabled) so entries persist when toggled off.
    // Entries are only removed when a domain is deleted.
    let hosts_entries: Vec<HostsEntry> = domains
        .iter()
        .map(|d| HostsEntry {
            domain: d.name.clone(),
            ip: "127.0.0.1".to_string(),
        })
        .collect();

    // Caddy: only ENABLED domains — toggling off stops the proxy/routing
    let enabled_domains: Vec<_> = domains.iter().filter(|d| d.enabled).collect();

    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Ok(());
    }

    // Sync hosts first (most important - enables domain resolution)
    client
        .sync_hosts(hosts_entries)
        .map_err(|e| AppError::Daemon(e.to_string()))?;

    // For HTTPS domains, generate CA + certs via daemon
    let needs_https = enabled_domains
        .iter()
        .any(|d| d.protocol == "https" || d.protocol == "both");

    if needs_https {
        // Ensure CA exists (daemon generates it)
        client
            .generate_ca()
            .map_err(|e| AppError::Daemon(format!("CA generation failed: {}", e)))?;
        // CA trust is NOT attempted here — on macOS the daemon can't do it
        // non-interactively. Use the explicit `trust_ca` command instead,
        // which falls back to osascript with an admin prompt.
    }

    // Partition enabled domains into proxy and XAMPP types (for Caddy routing)
    let proxy_domains: Vec<_> = enabled_domains
        .iter()
        .filter(|d| d.domain_type != "xampp")
        .collect();
    let xampp_enabled: Vec<_> = enabled_domains
        .iter()
        .filter(|d| d.domain_type == "xampp")
        .collect();

    // ALL XAMPP domains (enabled + disabled) for Apache VirtualHost config.
    // VirtualHosts persist when toggled off; only removed on delete.
    let all_xampp_domains: Vec<_> = domains
        .iter()
        .filter(|d| d.domain_type == "xampp")
        .collect();

    // Get XAMPP path and ports for routing XAMPP domains through Caddy
    let conn = state.db.lock().unwrap();
    let xampp_path = models::get_setting(&conn, "xampp_path")
        .ok()
        .flatten()
        .unwrap_or_default();
    drop(conn);

    let (xampp_http_port, _xampp_ssl_port) = if !xampp_path.is_empty() {
        xampp::get_xampp_ports(&xampp_path)
    } else {
        (80u16, 443u16)
    };

    // Build Caddy configs for proxy domains
    let mut caddy_configs: Vec<CaddyDomainConfig> = Vec::new();
    for d in &proxy_domains {
        // Skip domains without a target port (hosts-only domains)
        if d.target_port <= 0 {
            continue;
        }

        let wants_https = d.protocol == "https" || d.protocol == "both";
        let mut cert_path = None;
        let mut key_path = None;

        if wants_https {
            let result = client.generate_cert(&d.name).map_err(|e| {
                AppError::Daemon(format!("cert generation failed for {}: {}", d.name, e))
            })?;
            cert_path = Some(result.cert_path);
            key_path = Some(result.key_path);
        }

        caddy_configs.push(CaddyDomainConfig {
            name: d.name.clone(),
            target_host: d.target_host.clone(),
            target_port: d.target_port as u16,
            protocol: d.protocol.clone(),
            cert_path,
            key_path,
            access_log: d.access_log,
        });
    }

    // Route enabled XAMPP domains through Caddy — Caddy on standard ports (80/443)
    // reverse proxies to Apache's HTTP port. This ensures XAMPP domains work
    // regardless of Apache's configured port (e.g., 8080, 444 for SSL).
    for d in &xampp_enabled {
        let wants_https = d.protocol == "https" || d.protocol == "both";
        let mut cert_path = None;
        let mut key_path = None;

        if wants_https {
            let result = client.generate_cert(&d.name).map_err(|e| {
                AppError::Daemon(format!("cert generation failed for {}: {}", d.name, e))
            })?;
            cert_path = Some(result.cert_path);
            key_path = Some(result.key_path);
        }

        caddy_configs.push(CaddyDomainConfig {
            name: d.name.clone(),
            target_host: "127.0.0.1".to_string(),
            target_port: xampp_http_port,
            protocol: d.protocol.clone(),
            cert_path,
            key_path,
            access_log: d.access_log,
        });
    }

    // Sync Caddy config with port settings
    let (http_port, https_port) = get_port_settings(state);
    client
        .sync_caddy_config(caddy_configs, http_port, https_port)
        .map_err(|e| AppError::Daemon(format!("Caddy config sync failed: {}", e)))?;

    // Ensure Caddy is running (safety net — start_caddy is a no-op if already running)
    client
        .start_caddy()
        .map_err(|e| AppError::Daemon(format!("Caddy start failed: {}", e)))?;

    // Sync XAMPP VirtualHost config via daemon (HTTP only — Caddy handles HTTPS on port 443).
    // Uses ALL XAMPP domains so VirtualHosts persist when toggled off.
    // The daemon handles config write + config test + Apache restart as root,
    // so no interactive password prompt is needed.
    {
        if !xampp_path.is_empty() && !all_xampp_domains.is_empty() {
            let xampp_configs: Vec<XamppVhostConfig> = all_xampp_domains
                .iter()
                .map(|d| XamppVhostConfig {
                    name: d.name.clone(),
                    document_root: d.document_root.clone(),
                    protocol: "http".to_string(),
                    cert_path: None,
                    key_path: None,
                })
                .collect();

            if let Err(e) = client.sync_xampp_config(xampp_configs, &xampp_path) {
                eprintln!("XAMPP config sync via daemon failed: {}", e);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn list_domains(state: State<AppState>) -> Result<Vec<Domain>, AppError> {
    let conn = state.db.lock().unwrap();
    Ok(models::list_domains(&conn)?)
}

#[tauri::command]
pub async fn create_domain(app: AppHandle, request: CreateDomainRequest) -> Result<Domain, AppError> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();

        validate_domain_name(&request.name).map_err(AppError::Validation)?;
        if let Some(port) = request.target_port {
            validate_port(port as u16).map_err(AppError::Validation)?;
        }

        // XAMPP domain validation + auto-set port from XAMPP config
        let mut request = request;
        if request.domain_type.as_deref() == Some("xampp") {
            let doc_root = request.document_root.as_deref().unwrap_or("");
            validate_document_root(doc_root).map_err(AppError::Validation)?;

            // Auto-detect port from XAMPP config if not explicitly set
            if request.target_port.is_none() {
                let conn = state.db.lock().unwrap();
                let xampp_path = models::get_setting(&conn, "xampp_path")
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                drop(conn);

                if !xampp_path.is_empty() {
                    let (http_port, _) = xampp::get_xampp_ports(&xampp_path);
                    request.target_port = Some(http_port as i32);
                }
            }
        }

        let domain = {
            let conn = state.db.lock().unwrap();
            let domain = models::create_domain(&conn, &request)?;
            models::insert_audit_log(
                &conn,
                "domain_created",
                Some(&domain.id),
                Some(&serde_json::to_string(&request).unwrap_or_default()),
            )?;
            domain
        };

        sync_state_to_daemon(state.inner())?;
        tray::refresh_tray_menu(&app_handle);
        Ok(domain)
    })
    .await
    .map_err(|e| AppError::Other(format!("create_domain join error: {}", e)))?
}

#[tauri::command]
pub async fn update_domain(app: AppHandle, request: UpdateDomainRequest) -> Result<Domain, AppError> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();

        if let Some(ref name) = request.name {
            validate_domain_name(name).map_err(AppError::Validation)?;
        }
        if let Some(port) = request.target_port {
            validate_port(port as u16).map_err(AppError::Validation)?;
        }

        // XAMPP domain validation
        if request.domain_type.as_deref() == Some("xampp") {
            if let Some(ref doc_root) = request.document_root {
                validate_document_root(doc_root).map_err(AppError::Validation)?;
            }
        }

        let domain = {
            let conn = state.db.lock().unwrap();
            let domain = models::update_domain(&conn, &request)?
                .ok_or_else(|| AppError::Validation("Domain not found".to_string()))?;
            models::insert_audit_log(
                &conn,
                "domain_updated",
                Some(&domain.id),
                Some(&serde_json::to_string(&request).unwrap_or_default()),
            )?;
            domain
        };

        sync_state_to_daemon(state.inner())?;
        tray::refresh_tray_menu(&app_handle);
        Ok(domain)
    })
    .await
    .map_err(|e| AppError::Other(format!("update_domain join error: {}", e)))?
}

#[tauri::command]
pub async fn delete_domain(app: AppHandle, id: String) -> Result<(), AppError> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        {
            let conn = state.db.lock().unwrap();
            // Fetch name before deletion for audit log and tunnel cleanup
            let domain = models::get_domain(&conn, &id)?;
            let name = domain.as_ref().map(|d| d.name.clone());

            // Stop any running tunnel for this domain
            if let Some(ref domain_name) = name {
                let client = state.daemon_client.lock().unwrap();
                if client.is_daemon_running() {
                    client.stop_tunnel(domain_name).ok();
                }
            }

            let deleted = models::delete_domain(&conn, &id)?;
            if !deleted {
                return Err(AppError::Validation("Domain not found".to_string()));
            }
            models::insert_audit_log(&conn, "domain_deleted", Some(&id), name.as_deref())?;
        }

        sync_state_to_daemon(state.inner())?;
        tray::refresh_tray_menu(&app_handle);
        Ok(())
    })
    .await
    .map_err(|e| AppError::Other(format!("delete_domain join error: {}", e)))?
}

#[tauri::command]
pub async fn toggle_domain(app: AppHandle, id: String, enabled: bool) -> Result<Domain, AppError> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let domain = {
            let conn = state.db.lock().unwrap();
            let domain = models::toggle_domain(&conn, &id, enabled)?
                .ok_or_else(|| AppError::Validation("Domain not found".to_string()))?;
            models::insert_audit_log(
                &conn,
                if enabled {
                    "domain_enabled"
                } else {
                    "domain_disabled"
                },
                Some(&id),
                Some(&domain.name),
            )?;
            domain
        };

        sync_state_to_daemon(state.inner())?;
        tray::refresh_tray_menu(&app_handle);
        Ok(domain)
    })
    .await
    .map_err(|e| AppError::Other(format!("toggle_domain join error: {}", e)))?
}

#[tauri::command]
pub async fn toggle_access_log(
    app: AppHandle,
    id: String,
    enabled: bool,
) -> Result<Domain, AppError> {
    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        let domain = {
            let conn = state.db.lock().unwrap();
            let domain = models::set_access_log(&conn, &id, enabled)?
                .ok_or_else(|| AppError::Validation("Domain not found".to_string()))?;
            models::insert_audit_log(
                &conn,
                if enabled {
                    "access_log_enabled"
                } else {
                    "access_log_disabled"
                },
                Some(&id),
                Some(&domain.name),
            )?;
            domain
        };

        sync_state_to_daemon(state.inner())?;
        Ok(domain)
    })
    .await
    .map_err(|e| AppError::Other(format!("toggle_access_log join error: {}", e)))?
}

#[tauri::command]
pub fn trust_ca(state: State<AppState>) -> Result<(), AppError> {
    let client = state.daemon_client.lock().unwrap();
    if !client.is_daemon_running() {
        return Err(AppError::Daemon("Daemon is not running".to_string()));
    }

    // Ensure CA exists first
    client
        .generate_ca()
        .map_err(|e| AppError::Daemon(format!("CA generation failed: {}", e)))?;

    // Try trust via daemon first (works on Linux/Windows where root/SYSTEM can do it)
    match client.install_ca_trust() {
        Ok(()) => return Ok(()),
        Err(e) => {
            let msg = e.to_string();
            // On macOS the daemon can't add to System Keychain non-interactively;
            // fall back to osascript which can show the admin password prompt.
            if !msg.contains("non-interactive") {
                return Err(AppError::Daemon(format!("CA trust failed: {}", msg)));
            }
        }
    }

    // macOS fallback: use osascript to prompt the user for admin privileges
    #[cfg(target_os = "macos")]
    {
        let ca_cert = "/var/lib/localdomain/certs/localdomain-ca.crt";
        let script = format!(
            r#"do shell script "security add-trusted-cert -d -r trustRoot -p ssl -k /Library/Keychains/System.keychain '{}'" with administrator privileges"#,
            ca_cert
        );
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| AppError::Daemon(format!("Failed to run osascript: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Daemon(format!(
                "CA trust via admin prompt failed: {}",
                stderr.trim()
            )));
        }
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(AppError::Daemon(
        "CA trust failed: non-interactive".to_string(),
    ))
}
