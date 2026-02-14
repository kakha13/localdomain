use anyhow::{Context, Result};
use localdomain_shared::protocol::{JsonRpcRequest, JsonRpcResponse};
use std::io::{BufRead, BufReader, Write};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::paths;

pub struct DaemonClient {
    next_id: AtomicU64,
}

#[cfg(unix)]
fn connect_to_daemon() -> Result<std::os::unix::net::UnixStream> {
    let stream = std::os::unix::net::UnixStream::connect(paths::SOCKET_PATH)
        .context("Failed to connect to daemon")?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(60)))
        .ok();
    Ok(stream)
}

#[cfg(windows)]
fn connect_to_daemon() -> Result<std::fs::File> {
    use std::fs::OpenOptions;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(paths::PIPE_NAME)
        .context("Failed to connect to daemon")?;
    Ok(file)
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let id = self.next_id();
        let request = JsonRpcRequest::new(method, params, id);
        let request_json = serde_json::to_string(&request)?;

        let mut stream = connect_to_daemon()?;

        writeln!(stream, "{}", request_json)?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;

        let response: JsonRpcResponse =
            serde_json::from_str(&response_line).context("Failed to parse daemon response")?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("Daemon error: {}", error.message));
        }

        Ok(response.result.unwrap_or(serde_json::Value::Null))
    }

    pub fn ping(&self) -> Result<bool> {
        let result = self.call("ping", serde_json::Value::Null)?;
        Ok(result.as_str() == Some("pong"))
    }

    pub fn status(&self) -> Result<localdomain_shared::protocol::StatusResult> {
        let result = self.call("status", serde_json::Value::Null)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn sync_hosts(&self, entries: Vec<localdomain_shared::domain::HostsEntry>) -> Result<()> {
        let params =
            serde_json::to_value(localdomain_shared::protocol::SyncHostsParams { entries })?;
        self.call("sync_hosts", params)?;
        Ok(())
    }

    pub fn sync_caddy_config(
        &self,
        domains: Vec<localdomain_shared::domain::CaddyDomainConfig>,
        http_port: u16,
        https_port: u16,
    ) -> Result<()> {
        let params = serde_json::to_value(localdomain_shared::protocol::SyncCaddyConfigParams {
            domains,
            http_port,
            https_port,
        })?;
        self.call("sync_caddy_config", params)?;
        Ok(())
    }

    pub fn start_caddy(&self) -> Result<()> {
        self.call("start_caddy", serde_json::Value::Null)?;
        Ok(())
    }

    pub fn stop_caddy(&self) -> Result<()> {
        self.call("stop_caddy", serde_json::Value::Null)?;
        Ok(())
    }

    pub fn generate_ca(&self) -> Result<()> {
        self.call("generate_ca", serde_json::Value::Null)?;
        Ok(())
    }

    pub fn install_ca_trust(&self) -> Result<()> {
        self.call("install_ca_trust", serde_json::Value::Null)?;
        Ok(())
    }

    pub fn generate_cert(
        &self,
        domain: &str,
    ) -> Result<localdomain_shared::protocol::GenerateCertResult> {
        let params = serde_json::to_value(localdomain_shared::protocol::GenerateCertParams {
            domain: domain.to_string(),
        })?;
        let result = self.call("generate_cert", params)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn get_access_log(
        &self,
        domain: &str,
        limit: Option<u64>,
    ) -> Result<Vec<localdomain_shared::protocol::AccessLogEntry>> {
        let params = serde_json::to_value(localdomain_shared::protocol::GetAccessLogParams {
            domain: domain.to_string(),
            limit,
        })?;
        let result = self.call("get_access_log", params)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn clear_access_log(&self, domain: &str) -> Result<()> {
        let params = serde_json::to_value(localdomain_shared::protocol::ClearAccessLogParams {
            domain: domain.to_string(),
        })?;
        self.call("clear_access_log", params)?;
        Ok(())
    }

    pub fn start_tunnel(
        &self,
        params: localdomain_shared::protocol::StartTunnelParams,
    ) -> Result<localdomain_shared::protocol::StartTunnelResult> {
        let params = serde_json::to_value(params)?;
        let result = self.call("start_tunnel", params)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn stop_tunnel(&self, domain: &str) -> Result<()> {
        let params = serde_json::to_value(localdomain_shared::protocol::StopTunnelParams {
            domain: domain.to_string(),
        })?;
        self.call("stop_tunnel", params)?;
        Ok(())
    }

    pub fn tunnel_status(
        &self,
        domain: &str,
    ) -> Result<localdomain_shared::protocol::TunnelStatusResult> {
        let params = serde_json::to_value(localdomain_shared::protocol::TunnelStatusParams {
            domain: domain.to_string(),
        })?;
        let result = self.call("tunnel_status", params)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn list_tunnels(&self) -> Result<localdomain_shared::protocol::ListTunnelsResult> {
        let result = self.call("list_tunnels", serde_json::Value::Null)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn ensure_cloudflared(
        &self,
    ) -> Result<localdomain_shared::protocol::EnsureCloudflaredResult> {
        let result = self.call("ensure_cloudflared", serde_json::Value::Null)?;
        Ok(serde_json::from_value(result)?)
    }

    #[allow(dead_code)]
    pub fn stop_all_tunnels(&self) -> Result<()> {
        self.call("stop_all_tunnels", serde_json::Value::Null)?;
        Ok(())
    }

    pub fn start_apache(&self, xampp_path: &str) -> Result<()> {
        let params =
            serde_json::to_value(localdomain_shared::protocol::XamppActionParams {
                xampp_path: xampp_path.to_string(),
            })?;
        self.call("start_apache", params)?;
        Ok(())
    }

    pub fn stop_apache(&self, xampp_path: &str) -> Result<()> {
        let params =
            serde_json::to_value(localdomain_shared::protocol::XamppActionParams {
                xampp_path: xampp_path.to_string(),
            })?;
        self.call("stop_apache", params)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn sync_xampp_config(
        &self,
        vhosts: Vec<localdomain_shared::domain::XamppVhostConfig>,
        xampp_path: &str,
    ) -> Result<()> {
        let params = serde_json::to_value(localdomain_shared::protocol::SyncXamppConfigParams {
            vhosts,
            xampp_path: xampp_path.to_string(),
        })?;
        self.call("sync_xampp_config", params)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn detect_xampp(&self) -> Result<localdomain_shared::protocol::DetectXamppResult> {
        let result = self.call("detect_xampp", serde_json::Value::Null)?;
        Ok(serde_json::from_value(result)?)
    }

    pub fn is_daemon_running(&self) -> bool {
        self.ping().is_ok()
    }
}
