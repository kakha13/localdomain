# Project Audit Report

**Date:** 2026-03-06
**Scope:** Full codebase audit — Rust backend, TypeScript frontend, configuration & build scripts

---

## Summary

| Area | Critical | High | Medium | Low | Total |
|------|----------|------|--------|-----|-------|
| Rust (backend + daemon) | 4 | 6 | 10 | 6 | 26 |
| Frontend (React/TS) | 0 | 0 | 9 | 18 | 27 |
| Config & Build | 0 | 4 | 10 | 8 | 22 |
| **Total** | **4** | **10** | **29** | **32** | **75** |

---

## CRITICAL Issues (4)

### 1. Command Injection via `xampp_path` in shell commands
- **File:** `src-tauri/src/xampp.rs:365-398`
- **Category:** Security — Command Injection
- On macOS, `xampp_path` is interpolated into an `osascript do shell script` string with only `"` escaping — backticks, `$()`, or single quotes bypass this and allow arbitrary command execution **with administrator privileges**. On Linux, `xampp_path` is interpolated into a `bash -c` string passed to `pkexec` with zero escaping, giving direct **root-level command injection**.

### 2. Path Traversal in access log domain parameter
- **File:** `daemon/src/logs.rs:8-13`
- **Category:** Security — Path Traversal
- `log_path` builds a filesystem path using the raw `domain` string: `format!("{}.access.log", domain)`. A domain value like `../../etc/passwd` allows reading or truncating arbitrary files as root. The `clear_access_log` function (line 65-71) is destructive — it truncates the resolved path to zero bytes.

### 3. Path Traversal in certificate generation
- **File:** `daemon/src/certs/domain.rs:60-67`
- **Category:** Security — Path Traversal
- The `domain` parameter is used directly in file paths (`format!("{}.crt", domain)`) without validation. A crafted domain with `../` sequences writes certificate and key files to arbitrary filesystem locations as root.

### 4. No input validation in the daemon IPC server
- **File:** `daemon/src/server.rs:156-371`
- **Category:** Security — Missing Input Validation
- The daemon's `dispatch` function accepts requests from any local user (the Unix socket has mode `0o666`). It trusts all incoming parameters without re-validation. This is the **root cause** enabling issues 2, 3, and the HIGH-severity injection issues below — any local user can craft JSON-RPC requests to exploit every daemon operation.

---

## HIGH Issues (10)

### 5. Hosts file injection via unvalidated fields
- **File:** `daemon/src/hosts/mod.rs`
- **Category:** Security — Injection
- `HostsEntry.ip` and `HostsEntry.domain` are written directly into `/etc/hosts`. If either field contains newlines or tabs, additional hosts entries can be injected.

### 6. Caddyfile injection via domain name and target_host
- **File:** `daemon/src/caddy/config.rs`
- **Category:** Security — Injection
- Domain names and target hosts are interpolated into Caddyfile syntax without escaping. A domain containing `}` or newlines could break out of the Caddy configuration block and inject arbitrary directives.

### 7. XAMPP VirtualHost injection
- **File:** `daemon/src/xampp/config.rs:211-255`
- **Category:** Security — Injection
- `vhost.name` and `vhost.document_root` are interpolated into Apache VirtualHost blocks without escaping. A `document_root` containing `"` escapes the `DocumentRoot` directive and injects arbitrary Apache config run as root.

### 8. Unsafe integer cast: i32 to u16 without bounds checking
- **File:** `src-tauri/src/commands/domains.rs:124`
- **Category:** Bug — Integer Overflow
- `d.target_port as u16` silently truncates. The check on line 105 only catches zero and negatives. Values above 65535 (e.g., `70000`) silently wrap to incorrect port numbers.

### 9. SSH tunnel with `StrictHostKeyChecking=no`
- **File:** `daemon/src/tunnel/ssh.rs:22`
- **Category:** Security — MITM Vulnerability
- Hardcoded `StrictHostKeyChecking=no` makes SSH tunnels vulnerable to man-in-the-middle attacks.

### 10. Potential deadlock: inconsistent lock ordering
- **File:** `src-tauri/src/commands/domains.rs:302-309`
- **Category:** Bug — Potential Deadlock
- In `delete_domain`, the `db` mutex is held while `daemon_client` is acquired. Other code paths acquire these in the opposite order. This creates a classic deadlock cycle.

### 11. CSP disabled in Tauri config
- **File:** `src-tauri/tauri.conf.json:24`
- **Category:** Security
- `"csp": null` disables Content Security Policy entirely, allowing arbitrary script execution in the webview.

### 12. Hardcoded developer-specific path in `reinstall-service.ps1`
- **File:** `scripts/reinstall-service.ps1:5`
- **Category:** Build — Portability
- `$DaemonSrc = "E:\AppsD\localdomain\target\debug\localdomain-daemon.exe"` is specific to one developer's machine.

### 13. Windows CI step uses bash syntax without specifying bash shell
- **File:** `.github/workflows/build-all.yml:176-179`
- **Category:** CI — Build Failure
- The `Install Tauri CLI` step uses bash syntax on `windows-latest` (which defaults to PowerShell).

### 14. Signing identity hardcoded in two places
- **File:** `src-tauri/tauri.conf.json:42`, `scripts/pre-bundle.cjs:6`
- **Category:** Build — Maintainability
- The identity string is duplicated; if it changes, both files must be updated.

---

## MEDIUM Issues (29)

### Rust Backend (10)

| # | Issue | File | Description |
|---|-------|------|-------------|
| 15 | `Mutex::lock().unwrap()` everywhere | Multiple `src-tauri/src/` files | One panic poisons the mutex, crashing the entire app on subsequent accesses |
| 16 | Migration without transactions | `src-tauri/src/db/migrations.rs:61-69` | Multi-statement migrations can leave DB in unrecoverable state |
| 17 | Race condition in `start_caddy` | `daemon/src/caddy/process.rs:34-38` | Check-then-start is non-atomic; concurrent IPC requests spawn duplicate Caddy processes |
| 18 | Blocking I/O in async context | `daemon/src/tunnel/cloudflared.rs` | `std::thread::sleep` (up to 15s) blocks the Tokio runtime thread |
| 19 | Blocking synchronous daemon IPC | `src-tauri/src/daemon_client/mod.rs:44-66` | 60-second timeout blocks Tauri thread pool if daemon is slow |
| 20 | Settings can never be cleared | `src-tauri/src/commands/settings.rs:73-95` | `None` values skip DB update; old values persist when user intends to clear |
| 21 | `stop_caddy` doesn't force-kill | `daemon/src/caddy/process.rs:94-110` | After SIGTERM timeout, removes PID file without SIGKILL — Caddy runs untracked |
| 22 | Caddy child handle leaked (zombie) | `daemon/src/caddy/process.rs:55-63` | `Child` handle dropped immediately creates zombie process on Unix |
| 23 | `ListenBacklog` falsely matched as `Listen` | `daemon/src/xampp/config.rs:58` | `strip_prefix("Listen")` matches `ListenBacklog 512` as port |
| 24 | Named pipe `SECURITY_DESCRIPTOR` lifetime | `daemon/src/server.rs:86-96` | Stack-allocated descriptor referenced by pointer — fragile to refactoring |

### Frontend (9)

| # | Issue | File | Description |
|---|-------|------|-------------|
| 25 | Duplicate `useServiceStatus` polling | `src/components/ServiceStatusBar.tsx:9` | Two independent 5s polling loops; status can momentarily disagree |
| 26 | Stale index in RequestInspector | `src/components/RequestInspector.tsx:169-172` | `selectedIndex` shifts when new log entries arrive during polling |
| 27 | No error handling on `confirmDelete` | `src/components/DomainList.tsx:156-159` | If `remove()` throws, user gets stuck in delete modal |
| 28 | No error state on settings load | `src/components/SettingsView.tsx:29-33` | Infinite "Loading settings..." if API call fails |
| 29 | No error state on XAMPP settings load | `src/components/XamppView.tsx:23-28` | Same infinite loading problem |
| 30 | No confirmation for "Clear All" audit log | `src/components/AuditLogView.tsx:68-71` | Permanently deletes all audit history with single click |
| 31 | Excessive interval recreation for tunnel polling | `src/components/DomainList.tsx:89-114` | Interval torn down and recreated on every domain change |
| 32 | Toggle switch lacks accessible label | `src/components/DomainCard.tsx:107-115` | Screen readers see only "checkbox" with no domain context |
| 33 | Modals lack focus trapping and ARIA | Multiple modal components | Missing `role="dialog"`, `aria-modal`, focus trap, Escape key handling |

### Config & Build (10)

| # | Issue | File | Description |
|---|-------|------|-------------|
| 34 | Schema URL points to third-party fork | `src-tauri/tauri.conf.json:2` | References `nicoulaj/tauri` instead of official `tauri-apps/tauri` |
| 35 | systemd service has no security hardening | `resources/localdomain-daemon.service` | Runs as root with no sandboxing directives |
| 36 | launchd plist has no resource limits | `resources/com.localdomain.daemon.plist` | Root daemon with `KeepAlive` but no memory/fd limits |
| 37 | `pre-bundle.cjs` silently succeeds on CI codesigning failure | `scripts/pre-bundle.cjs:24-28` | Produces unsigned binary that fails later at notarization |
| 38 | CI version update misses Cargo.toml files | `.github/workflows/build-all.yml:32-41` | Only patches JSON files; three Cargo.toml files keep hardcoded `0.1.2` |
| 39 | Reload scripts build debug binaries but install to production paths | `scripts/reload-daemon.ps1`, `reload-daemon-linux.sh` | Copies unoptimized debug binaries to `/usr/local/bin/` |
| 40 | macOS scripts use deprecated launchctl commands | `scripts/install-daemon.sh`, `resources/uninstall.sh` | `launchctl load/unload` deprecated since macOS 10.10 |
| 41 | CI uses deprecated `softprops/action-gh-release@v1` | `.github/workflows/build-all.yml:297` | v1 uses deprecated Node.js 16 |
| 42 | Download scripts do not verify checksums | `scripts/download-caddy*.sh` | Caddy binaries downloaded with no SHA256 verification |
| 43 | Windows Caddy download has no version pinning | `scripts/download-caddy.ps1` | Always downloads "latest" while macOS/Linux pin to 2.8.4 |

---

## LOW Issues (32)

### Rust (6)
- Regex compiled on every validation call (`shared/src/domain.rs:70-71`)
- `import_xampp_vhosts` queries all domains per iteration — O(n*m) (`src-tauri/src/commands/settings.rs:193-219`)
- Windows `is_process_alive` false positive on substring PID match (`daemon/src/caddy/process.rs:14-23`)
- Access log reads entire file into memory before truncating (`daemon/src/logs.rs:15-63`)
- Cloudflare credentials returned to frontend (`src-tauri/src/commands/tunnel.rs:400-407`)
- `ensure_vhosts_include` strips line indentation when uncommenting (`daemon/src/xampp/config.rs:280-297`)

### Frontend (18)
- Missing `useEffect` dependency in SetupScreen (`src/components/SetupScreen.tsx:44-48`)
- Missing `useEffect` dependency in DomainFormModal (`src/components/DomainFormModal.tsx:39-58`)
- Missing error handling on access log clear (`src/hooks/useAccessLog.ts`)
- No error handling on `getVersion()` (`src/components/AboutView.tsx:9-11`)
- No loading state on delete button (`src/components/DomainList.tsx:272`)
- No confirmation for access log clear (`src/components/RequestInspector.tsx:204`)
- No client-side domain name format validation (`src/components/DomainFormModal.tsx`)
- `parseInt` can produce NaN (`src/components/DomainFormModal.tsx:66`)
- Nav buttons lack `aria-current="page"` (`src/components/Layout.tsx:90-109`)
- Sidebar buttons lack `aria-label` when collapsed (`src/components/Layout.tsx:113-126`)
- Action buttons use `title` instead of `aria-label` (`src/components/DomainCard.tsx:196-257`)
- Table rows not keyboard-accessible (`src/components/RequestInspector.tsx:237-255`)
- `catch (e: any)` bypasses type safety (`src/components/DomainList.tsx:183`)
- Unused `toggleProgress` prop (`src/components/DomainCard.tsx:24`)
- Silent error swallowing in hooks (`src/hooks/useAuditLog.ts`, `useAccessLog.ts`)
- Theme initialization logic duplication (`src/main.tsx:8-15` vs `useTheme.ts`)
- Type assertion on `domain_type` (`src/components/DomainFormModal.tsx:33`)
- Unhandled error on XAMPP path save (`src/components/XamppView.tsx:63-68`)

### Config & Build (8)
- `rusqlite` 0.31 is one major version behind (`src-tauri/Cargo.toml:16`)
- `once_cell` crate is unnecessary — `std::sync::OnceLock` is stable (`daemon/Cargo.toml:18`)
- `build-linux-deb.sh` output message has wrong path (`scripts/build-linux-deb.sh:22`)
- Windows daemon binary checked into git (`resources/localdomain-daemon.exe`)
- `shell:default` capability may be overly broad (`src-tauri/capabilities/default.json:9`)
- Caddy version 2.8.4 is outdated (`scripts/download-caddy.sh`)
- `install-daemon.sh` does not set ownership on plist (`scripts/install-daemon.sh`)
- No enforcement of consistent local dependency versions (`package.json`)

---

## Top Priority Recommendations

1. **Add input validation in the daemon** (fixes issues 2, 3, 5, 6, 7 in one pass) — The daemon IPC socket is world-writable (`0o666`). Domain name validation, path canonicalization, and IP format validation at the daemon entry point would neutralize all path traversal and injection attacks.

2. **Rewrite XAMPP shell command construction** (issue 1) — Replace string interpolation with proper argument arrays to eliminate the root-level command injection.

3. **Set a proper CSP in `tauri.conf.json`** (issue 11) — At minimum: `"default-src 'self'; script-src 'self'"`.

4. **Fix lock ordering** (issue 10) — Establish a consistent order (always `db` before `daemon_client` or vice versa) across all command handlers.

5. **Wrap multi-statement migrations in transactions** (issue 16) — Prevents unrecoverable database states.

6. **Add systemd/launchd hardening** (issues 35, 36) — The daemon runs as root; add `ProtectSystem=strict`, `ProtectHome=yes`, `NoNewPrivileges=yes` etc.
