use crate::db::models;
use crate::error::AppError;
use crate::state::AppState;
use crate::tray;
use serde::Serialize;
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub daemon_running: bool,
    pub caddy_running: bool,
    pub ca_installed: bool,
    pub ca_trusted: bool,
    pub xampp_running: bool,
}

#[tauri::command]
pub fn get_service_status(state: State<AppState>) -> Result<ServiceStatus, AppError> {
    let client = state.daemon_client.lock().unwrap();
    let xampp_running = check_xampp_running(state.inner());
    match client.status() {
        Ok(status) => Ok(ServiceStatus {
            daemon_running: status.daemon_running,
            caddy_running: status.caddy_running,
            ca_installed: status.ca_installed,
            ca_trusted: status.ca_trusted,
            xampp_running,
        }),
        Err(_) => Ok(ServiceStatus {
            daemon_running: false,
            caddy_running: false,
            ca_installed: false,
            ca_trusted: false,
            xampp_running,
        }),
    }
}

/// Get the configured or default XAMPP path.
fn get_xampp_path(state: &AppState) -> String {
    let xampp_path = {
        let conn = state.db.lock().unwrap();
        models::get_setting(&conn, "xampp_path")
            .ok()
            .flatten()
    };
    match xampp_path.as_deref() {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => {
            #[cfg(target_os = "macos")]
            let default = "/Applications/XAMPP/xamppfiles";
            #[cfg(target_os = "linux")]
            let default = "/opt/lampp";
            #[cfg(target_os = "windows")]
            let default = "C:\\xampp";
            default.to_string()
        }
    }
}

/// Check if XAMPP Apache is running directly (no daemon needed).
fn check_xampp_running(state: &AppState) -> bool {
    let path = get_xampp_path(state);
    is_apache_running(&path)
}

#[cfg(unix)]
fn is_apache_running(xampp_path: &str) -> bool {
    let httpd = format!("{}/bin/httpd", xampp_path);
    std::process::Command::new("pgrep")
        .arg("-f")
        .arg(&httpd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn is_apache_running(_xampp_path: &str) -> bool {
    localdomain_shared::silent_cmd("tasklist")
        .args(["/FI", "IMAGENAME eq httpd.exe"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains("httpd.exe")
        })
        .unwrap_or(false)
}

#[tauri::command]
pub fn start_service(app: AppHandle, state: State<AppState>) -> Result<(), AppError> {
    let client = state.daemon_client.lock().unwrap();
    client
        .start_caddy()
        .map_err(|e| AppError::Daemon(e.to_string()))?;
    drop(client);
    tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub fn stop_service(app: AppHandle, state: State<AppState>) -> Result<(), AppError> {
    let client = state.daemon_client.lock().unwrap();
    client
        .stop_caddy()
        .map_err(|e| AppError::Daemon(e.to_string()))?;
    drop(client);
    tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub fn start_apache(app: AppHandle, state: State<AppState>) -> Result<(), AppError> {
    let path = get_xampp_path(state.inner());
    let client = state.daemon_client.lock().unwrap();
    client
        .start_apache(&path)
        .map_err(|e| AppError::Daemon(e.to_string()))?;
    drop(client);
    tray::refresh_tray_menu(&app);
    Ok(())
}

#[tauri::command]
pub fn stop_apache(app: AppHandle, state: State<AppState>) -> Result<(), AppError> {
    let path = get_xampp_path(state.inner());
    let client = state.daemon_client.lock().unwrap();
    client
        .stop_apache(&path)
        .map_err(|e| AppError::Daemon(e.to_string()))?;
    drop(client);
    tray::refresh_tray_menu(&app);
    Ok(())
}

/// Find the daemon binary. In dev mode it's in target/debug/, in production it's bundled.
fn find_daemon_binary() -> Result<std::path::PathBuf, AppError> {
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap();

    #[cfg(unix)]
    let binary_name = "localdomain-daemon";
    #[cfg(windows)]
    let binary_name = "localdomain-daemon.exe";

    // Check next to current exe (target/debug/)
    let dev_path = exe_dir.join(binary_name);
    if dev_path.exists() {
        return Ok(dev_path);
    }

    // Check in bundled resources
    #[cfg(target_os = "macos")]
    {
        // Tauri bundles "../resources/*" into Contents/Resources/_up_/resources/
        let bundled_path = exe_dir.join("../Resources/_up_/resources").join(binary_name);
        if bundled_path.exists() {
            return Ok(bundled_path);
        }
        // Fallback: directly in Contents/Resources/
        let resources_path = exe_dir.join("../Resources").join(binary_name);
        if resources_path.exists() {
            return Ok(resources_path);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Tauri on Linux places resources next to the binary
        let bundled_path = exe_dir.join("resources").join(binary_name);
        if bundled_path.exists() {
            return Ok(bundled_path);
        }
        // Tauri NSIS-style _up_/resources/ structure
        let nsis_path = exe_dir.join("_up_").join("resources").join(binary_name);
        if nsis_path.exists() {
            return Ok(nsis_path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Standard resources path (WiX/MSI installs)
        let bundled_path = exe_dir.join("resources").join(binary_name);
        if bundled_path.exists() {
            return Ok(bundled_path);
        }
        // Tauri NSIS installer uses _up_/resources/ structure
        let nsis_path = exe_dir.join("_up_").join("resources").join(binary_name);
        if nsis_path.exists() {
            return Ok(nsis_path);
        }
    }

    Err(AppError::Daemon(format!(
        "Daemon binary not found. Looked at:\n  {}\n  {}\nBuild it first: cargo build -p localdomain-daemon",
        dev_path.display(),
        exe_dir.join("../Resources/_up_/resources").join(binary_name).display(),
    )))
}

// ---- macOS: launchd-based install/uninstall ----

#[cfg(target_os = "macos")]
fn find_plist() -> Result<std::path::PathBuf, AppError> {
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap();

    let dev_path = exe_dir
        .join("../../resources/com.localdomain.daemon.plist")
        .canonicalize()
        .ok();
    if let Some(ref p) = dev_path {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    // Tauri bundles "../resources/*" into Contents/Resources/_up_/resources/
    let bundled_path = exe_dir.join("../Resources/_up_/resources/com.localdomain.daemon.plist");
    if bundled_path.exists() {
        return Ok(bundled_path);
    }
    // Fallback: directly in Contents/Resources/
    let resources_path = exe_dir.join("../Resources/com.localdomain.daemon.plist");
    if resources_path.exists() {
        return Ok(resources_path);
    }

    Err(AppError::Daemon(
        "Plist file not found. Ensure resources/com.localdomain.daemon.plist exists.".to_string(),
    ))
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn install_daemon() -> Result<(), AppError> {
    let daemon_binary = find_daemon_binary()?;
    let plist_src = find_plist()?;

    let daemon_path = daemon_binary.display().to_string().replace('\'', "'\\''");
    let plist_path = plist_src.display().to_string().replace('\'', "'\\''");

    let install_script = format!(
        r#"do shell script "
mkdir -p /usr/local/bin && \
cp '{daemon_path}' /usr/local/bin/localdomain-daemon && \
chmod 755 /usr/local/bin/localdomain-daemon && \
mkdir -p /var/lib/localdomain/certs && \
mkdir -p /var/lib/localdomain/caddy && \
cp '{plist_path}' /Library/LaunchDaemons/com.localdomain.daemon.plist && \
launchctl bootout system/com.localdomain.daemon 2>/dev/null; \
launchctl bootstrap system /Library/LaunchDaemons/com.localdomain.daemon.plist
" with administrator privileges"#
    );

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&install_script)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::Daemon(format!(
            "Failed to install daemon.\nstderr: {}\nstdout: {}",
            stderr.trim(),
            stdout.trim()
        )));
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}

#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn uninstall_daemon() -> Result<(), AppError> {
    let uninstall_script = r#"do shell script "
launchctl bootout system/com.localdomain.daemon 2>/dev/null; \
rm -f /Library/LaunchDaemons/com.localdomain.daemon.plist; \
rm -f /usr/local/bin/localdomain-daemon; \
rm -f /var/run/localdomain.sock
" with administrator privileges"#;

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(uninstall_script)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Daemon(format!(
            "Failed to uninstall daemon: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

// ---- Linux: systemd-based install/uninstall ----

#[cfg(target_os = "linux")]
fn find_service_file() -> Result<std::path::PathBuf, AppError> {
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap();

    // Dev mode: resources/ relative to project root
    let dev_path = exe_dir
        .join("../../resources/localdomain-daemon.service")
        .canonicalize()
        .ok();
    if let Some(ref p) = dev_path {
        if p.exists() {
            return Ok(p.clone());
        }
    }

    // Bundled: next to binary
    let bundled_path = exe_dir.join("resources/localdomain-daemon.service");
    if bundled_path.exists() {
        return Ok(bundled_path);
    }

    Err(AppError::Daemon(
        "Service file not found. Ensure resources/localdomain-daemon.service exists.".to_string(),
    ))
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn install_daemon() -> Result<(), AppError> {
    let daemon_binary = find_daemon_binary()?;
    let service_src = find_service_file()?;

    let daemon_path = daemon_binary.display().to_string().replace('\'', "'\\''");
    let service_path = service_src.display().to_string().replace('\'', "'\\''");

    let install_script = format!(
        r#"set -e
mkdir -p /usr/local/bin
cp '{daemon_path}' /usr/local/bin/localdomain-daemon
chmod 755 /usr/local/bin/localdomain-daemon
mkdir -p /var/lib/localdomain/certs
mkdir -p /var/lib/localdomain/caddy
mkdir -p /var/lib/localdomain/logs

# Download Caddy if not already installed
if [ ! -f /usr/local/bin/caddy ]; then
    CADDY_VERSION="2.8.4"
    ARCH=$(uname -m)
    if [ "$ARCH" = "aarch64" ]; then
        PLATFORM="linux_arm64"
    elif [ "$ARCH" = "x86_64" ]; then
        PLATFORM="linux_amd64"
    else
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
    fi
    DOWNLOAD_URL="https://github.com/caddyserver/caddy/releases/download/v${{CADDY_VERSION}}/caddy_${{CADDY_VERSION}}_${{PLATFORM}}.tar.gz"
    curl -fSL "$DOWNLOAD_URL" -o /tmp/caddy.tar.gz
    tar -xzf /tmp/caddy.tar.gz -C /tmp caddy
    mv /tmp/caddy /usr/local/bin/caddy
    chmod 755 /usr/local/bin/caddy
    rm -f /tmp/caddy.tar.gz
fi

cp '{service_path}' /etc/systemd/system/localdomain-daemon.service
systemctl daemon-reload
systemctl enable --now localdomain-daemon"#,
        daemon_path = daemon_path,
        service_path = service_path
    );

    let output = std::process::Command::new("pkexec")
        .args(["bash", "-c", &install_script])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(AppError::Daemon(format!(
            "Failed to install daemon.\nstderr: {}\nstdout: {}",
            stderr.trim(),
            stdout.trim()
        )));
    }

    std::thread::sleep(std::time::Duration::from_secs(1));
    Ok(())
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub async fn uninstall_daemon() -> Result<(), AppError> {
    let uninstall_script = "\
        systemctl stop localdomain-daemon 2>/dev/null; \
        systemctl disable localdomain-daemon 2>/dev/null; \
        rm -f /etc/systemd/system/localdomain-daemon.service && \
        systemctl daemon-reload; \
        rm -f /usr/local/bin/localdomain-daemon; \
        rm -f /var/run/localdomain.sock";

    let output = std::process::Command::new("pkexec")
        .args(["bash", "-c", uninstall_script])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Daemon(format!(
            "Failed to uninstall daemon: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

// ---- Windows: Windows Service-based install/uninstall ----

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn install_daemon() -> Result<(), AppError> {
    let daemon_binary = find_daemon_binary()?;
    let src = daemon_binary.display().to_string();
    let dest = r"C:\ProgramData\LocalDomain\bin\localdomain-daemon.exe";
    let log_path = r"C:\ProgramData\LocalDomain\logs\install.log";

    // PowerShell script: create directories, download Caddy, copy binary, create and start Windows Service
    let ps_script = format!(
        r#"$ErrorActionPreference = 'Stop'
$logFile = '{log_path}'
$logDir = Split-Path -Parent $logFile
try {{
    New-Item -ItemType Directory -Force -Path $logDir | Out-Null
    "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - Starting daemon install" | Out-File $logFile -Append

    New-Item -ItemType Directory -Force -Path 'C:\ProgramData\LocalDomain\bin' | Out-Null
    New-Item -ItemType Directory -Force -Path 'C:\ProgramData\LocalDomain\certs' | Out-Null
    New-Item -ItemType Directory -Force -Path 'C:\ProgramData\LocalDomain\caddy' | Out-Null
    New-Item -ItemType Directory -Force -Path 'C:\ProgramData\LocalDomain\logs' | Out-Null

    # Download Caddy if not present
    $CaddyPath = 'C:\ProgramData\LocalDomain\bin\caddy.exe'
    if (-not (Test-Path $CaddyPath)) {{
        $arch = if ([Environment]::Is64BitOperatingSystem) {{ 'amd64' }} else {{ '386' }}
        $url = "https://caddyserver.com/api/download?os=windows&arch=$arch"
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $url -OutFile $CaddyPath -UseBasicParsing
    }}

    Copy-Item -Path '{src}' -Destination '{dest}' -Force
    if (-not (Test-Path '{dest}')) {{
        throw "Daemon binary copy failed: destination not found"
    }}

    $svc = Get-Service -Name 'localdomain-daemon' -ErrorAction SilentlyContinue
    if ($svc) {{
        Stop-Service -Name 'localdomain-daemon' -Force -ErrorAction SilentlyContinue
        sc.exe delete 'localdomain-daemon' | Out-Null
        Start-Sleep -Seconds 1
    }}

    sc.exe create 'localdomain-daemon' binPath='{dest}' start=auto DisplayName='LocalDomain Daemon'
    if ($LASTEXITCODE -ne 0) {{ throw "sc.exe create failed with exit code $LASTEXITCODE" }}

    sc.exe start 'localdomain-daemon'
    $startExitCode = $LASTEXITCODE

    # sc.exe start can return 1053 while the service is still transitioning.
    # Poll for up to 15s and only fail if it never reaches Running.
    $running = $false
    for ($i = 0; $i -lt 15; $i++) {{
        Start-Sleep -Seconds 1
        $svc = Get-Service -Name 'localdomain-daemon' -ErrorAction SilentlyContinue
        if ($svc -and $svc.Status -eq 'Running') {{
            $running = $true
            break
        }}
    }}
    if (-not $running) {{
        $query = (sc.exe query 'localdomain-daemon' | Out-String)
        throw "Service failed to reach Running state (sc.exe start exit code: $startExitCode). Query: $query"
    }}

    "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - Installation successful" | Out-File $logFile -Append
}} catch {{
    $errMsg = $_.Exception.Message
    try {{
        New-Item -ItemType Directory -Force -Path $logDir | Out-Null
        "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - Installation failed: $errMsg" | Out-File $logFile -Append
    }} catch {{}}
    Write-Output "INSTALL_ERROR: $errMsg"
    exit 1
}}"#,
        src = src.replace('\'', "''"),
        dest = dest,
        log_path = log_path
    );

    // Write script to temp file to avoid command-line escaping issues
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("localdomain-install.ps1");
    std::fs::write(&script_path, &ps_script)?;

    // Run elevated via Start-Process -Verb RunAs with exit code propagation
    let script_path_str = script_path.display().to_string().replace('\'', "''");
    let output = localdomain_shared::silent_cmd("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command",
            &format!(
                "$p = Start-Process powershell -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','{script_path_str}') -Verb RunAs -Wait -PassThru; exit $p.ExitCode"
            ),
        ])
        .output()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        // Read install log for details about what failed
        let log = std::fs::read_to_string(log_path).unwrap_or_default();
        let daemon_log = std::fs::read_to_string(r"C:\ProgramData\LocalDomain\logs\daemon.log")
            .unwrap_or_default();
        let daemon_log_tail = daemon_log
            .lines()
            .rev()
            .take(20)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Daemon(format!(
            "Failed to install daemon.\nLog: {}\nstdout: {}\nstderr: {}\ndaemon.log tail:\n{}",
            log.lines().last().unwrap_or("no log"),
            stdout.trim(),
            stderr.trim(),
            if daemon_log_tail.is_empty() {
                "<empty>"
            } else {
                &daemon_log_tail
            }
        )));
    }

    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn uninstall_daemon() -> Result<(), AppError> {
    let ps_script = r#"$ErrorActionPreference = 'Stop'
Stop-Service -Name 'localdomain-daemon' -Force -ErrorAction SilentlyContinue
sc.exe delete 'localdomain-daemon'
Remove-Item -Path 'C:\ProgramData\LocalDomain\bin\localdomain-daemon.exe' -Force -ErrorAction SilentlyContinue
"#;

    // Write script to temp file
    let temp_dir = std::env::temp_dir();
    let script_path = temp_dir.join("localdomain-uninstall.ps1");
    std::fs::write(&script_path, ps_script)?;

    let script_path_str = script_path.display().to_string().replace('\'', "''");
    let output = localdomain_shared::silent_cmd("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command",
            &format!(
                "$p = Start-Process powershell -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-File','{script_path_str}') -Verb RunAs -Wait -PassThru; exit $p.ExitCode"
            ),
        ])
        .output()?;

    let _ = std::fs::remove_file(&script_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Daemon(format!(
            "Failed to uninstall daemon: {}",
            stderr.trim()
        )));
    }

    Ok(())
}
