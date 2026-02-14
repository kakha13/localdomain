use anyhow::Result;
use localdomain_shared::domain::CaddyDomainConfig;
use std::fs;
use std::io::Write;
use tracing::info;

use crate::paths;

pub fn generate_caddyfile(
    domains: &[CaddyDomainConfig],
    http_port: u16,
    https_port: u16,
) -> Result<()> {
    let content = build_caddyfile(domains, http_port, https_port);

    let mut f = fs::File::create(paths::CADDYFILE)?;
    f.write_all(content.as_bytes())?;
    f.sync_all()?;

    info!(
        "Generated Caddyfile with {} domains (HTTP:{}, HTTPS:{})",
        domains.len(),
        http_port,
        https_port
    );
    Ok(())
}

fn build_caddyfile(domains: &[CaddyDomainConfig], http_port: u16, https_port: u16) -> String {
    let mut out = String::new();

    // Global options
    out.push_str("{\n");
    out.push_str("\tadmin off\n");
    if http_port != 80 {
        out.push_str(&format!("\thttp_port {}\n", http_port));
    }
    if https_port != 443 {
        out.push_str(&format!("\thttps_port {}\n", https_port));
    }
    out.push_str("}\n\n");

    if domains.is_empty() {
        out.push_str(":65535 {\n");
        out.push_str("\trespond \"LocalDomain placeholder\" 200\n");
        out.push_str("}\n");
        return out;
    }

    for domain in domains {
        let wants_https = domain.protocol == "https" || domain.protocol == "both";
        let wants_http = domain.protocol == "http" || domain.protocol == "both";

        // HTTPS block
        if wants_https {
            if let (Some(cert), Some(key)) = (&domain.cert_path, &domain.key_path) {
                if https_port != 443 {
                    out.push_str(&format!("https://{}:{} {{\n", domain.name, https_port));
                } else {
                    out.push_str(&format!("https://{} {{\n", domain.name));
                }
                out.push_str(&format!("\ttls {} {}\n", cert, key));
                out.push_str(&format!(
                    "\treverse_proxy {}:{} {{\n\t\theader_up Host {{host}}\n\t}}\n",
                    domain.target_host, domain.target_port
                ));
                out.push_str("\tbind 127.0.0.1\n");
                if domain.access_log {
                    append_log_directive(&mut out, &domain.name);
                }
                out.push_str("}\n\n");
            }
        }

        // HTTP block
        if wants_http {
            if http_port != 80 {
                out.push_str(&format!("http://{}:{} {{\n", domain.name, http_port));
            } else {
                out.push_str(&format!("http://{} {{\n", domain.name));
            }
            out.push_str(&format!(
                "\treverse_proxy {}:{} {{\n\t\theader_up Host {{host}}\n\t}}\n",
                domain.target_host, domain.target_port
            ));
            out.push_str("\tbind 127.0.0.1\n");
            if domain.access_log {
                append_log_directive(&mut out, &domain.name);
            }
            out.push_str("}\n\n");
        }
    }

    out
}

fn append_log_directive(out: &mut String, domain_name: &str) {
    out.push_str("\tlog {\n");
    let log_path = std::path::Path::new(crate::paths::LOGS_DIR)
        .join(format!("{}.access.log", domain_name))
        .to_string_lossy()
        .to_string();
    out.push_str(&format!("\t\toutput file {} {{\n", log_path));
    out.push_str("\t\t\troll_size 10mb\n");
    out.push_str("\t\t\troll_keep 1\n");
    out.push_str("\t\t}\n");
    out.push_str("\t\tformat json\n");
    out.push_str("\t}\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_caddyfile() {
        let result = build_caddyfile(&[], 8080, 8443);
        assert!(result.contains("admin off"));
        assert!(result.contains("http_port 8080"));
        assert!(result.contains("https_port 8443"));
        assert!(result.contains(":65535"));
    }

    #[test]
    fn test_http_only_domain() {
        let domains = vec![CaddyDomainConfig {
            name: "project.test".to_string(),
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
            access_log: false,
        }];
        let result = build_caddyfile(&domains, 8080, 8443);
        assert!(result.contains("http://project.test:8080"));
        assert!(result.contains("reverse_proxy 127.0.0.1:3000"));
        assert!(result.contains("header_up Host {host}"));
        assert!(!result.contains("https://"));
    }

    #[test]
    fn test_https_only_domain() {
        let domains = vec![CaddyDomainConfig {
            name: "secure.test".to_string(),
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            protocol: "https".to_string(),
            cert_path: Some("/var/lib/localdomain/certs/secure.test.crt".to_string()),
            key_path: Some("/var/lib/localdomain/certs/secure.test.key".to_string()),
            access_log: false,
        }];
        let result = build_caddyfile(&domains, 8080, 8443);
        assert!(result.contains("https://secure.test:8443"));
        assert!(result.contains("tls"));
        assert!(!result.contains("http://secure.test"));
    }

    #[test]
    fn test_both_protocol_domain() {
        let domains = vec![CaddyDomainConfig {
            name: "both.test".to_string(),
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            protocol: "both".to_string(),
            cert_path: Some("/var/lib/localdomain/certs/both.test.crt".to_string()),
            key_path: Some("/var/lib/localdomain/certs/both.test.key".to_string()),
            access_log: false,
        }];
        let result = build_caddyfile(&domains, 8080, 8443);
        assert!(result.contains("https://both.test:8443"));
        assert!(result.contains("http://both.test:8080"));
        assert!(result.contains("tls"));
    }

    #[test]
    fn test_access_log_directive() {
        let domains = vec![CaddyDomainConfig {
            name: "logged.test".to_string(),
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
            access_log: true,
        }];
        let result = build_caddyfile(&domains, 80, 443);
        assert!(result.contains("log {"));
        let expected_log_path = std::path::Path::new(crate::paths::LOGS_DIR)
            .join("logged.test.access.log")
            .to_string_lossy()
            .to_string();
        assert!(result.contains(&format!("output file {}", expected_log_path)));
        assert!(result.contains("format json"));
    }

    #[test]
    fn test_no_access_log_by_default() {
        let domains = vec![CaddyDomainConfig {
            name: "nolog.test".to_string(),
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            protocol: "http".to_string(),
            cert_path: None,
            key_path: None,
            access_log: false,
        }];
        let result = build_caddyfile(&domains, 80, 443);
        assert!(!result.contains("log {"));
    }

    #[test]
    fn test_standard_ports() {
        let domains = vec![CaddyDomainConfig {
            name: "project.test".to_string(),
            target_host: "127.0.0.1".to_string(),
            target_port: 3000,
            protocol: "both".to_string(),
            cert_path: Some("/certs/project.test.crt".to_string()),
            key_path: Some("/certs/project.test.key".to_string()),
            access_log: false,
        }];
        let result = build_caddyfile(&domains, 80, 443);
        assert!(result.contains("http://project.test {"));
        assert!(result.contains("https://project.test {"));
        assert!(!result.contains("http_port"));
        assert!(!result.contains("https_port"));
    }
}
