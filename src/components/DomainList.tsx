import { useState, useEffect, useCallback, useRef } from "react";
import { flushSync } from "react-dom";
import type {
  Domain,
  CreateDomainRequest,
  UpdateDomainRequest,
  TunnelStatusResult,
  TunnelType,
  AppSettings,
} from "../lib/types";
import { useDomains } from "../hooks/useDomains";
import { useLoading } from "../hooks/useLoading";
import * as api from "../lib/api";
import { DomainCard } from "./DomainCard";
import { DomainFormModal } from "./DomainFormModal";
import { TunnelModal } from "./TunnelModal";
import { PlusIcon } from "./Icons";
import type { DomainFilter } from "./Layout";

interface DomainListProps {
  onInspect: (id: string, name: string, accessLog: boolean) => void;
  searchQuery: string;
  domainFilter: DomainFilter;
  addTrigger: number;
}

export function DomainList({ onInspect, searchQuery, domainFilter, addTrigger }: DomainListProps) {
  const { domains, loading, error, create, update, remove, toggle } = useDomains();
  const { track } = useLoading();
  const [showModal, setShowModal] = useState(false);
  const [editingDomain, setEditingDomain] = useState<Domain | null>(null);
  const [deletingDomain, setDeletingDomain] = useState<Domain | null>(null);
  const [sharingDomain, setSharingDomain] = useState<Domain | null>(null);
  const [tunnelStatuses, setTunnelStatuses] = useState<Record<string, TunnelStatusResult>>({});
  const [tunnelLoading, setTunnelLoading] = useState(false);
  const [tunnelError, setTunnelError] = useState<string | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [toggleProgressById, setToggleProgressById] = useState<Record<string, number>>({});

  const handleToggle = (id: string, enabled: boolean) => {
    // Prevent duplicate toggles while one is already in flight for this card.
    if (toggleProgressById[id] !== undefined) return;

    // Show per-card loading immediately at 10%.
    flushSync(() => {
      setToggleProgressById((prev) => ({ ...prev, [id]: 10 }));
    });

    // Schedule toggle AFTER the next paint — return now so the browser can paint
    requestAnimationFrame(() => {
      (async () => {
        let creepTimer: ReturnType<typeof setInterval> | null = null;
        try {
          creepTimer = setInterval(() => {
            setToggleProgressById((prev) => {
              const current = prev[id];
              if (current === undefined || current >= 90) return prev;
              const remaining = 90 - current;
              const next = Math.min(90, current + Math.max(1, remaining * 0.18));
              return { ...prev, [id]: next };
            });
          }, 120);

          await toggle(id, enabled);
        } finally {
          if (creepTimer) clearInterval(creepTimer);

          // Complete to 100% first, then remove the bar.
          setToggleProgressById((prev) => ({ ...prev, [id]: 100 }));
          setTimeout(() => {
            setToggleProgressById((prev) => {
              if (prev[id] === undefined) return prev;
              const next = { ...prev };
              delete next[id];
              return next;
            });
          }, 220);
        }
      })();
    });
  };

  // Load settings for tunnel defaults
  useEffect(() => {
    api.getSettings().then(setSettings).catch(() => {});
  }, []);

  // Poll tunnel statuses
  const pollTunnels = useCallback(async () => {
    try {
      const result = await api.listTunnels();
      const statusMap: Record<string, TunnelStatusResult> = {};
      for (const tunnel of result.tunnels) {
        // Find domain by name to get its ID
        const domain = domains.find((d) => d.name === tunnel.domain);
        if (domain) {
          statusMap[domain.id] = {
            active: true,
            public_url: tunnel.public_url,
            tunnel_type: tunnel.tunnel_type,
          };
        }
      }
      setTunnelStatuses(statusMap);
    } catch {
      // ignore polling errors
    }
  }, [domains]);

  useEffect(() => {
    pollTunnels();
    const interval = setInterval(pollTunnels, 3000);
    return () => clearInterval(interval);
  }, [pollTunnels]);

  const filteredDomains = domains.filter((d) => {
    if (searchQuery && !d.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
    if (domainFilter === "active" && !d.enabled) return false;
    if (domainFilter === "inactive" && d.enabled) return false;
    return true;
  });

  const handleEdit = (domain: Domain) => {
    setEditingDomain(domain);
    setShowModal(true);
  };

  // Open add modal when header "Add Domain" button is clicked
  const prevTrigger = useRef(addTrigger);
  useEffect(() => {
    if (addTrigger !== prevTrigger.current) {
      prevTrigger.current = addTrigger;
      setEditingDomain(null);
      setShowModal(true);
    }
  }, [addTrigger]);

  const handleAdd = () => {
    setEditingDomain(null);
    setShowModal(true);
  };

  const handleSave = async (request: CreateDomainRequest | UpdateDomainRequest) => {
    if ("id" in request) {
      await update(request);
    } else {
      await create(request);
    }
  };

  const handleDelete = (id: string) => {
    const domain = domains.find((d) => d.id === id);
    if (domain) setDeletingDomain(domain);
  };

  const confirmDelete = async () => {
    if (!deletingDomain) return;
    await remove(deletingDomain.id);
    setDeletingDomain(null);
  };

  const handleShare = (domain: Domain) => {
    setTunnelError(null);
    setSharingDomain(domain);
  };

  const handleStartTunnel = async (tunnelType: TunnelType) => {
    if (!sharingDomain) return;
    setTunnelLoading(true);
    setTunnelError(null);
    try {
      // Ensure cloudflared is installed for Cloudflare tunnels
      if (tunnelType.type === "QuickTunnel" || tunnelType.type === "NamedTunnel") {
        await track(api.ensureCloudflared());
      }
      await track(api.startTunnel({
        domain_id: sharingDomain.id,
        tunnel_type: tunnelType,
      }));
      setSharingDomain(null);
      setTunnelError(null);
      await pollTunnels();
    } catch (e: any) {
      const msg = typeof e === "string" ? e : e?.message || "Failed to start tunnel";
      setTunnelError(msg);
    } finally {
      setTunnelLoading(false);
    }
  };

  const handleStopTunnel = async (domainId: string) => {
    try {
      await track(api.stopTunnel(domainId));
      setTunnelStatuses((prev) => {
        const next = { ...prev };
        delete next[domainId];
        return next;
      });
    } catch (e) {
      console.error("Failed to stop tunnel:", e);
    }
  };

  if (loading) {
    return <div className="loading">Loading domains...</div>;
  }

  return (
    <div className="domain-list">
      {error && <div className="error-banner">{error}</div>}

      {filteredDomains.length === 0 && domains.length === 0 ? (
        <div className="empty-state">
          <p>No domains configured yet.</p>
          <p>Click "Add Domain" to get started.</p>
        </div>
      ) : filteredDomains.length === 0 ? (
        <div className="empty-state">
          <p>No domains matching "{searchQuery}"</p>
        </div>
      ) : (
        <div className="domain-grid">
          {filteredDomains.map((domain) => (
            <DomainCard
              key={domain.id}
              domain={domain}
              onToggle={handleToggle}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onInspect={(d) => onInspect(d.id, d.name, d.access_log)}
              onShare={handleShare}
              onStopTunnel={handleStopTunnel}
              tunnelStatus={tunnelStatuses[domain.id]}
              isToggling={toggleProgressById[domain.id] !== undefined && toggleProgressById[domain.id] < 100}
              toggleProgress={toggleProgressById[domain.id]}
            />
          ))}
          <button className="domain-card-add" onClick={handleAdd}>
            <div className="domain-card-add-icon">
              <PlusIcon />
            </div>
            <span>Configure a new domain</span>
          </button>
        </div>
      )}

      {showModal && (
        <DomainFormModal
          domain={editingDomain}
          onSave={handleSave}
          onClose={() => setShowModal(false)}
        />
      )}

      {deletingDomain && (
        <div className="modal-overlay" onClick={() => setDeletingDomain(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Remove Domain</h2>
              <button className="modal-close" onClick={() => setDeletingDomain(null)}>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
            <div className="modal-body">
              <p>Are you sure you want to remove <strong>{deletingDomain.name}</strong>? This will delete the domain configuration and remove it from your hosts file.</p>
            </div>
            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={() => setDeletingDomain(null)}>Cancel</button>
              <button className="btn btn-danger" onClick={confirmDelete}>Remove</button>
            </div>
          </div>
        </div>
      )}

      {sharingDomain && settings && (
        <TunnelModal
          domainId={sharingDomain.id}
          domainName={sharingDomain.name}
          targetPort={sharingDomain.target_port}
          savedSubdomain={sharingDomain.tunnel_subdomain}
          savedDomain={sharingDomain.tunnel_domain}
          settings={settings}
          onStart={handleStartTunnel}
          onClose={() => { setSharingDomain(null); setTunnelError(null); }}
          loading={tunnelLoading}
          error={tunnelError}
        />
      )}
    </div>
  );
}
