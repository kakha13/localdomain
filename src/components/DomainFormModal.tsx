import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../lib/api";
import type { Domain, CreateDomainRequest, UpdateDomainRequest } from "../lib/types";

interface DomainFormModalProps {
  domain: Domain | null;
  onSave: (request: CreateDomainRequest | UpdateDomainRequest) => Promise<void>;
  onClose: () => void;
}

export function DomainFormModal({ domain, onSave, onClose }: DomainFormModalProps) {
  const [name, setName] = useState("");
  const [targetHost, setTargetHost] = useState("");
  const [targetPort, setTargetPort] = useState("");
  const [protocol, setProtocol] = useState("http");
  const [wildcard, setWildcard] = useState(false);
  const [domainType, setDomainType] = useState<"proxy" | "xampp">("proxy");
  const [documentRoot, setDocumentRoot] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [xamppPort, setXamppPort] = useState<number | null>(null);

  const isEditing = domain !== null;

  useEffect(() => {
    if (domain) {
      setName(domain.name);
      setTargetHost(domain.target_host || "");
      setTargetPort(domain.target_port > 0 ? String(domain.target_port) : "");
      setProtocol(domain.protocol);
      setWildcard(domain.wildcard);
      setDomainType((domain.domain_type as "proxy" | "xampp") || "proxy");
      setDocumentRoot(domain.document_root || "");
    }
  }, [domain]);

  // Auto-fetch XAMPP default port when switching to XAMPP type
  useEffect(() => {
    if (domainType !== "xampp") return;
    let cancelled = false;
    (async () => {
      try {
        const settings = await api.getSettings();
        if (!settings.xampp_path || cancelled) return;
        const ports = await api.getXamppDefaultPort(settings.xampp_path);
        if (cancelled) return;
        setXamppPort(ports.http_port);
        // Auto-set target port if not already set by user
        if (!targetPort) {
          setTargetPort(String(ports.http_port));
        }
      } catch {
        // Ignore - port detection is best-effort
      }
    })();
    return () => { cancelled = true; };
  }, [domainType]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);

    try {
      const portNum = targetPort ? parseInt(targetPort, 10) : undefined;

      if (isEditing) {
        const request: UpdateDomainRequest = {
          id: domain.id,
          name,
          target_host: targetHost || undefined,
          target_port: portNum,
          protocol,
          wildcard,
          domain_type: domainType,
          document_root: domainType === "xampp" ? documentRoot : undefined,
        };
        await onSave(request);
      } else {
        const request: CreateDomainRequest = {
          name,
          target_host: targetHost || undefined,
          target_port: portNum,
          protocol,
          wildcard,
          domain_type: domainType,
          document_root: domainType === "xampp" ? documentRoot : undefined,
        };
        await onSave(request);
      }
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{isEditing ? "Edit Domain" : "Add Domain"}</h2>
          <button className="modal-close" onClick={onClose}>
            &times;
          </button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="name">Domain Name</label>
            <input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="project.test"
              required
              autoFocus
            />
          </div>
          <div className="form-group">
            <label>Domain Type</label>
            <div className="form-row" style={{ gap: 0 }}>
              <button
                type="button"
                className={`btn btn-sm ${domainType === "proxy" ? "btn-primary" : ""}`}
                style={{ borderRadius: "6px 0 0 6px", flex: 1 }}
                onClick={() => setDomainType("proxy")}
              >
                Reverse Proxy
              </button>
              <button
                type="button"
                className={`btn btn-sm ${domainType === "xampp" ? "btn-primary" : ""}`}
                style={{ borderRadius: "0 6px 6px 0", flex: 1 }}
                onClick={() => setDomainType("xampp")}
              >
                XAMPP
              </button>
            </div>
          </div>
          {domainType === "proxy" && (
            <>
              <div className="form-row">
                <div className="form-group">
                  <label htmlFor="targetHost">Target Host <span className="form-optional">(optional)</span></label>
                  <input
                    id="targetHost"
                    type="text"
                    value={targetHost}
                    onChange={(e) => setTargetHost(e.target.value)}
                    placeholder="127.0.0.1"
                  />
                </div>
                <div className="form-group">
                  <label htmlFor="targetPort">Target Port <span className="form-optional">(optional)</span></label>
                  <input
                    id="targetPort"
                    type="number"
                    value={targetPort}
                    onChange={(e) => setTargetPort(e.target.value)}
                    placeholder="3000"
                    min="1"
                    max="65535"
                  />
                </div>
              </div>
              <div className="form-hint">
                Only the domain name is required. The domain will resolve to 127.0.0.1. Set a target port to enable reverse proxying.
              </div>
            </>
          )}
          {domainType === "xampp" && (
            <>
              <div className="form-group">
                <label htmlFor="documentRoot">Document Root</label>
                <div className="input-with-btn">
                  <input
                    id="documentRoot"
                    type="text"
                    value={documentRoot}
                    onChange={(e) => setDocumentRoot(e.target.value)}
                    placeholder="/var/www/mysite"
                    required
                  />
                  <button
                    type="button"
                    className="btn btn-browse"
                    onClick={async () => {
                      const selected = await open({
                        directory: true,
                        multiple: false,
                        title: "Select Document Root",
                      });
                      if (selected) setDocumentRoot(selected as string);
                    }}
                  >
                    Browse
                  </button>
                </div>
                <div className="form-hint">
                  Absolute path to the website files directory.
                </div>
              </div>
              {xamppPort !== null && (
                <div className="form-hint">
                  XAMPP Apache port: <strong>{xamppPort}</strong>
                </div>
              )}
            </>
          )}
          <div className="form-group">
            <label htmlFor="protocol">Protocol</label>
            <select
              id="protocol"
              value={protocol}
              onChange={(e) => setProtocol(e.target.value)}
            >
              <option value="http">HTTP</option>
              <option value="https">HTTPS</option>
              <option value="both">Both</option>
            </select>
          </div>
          {(protocol === "https" || protocol === "both") && (
            <div className="form-hint form-hint-info" style={{ color: "var(--text-tertiary)" }}>
              HTTPS requires a trusted root certificate. It will be installed automatically if needed. You may need to restart your browser afterward.
            </div>
          )}
          <div className="form-group">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={wildcard}
                onChange={(e) => setWildcard(e.target.checked)}
              />
              Wildcard (*.domain)
            </label>
          </div>
          {error && <div className="form-error">{error}</div>}
          <div className="modal-actions">
            <button type="button" className="btn" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn btn-primary" disabled={saving}>
              {saving ? "Saving..." : isEditing ? "Update" : "Create"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
