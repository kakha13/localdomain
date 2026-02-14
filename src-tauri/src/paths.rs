/// Platform-specific paths for the Tauri app side.

#[cfg(target_os = "macos")]
pub const CA_CERT: &str = "/var/lib/localdomain/certs/localdomain-ca.crt";
#[cfg(target_os = "macos")]
pub const SOCKET_PATH: &str = "/var/run/localdomain.sock";

#[cfg(target_os = "linux")]
pub const CA_CERT: &str = "/var/lib/localdomain/certs/localdomain-ca.crt";
#[cfg(target_os = "linux")]
pub const SOCKET_PATH: &str = "/var/run/localdomain.sock";

#[cfg(target_os = "windows")]
pub const CA_CERT: &str = "C:\\ProgramData\\LocalDomain\\certs\\localdomain-ca.crt";
#[cfg(target_os = "windows")]
pub const PIPE_NAME: &str = r"\\.\pipe\localdomain";
