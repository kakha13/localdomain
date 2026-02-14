import type { View } from "../lib/types";
import { useServiceStatus } from "../hooks/useServiceStatus";

interface ServiceStatusBarProps {
  onNavigate?: (view: View) => void;
}

export function ServiceStatusBar({ onNavigate }: ServiceStatusBarProps) {
  const { status, loading, error, clearError } = useServiceStatus();

  if (loading) {
    return <div className="status-bar">Loading status...</div>;
  }

  return (
    <div className="status-bar-wrapper">
      {error && (
        <div className="status-error">
          <span>{error}</span>
          <button className="status-error-close" onClick={clearError}>&times;</button>
        </div>
      )}
      <div className="status-bar">
        <div className="status-indicators">
          <span className={`status-dot ${status.daemon_running ? "green" : "red"}`} />
          <span>Daemon: {status.daemon_running ? "Running" : "Stopped"}</span>

          <span className={`status-dot ${status.caddy_running ? "green" : "red"}`} />
          <span>Caddy: {status.caddy_running ? "Running" : "Stopped"}</span>

          {status.ca_installed && status.ca_trusted && (
            <>
              <span className="status-dot green" />
              <span>CA Trusted</span>
            </>
          )}
          {status.ca_installed && !status.ca_trusted && (
            <>
              <span className="status-dot yellow" />
              <span
                className="status-link"
                onClick={() => onNavigate?.("settings")}
              >
                CA Not Trusted
              </span>
            </>
          )}
        </div>

      </div>
    </div>
  );
}
