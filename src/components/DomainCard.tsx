import { useState, useRef, useEffect } from "react";
import { open } from "@tauri-apps/plugin-shell";
import type { Domain, TunnelStatusResult } from "../lib/types";
import {
  MonitorIcon,
  GlobeIcon,
  ExternalLinkIcon,
  CopyIcon,
  EditIcon,
  TrashIcon,
  XamppIcon,
} from "./Icons";

interface DomainCardProps {
  domain: Domain;
  onToggle: (id: string, enabled: boolean) => void;
  onEdit: (domain: Domain) => void;
  onDelete: (id: string) => void;
  onInspect: (domain: Domain) => void;
  onShare: (domain: Domain) => void;
  onStopTunnel: (domainId: string) => void;
  tunnelStatus?: TunnelStatusResult;
  isToggling?: boolean;
  toggleProgress?: number;
}

export function DomainCard({
  domain,
  onToggle,
  onEdit,
  onDelete,
  onInspect,
  onShare,
  onStopTunnel,
  tunnelStatus,
  isToggling
}: DomainCardProps) {
  const hasProxy = domain.target_port > 0;
  const protocol =
    domain.protocol === "https" || domain.protocol === "both"
      ? "https"
      : "http";
  const domainUrl = `${protocol}://${domain.name}`;
  const [tunnelMenuOpen, setTunnelMenuOpen] = useState(false);
  const tunnelMenuRef = useRef<HTMLDivElement>(null);
  const isTunnelActive = tunnelStatus?.active && tunnelStatus.public_url;

  // Close menus on outside click
  useEffect(() => {
    if (!tunnelMenuOpen) return;
    const handleClick = (e: MouseEvent) => {
      if (
        tunnelMenuOpen &&
        tunnelMenuRef.current &&
        !tunnelMenuRef.current.contains(e.target as Node)
      ) {
        setTunnelMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [tunnelMenuOpen]);

  const handleOpenDomain = (e: React.MouseEvent) => {
    e.preventDefault();
    open(domainUrl);
  };

  const handleCopyUrl = async (url: string) => {
    try {
      await navigator.clipboard.writeText(url);
    } catch {
      // ignore
    }
  };


  const isXampp = domain.domain_type === "xampp";
  const targetDisplay = isXampp
    ? domain.document_root
    : hasProxy
      ? `${domain.target_host}:${domain.target_port}`
      : "hosts only";

  return (
    <div
      className={`domain-card ${domain.enabled ? "" : "disabled-card"}`}
    >
      

      {/* Header: Icon + Name + Target + Badge + Toggle */}
      <div className="domain-card-header">
        <div className="domain-card-left">
          <div className="domain-card-icon">
            {isXampp ? <XamppIcon /> : isTunnelActive ? <GlobeIcon size={20} /> : <MonitorIcon />}
          </div>
          <div className="domain-card-info">
            <div className="domain-card-name">
              <a href={domainUrl} onClick={handleOpenDomain}>
                {domain.name}
              </a>
            </div>
            <div className="domain-card-target">{targetDisplay}</div>
          </div>
        </div>
        <div className="domain-card-right">
          <label className={`toggle-switch ${isToggling ? "toggling" : ""}`}>
            <input
              type="checkbox"
              checked={domain.enabled}
              onChange={() => onToggle(domain.id, !domain.enabled)}
              disabled={isToggling}
            />
            <span className="toggle-slider" />
          </label>
        </div>
      </div>

      {/* Tunnel pill (if active) */}
      {isTunnelActive && (
        <div
          className="tunnel-pill-wrapper"
          ref={tunnelMenuRef}
          style={{ marginBottom: 8 }}
        >
          <button
            className="tunnel-pill"
            onClick={() => setTunnelMenuOpen(!tunnelMenuOpen)}
            title={tunnelStatus.public_url}
          >
            <svg
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="12" cy="12" r="10" />
              <line x1="2" y1="12" x2="22" y2="12" />
              <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            <span className="tunnel-pill-url">
              {tunnelStatus.public_url!.replace(/^https?:\/\//, "")}
            </span>
            <svg
              width="10"
              height="10"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </button>
          {tunnelMenuOpen && (
            <div className="dropdown-menu">
              <button
                onClick={() => {
                  open(tunnelStatus.public_url!);
                  setTunnelMenuOpen(false);
                }}
              >
                Open in browser
              </button>
              <button
                onClick={() => {
                  handleCopyUrl(tunnelStatus.public_url!);
                  setTunnelMenuOpen(false);
                }}
              >
                Copy URL
              </button>
              <button
                className="dropdown-danger"
                onClick={() => {
                  onStopTunnel(domain.id);
                  setTunnelMenuOpen(false);
                }}
              >
                Stop sharing
              </button>
            </div>
          )}
        </div>
      )}

      {/* Footer: Action buttons + Share/Remove */}
      <div className="domain-card-meta">
        <div className="domain-card-actions">
          <button
            className="card-action-btn"
            onClick={handleOpenDomain}
            title="Open in browser"
          >
            <ExternalLinkIcon />
          </button>
          <button
            className="card-action-btn"
            onClick={() => handleCopyUrl(domainUrl)}
            title="Copy URL"
          >
            <CopyIcon />
          </button>
          <button
            className="card-action-btn"
            onClick={() => onEdit(domain)}
            title="Edit"
          >
            <EditIcon />
          </button>
          <button
            className="card-action-btn"
            onClick={() => onInspect(domain)}
            title="Inspect"
          >
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
          </button>
          {(hasProxy || domain.domain_type === "xampp") && !isTunnelActive && (
            <button
              className="card-action-btn"
              onClick={() => onShare(domain)}
              title="Share via tunnel"
            >
              <svg
                width="15"
                height="15"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="2" y1="12" x2="22" y2="12" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              </svg>
            </button>
          )}
        </div>
        <button
          className="card-remove-btn"
          onClick={() => onDelete(domain.id)}
        >
          <TrashIcon />
        </button>
      </div>
    </div>
  );
}
