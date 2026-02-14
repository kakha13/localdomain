use anyhow::{Context, Result};
use localdomain_shared::domain::XamppVhostConfig;
use std::fs;
use std::io::Write;
use tracing::info;

const SENTINEL_START: &str = "# BEGIN LOCALDOMAIN MANAGED VHOSTS";
const SENTINEL_END: &str = "# END LOCALDOMAIN MANAGED VHOSTS";

/// Get the path to httpd-vhosts.conf for a given XAMPP installation.
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn vhosts_conf_path(xampp_path: &str) -> String {
    format!("{}/etc/extra/httpd-vhosts.conf", xampp_path)
}

#[cfg(target_os = "windows")]
pub fn vhosts_conf_path(xampp_path: &str) -> String {
    format!("{}\\apache\\conf\\extra\\httpd-vhosts.conf", xampp_path)
}

/// Get the path to httpd.conf for a given XAMPP installation.
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

/// Parse XAMPP httpd.conf and httpd-ssl.conf to detect configured Listen ports.
/// Returns (http_port, ssl_port).
pub fn get_xampp_ports(xampp_path: &str) -> (u16, u16) {
    let http_port = parse_listen_port(&httpd_conf_path(xampp_path)).unwrap_or(80);
    let ssl_port = parse_listen_port(&httpd_ssl_conf_path(xampp_path)).unwrap_or(443);
    (http_port, ssl_port)
}

/// Parse the first uncommented `Listen` directive from an Apache config file.
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
            if let Some(colon_pos) = value.rfind(':') {
                let port_str: String = value[colon_pos + 1..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                if let Ok(port) = port_str.parse() {
                    return Some(port);
                }
            }
            let port_str: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(port) = port_str.parse() {
                return Some(port);
            }
        }
    }
    None
}

/// Sync XAMPP VirtualHost configuration.
/// Writes managed vhosts to httpd-vhosts.conf using sentinel markers,
/// preserving any user-defined entries outside the managed block.
/// Uses the detected XAMPP ports for VirtualHost directives.
pub fn sync_vhosts_config(vhosts: &[XamppVhostConfig], xampp_path: &str) -> Result<()> {
    let conf_path = vhosts_conf_path(xampp_path);
    let (http_port, ssl_port) = get_xampp_ports(xampp_path);

    let current = if std::path::Path::new(&conf_path).exists() {
        fs::read_to_string(&conf_path).context("Failed to read httpd-vhosts.conf")?
    } else {
        String::new()
    };

    // Create backup
    let backup_path = format!("{}.localdomain.bak", conf_path);
    fs::write(&backup_path, &current).context("Failed to create vhosts backup")?;

    let new_content = build_vhosts_content(&current, vhosts, xampp_path, http_port, ssl_port);

    // Atomic write via temp file
    let tmp_path = format!("{}.localdomain.tmp", conf_path);
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(new_content.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, &conf_path).context("Failed to replace httpd-vhosts.conf")?;

    // Ensure the vhosts include is enabled in httpd.conf
    ensure_vhosts_include(xampp_path)?;

    // Enable SSL module if any HTTPS domains exist
    let needs_ssl = vhosts
        .iter()
        .any(|v| v.protocol == "https" || v.protocol == "both");
    if needs_ssl {
        ensure_ssl_module(xampp_path)?;
    }

    info!(
        "Updated httpd-vhosts.conf with {} vhosts",
        vhosts.len()
    );
    Ok(())
}

/// Restore httpd-vhosts.conf from backup.
pub fn rollback_vhosts(xampp_path: &str) -> Result<()> {
    let conf_path = vhosts_conf_path(xampp_path);
    let backup_path = format!("{}.localdomain.bak", conf_path);

    if std::path::Path::new(&backup_path).exists() {
        fs::copy(&backup_path, &conf_path).context("Failed to restore vhosts from backup")?;
        info!("Rolled back httpd-vhosts.conf from backup");
    }
    Ok(())
}

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

/// Ensure the `Include conf/extra/httpd-vhosts.conf` line is uncommented in httpd.conf.
pub fn ensure_vhosts_include(xampp_path: &str) -> Result<()> {
    let conf_path = httpd_conf_path(xampp_path);
    if !std::path::Path::new(&conf_path).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&conf_path).context("Failed to read httpd.conf")?;

    // Check if vhosts include is already enabled
    let has_active_include = content
        .lines()
        .any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#')
                && trimmed.contains("Include")
                && trimmed.contains("httpd-vhosts.conf")
        });

    if has_active_include {
        return Ok(());
    }

    // Try to uncomment the line
    let new_content = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#')
                && trimmed.contains("Include")
                && trimmed.contains("httpd-vhosts.conf")
            {
                // Remove leading # and optional space
                let uncommented = trimmed.trim_start_matches('#').trim_start();
                uncommented.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if new_content != content {
        fs::write(&conf_path, &new_content).context("Failed to update httpd.conf")?;
        info!("Enabled vhosts include in httpd.conf");
    }

    Ok(())
}

/// Ensure the SSL module is loaded in httpd.conf by uncommenting the LoadModule line.
pub fn ensure_ssl_module(xampp_path: &str) -> Result<()> {
    let conf_path = httpd_conf_path(xampp_path);
    if !std::path::Path::new(&conf_path).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&conf_path).context("Failed to read httpd.conf")?;

    let has_active_ssl = content
        .lines()
        .any(|line| {
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
        fs::write(&conf_path, &new_content).context("Failed to update httpd.conf")?;
        info!("Enabled ssl_module in httpd.conf");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XAMPP_PATH: &str = "/opt/lampp";

    #[test]
    fn test_build_vhosts_content_empty() {
        let result = build_vhosts_content("", &[], TEST_XAMPP_PATH, 80, 443);
        assert_eq!(result, "");
    }

    #[test]
    fn test_build_vhosts_content_http_only() {
        let vhosts = vec![XamppVhostConfig {
            name: "mysite.test".to_string(),
            document_root: "/var/www/mysite".to_string(),
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
        }];
        let result = build_vhosts_content("", &vhosts, TEST_XAMPP_PATH, 80, 443);
        assert!(result.contains(SENTINEL_START));
        assert!(result.contains(SENTINEL_END));
        assert!(result.contains("<VirtualHost *:80>"));
        assert!(result.contains("ServerName mysite.test"));
        assert!(result.contains("DocumentRoot \"/var/www/mysite\""));
        assert!(!result.contains("<VirtualHost *:443>"));
    }

    #[test]
    fn test_build_vhosts_content_https_only() {
        let vhosts = vec![XamppVhostConfig {
            name: "secure.test".to_string(),
            document_root: "/var/www/secure".to_string(),
            protocol: "https".to_string(),
            cert_path: Some("/certs/secure.crt".to_string()),
            key_path: Some("/certs/secure.key".to_string()),
        }];
        let result = build_vhosts_content("", &vhosts, TEST_XAMPP_PATH, 80, 443);
        assert!(result.contains("<VirtualHost *:443>"));
        assert!(result.contains("SSLEngine on"));
        assert!(result.contains("SSLCertificateFile \"/certs/secure.crt\""));
        assert!(result.contains("SSLCertificateKeyFile \"/certs/secure.key\""));
        // Should not have HTTP vhost for this domain
        let http_count = result.matches("<VirtualHost *:80>").count();
        // Only localhost preservation vhost on port 80
        assert_eq!(http_count, 1);
    }

    #[test]
    fn test_build_vhosts_content_both_protocols() {
        let vhosts = vec![XamppVhostConfig {
            name: "both.test".to_string(),
            document_root: "/var/www/both".to_string(),
            protocol: "both".to_string(),
            cert_path: Some("/certs/both.crt".to_string()),
            key_path: Some("/certs/both.key".to_string()),
        }];
        let result = build_vhosts_content("", &vhosts, TEST_XAMPP_PATH, 80, 443);
        // Should have both HTTP and HTTPS vhosts for the domain, plus localhost on :80
        let http_count = result.matches("<VirtualHost *:80>").count();
        let https_count = result.matches("<VirtualHost *:443>").count();
        assert_eq!(http_count, 2); // localhost + domain
        assert_eq!(https_count, 1);
    }

    #[test]
    fn test_build_vhosts_content_preserves_user_entries() {
        let existing = "# My custom vhost\n<VirtualHost *:80>\n    ServerName custom.local\n</VirtualHost>\n";
        let vhosts = vec![XamppVhostConfig {
            name: "managed.test".to_string(),
            document_root: "/var/www/managed".to_string(),
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
        }];
        let result = build_vhosts_content(existing, &vhosts, TEST_XAMPP_PATH, 80, 443);
        assert!(result.contains("custom.local"));
        assert!(result.contains("managed.test"));
    }

    #[test]
    fn test_build_vhosts_content_replaces_managed_block() {
        let existing = format!(
            "# User stuff\n\n{}\n<VirtualHost *:80>\n    ServerName old.test\n</VirtualHost>\n{}\n",
            SENTINEL_START, SENTINEL_END
        );
        let vhosts = vec![XamppVhostConfig {
            name: "new.test".to_string(),
            document_root: "/var/www/new".to_string(),
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
        }];
        let result = build_vhosts_content(&existing, &vhosts, TEST_XAMPP_PATH, 80, 443);
        assert!(!result.contains("old.test"));
        assert!(result.contains("new.test"));
        assert!(result.contains("# User stuff"));
    }

    #[test]
    fn test_build_vhosts_content_localhost_preservation() {
        let vhosts = vec![XamppVhostConfig {
            name: "mysite.test".to_string(),
            document_root: "/var/www/mysite".to_string(),
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
        }];
        let result = build_vhosts_content("", &vhosts, TEST_XAMPP_PATH, 80, 443);
        assert!(result.contains("ServerName localhost"));
        assert!(result.contains(r#"DocumentRoot "/opt/lampp/htdocs""#));
    }

    #[test]
    fn test_build_vhosts_content_localhost_uses_xampp_path() {
        let vhosts = vec![XamppVhostConfig {
            name: "mysite.test".to_string(),
            document_root: "/var/www/mysite".to_string(),
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
        }];
        let result = build_vhosts_content("", &vhosts, "/Applications/XAMPP/xamppfiles", 80, 443);
        assert!(result.contains(r#"DocumentRoot "/Applications/XAMPP/xamppfiles/htdocs""#));
    }

    #[test]
    fn test_build_vhosts_content_custom_ports() {
        let vhosts = vec![XamppVhostConfig {
            name: "mysite.test".to_string(),
            document_root: "/var/www/mysite".to_string(),
            protocol: "both".to_string(),
            cert_path: Some("/certs/mysite.crt".to_string()),
            key_path: Some("/certs/mysite.key".to_string()),
        }];
        let result = build_vhosts_content("", &vhosts, TEST_XAMPP_PATH, 8080, 4443);
        assert!(result.contains("<VirtualHost *:8080>"));
        assert!(result.contains("<VirtualHost *:4443>"));
        assert!(!result.contains("<VirtualHost *:80>"));
        assert!(!result.contains("<VirtualHost *:443>"));
    }

    #[test]
    fn test_ensure_vhosts_include_uncomments() {
        let input = "# Some config\n#Include conf/extra/httpd-vhosts.conf\n# More config\n";
        let new_content = input
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
        assert!(new_content.contains("Include conf/extra/httpd-vhosts.conf"));
        assert!(!new_content.contains("#Include"));
    }
}
