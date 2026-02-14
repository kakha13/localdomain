use anyhow::Result;
use localdomain_shared::protocol::AccessLogEntry;
use std::fs;
use std::io::{BufRead, BufReader};

use crate::paths;

fn log_path(domain: &str) -> String {
    std::path::Path::new(paths::LOGS_DIR)
        .join(format!("{}.access.log", domain))
        .to_string_lossy()
        .to_string()
}

pub fn read_access_log(domain: &str, limit: u64) -> Result<Vec<AccessLogEntry>> {
    let path = log_path(domain);
    let file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.into()),
    };

    let reader = BufReader::new(file);
    let mut entries: Vec<AccessLogEntry> = Vec::new();

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
        entries.push(entry);
    }

    // Return last N entries, newest first
    if entries.len() > limit as usize {
        entries = entries.split_off(entries.len() - limit as usize);
    }
    entries.reverse();

    Ok(entries)
}

pub fn clear_access_log(domain: &str) -> Result<()> {
    let path = log_path(domain);
    // Truncate to 0 bytes (Caddy keeps file handle open and will continue writing)
    if std::path::Path::new(&path).exists() {
        fs::write(&path, b"")?;
    }
    Ok(())
}
