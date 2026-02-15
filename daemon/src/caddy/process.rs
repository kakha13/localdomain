use anyhow::{Context, Result};
use localdomain_shared::silent_cmd;
use std::fs;
use tracing::info;

use crate::paths;

#[cfg(unix)]
fn is_process_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(windows)]
fn is_process_alive(pid: i32) -> bool {
    silent_cmd("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

pub fn is_caddy_running() -> bool {
    if let Ok(pid_str) = fs::read_to_string(paths::CADDY_PID) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            return is_process_alive(pid);
        }
    }
    false
}

pub fn start_caddy() -> Result<()> {
    if is_caddy_running() {
        info!("Caddy already running");
        return Ok(());
    }

    // Ensure Caddyfile exists
    if !std::path::Path::new(paths::CADDYFILE).exists() {
        fs::write(
            paths::CADDYFILE,
            "{\n\tadmin off\n}\n\n:65535 {\n\trespond \"LocalDomain placeholder\" 200\n}\n",
        )?;
    }

    if !std::path::Path::new(paths::CADDY_BINARY).exists() {
        anyhow::bail!(
            "Caddy binary not found at {}. Please reinstall the service to download it.",
            paths::CADDY_BINARY
        );
    }

    let child = silent_cmd(paths::CADDY_BINARY)
        .args(["run", "--config", paths::CADDYFILE])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to start Caddy")?;

    fs::write(paths::CADDY_PID, child.id().to_string())?;
    info!("Caddy started with PID {}", child.id());
    Ok(())
}

#[cfg(unix)]
fn kill_process(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
}

#[cfg(windows)]
fn kill_process(pid: i32) {
    match silent_cmd("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output()
    {
        Ok(output) if !output.status.success() => {
            tracing::warn!(
                "taskkill failed for PID {}: {}",
                pid,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            tracing::warn!("Failed to run taskkill for PID {}: {}", pid, e);
        }
        _ => {}
    }
}

pub fn stop_caddy() -> Result<()> {
    if let Ok(pid_str) = fs::read_to_string(paths::CADDY_PID) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            kill_process(pid);
            // Wait for process to fully exit (up to 5s) so ports are released
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(250));
                if !is_process_alive(pid) {
                    break;
                }
            }
            info!("Stopped Caddy (PID {})", pid);
        }
    }
    let _ = fs::remove_file(paths::CADDY_PID);
    Ok(())
}

pub fn reload_caddy() -> Result<()> {
    // Since we use `admin off`, we can't use `caddy reload`.
    // Instead, stop and restart Caddy with the new config.
    if is_caddy_running() {
        stop_caddy()?;
    }
    start_caddy()
}
