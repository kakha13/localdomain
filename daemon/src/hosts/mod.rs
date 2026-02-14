use anyhow::{Context, Result};
use localdomain_shared::domain::HostsEntry;
use std::fs;
use std::io::Write;
use tracing::info;

use crate::paths;

const SENTINEL_START: &str = "# LocalDomain Start";
const SENTINEL_END: &str = "# LocalDomain End";

pub fn sync_hosts(entries: &[HostsEntry]) -> Result<()> {
    let hosts_path = paths::HOSTS_FILE;
    let current = fs::read_to_string(hosts_path).context("Failed to read hosts file")?;

    // Create backup
    let backup_path = format!("{}.localdomain.bak", hosts_path);
    fs::write(&backup_path, &current).context("Failed to create hosts backup")?;

    let new_content = build_hosts_content(&current, entries);

    // Atomic write via temp file
    let tmp_path = format!("{}.localdomain.tmp", hosts_path);
    {
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(new_content.as_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp_path, hosts_path).context("Failed to replace hosts file")?;

    info!("Updated hosts file with {} entries", entries.len());
    Ok(())
}

fn build_hosts_content(current: &str, entries: &[HostsEntry]) -> String {
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
    result.push('\n');

    if !entries.is_empty() {
        result.push('\n');
        result.push_str(SENTINEL_START);
        result.push('\n');
        for entry in entries {
            result.push_str(&format!("{}\t{}\n", entry.ip, entry.domain));
        }
        result.push_str(SENTINEL_END);
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_hosts_content_empty_entries() {
        let current = "127.0.0.1\tlocalhost\n";
        let result = build_hosts_content(current, &[]);
        assert_eq!(result, "127.0.0.1\tlocalhost\n");
    }

    #[test]
    fn test_build_hosts_content_with_entries() {
        let current = "127.0.0.1\tlocalhost\n";
        let entries = vec![HostsEntry {
            domain: "project.test".to_string(),
            ip: "127.0.0.1".to_string(),
        }];
        let result = build_hosts_content(current, &entries);
        assert!(result.contains(SENTINEL_START));
        assert!(result.contains("127.0.0.1\tproject.test"));
        assert!(result.contains(SENTINEL_END));
    }

    #[test]
    fn test_build_hosts_content_replaces_existing_block() {
        let current = format!(
            "127.0.0.1\tlocalhost\n\n{}\n127.0.0.1\told.test\n{}\n",
            SENTINEL_START, SENTINEL_END
        );
        let entries = vec![HostsEntry {
            domain: "new.test".to_string(),
            ip: "127.0.0.1".to_string(),
        }];
        let result = build_hosts_content(&current, &entries);
        assert!(!result.contains("old.test"));
        assert!(result.contains("new.test"));
    }
}
