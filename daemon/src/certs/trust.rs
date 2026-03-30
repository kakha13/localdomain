use anyhow::{Context, Result};
use localdomain_shared::silent_cmd;
use tracing::{info, warn};

use super::ca;

// ---- macOS: System Keychain ----

/// Remove stale "LocalDomain Root CA" certs from System Keychain.
/// This handles duplicates from prior CA regenerations.
#[cfg(target_os = "macos")]
fn cleanup_stale_ca_certs() {
    for _ in 0..10 {
        let output = silent_cmd("security")
            .args([
                "delete-certificate",
                "-c",
                "LocalDomain Root CA",
                "/Library/Keychains/System.keychain",
            ])
            .output();
        match output {
            Ok(o) if o.status.success() => continue,
            _ => break,
        }
    }
}

#[cfg(target_os = "macos")]
pub fn install_ca_trust() -> Result<()> {
    if !ca::ca_exists() {
        anyhow::bail!("CA certificate does not exist. Generate it first.");
    }

    // Clean up stale certs before adding
    cleanup_stale_ca_certs();

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
    // Check user-level trust settings (visible when running as the same user)
    if let Ok(output) = silent_cmd("security")
        .args(["dump-trust-settings"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("LocalDomain Root CA") {
            return true;
        }
    }
    // Fallback for macOS Sequoia+: trust settings may be at user-level (not
    // visible to the daemon running as root). Check if the current CA cert
    // is in the System Keychain — its presence means it was added via
    // `security add-trusted-cert`, so trust was established for the user.
    ca_cert_in_system_keychain()
}

/// Check if the current CA cert on disk is present in the System Keychain.
#[cfg(target_os = "macos")]
fn ca_cert_in_system_keychain() -> bool {
    let ca_pem = match std::fs::read_to_string(ca::ca_cert_path()) {
        Ok(pem) => normalize_pem(&pem),
        Err(_) => return false,
    };
    if ca_pem.is_empty() {
        return false;
    }

    // Export all "LocalDomain Root CA" certs from System Keychain as PEM
    let output = match silent_cmd("security")
        .args([
            "find-certificate",
            "-c",
            "LocalDomain Root CA",
            "-a",
            "-p",
            "/Library/Keychains/System.keychain",
        ])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };

    let keychain_pems = String::from_utf8_lossy(&output.stdout);
    // Split the concatenated PEM output into individual certs and compare
    for cert_block in keychain_pems.split("-----END CERTIFICATE-----") {
        let normalized = normalize_pem(cert_block);
        if !normalized.is_empty() && normalized == ca_pem {
            return true;
        }
    }
    false
}

/// Strip PEM headers, whitespace, and newlines to get just the base64 content.
#[cfg(target_os = "macos")]
fn normalize_pem(pem: &str) -> String {
    pem.lines()
        .map(|l| l.trim())
        .filter(|l| !l.starts_with("-----") && !l.is_empty())
        .collect::<String>()
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
