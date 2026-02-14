import { useAuditLog } from "../hooks/useAuditLog";
import type { AuditLogEntry } from "../lib/types";
import { useLoading } from "../hooks/useLoading";
import * as api from "../lib/api";

function formatAction(action: string): { label: string; className: string } {
  const map: Record<string, { label: string; className: string }> = {
    domain_created: { label: "Created", className: "audit-badge audit-badge-created" },
    domain_updated: { label: "Updated", className: "audit-badge audit-badge-updated" },
    domain_deleted: { label: "Deleted", className: "audit-badge audit-badge-deleted" },
    domain_enabled: { label: "Enabled", className: "audit-badge audit-badge-enabled" },
    domain_disabled: { label: "Disabled", className: "audit-badge audit-badge-disabled" },
    access_log_enabled: { label: "Log On", className: "audit-badge audit-badge-enabled" },
    access_log_disabled: { label: "Log Off", className: "audit-badge audit-badge-disabled" },
  };
  return map[action] ?? { label: action, className: "audit-badge" };
}

function formatTimestamp(raw: string): string {
  try {
    // SQLite datetime('now') stores UTC without timezone indicator.
    // Append 'Z' so JS interprets it as UTC, then display in local time.
    const normalized = raw.includes("Z") || raw.includes("+") ? raw : raw.replace(" ", "T") + "Z";
    const d = new Date(normalized);
    if (isNaN(d.getTime())) return raw;
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    if (diffMin < 1) return "Just now";
    if (diffMin < 60) return `${diffMin}m ago`;
    const diffHrs = Math.floor(diffMin / 60);
    if (diffHrs < 24) return `${diffHrs}h ago`;
    const diffDays = Math.floor(diffHrs / 24);
    if (diffDays < 7) return `${diffDays}d ago`;
    return d.toLocaleDateString();
  } catch {
    return raw;
  }
}

function formatFullTimestamp(raw: string): string {
  try {
    const normalized = raw.includes("Z") || raw.includes("+") ? raw : raw.replace(" ", "T") + "Z";
    const d = new Date(normalized);
    if (isNaN(d.getTime())) return raw;
    return d.toLocaleString();
  } catch {
    return raw;
  }
}

function formatDetails(entry: AuditLogEntry): string | null {
  if (!entry.details) return null;
  try {
    const parsed = JSON.parse(entry.details);
    if (parsed.name) return parsed.name;
    if (parsed.domain) return parsed.domain;
    return null;
  } catch {
    return entry.details;
  }
}

export function AuditLogView() {
  const { entries, loading, hasMore, refresh, loadMore } = useAuditLog();
  const { track } = useLoading();

  const handleClear = async () => {
    await track(api.clearAuditLog());
    refresh();
  };

  if (loading) {
    return <div className="loading">Loading audit log...</div>;
  }

  return (
    <div className="audit-log-view">
      <div className="audit-log-header">
        {entries.length > 0 && (
          <button className="btn btn-sm btn-danger" onClick={handleClear}>
            Clear All
          </button>
        )}
      </div>

      {entries.length === 0 ? (
        <div className="empty-state">
          <p>No audit log entries yet.</p>
        </div>
      ) : (
        <>
          <div className="audit-list">
            {entries.map((entry) => {
              const { label, className } = formatAction(entry.action);
              const detail = formatDetails(entry);
              return (
                <div key={entry.id} className="audit-entry">
                  <span className={className}>{label}</span>
                  <span className="audit-entry-detail">
                    {detail || <span className="text-muted">-</span>}
                  </span>
                  <span className="audit-entry-time" title={formatFullTimestamp(entry.created_at)}>
                    {formatTimestamp(entry.created_at)}
                  </span>
                </div>
              );
            })}
          </div>

          {hasMore && (
            <button className="btn audit-load-more" onClick={loadMore}>
              Load More
            </button>
          )}
        </>
      )}
    </div>
  );
}
