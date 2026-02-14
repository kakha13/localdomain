import { useState, useEffect } from "react";
import type { AppSettings, ServiceStatus } from "../lib/types";
import { useLoading } from "../hooks/useLoading";
import * as api from "../lib/api";

interface SettingsViewProps {
  status: ServiceStatus;
  onStart: () => Promise<void>;
  onStop: () => Promise<void>;
  onUninstall: () => Promise<void>;
  onTrustCa: () => Promise<void>;
}

export function SettingsView({ status, onStart, onStop, onUninstall, onTrustCa }: SettingsViewProps) {
  const { track } = useLoading();
  const [settings, setSettings] = useState<AppSettings>({
    start_on_boot: false,
    http_port: 80,
    https_port: 443,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [confirmUninstall, setConfirmUninstall] = useState(false);
  const [actionLoading, setActionLoading] = useState(false);
  const [trustingCa, setTrustingCa] = useState(false);
  const [trustHint, setTrustHint] = useState(false);

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      setLoading(false);
    });
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setMessage(null);
    try {
      await track(api.saveSettings(settings));
      setMessage("Settings saved.");
    } catch (e) {
      setMessage(`Error: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleToggleProxy = async () => {
    setActionLoading(true);
    try {
      if (status.caddy_running) {
        await onStop();
      } else {
        await onStart();
      }
    } finally {
      setActionLoading(false);
    }
  };

  const handleUninstall = async () => {
    if (!confirmUninstall) {
      setConfirmUninstall(true);
      setTimeout(() => setConfirmUninstall(false), 4000);
      return;
    }
    setActionLoading(true);
    try {
      await onUninstall();
    } finally {
      setActionLoading(false);
      setConfirmUninstall(false);
    }
  };

  if (loading) {
    return <div className="loading">Loading settings...</div>;
  }

  return (
    <div className="settings-view">
      <section className="settings-section">
        <h3>General</h3>
        <div className="form-group">
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={settings.start_on_boot}
              onChange={(e) =>
                setSettings({ ...settings, start_on_boot: e.target.checked })
              }
            />
            Start on boot
          </label>
        </div>

        <div className="form-row">
          <div className="form-group">
            <label htmlFor="httpPort">HTTP Port</label>
            <input
              id="httpPort"
              type="number"
              value={settings.http_port}
              onChange={(e) =>
                setSettings({ ...settings, http_port: parseInt(e.target.value, 10) || 80 })
              }
              min="1"
              max="65535"
            />
          </div>
          <div className="form-group">
            <label htmlFor="httpsPort">HTTPS Port</label>
            <input
              id="httpsPort"
              type="number"
              value={settings.https_port}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  https_port: parseInt(e.target.value, 10) || 443,
                })
              }
              min="1"
              max="65535"
            />
          </div>
        </div>

        {message && <div className="form-message">{message}</div>}

        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </section>

      <section className="settings-section">
        <h3>Service</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">
              Proxy server
              <span className={`status-badge ${status.caddy_running ? "status-badge-active" : "status-badge-inactive"}`}>
                {status.caddy_running ? "Running" : "Stopped"}
              </span>
            </div>
            <p className="settings-service-desc">
              The reverse proxy handles domain routing on your machine.
            </p>
          </div>
          <button
            className={`btn btn-sm ${status.caddy_running ? "" : "btn-primary"}`}
            onClick={handleToggleProxy}
            disabled={actionLoading}
          >
            {actionLoading ? "..." : status.caddy_running ? "Stop" : "Start"}
          </button>
        </div>
      </section>

      <section className="settings-section">
        <h3>Certificate Authority</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">
              Root CA
              {!status.ca_installed && (
                <span className="status-badge status-badge-inactive">Not Generated</span>
              )}
              {status.ca_installed && status.ca_trusted && (
                <span className="status-badge status-badge-active">Trusted</span>
              )}
              {status.ca_installed && !status.ca_trusted && (
                <span className="status-badge status-badge-warning">Not Trusted</span>
              )}
            </div>
            <p className="settings-service-desc">
              The root certificate is used to sign HTTPS certificates for your local domains.
            </p>
            {trustHint && (
              <p className="form-hint form-hint-info">
                Certificate trusted. You may need to restart your browser for changes to take effect.
              </p>
            )}
          </div>
          {status.ca_installed && !status.ca_trusted && (
            <button
              className="btn btn-sm btn-primary"
              onClick={async () => {
                setTrustingCa(true);
                await onTrustCa();
                setTrustingCa(false);
                setTrustHint(true);
                setTimeout(() => setTrustHint(false), 8000);
              }}
              disabled={trustingCa}
            >
              {trustingCa ? "..." : "Trust Certificate"}
            </button>
          )}
        </div>
      </section>

      <section className="settings-section">
        <h3>Tunnels</h3>
        <div className="form-group">
          <label htmlFor="cfToken">Cloudflare Tunnel Token <span className="form-optional">(optional)</span></label>
          <input
            id="cfToken"
            type="password"
            value={settings.cloudflare_tunnel_token || ""}
            onChange={(e) =>
              setSettings({ ...settings, cloudflare_tunnel_token: e.target.value || undefined })
            }
            placeholder="eyJhIjoiN..."
          />
        </div>
        <p className="form-hint">Used for Named Tunnels. Get your token from the Cloudflare Zero Trust dashboard.</p>

        <div className="form-row">
          <div className="form-group">
            <label htmlFor="sshHost">Default SSH Host <span className="form-optional">(optional)</span></label>
            <input
              id="sshHost"
              type="text"
              value={settings.default_ssh_host || ""}
              onChange={(e) =>
                setSettings({ ...settings, default_ssh_host: e.target.value || undefined })
              }
              placeholder="vps.example.com"
            />
          </div>
          <div className="form-group">
            <label htmlFor="sshUser">Default SSH User <span className="form-optional">(optional)</span></label>
            <input
              id="sshUser"
              type="text"
              value={settings.default_ssh_user || ""}
              onChange={(e) =>
                setSettings({ ...settings, default_ssh_user: e.target.value || undefined })
              }
              placeholder="root"
            />
          </div>
        </div>
        <div className="form-group">
          <label htmlFor="sshKeyPath">Default SSH Key Path <span className="form-optional">(optional)</span></label>
          <input
            id="sshKeyPath"
            type="text"
            value={settings.default_ssh_key_path || ""}
            onChange={(e) =>
              setSettings({ ...settings, default_ssh_key_path: e.target.value || undefined })
            }
            placeholder="~/.ssh/id_rsa"
          />
        </div>
      </section>

      <section className="settings-section settings-danger-zone">
        <h3>Danger Zone</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">Uninstall service</div>
            <p className="settings-service-desc">
              Removes the background service, stops all proxying, and cleans up hosts entries.
            </p>
          </div>
          <button
            className="btn btn-sm btn-danger"
            onClick={handleUninstall}
            disabled={actionLoading}
          >
            {confirmUninstall ? "Confirm Uninstall" : "Uninstall"}
          </button>
        </div>
      </section>
    </div>
  );
}
