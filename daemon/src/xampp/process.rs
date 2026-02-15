use anyhow::{Context, Result};
use tracing::info;

use super::detect;

/// Test the Apache configuration for syntax errors.
pub fn test_apache_config(xampp_path: &str) -> Result<()> {
    let httpd = detect::get_httpd_binary(xampp_path);

    let output = localdomain_shared::silent_cmd(&httpd)
        .arg("-t")
        .output()
        .context("Failed to run httpd -t")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache config test failed: {}", stderr.trim()));
    }

    info!("Apache config test passed");
    Ok(())
}

/// Start Apache using the platform-appropriate method.
pub fn start_apache(xampp_path: &str) -> Result<()> {
    start_apache_platform(xampp_path)
}

/// Stop Apache using the platform-appropriate method.
pub fn stop_apache(xampp_path: &str) -> Result<()> {
    stop_apache_platform(xampp_path)
}

/// Restart Apache using the platform-appropriate method.
pub fn restart_apache(xampp_path: &str) -> Result<()> {
    restart_apache_platform(xampp_path)
}

#[cfg(target_os = "macos")]
fn restart_apache_platform(xampp_path: &str) -> Result<()> {
    let apachectl = format!("{}/bin/apachectl", xampp_path);
    let output = std::process::Command::new(&apachectl)
        .arg("restart")
        .output()
        .context("Failed to restart Apache via apachectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache restart failed: {}", stderr.trim()));
    }

    info!("Apache restarted successfully");
    Ok(())
}

#[cfg(target_os = "linux")]
fn restart_apache_platform(xampp_path: &str) -> Result<()> {
    let lampp = format!("{}/lampp", xampp_path);
    let output = std::process::Command::new(&lampp)
        .arg("restartapache")
        .output()
        .context("Failed to restart Apache via lampp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache restart failed: {}", stderr.trim()));
    }

    info!("Apache restarted successfully");
    Ok(())
}

#[cfg(target_os = "windows")]
fn restart_apache_platform(xampp_path: &str) -> Result<()> {
    let httpd = format!("{}\\apache\\bin\\httpd.exe", xampp_path);
    let output = localdomain_shared::silent_cmd(&httpd)
        .arg("-k")
        .arg("restart")
        .output()
        .context("Failed to restart Apache")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache restart failed: {}", stderr.trim()));
    }

    info!("Apache restarted successfully");
    Ok(())
}

#[cfg(target_os = "macos")]
fn start_apache_platform(xampp_path: &str) -> Result<()> {
    let apachectl = format!("{}/bin/apachectl", xampp_path);
    let output = std::process::Command::new(&apachectl)
        .arg("start")
        .output()
        .context("Failed to start Apache via apachectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache start failed: {}", stderr.trim()));
    }

    info!("Apache started successfully");
    Ok(())
}

#[cfg(target_os = "linux")]
fn start_apache_platform(xampp_path: &str) -> Result<()> {
    let lampp = format!("{}/lampp", xampp_path);
    let output = std::process::Command::new(&lampp)
        .arg("startapache")
        .output()
        .context("Failed to start Apache via lampp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache start failed: {}", stderr.trim()));
    }

    info!("Apache started successfully");
    Ok(())
}

#[cfg(target_os = "windows")]
fn start_apache_platform(xampp_path: &str) -> Result<()> {
    let httpd = format!("{}\\apache\\bin\\httpd.exe", xampp_path);
    let output = localdomain_shared::silent_cmd(&httpd)
        .arg("-k")
        .arg("start")
        .output()
        .context("Failed to start Apache")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache start failed: {}", stderr.trim()));
    }

    info!("Apache started successfully");
    Ok(())
}

#[cfg(target_os = "macos")]
fn stop_apache_platform(xampp_path: &str) -> Result<()> {
    let apachectl = format!("{}/bin/apachectl", xampp_path);
    let output = std::process::Command::new(&apachectl)
        .arg("stop")
        .output()
        .context("Failed to stop Apache via apachectl")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache stop failed: {}", stderr.trim()));
    }

    info!("Apache stopped successfully");
    Ok(())
}

#[cfg(target_os = "linux")]
fn stop_apache_platform(xampp_path: &str) -> Result<()> {
    let lampp = format!("{}/lampp", xampp_path);
    let output = std::process::Command::new(&lampp)
        .arg("stopapache")
        .output()
        .context("Failed to stop Apache via lampp")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache stop failed: {}", stderr.trim()));
    }

    info!("Apache stopped successfully");
    Ok(())
}

#[cfg(target_os = "windows")]
fn stop_apache_platform(xampp_path: &str) -> Result<()> {
    let httpd = format!("{}\\apache\\bin\\httpd.exe", xampp_path);
    let output = localdomain_shared::silent_cmd(&httpd)
        .arg("-k")
        .arg("stop")
        .output()
        .context("Failed to stop Apache")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Apache stop failed: {}", stderr.trim()));
    }

    info!("Apache stopped successfully");
    Ok(())
}

/// Check if Apache is running by looking for the httpd process.
pub fn is_apache_running(xampp_path: &str) -> bool {
    is_apache_running_platform(xampp_path)
}

#[cfg(unix)]
fn is_apache_running_platform(xampp_path: &str) -> bool {
    let httpd = detect::get_httpd_binary(xampp_path);
    // Use pgrep to check for httpd process
    std::process::Command::new("pgrep")
        .arg("-f")
        .arg(&httpd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn is_apache_running_platform(_xampp_path: &str) -> bool {
    localdomain_shared::silent_cmd("tasklist")
        .args(["/FI", "IMAGENAME eq httpd.exe"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("httpd.exe")
        })
        .unwrap_or(false)
}
