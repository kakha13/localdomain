use anyhow::Result;
use localdomain_shared::protocol::{
    ListTunnelsResult, StartTunnelParams, StartTunnelResult, StopTunnelParams, TunnelInfo,
    TunnelStatusParams, TunnelStatusResult, TunnelType,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::info;

use super::cloudflared;
use super::ssh;

pub struct TunnelProcess {
    pub domain: String,
    pub public_url: String,
    pub tunnel_type: TunnelType,
    pub pid: u32,
}

static TUNNELS: Lazy<Mutex<HashMap<String, TunnelProcess>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(unix)]
fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn is_process_alive(pid: u32) -> bool {
    localdomain_shared::silent_cmd("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.contains(&pid.to_string())
        })
        .unwrap_or(false)
}

#[cfg(unix)]
fn kill_process(pid: u32) {
    unsafe {
        libc::kill(pid as i32, libc::SIGTERM);
    }
}

#[cfg(windows)]
fn kill_process(pid: u32) {
    let _ = localdomain_shared::silent_cmd("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output();
}

pub fn start_tunnel(params: StartTunnelParams) -> Result<StartTunnelResult> {
    // Stop existing tunnel for this domain if any
    let _ = stop_tunnel(StopTunnelParams {
        domain: params.domain.clone(),
    });

    let (public_url, pid) = match &params.tunnel_type {
        TunnelType::QuickTunnel => {
            cloudflared::start_quick_tunnel(&params.domain, params.local_port)?
        }
        TunnelType::NamedTunnel {
            token,
            subdomain,
            cloudflare_domain,
            credentials_json,
            tunnel_uuid,
        } => cloudflared::start_named_tunnel(
            &params.domain,
            params.local_port,
            token,
            subdomain,
            cloudflare_domain,
            credentials_json,
            tunnel_uuid,
        )?,
        TunnelType::SshTunnel {
            host,
            port,
            user,
            key,
            remote_port,
        } => ssh::start_ssh_tunnel(
            &params.domain,
            params.local_port,
            host,
            *port,
            user,
            key,
            *remote_port,
        )?,
    };

    let tunnel_id = format!("tunnel-{}", uuid::Uuid::new_v4());

    let process = TunnelProcess {
        domain: params.domain.clone(),
        public_url: public_url.clone(),
        tunnel_type: params.tunnel_type,
        pid,
    };

    let mut tunnels = TUNNELS.lock().unwrap();
    tunnels.insert(params.domain.clone(), process);

    info!(
        "Tunnel started for {} -> {} (PID {})",
        params.domain, public_url, pid
    );

    Ok(StartTunnelResult {
        public_url,
        tunnel_id,
    })
}

pub fn stop_tunnel(params: StopTunnelParams) -> Result<()> {
    let mut tunnels = TUNNELS.lock().unwrap();
    if let Some(process) = tunnels.remove(&params.domain) {
        if is_process_alive(process.pid) {
            kill_process(process.pid);
            // Wait for process to exit (up to 3s)
            for _ in 0..12 {
                std::thread::sleep(std::time::Duration::from_millis(250));
                if !is_process_alive(process.pid) {
                    break;
                }
            }
        }
        info!("Tunnel stopped for {} (PID {})", params.domain, process.pid);
    }
    Ok(())
}

pub fn tunnel_status(params: TunnelStatusParams) -> Result<TunnelStatusResult> {
    let tunnels = TUNNELS.lock().unwrap();
    if let Some(process) = tunnels.get(&params.domain) {
        let alive = is_process_alive(process.pid);
        Ok(TunnelStatusResult {
            active: alive,
            public_url: if alive {
                Some(process.public_url.clone())
            } else {
                None
            },
            tunnel_type: if alive {
                Some(process.tunnel_type.clone())
            } else {
                None
            },
            error: if !alive {
                Some("Tunnel process is no longer running".to_string())
            } else {
                None
            },
        })
    } else {
        Ok(TunnelStatusResult {
            active: false,
            public_url: None,
            tunnel_type: None,
            error: None,
        })
    }
}

pub fn list_tunnels() -> Result<ListTunnelsResult> {
    let mut tunnels_lock = TUNNELS.lock().unwrap();
    // Clean up dead processes
    let dead_domains: Vec<String> = tunnels_lock
        .iter()
        .filter(|(_, p)| !is_process_alive(p.pid))
        .map(|(d, _)| d.clone())
        .collect();
    for d in &dead_domains {
        tunnels_lock.remove(d);
    }

    let tunnels = tunnels_lock
        .values()
        .map(|p| TunnelInfo {
            domain: p.domain.clone(),
            public_url: p.public_url.clone(),
            tunnel_type: p.tunnel_type.clone(),
            pid: p.pid,
        })
        .collect();

    Ok(ListTunnelsResult { tunnels })
}

pub fn stop_all_tunnels() -> Result<()> {
    let mut tunnels = TUNNELS.lock().unwrap();
    for (domain, process) in tunnels.drain() {
        if is_process_alive(process.pid) {
            kill_process(process.pid);
            info!("Stopped tunnel for {} (PID {})", domain, process.pid);
        }
    }
    Ok(())
}
