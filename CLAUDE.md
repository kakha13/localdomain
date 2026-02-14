# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development (all platforms)
cargo tauri dev              # Launch app with Vite hot-reload (port 1420)

# Checks (all platforms)
cargo check --workspace      # Type-check all Rust crates
cargo test --workspace       # Run all Rust tests (13 tests)
npx tsc --noEmit             # Type-check TypeScript

# Production build
npx tauri build --bundles dmg            # macOS — signed .dmg
npx tauri build                          # Windows — .msi + .nsis installer (targets: all)
npx tauri build --bundles deb            # Linux — .deb package

# Daemon — macOS (requires sudo)
cargo build -p localdomain-daemon --release
sudo cp target/release/localdomain-daemon /usr/local/bin/localdomain-daemon
sudo launchctl kickstart -k system/com.localdomain.daemon
# Or use: ./scripts/reload-daemon.sh

# Daemon — Linux (requires sudo)
cargo build -p localdomain-daemon --release
sudo cp target/release/localdomain-daemon /usr/local/bin/localdomain-daemon
sudo cp resources/localdomain-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload && sudo systemctl enable --now localdomain-daemon
# Or use: ./scripts/reload-daemon-linux.sh

# Daemon — Windows (requires Administrator)
cargo build -p localdomain-daemon --release
# Use: .\scripts\reload-daemon.ps1   (rebuilds + reinstalls service)
# Or:  .\scripts\install-daemon.ps1  (first-time install as Windows Service)
# Debug: target\release\localdomain-daemon.exe --console
```

## Architecture

```
React UI ──tauri invoke()──▶ Tauri App ──JSON-RPC 2.0──▶ Daemon (privileged)
                              │                               │
                           SQLite DB                    hosts file
                           (owns state)                 Caddy proxy
                                                        TLS certs (rcgen)

IPC transport:
  macOS/Linux: Unix socket  /var/run/localdomain.sock
  Windows:     Named pipe   \\.\pipe\localdomain

Daemon runs as:
  macOS:   launchd system daemon  (com.localdomain.daemon)
  Linux:   systemd service        (localdomain-daemon)
  Windows: Windows Service        (localdomain-daemon)
```

**Three Rust crates** in a Cargo workspace:
- **`src-tauri`** — Tauri v2 app process. Owns SQLite database, validates input, coordinates with daemon via `DaemonClient`. All Tauri commands in `src-tauri/src/commands/`.
- **`daemon`** — Privileged service. Manages hosts file, generates Caddyfile, generates TLS certs with rcgen, controls Caddy process. macOS: launchd + Unix socket. Linux: systemd + Unix socket. Windows: Windows Service + Named pipe.
- **`shared`** — Types and validation shared between app and daemon. Protocol structs (`AccessLogEntry`, `CaddyDomainConfig`, `HostsEntry`), domain validation, JSON-RPC message types.

**Key design principle**: The daemon is stateless. The app owns all state in SQLite and sends the complete configuration to the daemon on every sync via `sync_state_to_daemon()`.

## System Tray

The app has a native system tray icon (`src-tauri/src/tray.rs`) with a context menu showing daemon/caddy status, domain toggles, and quick actions (start/stop services, open window, quit). The tray requires `"tray-icon"` and `"image-png"` Tauri features.

**Key behaviors:**
- Closing the window hides to tray (via `on_window_event` intercepting `CloseRequested`), does NOT quit
- The tray menu is rebuilt after every mutation (domain toggle, service start/stop) — not on click, to avoid Windows `TrackPopupMenu` race
- `tray::refresh_tray_menu(app)` is called from both tray event handlers and Tauri commands to keep the menu in sync
- Tray mutations emit a `"state-changed"` event so the frontend re-fetches via `listen()` in `useDomains` and `useServiceStatus`
- The `TrayIconBuilder` uses `with_id("main-tray")` so `app.tray_by_id("main-tray")` can find it for menu updates

## Cross-Platform Paths

| Resource | macOS | Linux | Windows |
|---|---|---|---|
| Data root | `/var/lib/localdomain/` | `/var/lib/localdomain/` | `C:\ProgramData\LocalDomain\` |
| Certs | `/var/lib/localdomain/certs/` | `/var/lib/localdomain/certs/` | `C:\ProgramData\LocalDomain\certs\` |
| Caddy dir | `/var/lib/localdomain/caddy/` | `/var/lib/localdomain/caddy/` | `C:\ProgramData\LocalDomain\caddy\` |
| Caddy binary | `/usr/local/bin/caddy` | `/usr/local/bin/caddy` | `C:\ProgramData\LocalDomain\bin\caddy.exe` |
| Daemon binary | `/usr/local/bin/localdomain-daemon` | `/usr/local/bin/localdomain-daemon` | `C:\ProgramData\LocalDomain\bin\localdomain-daemon.exe` |
| Hosts file | `/etc/hosts` | `/etc/hosts` | `C:\Windows\System32\drivers\etc\hosts` |
| IPC endpoint | `/var/run/localdomain.sock` | `/var/run/localdomain.sock` | `\\.\pipe\localdomain` |
| SQLite DB | `~/.local/share/com.localdomain.app/localdomain.db` | `~/.local/share/com.localdomain.app/localdomain.db` | `%APPDATA%\com.localdomain.app\localdomain.db` |

Paths are centralized in `daemon/src/paths.rs` and `src-tauri/src/paths.rs` using `#[cfg(target_os)]`.

## Frontend Structure

React 19 + TypeScript + Vite. Views are swapped via a `View` union type (`"domains" | "settings" | "audit" | "inspect" | "about"`), not a router. State lives in custom hooks (`useDomains`, `useServiceStatus`, `useAccessLog`, `useAuditLog`, `useTheme`). API calls to Tauri are in `src/lib/api.ts` via `@tauri-apps/api/core` `invoke()`.

Both `useDomains` and `useServiceStatus` hooks listen for the `"state-changed"` Tauri event (emitted by tray actions) and auto-refresh when it fires, keeping the window and tray in sync.

## IPC Protocol

App-Daemon communication uses JSON-RPC 2.0. The `DaemonClient` (in `src-tauri/src/daemon_client/`) serializes requests and deserializes responses. Key methods: `sync_hosts`, `sync_caddy_config`, `generate_ca`, `install_ca_trust`, `generate_cert`, `start_caddy`, `stop_caddy`, `get_access_log`.

Transport: Unix socket on macOS/Linux, Named pipe on Windows. The daemon uses a generic `handle_connection<R: AsyncRead, W: AsyncWrite>` for platform-agnostic request handling. The app connects via `UnixStream` (macOS/Linux) or `std::fs::OpenOptions` on the named pipe (Windows).

## Database

SQLite via `dirs::data_dir()`. Three tables: `domains`, `audit_log`, `settings`. Migrations auto-run on startup from `src-tauri/src/db/migrations.rs`. All DB access through `src-tauri/src/db/models.rs`.

## Cross-Platform Patterns

```rust
#[cfg(unix)]                    // Unix socket, libc imports (macOS + Linux)
#[cfg(windows)]                 // Named pipe, Windows Service, taskkill
#[cfg(target_os = "macos")]     // launchd, `security` cert trust
#[cfg(target_os = "linux")]     // systemd, `update-ca-certificates`, `pkexec`
#[cfg(target_os = "windows")]   // sc.exe service control, `certutil` cert trust
```

Key platform-split implementations:
- `verify_privileged()` — `geteuid() == 0` vs `net session`
- `run_server()` — Unix socket listener vs Named pipe server
- `is_process_alive()` / `kill_process()` — `libc::kill` vs `tasklist`/`taskkill`
- CA trust — `security add-trusted-cert` vs `update-ca-certificates` vs `certutil -addstore Root`
- Service install — launchd plist vs systemd service vs `sc.exe create`
- Privilege elevation (app) — `osascript` (macOS) vs `pkexec` (Linux) vs PowerShell `RunAs` (Windows)

## Important Technical Notes

- rcgen v0.13 has no `CertificateParams::from_ca_cert_pem()` — CA params must be recreated and self-signed to get a `Certificate` for signing domain certs
- Tauri v2 `app` config does NOT have a `title` field; window title goes in `windows[]` only
- `cargo tauri dev` does NOT rebuild the daemon — use platform-specific reload scripts after daemon code changes
- Timestamps in SQLite use `datetime('now')` which stores UTC; frontend must append 'Z' when parsing
- `#[serde(default)]` on struct fields for backwards-compatible evolution between app and daemon
- macOS signing identity: `Developer ID Application: KAKHA GIORGASHVILI (9KW3WQ579Q)`
- Windows Service: MUST use `define_windows_service!` macro from `windows-service` v0.7 for FFI entry point — do NOT manually write `extern "system" fn`
- Windows named pipe security: Uses `windows-sys` NULL DACL so unprivileged app can connect to SYSTEM-owned pipe
- Windows daemon supports `--console` flag for debugging outside the service host
- Windows process mgmt: `tasklist /FI "PID eq {pid}"` to check alive, `taskkill /PID {pid} /F` to kill
- Linux CA trust via `update-ca-certificates` covers system tools (curl, wget) but NOT Firefox/Chrome (they use NSS/their own stores)
- Linux privilege elevation uses `pkexec` (polkit) — requires `policykit-1` package
- Linux daemon managed via systemd: unit file at `resources/localdomain-daemon.service`, installed to `/etc/systemd/system/`
- Linux build deps: `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf libssl-dev build-essential pkg-config`
