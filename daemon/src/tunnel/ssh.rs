use anyhow::{Context, Result};
use localdomain_shared::silent_cmd;
use std::process::Stdio;
use tracing::info;

/// Start an SSH reverse tunnel.
/// Returns (public_url, pid).
pub fn start_ssh_tunnel(
    domain: &str,
    local_port: u16,
    host: &str,
    port: u16,
    user: &str,
    key: &str,
    remote_port: u16,
) -> Result<(String, u32)> {
    let mut cmd = silent_cmd("ssh");
    cmd.args([
        "-N",
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "ExitOnForwardFailure=yes",
        "-p",
        &port.to_string(),
        "-R",
        &format!("{}:localhost:{}", remote_port, local_port),
    ]);

    if !key.is_empty() {
        cmd.args(["-i", key]);
    }

    cmd.arg(format!("{}@{}", user, host));
    cmd.stdout(Stdio::null()).stderr(Stdio::null());

    let child = cmd.spawn().context("Failed to start SSH tunnel")?;
    let pid = child.id();

    let public_url = format!("http://{}:{}", host, remote_port);

    info!(
        "SSH tunnel spawned (PID {}) for {} -> {}@{}:{} (remote port {})",
        pid, domain, user, host, port, remote_port
    );

    // Give SSH a moment to establish the connection
    std::thread::sleep(std::time::Duration::from_secs(2));

    Ok((public_url, pid))
}
