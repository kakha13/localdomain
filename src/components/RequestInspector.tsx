import { useState } from "react";
import type { AccessLogEntry } from "../lib/types";
import { useAccessLog } from "../hooks/useAccessLog";
import { useLoading } from "../hooks/useLoading";
import * as api from "../lib/api";

interface RequestInspectorProps {
  domainId: string;
  domain: string;
  initialAccessLog: boolean;
  onBack: () => void;
}

function formatDuration(seconds: number): string {
  if (seconds < 0.001) return `${(seconds * 1_000_000).toFixed(0)}us`;
  if (seconds < 1) return `${(seconds * 1000).toFixed(0)}ms`;
  return `${seconds.toFixed(2)}s`;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "-";
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}

function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString();
}

function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

function statusClass(status: number): string {
  if (status >= 500) return "status-5xx";
  if (status >= 400) return "status-4xx";
  if (status >= 300) return "status-3xx";
  return "status-2xx";
}

function parseQueryParams(uri: string): [string, string][] {
  const qIndex = uri.indexOf("?");
  if (qIndex === -1) return [];
  const search = uri.slice(qIndex + 1);
  const params: [string, string][] = [];
  for (const part of search.split("&")) {
    const eqIndex = part.indexOf("=");
    if (eqIndex === -1) {
      params.push([decodeURIComponent(part), ""]);
    } else {
      params.push([
        decodeURIComponent(part.slice(0, eqIndex)),
        decodeURIComponent(part.slice(eqIndex + 1)),
      ]);
    }
  }
  return params;
}

function HeadersTable({ headers }: { headers: Record<string, string[]> | null }) {
  if (!headers || typeof headers !== "object") return null;
  const entries = Object.entries(headers);
  if (entries.length === 0) return null;

  return (
    <table className="detail-kv-table">
      <tbody>
        {entries.map(([key, values]) => (
          <tr key={key}>
            <td className="detail-kv-key">{key}</td>
            <td className="detail-kv-value">{Array.isArray(values) ? values.join(", ") : String(values)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function DetailPanel({ entry }: { entry: AccessLogEntry }) {
  const queryParams = parseQueryParams(entry.uri);

  return (
    <div className="detail-panel-content">
      <div className="detail-section">
        <h4 className="detail-section-title">General</h4>
        <table className="detail-kv-table">
          <tbody>
            <tr>
              <td className="detail-kv-key">Method</td>
              <td className="detail-kv-value">{entry.method}</td>
            </tr>
            <tr>
              <td className="detail-kv-key">URI</td>
              <td className="detail-kv-value">{entry.uri}</td>
            </tr>
            <tr>
              <td className="detail-kv-key">Status</td>
              <td className="detail-kv-value">
                <span className={`inspector-status ${statusClass(entry.status)}`}>{entry.status}</span>
              </td>
            </tr>
            <tr>
              <td className="detail-kv-key">Duration</td>
              <td className="detail-kv-value">{formatDuration(entry.duration)}</td>
            </tr>
            <tr>
              <td className="detail-kv-key">Size</td>
              <td className="detail-kv-value">{formatSize(entry.size)}</td>
            </tr>
            {entry.remote_ip && (
              <tr>
                <td className="detail-kv-key">Remote IP</td>
                <td className="detail-kv-value">{entry.remote_ip}</td>
              </tr>
            )}
            {entry.proto && (
              <tr>
                <td className="detail-kv-key">Protocol</td>
                <td className="detail-kv-value">{entry.proto}</td>
              </tr>
            )}
            <tr>
              <td className="detail-kv-key">Timestamp</td>
              <td className="detail-kv-value">{formatTimestamp(entry.timestamp)}</td>
            </tr>
          </tbody>
        </table>
      </div>

      {queryParams.length > 0 && (
        <div className="detail-section">
          <h4 className="detail-section-title">Query Parameters</h4>
          <table className="detail-kv-table">
            <tbody>
              {queryParams.map(([key, value], i) => (
                <tr key={i}>
                  <td className="detail-kv-key">{key}</td>
                  <td className="detail-kv-value">{value}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <div className="detail-section">
        <h4 className="detail-section-title">Request Headers</h4>
        <HeadersTable headers={entry.headers} />
        {(!entry.headers || Object.keys(entry.headers).length === 0) && (
          <p className="detail-empty">No request headers captured</p>
        )}
      </div>

      <div className="detail-section">
        <h4 className="detail-section-title">Response Headers</h4>
        <HeadersTable headers={entry.resp_headers} />
        {(!entry.resp_headers || Object.keys(entry.resp_headers).length === 0) && (
          <p className="detail-empty">No response headers captured</p>
        )}
      </div>
    </div>
  );
}

export function RequestInspector({ domainId, domain, initialAccessLog, onBack }: RequestInspectorProps) {
  const { entries, loading, clear, refresh } = useAccessLog(domain);
  const { track } = useLoading();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [accessLog, setAccessLog] = useState(initialAccessLog);

  const selectedEntry = selectedIndex !== null ? entries[selectedIndex] ?? null : null;

  const handleToggleAccessLog = async () => {
    try {
      const updated = await track(api.toggleAccessLog(domainId, !accessLog));
      setAccessLog(updated.access_log);
    } catch {
      // ignore
    }
  };

  return (
    <div className="inspector-fullpage">
      <div className="inspector-header">
        <div className="inspector-header-left">
          <button className="inspector-back-btn" onClick={onBack} title="Back to domains">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </button>
          <h3>Inspect<span className="inspector-separator">/</span><span className="inspector-domain-name">{domain}</span></h3>
        </div>
        <div className="inspector-actions">
          <label className="toggle-switch" title={accessLog ? "Logging enabled" : "Logging disabled"}>
            <input
              type="checkbox"
              checked={accessLog}
              onChange={handleToggleAccessLog}
            />
            <span className="toggle-slider" />
          </label>
          <button className="btn btn-sm" onClick={() => track(refresh())}>Reload</button>
          <button className="btn btn-sm" onClick={() => track(clear())}>Clear</button>
        </div>
      </div>

      {loading ? (
        <div className="loading">Loading access log...</div>
      ) : !accessLog && entries.length === 0 ? (
        <div className="empty-state">
          <p>Access logging is disabled.</p>
          <p>Enable the toggle above to start capturing requests.</p>
        </div>
      ) : entries.length === 0 ? (
        <div className="empty-state">
          <p>No requests captured yet.</p>
          <p>Make HTTP requests to {domain} and they will appear here.</p>
        </div>
      ) : (
        <div className="inspector-panels">
          <div className="inspector-list-panel">
            <div className="inspector-table-wrapper">
              <table className="inspector-table">
                <thead>
                  <tr>
                    <th>Time</th>
                    <th>Method</th>
                    <th>Path</th>
                    <th>Status</th>
                    <th>Duration</th>
                    <th>Size</th>
                  </tr>
                </thead>
                <tbody>
                  {entries.map((entry, i) => (
                    <tr
                      key={i}
                      className={`inspector-row-clickable${selectedIndex === i ? " inspector-row-selected" : ""}`}
                      onClick={() => setSelectedIndex(i)}
                    >
                      <td className="inspector-time">{formatTime(entry.timestamp)}</td>
                      <td>
                        <span className="method-badge">{entry.method}</span>
                      </td>
                      <td className="inspector-path">{entry.uri}</td>
                      <td>
                        <span className={`inspector-status ${statusClass(entry.status)}`}>
                          {entry.status}
                        </span>
                      </td>
                      <td className="inspector-duration">{formatDuration(entry.duration)}</td>
                      <td className="inspector-size">{formatSize(entry.size)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="inspector-detail-panel">
            {selectedEntry ? (
              <DetailPanel entry={selectedEntry} />
            ) : (
              <div className="detail-placeholder">
                <p>Select a request to view details</p>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
