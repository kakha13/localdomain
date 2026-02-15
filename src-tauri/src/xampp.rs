use crate::error::AppError;
use localdomain_shared::domain::XamppVhostConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

const SENTINEL_START: &str = "# BEGIN LOCALDOMAIN MANAGED VHOSTS";
const SENTINEL_END: &str = "# END LOCALDOMAIN MANAGED VHOSTS";

/// A VirtualHost discovered by scanning httpd-vhosts.conf outside the managed block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedVhost {
    pub server_name: String,
    pub document_root: String,
    pub port: u16,
    pub already_exists: bool,
}

/// Import request for a single vhost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportVhost {
    pub server_name: String,
    pub document_root: String,
}

// ---- Path helpers ----

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn vhosts_conf_path(xampp_path: &str) -> String {
    format!("{}/etc/extra/httpd-vhosts.conf", xampp_path)
}

#[cfg(target_os = "windows")]
pub fn vhosts_conf_path(xampp_path: &str) -> String {
    format!("{}\\apache\\conf\\extra\\httpd-vhosts.conf", xampp_path)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn httpd_conf_path(xampp_path: &str) -> String {
    format!("{}/etc/httpd.conf", xampp_path)
}

#[cfg(target_os = "windows")]
fn httpd_conf_path(xampp_path: &str) -> String {
    format!("{}\\apache\\conf\\httpd.conf", xampp_path)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn httpd_ssl_conf_path(xampp_path: &str) -> String {
    format!("{}/etc/extra/httpd-ssl.conf", xampp_path)
}

#[cfg(target_os = "windows")]
fn httpd_ssl_conf_path(xampp_path: &str) -> String {
    format!("{}\\apache\\conf\\extra\\httpd-ssl.conf", xampp_path)
}

// ---- Port detection ----

/// Parse XAMPP httpd.conf and httpd-ssl.conf to detect configured Listen ports.
/// Returns (http_port, ssl_port).
pub fn get_xampp_ports(xampp_path: &str) -> (u16, u16) {
    let http_port = parse_listen_port(&httpd_conf_path(xampp_path)).unwrap_or(80);
    let ssl_port = parse_listen_port(&httpd_ssl_conf_path(xampp_path)).unwrap_or(443);
    (http_port, ssl_port)
}

/// Parse the first uncommented `Listen` directive from an Apache config file.
/// Handles formats: `Listen 80`, `Listen 0.0.0.0:80`, `Listen [::]:80`.
fn parse_listen_port(conf_path: &str) -> Option<u16> {
    let content = fs::read_to_string(conf_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Listen") {
            let value = value.trim();
            if value.is_empty() {
                continue;
            }
            // Handle address:port format (e.g., 0.0.0.0:80, [::]:443)
            if let Some(colon_pos) = value.rfind(':') {
                let port_str: String = value[colon_pos + 1..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(port) = port_str.parse() {
                    return Some(port);
                }
            }
            // Just a port number
            let port_str: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(port) = port_str.parse() {
                return Some(port);
            }
        }
    }
    None
}

// ---- Config writing ----

/// Sync XAMPP VirtualHost configuration from the app side (no daemon needed).
/// Writes managed vhosts to httpd-vhosts.conf using sentinel markers,
/// preserving any user-defined entries outside the managed block.
/// Uses the detected XAMPP ports for VirtualHost directives.
pub fn sync_vhosts_config(vhosts: &[XamppVhostConfig], xampp_path: &str) -> Result<(), AppError> {
    let conf_path = vhosts_conf_path(xampp_path);
    let (http_port, ssl_port) = get_xampp_ports(xampp_path);

    let current = if std::path::Path::new(&conf_path).exists() {
        fs::read_to_string(&conf_path)
            .map_err(|e| AppError::Other(format!("Failed to read httpd-vhosts.conf: {}", e)))?
    } else {
        String::new()
    };

    // Create backup
    let backup_path = format!("{}.localdomain.bak", conf_path);
    fs::write(&backup_path, &current)
        .map_err(|e| AppError::Other(format!("Failed to create vhosts backup: {}", e)))?;

    let new_content = build_vhosts_content(&current, vhosts, xampp_path, http_port, ssl_port);

    // Atomic write via temp file
    let tmp_path = format!("{}.localdomain.tmp", conf_path);
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(new_content.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, &conf_path)
        .map_err(|e| AppError::Other(format!("Failed to replace httpd-vhosts.conf: {}", e)))?;

    // Ensure the vhosts include is enabled in httpd.conf
    ensure_vhosts_include(xampp_path)?;

    // Enable SSL module if any HTTPS domains exist
    let needs_ssl = vhosts
        .iter()
        .any(|v| v.protocol == "https" || v.protocol == "both");
    if needs_ssl {
        ensure_ssl_module(xampp_path)?;
    }

    Ok(())
}

/// Ensure the `Include conf/extra/httpd-vhosts.conf` line is uncommented in httpd.conf.
pub fn ensure_vhosts_include(xampp_path: &str) -> Result<(), AppError> {
    let conf_path = httpd_conf_path(xampp_path);
    if !std::path::Path::new(&conf_path).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&conf_path)
        .map_err(|e| AppError::Other(format!("Failed to read httpd.conf: {}", e)))?;

    let has_active_include = content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#')
            && trimmed.contains("Include")
            && trimmed.contains("httpd-vhosts.conf")
    });

    if has_active_include {
        return Ok(());
    }

    let new_content = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#')
                && trimmed.contains("Include")
                && trimmed.contains("httpd-vhosts.conf")
            {
                trimmed.trim_start_matches('#').trim_start().to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if new_content != content {
        fs::write(&conf_path, &new_content)
            .map_err(|e| AppError::Other(format!("Failed to update httpd.conf: {}", e)))?;
    }

    Ok(())
}

/// Ensure the SSL module is loaded in httpd.conf by uncommenting the LoadModule line.
pub fn ensure_ssl_module(xampp_path: &str) -> Result<(), AppError> {
    let conf_path = httpd_conf_path(xampp_path);
    if !std::path::Path::new(&conf_path).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&conf_path)
        .map_err(|e| AppError::Other(format!("Failed to read httpd.conf: {}", e)))?;

    let has_active_ssl = content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#')
            && trimmed.contains("LoadModule")
            && trimmed.contains("ssl_module")
    });

    if has_active_ssl {
        return Ok(());
    }

    let new_content = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#')
                && trimmed.contains("LoadModule")
                && trimmed.contains("ssl_module")
            {
                trimmed.trim_start_matches('#').trim_start().to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if new_content != content {
        fs::write(&conf_path, &new_content)
            .map_err(|e| AppError::Other(format!("Failed to update httpd.conf: {}", e)))?;
    }

    Ok(())
}

// ---- Content building ----

fn build_vhosts_content(
    current: &str,
    vhosts: &[XamppVhostConfig],
    xampp_path: &str,
    http_port: u16,
    ssl_port: u16,
) -> String {
    let mut lines: Vec<&str> = Vec::new();
    let mut in_block = false;

    for line in current.lines() {
        if line.trim() == SENTINEL_START {
            in_block = true;
            continue;
        }
        if line.trim() == SENTINEL_END {
            in_block = false;
            continue;
        }
        if !in_block {
            lines.push(line);
        }
    }

    // Remove trailing empty lines
    while lines.last().map_or(false, |l| l.is_empty()) {
        lines.pop();
    }

    let mut result = lines.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }

    if !vhosts.is_empty() {
        result.push('\n');
        result.push_str(SENTINEL_START);
        result.push('\n');

        // Localhost preservation: ensure localhost still works on the configured port
        result.push_str(&build_localhost_vhost(xampp_path, http_port));

        for vhost in vhosts {
            if vhost.protocol == "http" || vhost.protocol == "both" {
                result.push_str(&build_http_vhost(vhost, http_port));
            }
            if vhost.protocol == "https" || vhost.protocol == "both" {
                result.push_str(&build_https_vhost(vhost, ssl_port));
            }
        }

        result.push_str(SENTINEL_END);
        result.push('\n');
    }

    result
}

fn build_localhost_vhost(xampp_path: &str, http_port: u16) -> String {
    format!(
        r#"<VirtualHost *:{port}>
    ServerName localhost
    DocumentRoot "{xampp_path}/htdocs"
</VirtualHost>

"#,
        port = http_port,
        xampp_path = xampp_path
    )
}

fn build_http_vhost(vhost: &XamppVhostConfig, http_port: u16) -> String {
    format!(
        r#"<VirtualHost *:{port}>
    ServerName {name}
    DocumentRoot "{document_root}"
    <Directory "{document_root}">
        Options Indexes FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>
</VirtualHost>

"#,
        port = http_port,
        name = vhost.name,
        document_root = vhost.document_root,
    )
}

fn build_https_vhost(vhost: &XamppVhostConfig, ssl_port: u16) -> String {
    let cert_path = vhost.cert_path.as_deref().unwrap_or("");
    let key_path = vhost.key_path.as_deref().unwrap_or("");

    format!(
        r#"<VirtualHost *:{port}>
    ServerName {name}
    DocumentRoot "{document_root}"
    SSLEngine on
    SSLCertificateFile "{cert_path}"
    SSLCertificateKeyFile "{key_path}"
    <Directory "{document_root}">
        Options Indexes FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>
</VirtualHost>

"#,
        port = ssl_port,
        name = vhost.name,
        document_root = vhost.document_root,
        cert_path = cert_path,
        key_path = key_path,
    )
}

// ---- Apache restart ----

/// Restart XAMPP Apache with elevated privileges.
/// Uses stop + kill + start to handle stuck processes reliably.
#[cfg(target_os = "macos")]
pub fn restart_apache(xampp_path: &str) -> Result<(), AppError> {
    let escaped = xampp_path.replace('"', "\\\"");
    // Stop Apache, kill any remaining XAMPP httpd processes, then start fresh.
    // apachectl stop may fail if already stopped — that's fine.
    // killall ensures no zombie processes hold the port.
    let script = format!(
        r#"do shell script "{path}/bin/apachectl stop; sleep 1; killall -9 {path}/bin/httpd 2>/dev/null; sleep 1; {path}/bin/apachectl start" with administrator privileges"#,
        path = escaped
    );
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| AppError::Other(format!("Failed to run osascript: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!(
            "Apache restart failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn restart_apache(xampp_path: &str) -> Result<(), AppError> {
    let cmd = format!(
        "{path}/bin/apachectl stop; sleep 1; killall -9 {path}/bin/httpd 2>/dev/null; sleep 1; {path}/bin/apachectl start",
        path = xampp_path
    );
    let output = std::process::Command::new("pkexec")
        .args(["bash", "-c", &cmd])
        .output()
        .map_err(|e| AppError::Other(format!("Failed to restart Apache: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!(
            "Apache restart failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn restart_apache(xampp_path: &str) -> Result<(), AppError> {
    let ps_script = format!(
        r#"& '{}\\apache\\bin\\httpd.exe' -k restart"#,
        xampp_path.replace('\'', "''")
    );
    let output = localdomain_shared::silent_cmd("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_script,
        ])
        .output()
        .map_err(|e| AppError::Other(format!("Failed to restart Apache: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!(
            "Apache restart failed: {}",
            stderr.trim()
        )));
    }
    Ok(())
}

// ---- VHost scanning ----

/// Scan httpd-vhosts.conf for user-defined VirtualHost blocks outside the managed block.
/// Returns a list of discovered vhosts that can be imported.
pub fn scan_vhosts(
    xampp_path: &str,
    existing_names: &[String],
) -> Result<Vec<ScannedVhost>, AppError> {
    let conf_path = vhosts_conf_path(xampp_path);
    if !std::path::Path::new(&conf_path).exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&conf_path)
        .map_err(|e| AppError::Other(format!("Failed to read httpd-vhosts.conf: {}", e)))?;

    Ok(parse_vhosts_outside_managed(&content, existing_names))
}

/// Parse VirtualHost blocks outside the managed sentinel block.
fn parse_vhosts_outside_managed(content: &str, existing_names: &[String]) -> Vec<ScannedVhost> {
    let mut results = Vec::new();
    let mut in_managed = false;
    let mut in_vhost = false;
    let mut current_port: u16 = 80;
    let mut current_server_name = String::new();
    let mut current_doc_root = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == SENTINEL_START {
            in_managed = true;
            continue;
        }
        if trimmed == SENTINEL_END {
            in_managed = false;
            continue;
        }

        // Skip everything inside the managed block
        if in_managed {
            continue;
        }

        if trimmed.starts_with("<VirtualHost") {
            in_vhost = true;
            current_port = extract_port(trimmed);
            current_server_name.clear();
            current_doc_root.clear();
            continue;
        }

        if trimmed == "</VirtualHost>" {
            if in_vhost && !current_server_name.is_empty() && current_server_name != "localhost" {
                let already_exists = existing_names
                    .iter()
                    .any(|n| n.eq_ignore_ascii_case(&current_server_name));
                results.push(ScannedVhost {
                    server_name: current_server_name.clone(),
                    document_root: current_doc_root.clone(),
                    port: current_port,
                    already_exists,
                });
            }
            in_vhost = false;
            continue;
        }

        if in_vhost {
            if let Some(name) = trimmed.strip_prefix("ServerName") {
                current_server_name = name.trim().to_string();
            } else if let Some(root) = trimmed.strip_prefix("DocumentRoot") {
                current_doc_root = root.trim().trim_matches('"').to_string();
            }
        }
    }

    // Deduplicate by server_name (keep first occurrence)
    let mut seen = std::collections::HashSet::new();
    results.retain(|v| seen.insert(v.server_name.clone()));

    results
}

/// Extract port from a `<VirtualHost *:PORT>` line.
fn extract_port(line: &str) -> u16 {
    // Match patterns like <VirtualHost *:80>, <VirtualHost _default_:443>
    if let Some(colon_pos) = line.rfind(':') {
        let after_colon = &line[colon_pos + 1..];
        let port_str: String = after_colon
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        port_str.parse().unwrap_or(80)
    } else {
        80
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_vhosts_content_uses_xampp_path_for_localhost() {
        let vhosts = vec![XamppVhostConfig {
            name: "mysite.test".to_string(),
            document_root: "/var/www/mysite".to_string(),
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
        }];
        let result = build_vhosts_content("", &vhosts, "/Applications/XAMPP/xamppfiles", 80, 443);
        assert!(result.contains(r#"DocumentRoot "/Applications/XAMPP/xamppfiles/htdocs""#));
        assert!(!result.contains("/opt/lampp/htdocs"));
    }

    #[test]
    fn test_build_vhosts_content_uses_custom_ports() {
        let vhosts = vec![XamppVhostConfig {
            name: "mysite.test".to_string(),
            document_root: "/var/www/mysite".to_string(),
            protocol: "both".to_string(),
            cert_path: Some("/certs/mysite.crt".to_string()),
            key_path: Some("/certs/mysite.key".to_string()),
        }];
        let result = build_vhosts_content("", &vhosts, "/opt/lampp", 8080, 4443);
        assert!(result.contains("<VirtualHost *:8080>"));
        assert!(result.contains("<VirtualHost *:4443>"));
        assert!(!result.contains("<VirtualHost *:80>"));
        assert!(!result.contains("<VirtualHost *:443>"));
    }

    #[test]
    fn test_parse_listen_port_simple() {
        let dir = std::env::temp_dir().join("localdomain_test_listen");
        let _ = std::fs::create_dir_all(&dir);
        let conf = dir.join("httpd.conf");
        std::fs::write(&conf, "# Comment\nListen 8080\n").unwrap();
        assert_eq!(parse_listen_port(conf.to_str().unwrap()), Some(8080));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_listen_port_with_address() {
        let dir = std::env::temp_dir().join("localdomain_test_listen_addr");
        let _ = std::fs::create_dir_all(&dir);
        let conf = dir.join("httpd.conf");
        std::fs::write(&conf, "Listen 0.0.0.0:9090\n").unwrap();
        assert_eq!(parse_listen_port(conf.to_str().unwrap()), Some(9090));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_listen_port_skips_comments() {
        let dir = std::env::temp_dir().join("localdomain_test_listen_comment");
        let _ = std::fs::create_dir_all(&dir);
        let conf = dir.join("httpd.conf");
        std::fs::write(&conf, "#Listen 80\nListen 3000\n").unwrap();
        assert_eq!(parse_listen_port(conf.to_str().unwrap()), Some(3000));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_scan_vhosts_outside_managed() {
        let content = format!(
            r#"# User vhost
<VirtualHost *:80>
    ServerName myproject.local
    DocumentRoot "/Users/me/projects/myproject"
</VirtualHost>

{}
<VirtualHost *:80>
    ServerName localhost
    DocumentRoot "/Applications/XAMPP/xamppfiles/htdocs"
</VirtualHost>
<VirtualHost *:80>
    ServerName managed.test
    DocumentRoot "/var/www/managed"
</VirtualHost>
{}

<VirtualHost *:443>
    ServerName secure.local
    DocumentRoot "/Users/me/secure"
</VirtualHost>
"#,
            SENTINEL_START, SENTINEL_END
        );

        let existing = vec!["secure.local".to_string()];
        let results = parse_vhosts_outside_managed(&content, &existing);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].server_name, "myproject.local");
        assert_eq!(results[0].document_root, "/Users/me/projects/myproject");
        assert_eq!(results[0].port, 80);
        assert!(!results[0].already_exists);
        assert_eq!(results[1].server_name, "secure.local");
        assert_eq!(results[1].port, 443);
        assert!(results[1].already_exists);
    }

    #[test]
    fn test_scan_skips_localhost() {
        let content = r#"<VirtualHost *:80>
    ServerName localhost
    DocumentRoot "/opt/htdocs"
</VirtualHost>
"#;
        let results = parse_vhosts_outside_managed(content, &[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_extract_port() {
        assert_eq!(extract_port("<VirtualHost *:80>"), 80);
        assert_eq!(extract_port("<VirtualHost *:443>"), 443);
        assert_eq!(extract_port("<VirtualHost _default_:8080>"), 8080);
        assert_eq!(extract_port("<VirtualHost *>"), 80); // no port -> default 80
    }
}
