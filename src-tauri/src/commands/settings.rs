use crate::db::models::{self, CreateDomainRequest};
use crate::error::AppError;
use crate::state::AppState;
use crate::xampp::{self, ImportVhost, ScannedVhost};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub start_on_boot: bool,
    pub http_port: u16,
    pub https_port: u16,
    #[serde(default)]
    pub cloudflare_tunnel_token: Option<String>,
    #[serde(default)]
    pub default_ssh_host: Option<String>,
    #[serde(default)]
    pub default_ssh_user: Option<String>,
    #[serde(default)]
    pub default_ssh_key_path: Option<String>,
    #[serde(default)]
    pub xampp_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_on_boot: false,
            http_port: 80,
            https_port: 443,
            cloudflare_tunnel_token: None,
            default_ssh_host: None,
            default_ssh_user: None,
            default_ssh_key_path: None,
            xampp_path: None,
        }
    }
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, AppError> {
    let conn = state.db.lock().unwrap();
    let mut settings = AppSettings::default();

    if let Some(v) = models::get_setting(&conn, "start_on_boot")? {
        settings.start_on_boot = v == "true";
    }
    if let Some(v) = models::get_setting(&conn, "http_port")? {
        settings.http_port = v.parse().unwrap_or(80);
    }
    if let Some(v) = models::get_setting(&conn, "https_port")? {
        settings.https_port = v.parse().unwrap_or(443);
    }
    if let Some(v) = models::get_setting(&conn, "cloudflare_tunnel_token")? {
        settings.cloudflare_tunnel_token = Some(v);
    }
    if let Some(v) = models::get_setting(&conn, "default_ssh_host")? {
        settings.default_ssh_host = Some(v);
    }
    if let Some(v) = models::get_setting(&conn, "default_ssh_user")? {
        settings.default_ssh_user = Some(v);
    }
    if let Some(v) = models::get_setting(&conn, "default_ssh_key_path")? {
        settings.default_ssh_key_path = Some(v);
    }
    if let Some(v) = models::get_setting(&conn, "xampp_path")? {
        settings.xampp_path = Some(v);
    }

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(state: State<AppState>, settings: AppSettings) -> Result<(), AppError> {
    let conn = state.db.lock().unwrap();
    models::set_setting(&conn, "start_on_boot", &settings.start_on_boot.to_string())?;
    models::set_setting(&conn, "http_port", &settings.http_port.to_string())?;
    models::set_setting(&conn, "https_port", &settings.https_port.to_string())?;
    if let Some(ref token) = settings.cloudflare_tunnel_token {
        models::set_setting(&conn, "cloudflare_tunnel_token", token)?;
    }
    if let Some(ref host) = settings.default_ssh_host {
        models::set_setting(&conn, "default_ssh_host", host)?;
    }
    if let Some(ref user) = settings.default_ssh_user {
        models::set_setting(&conn, "default_ssh_user", user)?;
    }
    if let Some(ref key_path) = settings.default_ssh_key_path {
        models::set_setting(&conn, "default_ssh_key_path", key_path)?;
    }
    if let Some(ref path) = settings.xampp_path {
        models::set_setting(&conn, "xampp_path", path)?;
    }
    Ok(())
}

/// Detect XAMPP path directly (no daemon needed).
#[tauri::command]
pub fn detect_xampp_path() -> Result<Option<String>, AppError> {
    #[cfg(target_os = "macos")]
    let candidates = ["/Applications/XAMPP/xamppfiles"];
    #[cfg(target_os = "linux")]
    let candidates = ["/opt/lampp"];
    #[cfg(target_os = "windows")]
    let candidates = ["C:\\xampp", "D:\\xampp"];

    for path in &candidates {
        let httpd = get_httpd_path(path);
        if std::path::Path::new(&httpd).exists() {
            return Ok(Some(path.to_string()));
        }
    }
    Ok(None)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_httpd_path(xampp_path: &str) -> String {
    format!("{}/bin/httpd", xampp_path)
}

#[cfg(target_os = "windows")]
fn get_httpd_path(xampp_path: &str) -> String {
    format!("{}\\apache\\bin\\httpd.exe", xampp_path)
}

/// Detected XAMPP ports from httpd.conf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XamppPorts {
    pub http_port: u16,
    pub ssl_port: u16,
}

/// Get the configured Listen ports from XAMPP's httpd.conf and httpd-ssl.conf.
#[tauri::command]
pub fn get_xampp_default_port(xampp_path: String) -> Result<XamppPorts, AppError> {
    if xampp_path.is_empty() {
        return Err(AppError::Validation(
            "XAMPP path is not configured".to_string(),
        ));
    }
    let (http_port, ssl_port) = xampp::get_xampp_ports(&xampp_path);
    Ok(XamppPorts {
        http_port,
        ssl_port,
    })
}

/// Scan XAMPP httpd-vhosts.conf for user-defined VirtualHost blocks that can be imported.
#[tauri::command]
pub fn scan_xampp_vhosts(
    state: State<AppState>,
    xampp_path: String,
) -> Result<Vec<ScannedVhost>, AppError> {
    if xampp_path.is_empty() {
        return Err(AppError::Validation(
            "XAMPP path is not configured".to_string(),
        ));
    }

    // Get existing domain names to mark already_exists
    let conn = state.db.lock().unwrap();
    let domains = models::list_domains(&conn)?;
    drop(conn);

    let existing_names: Vec<String> = domains.iter().map(|d| d.name.clone()).collect();
    xampp::scan_vhosts(&xampp_path, &existing_names)
}

/// Import selected VirtualHost entries as XAMPP domains.
#[tauri::command]
pub fn import_xampp_vhosts(
    app: tauri::AppHandle,
    state: State<AppState>,
    vhosts: Vec<ImportVhost>,
) -> Result<Vec<models::Domain>, AppError> {
    let mut created = Vec::new();

    // Detect XAMPP port for imported domains
    let xampp_http_port = {
        let conn = state.db.lock().unwrap();
        let xampp_path = models::get_setting(&conn, "xampp_path")
            .ok()
            .flatten()
            .unwrap_or_default();
        if !xampp_path.is_empty() {
            let (http_port, _) = xampp::get_xampp_ports(&xampp_path);
            Some(http_port as i32)
        } else {
            None
        }
    };

    for vhost in &vhosts {
        let conn = state.db.lock().unwrap();
        // Skip if domain already exists
        let existing = models::list_domains(&conn)?;
        if existing.iter().any(|d| d.name == vhost.server_name) {
            continue;
        }

        let req = CreateDomainRequest {
            name: vhost.server_name.clone(),
            target_host: None,
            target_port: xampp_http_port,
            protocol: Some("http".to_string()),
            wildcard: None,
            domain_type: Some("xampp".to_string()),
            document_root: Some(vhost.document_root.clone()),
        };

        let domain = models::create_domain(&conn, &req)?;
        models::insert_audit_log(
            &conn,
            "domain_imported",
            Some(&domain.id),
            Some(&format!("Imported from XAMPP: {}", vhost.server_name)),
        )?;
        created.push(domain);
    }

    // Sync state after importing
    drop(state.db.lock()); // ensure lock released
    crate::commands::domains::sync_state_to_daemon(&state)?;
    crate::tray::refresh_tray_menu(&app);

    Ok(created)
}
