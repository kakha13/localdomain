use anyhow::{bail, Context, Result};
use localdomain_shared::silent_cmd;
use std::process::Stdio;
use tracing::info;

use crate::paths;

/// Start a Cloudflare Quick Tunnel (trycloudflare.com).
/// Returns (public_url, pid).
pub fn start_quick_tunnel(domain: &str, local_port: u16) -> Result<(String, u32)> {
    if !std::path::Path::new(paths::CLOUDFLARED_BINARY).exists() {
        bail!(
            "cloudflared not found at {}. Use ensure_cloudflared first.",
            paths::CLOUDFLARED_BINARY
        );
    }

    let log_file_path = format!("{}/{}.log", paths::TUNNEL_DIR, domain.replace('.', "_"));
    std::fs::create_dir_all(paths::TUNNEL_DIR).ok();

    // Spawn cloudflared with stderr going to a log file so we can parse the URL
    let log_file = std::fs::File::create(&log_file_path)
        .context("Failed to create tunnel log file")?;

    // Use the domain name in the origin URL instead of "localhost" so the HTTP
    // Host header is set naturally to the domain. This ensures Apache VirtualHost
    // matching works correctly for XAMPP domains. The domain resolves to 127.0.0.1
    // via the hosts file (synced before tunnel start). --http-host-header is kept
    // as a belt-and-suspenders override.
    let origin_url = format!("http://{}:{}", domain, local_port);

    let child = silent_cmd(paths::CLOUDFLARED_BINARY)
        .args([
            "tunnel",
            "--url",
            &origin_url,
            "--http-host-header",
            domain,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::from(log_file.try_clone()?))
        .spawn()
        .context("Failed to start cloudflared")?;

    let pid = child.id();
    info!(
        "cloudflared quick tunnel spawned (PID {}) for {} -> localhost:{}",
        pid, domain, local_port
    );

    // Poll the log file for the trycloudflare.com URL (up to 15 seconds)
    let mut public_url = None;
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Ok(contents) = std::fs::read_to_string(&log_file_path) {
            // cloudflared logs the URL like: https://xxx-xxx-xxx.trycloudflare.com
            for line in contents.lines() {
                if let Some(pos) = line.find("https://") {
                    let url_part = &line[pos..];
                    let url_end = url_part
                        .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                        .unwrap_or(url_part.len());
                    let url = &url_part[..url_end];
                    if url.contains("trycloudflare.com") {
                        public_url = Some(url.to_string());
                        break;
                    }
                }
            }
        }
        if public_url.is_some() {
            break;
        }
    }

    match public_url {
        Some(url) => {
            info!("Quick tunnel URL for {}: {}", domain, url);
            Ok((url, pid))
        }
        None => {
            // Kill the process since we couldn't get the URL
            #[cfg(unix)]
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
            #[cfg(windows)]
            {
                let _ = silent_cmd("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .output();
            }
            bail!("Timed out waiting for cloudflared to provide a public URL")
        }
    }
}

/// Start a Cloudflare Named Tunnel.
/// If credentials_json + tunnel_uuid are provided, uses local config mode (writes config.yml + credentials).
/// Otherwise uses --token mode (requires pre-configured tunnel in Zero Trust dashboard).
/// Returns (public_url, pid).
pub fn start_named_tunnel(
    domain: &str,
    local_port: u16,
    token: &str,
    subdomain: &str,
    cloudflare_domain: &str,
    credentials_json: &str,
    tunnel_uuid: &str,
) -> Result<(String, u32)> {
    if !std::path::Path::new(paths::CLOUDFLARED_BINARY).exists() {
        bail!(
            "cloudflared not found at {}. Use ensure_cloudflared first.",
            paths::CLOUDFLARED_BINARY
        );
    }

    let log_file_path = format!(
        "{}/{}_named.log",
        paths::TUNNEL_DIR,
        domain.replace('.', "_")
    );
    std::fs::create_dir_all(paths::TUNNEL_DIR).ok();

    let log_file =
        std::fs::File::create(&log_file_path).context("Failed to create tunnel log file")?;

    let use_config_mode = !credentials_json.is_empty() && !tunnel_uuid.is_empty();

    let child = if use_config_mode {
        // Config-file mode: write credentials + config.yml with ingress rules
        let creds_path = format!("{}/{}.json", paths::TUNNEL_DIR, tunnel_uuid);
        std::fs::write(&creds_path, credentials_json)
            .context("Failed to write tunnel credentials")?;

        let hostname = if !subdomain.is_empty() && !cloudflare_domain.is_empty() {
            format!("{}.{}", subdomain, cloudflare_domain)
        } else {
            String::new()
        };

        let config_content = format!(
            "tunnel: {tunnel_uuid}\ncredentials-file: {creds_path}\ningress:\n  - hostname: {hostname}\n    service: http://{domain}:{local_port}\n    originRequest:\n      httpHostHeader: {domain}\n  - service: http_status:404\n"
        );
        let config_path = format!("{}/{}_config.yml", paths::TUNNEL_DIR, domain.replace('.', "_"));
        std::fs::write(&config_path, &config_content)
            .context("Failed to write tunnel config")?;

        info!("Using config-file mode for tunnel {} ({})", domain, tunnel_uuid);
        silent_cmd(paths::CLOUDFLARED_BINARY)
            .args(["tunnel", "--config", &config_path, "run", tunnel_uuid])
            .stdout(Stdio::null())
            .stderr(Stdio::from(log_file.try_clone()?))
            .spawn()
            .context("Failed to start cloudflared named tunnel (config mode)")?
    } else {
        // Token mode: remotely managed (ingress configured in dashboard)
        silent_cmd(paths::CLOUDFLARED_BINARY)
            .args(["tunnel", "run", "--token", token])
            .stdout(Stdio::null())
            .stderr(Stdio::from(log_file.try_clone()?))
            .spawn()
            .context("Failed to start cloudflared named tunnel")?
    };

    let pid = child.id();
    info!(
        "cloudflared named tunnel spawned (PID {}) for {}",
        pid, domain
    );

    // Poll log for the registered hostname (up to 15 seconds)
    // cloudflared logs lines like: "... external hostname: https://sub.example.com"
    // or "... Route ... to ... https://sub.example.com"
    let mut public_url = None;
    for _ in 0..30 {
        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Ok(contents) = std::fs::read_to_string(&log_file_path) {
            for line in contents.lines() {
                if let Some(pos) = line.find("https://") {
                    let url_part = &line[pos..];
                    let url_end = url_part
                        .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
                        .unwrap_or(url_part.len());
                    let url = &url_part[..url_end];
                    // Skip trycloudflare URLs and cloudflared internal URLs
                    if !url.contains("trycloudflare.com")
                        && !url.contains("argotunnel.com")
                        && url.len() > "https://".len() + 3
                    {
                        public_url = Some(url.to_string());
                        break;
                    }
                }
            }
        }
        if public_url.is_some() {
            break;
        }
    }

    match public_url {
        Some(url) => {
            info!("Named tunnel URL for {}: {}", domain, url);
            Ok((url, pid))
        }
        None => {
            // Use the expected URL from subdomain + domain as fallback
            let fallback = if !subdomain.is_empty() && !cloudflare_domain.is_empty() {
                format!("https://{}.{}", subdomain, cloudflare_domain)
            } else {
                "https://tunnel-connecting...".to_string()
            };
            info!(
                "Could not detect public URL from logs for {}, using fallback: {}",
                domain, fallback
            );
            Ok((fallback, pid))
        }
    }
}
