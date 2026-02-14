import { invoke } from "@tauri-apps/api/core";
import type {
  Domain,
  CreateDomainRequest,
  UpdateDomainRequest,
  ServiceStatus,
  AuditLogEntry,
  AccessLogEntry,
  AppSettings,
  StartTunnelRequest,
  StartTunnelResult,
  TunnelStatusResult,
  ListTunnelsResult,
  EnsureCloudflaredResult,
  CloudflareLoginStatus,
  CloudflareSetupRequest,
  CloudflareSetupResult,
  XamppPorts,
  ScannedVhost,
  ImportVhost,
} from "./types";

export async function listDomains(): Promise<Domain[]> {
  return invoke("list_domains");
}

export async function createDomain(
  request: CreateDomainRequest
): Promise<Domain> {
  return invoke("create_domain", { request });
}

export async function updateDomain(
  request: UpdateDomainRequest
): Promise<Domain> {
  return invoke("update_domain", { request });
}

export async function deleteDomain(id: string): Promise<void> {
  return invoke("delete_domain", { id });
}

export async function toggleDomain(
  id: string,
  enabled: boolean
): Promise<Domain> {
  return invoke("toggle_domain", { id, enabled });
}

export async function getServiceStatus(): Promise<ServiceStatus> {
  return invoke("get_service_status");
}

export async function startService(): Promise<void> {
  return invoke("start_service");
}

export async function stopService(): Promise<void> {
  return invoke("stop_service");
}

export async function installDaemon(): Promise<void> {
  return invoke("install_daemon");
}

export async function uninstallDaemon(): Promise<void> {
  return invoke("uninstall_daemon");
}

export async function toggleAccessLog(
  id: string,
  enabled: boolean
): Promise<Domain> {
  return invoke("toggle_access_log", { id, enabled });
}

export async function getAccessLog(
  domain: string,
  limit?: number
): Promise<AccessLogEntry[]> {
  return invoke("get_access_log", { domain, limit });
}

export async function clearAccessLog(domain: string): Promise<void> {
  return invoke("clear_access_log", { domain });
}

export async function clearAuditLog(): Promise<void> {
  return invoke("clear_audit_log");
}

export async function getAuditLog(
  limit?: number,
  offset?: number
): Promise<AuditLogEntry[]> {
  return invoke("get_audit_log", { limit, offset });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function trustCa(): Promise<void> {
  return invoke("trust_ca");
}

export async function startTunnel(
  request: StartTunnelRequest
): Promise<StartTunnelResult> {
  return invoke("start_tunnel", { request });
}

export async function stopTunnel(domainId: string): Promise<void> {
  return invoke("stop_tunnel", { domainId });
}

export async function getTunnelStatus(
  domainId: string
): Promise<TunnelStatusResult> {
  return invoke("get_tunnel_status", { domainId });
}

export async function listTunnels(): Promise<ListTunnelsResult> {
  return invoke("list_tunnels");
}

export async function ensureCloudflared(): Promise<EnsureCloudflaredResult> {
  return invoke("ensure_cloudflared");
}

export async function saveTunnelConfig(
  domainId: string,
  subdomain: string,
  domain: string
): Promise<void> {
  return invoke("save_tunnel_config", { domainId, subdomain, domain });
}

export async function cloudflareCheckLogin(): Promise<CloudflareLoginStatus> {
  return invoke("cloudflare_check_login");
}

export async function cloudflareLogin(): Promise<CloudflareLoginStatus> {
  return invoke("cloudflare_login");
}

export async function cloudflareSetupTunnel(
  request: CloudflareSetupRequest
): Promise<CloudflareSetupResult> {
  return invoke("cloudflare_setup_tunnel", { request });
}

export async function startApache(): Promise<void> {
  return invoke("start_apache");
}

export async function stopApache(): Promise<void> {
  return invoke("stop_apache");
}

export async function detectXamppPath(): Promise<string | null> {
  return invoke("detect_xampp_path");
}

export async function getXamppDefaultPort(
  xamppPath: string
): Promise<XamppPorts> {
  return invoke("get_xampp_default_port", { xamppPath });
}

export async function scanXamppVhosts(
  xamppPath: string
): Promise<ScannedVhost[]> {
  return invoke("scan_xampp_vhosts", { xamppPath });
}

export async function importXamppVhosts(
  vhosts: ImportVhost[]
): Promise<Domain[]> {
  return invoke("import_xampp_vhosts", { vhosts });
}
