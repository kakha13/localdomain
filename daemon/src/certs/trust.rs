use anyhow::{Context, Result};
use localdomain_shared::silent_cmd;
use tracing::{info, warn};

use super::ca;

// ---- macOS: System Keychain ----

#[cfg(target_os = "macos")]
pub fn install_ca_trust() -> Result<()> {
    if !ca::ca_exists() {
        anyhow::bail!("CA certificate does not exist. Generate it first.");
    }

    let output = silent_cmd("security")
        .args([
            "add-trusted-cert",
            "-d",
            "-r",
            "trustRoot",
            "-p",
            "ssl",
            "-k",
            "/Library/Keychains/System.keychain",
            ca::ca_cert_path(),
        ])
        .output()
        .context("Failed to run security command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(
            "CA trust from daemon failed (expected in non-interactive mode): {}",
            stderr.trim()
        );
        anyhow::bail!("non-interactive");
    }

    info!("CA certificate trusted in System Keychain");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn remove_ca_trust() -> Result<()> {
    let output = silent_cmd("security")
        .args(["remove-trusted-cert", "-d", ca::ca_cert_path()])
        .output()
        .context("Failed to run security command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("not found") {
            anyhow::bail!("Failed to remove CA trust: {}", stderr);
        }
    }

    info!("CA certificate removed from System Keychain");
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn verify_ca_trust() -> bool {
    // Check admin-level trust settings (daemon installs there)
    if let Ok(output) = silent_cmd("security")
        .args(["dump-trust-settings", "-d"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("LocalDomain Root CA") {
            return true;
        }
    }
    // Also check user-level trust settings
    if let Ok(output) = silent_cmd("security")
        .args(["dump-trust-settings"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("LocalDomain Root CA") {
            return true;
        }
    }
    false
}

// ---- Linux: update-ca-certificates ----

#[cfg(target_os = "linux")]
pub fn install_ca_trust() -> Result<()> {
    if !ca::ca_exists() {
        anyhow::bail!("CA certificate does not exist. Generate it first.");
    }

    let dest = "/usr/local/share/ca-certificates/localdomain-ca.crt";
    std::fs::copy(ca::ca_cert_path(), dest)
        .context("Failed to copy CA cert to /usr/local/share/ca-certificates/")?;

    let output = silent_cmd("update-ca-certificates")
        .output()
        .context("Failed to run update-ca-certificates")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("CA trust failed: {}", stderr.trim());
        anyhow::bail!("update-ca-certificates failed: {}", stderr.trim());
    }

    info!("CA certificate trusted via update-ca-certificates");
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn remove_ca_trust() -> Result<()> {
    let dest = "/usr/local/share/ca-certificates/localdomain-ca.crt";
    if std::path::Path::new(dest).exists() {
        std::fs::remove_file(dest).context("Failed to remove CA cert")?;

        let output = silent_cmd("update-ca-certificates")
            .arg("--fresh")
            .output()
            .context("Failed to run update-ca-certificates")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to update CA trust: {}", stderr);
        }
    }

    info!("CA certificate removed from system trust store");
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn verify_ca_trust() -> bool {
    std::path::Path::new("/usr/local/share/ca-certificates/localdomain-ca.crt").exists()
}

// ---- Windows: Certificate Store via certutil ----

#[cfg(target_os = "windows")]
pub fn install_ca_trust() -> Result<()> {
    if !ca::ca_exists() {
        anyhow::bail!("CA certificate does not exist. Generate it first.");
    }

    let output = silent_cmd("certutil")
        .args(["-addstore", "Root", ca::ca_cert_path()])
        .output()
        .context("Failed to run certutil")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!("CA trust failed: {}", stderr.trim());
        anyhow::bail!("certutil failed: {}", stderr.trim());
    }

    info!("CA certificate trusted in Windows certificate store");
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn remove_ca_trust() -> Result<()> {
    let output = silent_cmd("certutil")
        .args(["-delstore", "Root", "LocalDomain Root CA"])
        .output()
        .context("Failed to run certutil")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.contains("not found") {
            anyhow::bail!("Failed to remove CA trust: {}", stderr);
        }
    }

    info!("CA certificate removed from Windows certificate store");
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn verify_ca_trust() -> bool {
    if let Ok(output) = silent_cmd("certutil")
        .args(["-verifystore", "Root", "LocalDomain Root CA"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Confirm stdout actually contains the cert info
            return stdout.contains("LocalDomain Root CA");
        }
    }
    false
}
