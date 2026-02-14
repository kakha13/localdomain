export interface Domain {
  id: string;
  name: string;
  target_host: string;
  target_port: number;
  protocol: "http" | "https" | "both";
  wildcard: boolean;
  enabled: boolean;
  access_log: boolean;
  created_at: string;
  updated_at: string;
  tunnel_subdomain: string;
  tunnel_domain: string;
  domain_type: "proxy" | "xampp";
  document_root: string;
}

export interface CreateDomainRequest {
  name: string;
  target_host?: string;
  target_port?: number;
  protocol?: string;
  wildcard?: boolean;
  domain_type?: "proxy" | "xampp";
  document_root?: string;
}

export interface UpdateDomainRequest {
  id: string;
  name?: string;
  target_host?: string;
  target_port?: number;
  protocol?: string;
  wildcard?: boolean;
  enabled?: boolean;
  domain_type?: "proxy" | "xampp";
  document_root?: string;
}

export interface ServiceStatus {
  daemon_running: boolean;
  caddy_running: boolean;
  ca_installed: boolean;
  ca_trusted: boolean;
  xampp_running: boolean;
}

export interface AuditLogEntry {
  id: number;
  action: string;
  domain_id: string | null;
  details: string | null;
  created_at: string;
}

export interface AppSettings {
  start_on_boot: boolean;
  http_port: number;
  https_port: number;
  cloudflare_tunnel_token?: string;
  default_ssh_host?: string;
  default_ssh_user?: string;
  default_ssh_key_path?: string;
  xampp_path?: string;
}

export interface AccessLogEntry {
  timestamp: number;
  method: string;
  uri: string;
  status: number;
  duration: number;
  size: number;
  host: string;
  headers: Record<string, string[]>;
  resp_headers: Record<string, string[]>;
  remote_ip: string;
  proto: string;
}

// Tunnel types

export interface QuickTunnelConfig {
  type: "QuickTunnel";
}

export interface NamedTunnelConfig {
  type: "NamedTunnel";
  token: string;
  subdomain: string;
  cloudflare_domain: string;
  credentials_json?: string;
  tunnel_uuid?: string;
}

export interface SshTunnelConfig {
  type: "SshTunnel";
  host: string;
  port: number;
  user: string;
  key: string;
  remote_port: number;
}

export type TunnelType = QuickTunnelConfig | NamedTunnelConfig | SshTunnelConfig;

export interface StartTunnelRequest {
  domain_id: string;
  tunnel_type: TunnelType;
}

export interface StartTunnelResult {
  public_url: string;
  tunnel_id: string;
}

export interface TunnelStatusResult {
  active: boolean;
  public_url?: string;
  tunnel_type?: TunnelType;
  error?: string;
}

export interface TunnelInfo {
  domain: string;
  public_url: string;
  tunnel_type: TunnelType;
  pid: number;
}

export interface ListTunnelsResult {
  tunnels: TunnelInfo[];
}

export interface EnsureCloudflaredResult {
  installed: boolean;
  path: string;
  version?: string;
}

// Cloudflare automated auth flow
export interface CloudflareLoginStatus {
  logged_in: boolean;
}

export interface CloudflareSetupRequest {
  subdomain: string;
  domain: string;
}

export interface CloudflareSetupResult {
  tunnel_name: string;
  tunnel_id: string;
  token: string;
  credentials_json: string;
  public_url: string;
}

export interface XamppPorts {
  http_port: number;
  ssl_port: number;
}

export interface ScannedVhost {
  server_name: string;
  document_root: string;
  port: number;
  already_exists: boolean;
}

export interface ImportVhost {
  server_name: string;
  document_root: string;
}

export type View = "domains" | "settings" | "audit" | "inspect" | "about" | "xampp";
