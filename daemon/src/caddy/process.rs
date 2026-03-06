use anyhow::{Context, Result};
use localdomain_shared::silent_cmd;
use std::fs;
use std::sync::Mutex;
use tracing::info;

use crate::paths;

/// Mutex to prevent concurrent start/stop of Caddy (race condition guard)
static CADDY_LOCK: Mutex<()> = Mutex::new(());

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
            let pid_str = pid.to_string();
            // Match PID as a whole word to avoid false positives (e.g., PID 123 matching 1234)
            stdout.split_whitespace().any(|word| word == pid_str)
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
    let _lock = CADDY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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

    let pid = child.id();
    fs::write(paths::CADDY_PID, pid.to_string())?;
    // Intentionally leak the Child handle to prevent zombie process on Unix.
    // We track the PID via the PID file and manage the process lifecycle explicitly.
    std::mem::forget(child);
    info!("Caddy started with PID {}", pid);
    Ok(())
}

#[cfg(unix)]
fn kill_process(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
}

#[cfg(unix)]
fn force_kill_process(pid: i32) {
    unsafe {
        libc::kill(pid, libc::SIGKILL);
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

#[cfg(windows)]
fn force_kill_process(pid: i32) {
    // On Windows, taskkill /F already does a force kill
    kill_process(pid);
}

pub fn stop_caddy() -> Result<()> {
    let _lock = CADDY_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
            // If still alive after SIGTERM, force kill
            if is_process_alive(pid) {
                force_kill_process(pid);
                std::thread::sleep(std::time::Duration::from_millis(500));
                tracing::warn!("Had to force-kill Caddy (PID {})", pid);
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
