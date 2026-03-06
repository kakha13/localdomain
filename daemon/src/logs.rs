use anyhow::{bail, Result};
use localdomain_shared::protocol::AccessLogEntry;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::paths;

/// Validate that a domain name is safe for use in file paths.
/// Rejects path traversal characters like '/', '..', '\', and null bytes.
fn validate_domain_for_path(domain: &str) -> Result<()> {
    if domain.is_empty() {
        bail!("Domain name cannot be empty");
    }
    if domain.contains('/') || domain.contains('\\') || domain.contains('\0') {
        bail!("Domain name contains invalid characters");
    }
    if domain == "." || domain == ".." || domain.contains("..") {
        bail!("Domain name contains path traversal sequence");
    }
    Ok(())
}

fn log_path(domain: &str) -> Result<String> {
    validate_domain_for_path(domain)?;
    Ok(std::path::Path::new(paths::LOGS_DIR)
        .join(format!("{}.access.log", domain))
        .to_string_lossy()
        .to_string())
}

pub fn read_access_log(domain: &str, limit: u64) -> Result<Vec<AccessLogEntry>> {
    let path = log_path(domain)?;
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.into()),
    };

    let reader = BufReader::new(file);
    // Use a ring buffer to keep only the last `limit` entries in memory
    let limit = limit as usize;
    let mut entries: std::collections::VecDeque<AccessLogEntry> = std::collections::VecDeque::with_capacity(limit.min(1024) + 1);

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let request = &parsed["request"];
        let entry = AccessLogEntry {
            timestamp: parsed["ts"].as_f64().unwrap_or(0.0),
            method: request["method"].as_str().unwrap_or("").to_string(),
            uri: request["uri"].as_str().unwrap_or("").to_string(),
            status: parsed["status"].as_u64().unwrap_or(0) as u16,
            duration: parsed["duration"].as_f64().unwrap_or(0.0),
            size: parsed["size"].as_u64().unwrap_or(0),
            host: request["host"].as_str().unwrap_or("").to_string(),
            headers: request["headers"].clone(),
            resp_headers: parsed["resp_headers"].clone(),
            remote_ip: request["remote_ip"].as_str().unwrap_or("").to_string(),
            proto: request["proto"].as_str().unwrap_or("").to_string(),
        };
        entries.push_back(entry);
        if entries.len() > limit {
            entries.pop_front();
        }
    }

    // Return entries newest first
    let mut result: Vec<AccessLogEntry> = entries.into();
    result.reverse();

    Ok(result)
}

pub fn clear_access_log(domain: &str) -> Result<()> {
    let path = log_path(domain)?;
    // Truncate to 0 bytes (Caddy keeps file handle open and will continue writing)
    if std::path::Path::new(&path).exists() {
        fs::write(&path, b"")?;
    }
    Ok(())
}
