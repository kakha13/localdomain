use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
    pub id: u64,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: serde_json::Value, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: u64, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

// --- RPC parameter types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHostsParams {
    pub entries: Vec<super::domain::HostsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCaddyConfigParams {
    pub domains: Vec<super::domain::CaddyDomainConfig>,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_https_port")]
    pub https_port: u16,
}

fn default_http_port() -> u16 {
    80
}
fn default_https_port() -> u16 {
    443
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCertParams {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateCertResult {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResult {
    pub daemon_running: bool,
    pub caddy_running: bool,
    pub ca_installed: bool,
    #[serde(default)]
    pub ca_trusted: bool,
    #[serde(default)]
    pub xampp_running: bool,
}

// --- XAMPP types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XamppActionParams {
    pub xampp_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncXamppConfigParams {
    pub vhosts: Vec<super::domain::XamppVhostConfig>,
    pub xampp_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectXamppResult {
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLogEntry {
    pub timestamp: f64,
    pub method: String,
    pub uri: String,
    pub status: u16,
    pub duration: f64,
    pub size: u64,
    pub host: String,
    #[serde(default)]
    pub headers: serde_json::Value,
    #[serde(default)]
    pub resp_headers: serde_json::Value,
    #[serde(default)]
    pub remote_ip: String,
    #[serde(default)]
    pub proto: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccessLogParams {
    pub domain: String,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearAccessLogParams {
    pub domain: String,
}

// --- Tunnel types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TunnelType {
    QuickTunnel,
    NamedTunnel {
        token: String,
        subdomain: String,
        cloudflare_domain: String,
        /// Credentials JSON from ~/.cloudflared/<UUID>.json (for config-file mode)
        #[serde(default)]
        credentials_json: String,
        /// Tunnel UUID (for config-file mode)
        #[serde(default)]
        tunnel_uuid: String,
    },
    SshTunnel {
        host: String,
        #[serde(default = "default_ssh_port")]
        port: u16,
        user: String,
        #[serde(default)]
        key: String,
        remote_port: u16,
    },
}

fn default_ssh_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTunnelParams {
    pub domain: String,
    pub local_port: u16,
    pub tunnel_type: TunnelType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTunnelResult {
    pub public_url: String,
    pub tunnel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopTunnelParams {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatusParams {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStatusResult {
    pub active: bool,
    #[serde(default)]
    pub public_url: Option<String>,
    #[serde(default)]
    pub tunnel_type: Option<TunnelType>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub domain: String,
    pub public_url: String,
    pub tunnel_type: TunnelType,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTunnelsResult {
    pub tunnels: Vec<TunnelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsureCloudflaredResult {
    pub installed: bool,
    pub path: String,
    #[serde(default)]
    pub version: Option<String>,
}
