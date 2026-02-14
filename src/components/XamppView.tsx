import { useState, useEffect } from "react";
import type { AppSettings, ServiceStatus, ScannedVhost } from "../lib/types";
import { useLoading } from "../hooks/useLoading";
import * as api from "../lib/api";

interface XamppViewProps {
  status: ServiceStatus;
}

export function XamppView({ status }: XamppViewProps) {
  const { track } = useLoading();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [message, setMessage] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);

  // VHost scanning state
  const [scannedVhosts, setScannedVhosts] = useState<ScannedVhost[] | null>(null);
  const [selectedVhosts, setSelectedVhosts] = useState<Set<string>>(new Set());
  const [scanning, setScanning] = useState(false);
  const [importing, setImporting] = useState(false);

  useEffect(() => {
    api.getSettings().then((s) => {
      setSettings(s);
      setLoading(false);
    });
  }, []);

  const handleToggleApache = async () => {
    setActionLoading(true);
    setMessage(null);
    try {
      if (status.xampp_running) {
        await track(api.stopApache());
      } else {
        await track(api.startApache());
      }
    } catch (e) {
      setMessage(`${e}`);
    } finally {
      setActionLoading(false);
    }
  };

  const handleDetectPath = async () => {
    setMessage(null);
    try {
      const path = await track(api.detectXamppPath());
      if (path && settings) {
        const updated = { ...settings, xampp_path: path };
        setSettings(updated);
        await track(api.saveSettings(updated));
        setMessage("XAMPP path detected and saved.");
      } else {
        setMessage("XAMPP not found on this system.");
      }
    } catch {
      setMessage("Failed to detect XAMPP path.");
    }
  };

  const handleSaveXamppPath = async (newPath: string) => {
    if (!settings) return;
    const updated = { ...settings, xampp_path: newPath || undefined };
    setSettings(updated);
    await track(api.saveSettings(updated));
  };

  const handleScanVhosts = async () => {
    const path = settings?.xampp_path;
    if (!path) {
      setMessage("Set XAMPP path first.");
      return;
    }
    setScanning(true);
    setMessage(null);
    try {
      const results = await track(api.scanXamppVhosts(path));
      setScannedVhosts(results);
      const importable = new Set(
        results.filter((v) => !v.already_exists).map((v) => v.server_name)
      );
      setSelectedVhosts(importable);
    } catch (e) {
      setMessage(`Scan failed: ${e}`);
    } finally {
      setScanning(false);
    }
  };

  const handleImportVhosts = async () => {
    if (!scannedVhosts || selectedVhosts.size === 0) return;
    setImporting(true);
    setMessage(null);
    try {
      const toImport = scannedVhosts
        .filter((v) => selectedVhosts.has(v.server_name) && !v.already_exists)
        .map((v) => ({ server_name: v.server_name, document_root: v.document_root }));
      const created = await track(api.importXamppVhosts(toImport));
      setMessage(`Imported ${created.length} domain${created.length !== 1 ? "s" : ""}.`);
      setScannedVhosts(null);
      setSelectedVhosts(new Set());
    } catch (e) {
      setMessage(`Import failed: ${e}`);
    } finally {
      setImporting(false);
    }
  };

  const toggleVhostSelection = (name: string) => {
    setSelectedVhosts((prev) => {
      const next = new Set(prev);
      if (next.has(name)) {
        next.delete(name);
      } else {
        next.add(name);
      }
      return next;
    });
  };

  if (loading || !settings) {
    return <div className="loading">Loading...</div>;
  }

  return (
    <div className="settings-view">
      <h2>XAMPP</h2>

      <section className="settings-section">
        <h3>Apache</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">
              Apache Server
              <span className={`status-badge ${status.xampp_running ? "status-badge-active" : "status-badge-inactive"}`}>
                {status.xampp_running ? "Running" : "Stopped"}
              </span>
            </div>
            <p className="settings-service-desc">
              XAMPP Apache serves PHP sites via VirtualHost configuration.
            </p>
          </div>
          <button
            className={`btn btn-sm ${status.xampp_running ? "" : "btn-primary"}`}
            onClick={handleToggleApache}
            disabled={actionLoading || !settings.xampp_path}
          >
            {actionLoading ? "..." : status.xampp_running ? "Stop" : "Start"}
          </button>
        </div>
      </section>

      <section className="settings-section">
        <h3>Configuration</h3>
        <div className="form-group">
          <label htmlFor="xamppPath">XAMPP Path</label>
          <div className="form-row">
            <input
              id="xamppPath"
              type="text"
              value={settings.xampp_path || ""}
              onChange={(e) =>
                setSettings({ ...settings, xampp_path: e.target.value || undefined })
              }
              onBlur={(e) => handleSaveXamppPath(e.target.value)}
              placeholder="/Applications/XAMPP/xamppfiles"
              style={{ flex: 1 }}
            />
            <button className="btn btn-sm" onClick={handleDetectPath}>
              Auto-Detect
            </button>
          </div>
        </div>
        {message && <div className="form-message">{message}</div>}
      </section>

      <section className="settings-section">
        <h3>VHost Scanner</h3>
        <p className="form-hint">
          Scan httpd-vhosts.conf for existing VirtualHost entries to import as domains.
        </p>
        <button
          className="btn btn-sm"
          onClick={handleScanVhosts}
          disabled={scanning || !settings.xampp_path}
        >
          {scanning ? "Scanning..." : "Scan VHosts"}
        </button>

        {scannedVhosts !== null && (
          <div className="scanned-vhosts">
            {scannedVhosts.length === 0 ? (
              <p className="form-hint">No importable VirtualHost entries found.</p>
            ) : (
              <>
                <div className="scanned-vhosts-list">
                  {scannedVhosts.map((v) => (
                    <label key={v.server_name} className="checkbox-label scanned-vhost-item">
                      <input
                        type="checkbox"
                        checked={selectedVhosts.has(v.server_name)}
                        disabled={v.already_exists}
                        onChange={() => toggleVhostSelection(v.server_name)}
                      />
                      <span className="scanned-vhost-info">
                        <span className="scanned-vhost-name">
                          {v.server_name}
                          {v.port !== 80 && <span className="scanned-vhost-port">:{v.port}</span>}
                        </span>
                        <span className="scanned-vhost-root">{v.document_root}</span>
                        {v.already_exists && (
                          <span className="status-badge status-badge-inactive">Already exists</span>
                        )}
                      </span>
                    </label>
                  ))}
                </div>
                <div className="scanned-vhosts-actions">
                  <button
                    className="btn btn-sm btn-primary"
                    onClick={handleImportVhosts}
                    disabled={importing || selectedVhosts.size === 0}
                  >
                    {importing ? "Importing..." : `Import ${selectedVhosts.size} Selected`}
                  </button>
                  <button
                    className="btn btn-sm"
                    onClick={() => { setScannedVhosts(null); setSelectedVhosts(new Set()); }}
                  >
                    Cancel
                  </button>
                </div>
              </>
            )}
          </div>
        )}
      </section>
    </div>
  );
}
