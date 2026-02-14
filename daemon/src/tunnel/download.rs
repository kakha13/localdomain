use anyhow::{bail, Context, Result};
use localdomain_shared::protocol::EnsureCloudflaredResult;
use std::path::Path;
use tracing::info;

use crate::paths;

fn get_cloudflared_version() -> Option<String> {
    std::process::Command::new(paths::CLOUDFLARED_BINARY)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout).to_string();
            // cloudflared version 2024.x.x (built ...)
            out.split_whitespace()
                .nth(2)
                .map(|v| v.to_string())
        })
}

#[cfg(unix)]
fn download_cloudflared() -> Result<()> {
    let url = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-arm64.tgz"
        } else {
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-darwin-amd64.tgz"
        }
    } else {
        // Linux
        if cfg!(target_arch = "aarch64") {
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-arm64"
        } else {
            "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64"
        }
    };

    if cfg!(target_os = "macos") {
        // macOS: download tgz, extract
        let tgz_path = format!("{}/cloudflared.tgz", paths::TUNNEL_DIR);
        let status = std::process::Command::new("curl")
            .args(["-fSL", "-o", &tgz_path, url])
            .status()
            .context("Failed to run curl")?;
        if !status.success() {
            bail!("Failed to download cloudflared");
        }
        let status = std::process::Command::new("tar")
            .args(["-xzf", &tgz_path, "-C", paths::TUNNEL_DIR])
            .status()
            .context("Failed to extract cloudflared")?;
        if !status.success() {
            bail!("Failed to extract cloudflared");
        }
        // Move extracted binary to final location
        let extracted = format!("{}/cloudflared", paths::TUNNEL_DIR);
        std::fs::rename(&extracted, paths::CLOUDFLARED_BINARY)
            .context("Failed to move cloudflared binary")?;
        let _ = std::fs::remove_file(&tgz_path);
    } else {
        // Linux: direct binary download
        let status = std::process::Command::new("curl")
            .args(["-fSL", "-o", paths::CLOUDFLARED_BINARY, url])
            .status()
            .context("Failed to run curl")?;
        if !status.success() {
            bail!("Failed to download cloudflared");
        }
    }

    // Make executable
    let status = std::process::Command::new("chmod")
        .args(["+x", paths::CLOUDFLARED_BINARY])
        .status()
        .context("Failed to chmod cloudflared")?;
    if !status.success() {
        bail!("Failed to set cloudflared as executable");
    }

    Ok(())
}

#[cfg(windows)]
fn download_cloudflared() -> Result<()> {
    let url = if cfg!(target_arch = "aarch64") {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-arm64.exe"
    } else {
        "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe"
    };

    // Ensure parent directory exists
    if let Some(parent) = Path::new(paths::CLOUDFLARED_BINARY).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                url, paths::CLOUDFLARED_BINARY
            ),
        ])
        .status()
        .context("Failed to run PowerShell")?;

    if !status.success() {
        bail!("Failed to download cloudflared");
    }

    Ok(())
}

pub fn ensure_cloudflared() -> Result<EnsureCloudflaredResult> {
    if Path::new(paths::CLOUDFLARED_BINARY).exists() {
        let version = get_cloudflared_version();
        info!("cloudflared already installed at {}", paths::CLOUDFLARED_BINARY);
        return Ok(EnsureCloudflaredResult {
            installed: true,
            path: paths::CLOUDFLARED_BINARY.to_string(),
            version,
        });
    }

    info!("Downloading cloudflared...");
    std::fs::create_dir_all(paths::TUNNEL_DIR).ok();
    download_cloudflared()?;

    let version = get_cloudflared_version();
    info!("cloudflared installed at {}", paths::CLOUDFLARED_BINARY);

    Ok(EnsureCloudflaredResult {
        installed: true,
        path: paths::CLOUDFLARED_BINARY.to_string(),
        version,
    })
}
