/// Platform-specific paths for all daemon data, binaries, and IPC endpoints.

#[cfg(target_os = "macos")]
pub const CERTS_DIR: &str = "/var/lib/localdomain/certs";
#[cfg(target_os = "macos")]
pub const CADDY_DIR: &str = "/var/lib/localdomain/caddy";
#[cfg(target_os = "macos")]
pub const LOGS_DIR: &str = "/var/lib/localdomain/logs";
#[cfg(target_os = "macos")]
pub const CADDY_BINARY: &str = "/usr/local/bin/caddy";
#[cfg(target_os = "macos")]
pub const CA_CERT: &str = "/var/lib/localdomain/certs/localdomain-ca.crt";
#[cfg(target_os = "macos")]
pub const CA_KEY: &str = "/var/lib/localdomain/certs/localdomain-ca.key";
#[cfg(target_os = "macos")]
pub const CADDYFILE: &str = "/var/lib/localdomain/caddy/Caddyfile";
#[cfg(target_os = "macos")]
pub const CADDY_PID: &str = "/var/lib/localdomain/caddy/caddy.pid";
#[cfg(target_os = "macos")]
pub const HOSTS_FILE: &str = "/etc/hosts";
#[cfg(target_os = "macos")]
pub const SOCKET_PATH: &str = "/var/run/localdomain.sock";

#[cfg(target_os = "linux")]
pub const CERTS_DIR: &str = "/var/lib/localdomain/certs";
#[cfg(target_os = "linux")]
pub const CADDY_DIR: &str = "/var/lib/localdomain/caddy";
#[cfg(target_os = "linux")]
pub const LOGS_DIR: &str = "/var/lib/localdomain/logs";
#[cfg(target_os = "linux")]
pub const CADDY_BINARY: &str = "/usr/local/bin/caddy";
#[cfg(target_os = "linux")]
pub const CA_CERT: &str = "/var/lib/localdomain/certs/localdomain-ca.crt";
#[cfg(target_os = "linux")]
pub const CA_KEY: &str = "/var/lib/localdomain/certs/localdomain-ca.key";
#[cfg(target_os = "linux")]
pub const CADDYFILE: &str = "/var/lib/localdomain/caddy/Caddyfile";
#[cfg(target_os = "linux")]
pub const CADDY_PID: &str = "/var/lib/localdomain/caddy/caddy.pid";
#[cfg(target_os = "linux")]
pub const HOSTS_FILE: &str = "/etc/hosts";
#[cfg(target_os = "linux")]
pub const SOCKET_PATH: &str = "/var/run/localdomain.sock";

#[cfg(target_os = "windows")]
pub const DATA_ROOT: &str = "C:\\ProgramData\\LocalDomain";
#[cfg(target_os = "windows")]
pub const CERTS_DIR: &str = "C:\\ProgramData\\LocalDomain\\certs";
#[cfg(target_os = "windows")]
pub const CADDY_DIR: &str = "C:\\ProgramData\\LocalDomain\\caddy";
#[cfg(target_os = "windows")]
pub const LOGS_DIR: &str = "C:\\ProgramData\\LocalDomain\\logs";
#[cfg(target_os = "windows")]
pub const CADDY_BINARY: &str = "C:\\ProgramData\\LocalDomain\\bin\\caddy.exe";
#[cfg(target_os = "windows")]
pub const CA_CERT: &str = "C:\\ProgramData\\LocalDomain\\certs\\localdomain-ca.crt";
#[cfg(target_os = "windows")]
pub const CA_KEY: &str = "C:\\ProgramData\\LocalDomain\\certs\\localdomain-ca.key";
#[cfg(target_os = "windows")]
pub const CADDYFILE: &str = "C:\\ProgramData\\LocalDomain\\caddy\\Caddyfile";
#[cfg(target_os = "windows")]
pub const CADDY_PID: &str = "C:\\ProgramData\\LocalDomain\\caddy\\caddy.pid";
#[cfg(target_os = "windows")]
pub const HOSTS_FILE: &str = "C:\\Windows\\System32\\drivers\\etc\\hosts";
#[cfg(target_os = "windows")]
pub const PIPE_NAME: &str = r"\\.\pipe\localdomain";

// Tunnel paths

#[cfg(target_os = "macos")]
pub const CLOUDFLARED_BINARY: &str = "/usr/local/bin/cloudflared";
#[cfg(target_os = "macos")]
pub const TUNNEL_DIR: &str = "/var/lib/localdomain/tunnels";

#[cfg(target_os = "linux")]
pub const CLOUDFLARED_BINARY: &str = "/usr/local/bin/cloudflared";
#[cfg(target_os = "linux")]
pub const TUNNEL_DIR: &str = "/var/lib/localdomain/tunnels";

#[cfg(target_os = "windows")]
pub const CLOUDFLARED_BINARY: &str = "C:\\ProgramData\\LocalDomain\\bin\\cloudflared.exe";
#[cfg(target_os = "windows")]
pub const TUNNEL_DIR: &str = "C:\\ProgramData\\LocalDomain\\tunnels";
