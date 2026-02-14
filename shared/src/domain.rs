use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
    Both,
}

impl Protocol {
    pub fn as_str(&self) -> &str {
        match self {
            Protocol::Http => "http",
            Protocol::Https => "https",
            Protocol::Both => "both",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "http" => Some(Protocol::Http),
            "https" => Some(Protocol::Https),
            "both" => Some(Protocol::Both),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub id: String,
    pub name: String,
    pub target_host: String,
    pub target_port: u16,
    pub protocol: Protocol,
    pub wildcard: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsEntry {
    pub domain: String,
    pub ip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaddyDomainConfig {
    pub name: String,
    pub target_host: String,
    pub target_port: u16,
    /// "http", "https", or "both"
    pub protocol: String,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    #[serde(default)]
    pub access_log: bool,
}

/// Validates a domain name for use in /etc/hosts.
/// Allows alphanumeric, hyphens, dots. Must end with a valid TLD-like segment.
pub fn validate_domain_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Domain name cannot be empty".to_string());
    }
    if name.len() > 253 {
        return Err("Domain name too long (max 253 characters)".to_string());
    }

    let re = regex_lite::Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?)*\.[a-zA-Z]{2,}$")
        .unwrap();

    if !re.is_match(name) {
        return Err(format!(
            "Invalid domain name '{}'. Use format like 'project.test'",
            name
        ));
    }

    Ok(())
}

/// XAMPP VirtualHost configuration sent to the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XamppVhostConfig {
    pub name: String,
    pub document_root: String,
    /// "http", "https", or "both"
    pub protocol: String,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

/// Validates a port number.
pub fn validate_port(port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("Port cannot be 0".to_string());
    }
    Ok(())
}

/// Validates a document root path for XAMPP domains.
/// Must be non-empty and an absolute path.
pub fn validate_document_root(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Document root cannot be empty".to_string());
    }
    if !std::path::Path::new(path).is_absolute() {
        return Err("Document root must be an absolute path".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_domain_names() {
        assert!(validate_domain_name("project.test").is_ok());
        assert!(validate_domain_name("my-app.test").is_ok());
        assert!(validate_domain_name("sub.domain.test").is_ok());
        assert!(validate_domain_name("a.dev").is_ok());
    }

    #[test]
    fn test_invalid_domain_names() {
        assert!(validate_domain_name("").is_err());
        assert!(validate_domain_name("project").is_err());
        assert!(validate_domain_name("-project.test").is_err());
        assert!(validate_domain_name("project-.test").is_err());
        assert!(validate_domain_name("project.t").is_err());
        assert!(validate_domain_name(".test").is_err());
    }

    #[test]
    fn test_port_validation() {
        assert!(validate_port(3000).is_ok());
        assert!(validate_port(80).is_ok());
        assert!(validate_port(0).is_err());
    }
}
