import { useState, useEffect, useCallback, useRef } from "react";
import type { ServiceStatus } from "../lib/types";
import * as api from "../lib/api";

type StepStatus = "pending" | "loading" | "done" | "error";

interface SetupScreenProps {
  onComplete: () => void;
  onSkip: () => void;
}

export function SetupScreen({ onComplete, onSkip }: SetupScreenProps) {
  const onCompleteRef = useRef(onComplete);
  onCompleteRef.current = onComplete;
  const [daemonStatus, setDaemonStatus] = useState<StepStatus>("pending");
  const [proxyStatus, setProxyStatus] = useState<StepStatus>("pending");
  const [domainStatus, setDomainStatus] = useState<StepStatus>("pending");
  const [error, setError] = useState<string | null>(null);

  const checkStatus = useCallback(async () => {
    try {
      const s: ServiceStatus = await api.getServiceStatus();
      if (s.daemon_running) setDaemonStatus("done");
      if (s.caddy_running) setProxyStatus("done");
    } catch {
      // daemon not reachable
    }

    try {
      const domains = await api.listDomains();
      if (domains.some((d) => d.name === "domain.local")) {
        setDomainStatus("done");
      }
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    checkStatus();
  }, [checkStatus]);

  // Auto-create default domain once proxy is ready
  useEffect(() => {
    if (proxyStatus === "done" && domainStatus === "pending") {
      createDefaultDomain();
    }
  }, [proxyStatus, domainStatus]);

  // Transition to main app when all steps complete
  useEffect(() => {
    if (daemonStatus === "done" && proxyStatus === "done" && domainStatus === "done") {
      const timer = setTimeout(() => onCompleteRef.current(), 600);
      return () => clearTimeout(timer);
    }
  }, [daemonStatus, proxyStatus, domainStatus]);

  const createDefaultDomain = async () => {
    setDomainStatus("loading");
    setError(null);
    try {
      const domains = await api.listDomains();
      if (!domains.some((d) => d.name === "domain.local")) {
        await api.createDomain({
          name: "domain.local",
          protocol: "http",
        });
      }
      setDomainStatus("done");
    } catch (e) {
      setError(String(e));
      setDomainStatus("error");
    }
  };

  const handleInstallDaemon = async () => {
    setDaemonStatus("loading");
    setError(null);
    try {
      await api.installDaemon();
      await new Promise((r) => setTimeout(r, 1500));
      setDaemonStatus("done");
      // Re-check to see if caddy also came up
      try {
        const s: ServiceStatus = await api.getServiceStatus();
        if (s.caddy_running) setProxyStatus("done");
      } catch {
        // ignore
      }
    } catch (e) {
      setError(String(e));
      setDaemonStatus("error");
    }
  };

  const handleStartProxy = async () => {
    setProxyStatus("loading");
    setError(null);
    try {
      await api.startService();
      setProxyStatus("done");
    } catch (e) {
      setError(String(e));
      setProxyStatus("error");
    }
  };

  const allDone =
    daemonStatus === "done" && proxyStatus === "done" && domainStatus === "done";

  return (
    <div className="setup-screen">
      <div className="setup-card">
        <div className="setup-header">
          <div>
            <h1 className="setup-title">Setup</h1>
            <p className="setup-desc">
              LocalDomain needs a few permissions to manage local domains on your machine.
            </p>
          </div>
          <button className="setup-close" onClick={onSkip} title="Skip setup">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div className="setup-steps">
          <SetupRow
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <rect x="2" y="2" width="20" height="8" rx="2" ry="2" />
                <rect x="2" y="14" width="20" height="8" rx="2" ry="2" />
                <line x1="6" y1="6" x2="6.01" y2="6" />
                <line x1="6" y1="18" x2="6.01" y2="18" />
              </svg>
            }
            title="Background Service"
            description="Manages /etc/hosts and runs the reverse proxy. Requires admin password."
            status={daemonStatus}
            onAction={handleInstallDaemon}
            actionLabel="Install"
            disabled={false}
          />

          <SetupRow
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
            }
            title="Reverse Proxy"
            description="Routes custom domains to your local dev servers."
            status={proxyStatus}
            onAction={handleStartProxy}
            actionLabel="Start"
            disabled={daemonStatus !== "done"}
          />

          <SetupRow
            icon={
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="12" cy="12" r="10" />
                <line x1="2" y1="12" x2="22" y2="12" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              </svg>
            }
            title="Default Domain"
            description="Creates domain.local → 127.0.0.1 with HTTP & HTTPS."
            status={domainStatus}
            onAction={createDefaultDomain}
            actionLabel="Create"
            disabled={proxyStatus !== "done"}
          />
        </div>

        {error && <div className="setup-error-text">{error}</div>}

        {allDone && (
          <div className="setup-done-msg">All set! Opening app...</div>
        )}
      </div>
    </div>
  );
}

function SetupRow({
  icon,
  title,
  description,
  status,
  onAction,
  actionLabel,
  disabled,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  status: StepStatus;
  onAction: () => void;
  actionLabel: string;
  disabled: boolean;
}) {
  return (
    <div className={`setup-row ${status === "done" ? "setup-row-done" : ""}`}>
      <div className="setup-row-icon">{icon}</div>
      <div className="setup-row-info">
        <div className="setup-row-title">{title}</div>
        <div className="setup-row-desc">{description}</div>
      </div>
      <div className="setup-row-action">
        {status === "done" ? (
          <span className="setup-check">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </span>
        ) : status === "loading" ? (
          <div className="spinner" />
        ) : (
          <button
            className="btn btn-primary btn-sm"
            onClick={onAction}
            disabled={disabled}
          >
            {status === "error" ? "Retry" : actionLabel}
          </button>
        )}
      </div>
    </div>
  );
}
