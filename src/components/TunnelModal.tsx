import { useState, useEffect } from "react";
import type { TunnelType, AppSettings } from "../lib/types";
import { useLoading } from "../hooks/useLoading";
import { cloudflareCheckLogin, cloudflareLogin, cloudflareSetupTunnel, saveTunnelConfig } from "../lib/api";

interface TunnelModalProps {
  domainId: string;
  domainName: string;
  targetPort: number;
  savedSubdomain: string;
  savedDomain: string;
  settings: AppSettings;
  onStart: (tunnelType: TunnelType) => void;
  onClose: () => void;
  loading: boolean;
  error?: string | null;
}

type TunnelTab = "quick" | "named" | "ssh";
type NamedMode = "auto" | "manual";
type AutoState = "idle" | "checking" | "logging_in" | "logged_in" | "setting_up" | "error";

export function TunnelModal({ domainId, domainName, targetPort, savedSubdomain, savedDomain, settings, onStart, onClose, loading, error }: TunnelModalProps) {
  const { track } = useLoading();
  const [tab, setTab] = useState<TunnelTab>("quick");

  // Named tunnel — mode toggle
  const [namedMode, setNamedMode] = useState<NamedMode>("auto");

  // Named tunnel — automatic mode
  const [autoState, setAutoState] = useState<AutoState>("idle");
  const [autoError, setAutoError] = useState<string | null>(null);
  const [autoSubdomain, setAutoSubdomain] = useState(savedSubdomain);
  const [autoDomain, setAutoDomain] = useState(savedDomain);

  // Named tunnel — manual mode
  const [namedToken, setNamedToken] = useState(settings.cloudflare_tunnel_token || "");
  const [namedSubdomain, setNamedSubdomain] = useState("");
  const [namedDomain, setNamedDomain] = useState("");

  // SSH tunnel
  const [sshHost, setSshHost] = useState(settings.default_ssh_host || "");
  const [sshPort, setSshPort] = useState(22);
  const [sshUser, setSshUser] = useState(settings.default_ssh_user || "");
  const [sshKey, setSshKey] = useState(settings.default_ssh_key_path || "");
  const [sshRemotePort, setSshRemotePort] = useState(8080);

  // Check Cloudflare login status when switching to auto mode
  useEffect(() => {
    if (tab === "named" && namedMode === "auto" && autoState === "idle") {
      setAutoState("checking");
      cloudflareCheckLogin()
        .then((res) => {
          setAutoState(res.logged_in ? "logged_in" : "idle");
        })
        .catch(() => {
          setAutoState("idle");
        });
    }
  }, [tab, namedMode]);

  const handleCloudflareLogin = async () => {
    setAutoState("logging_in");
    setAutoError(null);
    try {
      const result = await track(cloudflareLogin());
      if (result.logged_in) {
        setAutoState("logged_in");
      } else {
        setAutoState("error");
        setAutoError("Authentication was not completed. Please try again.");
      }
    } catch (e: any) {
      setAutoState("error");
      setAutoError(e?.toString() || "Login failed");
    }
  };

  const handleAutoSetup = async () => {
    setAutoState("setting_up");
    setAutoError(null);
    try {
      const result = await track(cloudflareSetupTunnel({
        subdomain: autoSubdomain,
        domain: autoDomain,
      }));
      // Save subdomain/domain for next time
      saveTunnelConfig(domainId, autoSubdomain, autoDomain).catch(() => {});
      // Got token + credentials — start tunnel via existing flow (config-file mode)
      onStart({
        type: "NamedTunnel",
        token: result.token,
        subdomain: autoSubdomain,
        cloudflare_domain: autoDomain,
        credentials_json: result.credentials_json,
        tunnel_uuid: result.tunnel_id,
      });
    } catch (e: any) {
      setAutoState("error");
      setAutoError(e?.toString() || "Setup failed");
    }
  };

  const handleStart = () => {
    if (tab === "quick") {
      onStart({ type: "QuickTunnel" });
    } else if (tab === "named") {
      if (namedMode === "manual") {
        onStart({
          type: "NamedTunnel",
          token: namedToken,
          subdomain: namedSubdomain,
          cloudflare_domain: namedDomain,
        });
      }
      // Auto mode uses handleAutoSetup directly
    } else {
      onStart({
        type: "SshTunnel",
        host: sshHost,
        port: sshPort,
        user: sshUser,
        key: sshKey,
        remote_port: sshRemotePort,
      });
    }
  };

  const canStart =
    tab === "quick" ||
    (tab === "named" && namedMode === "manual" && namedToken && namedSubdomain && namedDomain) ||
    (tab === "ssh" && sshHost && sshUser && sshRemotePort > 0);

  const autoCanSetup = autoSubdomain && autoDomain;
  const isAutoWorking = autoState === "logging_in" || autoState === "setting_up" || autoState === "checking";

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Share {domainName}</h2>
          <button className="modal-close" onClick={onClose}>&times;</button>
        </div>

        <div className="tunnel-type-selector">
          <button
            className={`tunnel-type-option ${tab === "quick" ? "active" : ""}`}
            onClick={() => setTab("quick")}
          >
            <strong>Quick Tunnel</strong>
            <span>Free trycloudflare.com URL</span>
          </button>
          <button
            className={`tunnel-type-option ${tab === "named" ? "active" : ""}`}
            onClick={() => setTab("named")}
          >
            <strong>Named Tunnel</strong>
            <span>Custom Cloudflare domain</span>
          </button>
          <button
            className={`tunnel-type-option ${tab === "ssh" ? "active" : ""}`}
            onClick={() => setTab("ssh")}
          >
            <strong>SSH Tunnel</strong>
            <span>Reverse SSH to your VPS</span>
          </button>
        </div>

        {tab === "quick" && (
          <div className="tunnel-config">
            <p className="form-hint" style={{ marginTop: 8, marginBottom: 0 }}>
              Creates a free public URL via Cloudflare's Quick Tunnel service.
              No account needed. URL changes on every restart.
            </p>
          </div>
        )}

        {tab === "named" && (
          <div className="tunnel-config">
            <div className="named-mode-toggle">
              <button
                className={`named-mode-btn ${namedMode === "auto" ? "active" : ""}`}
                onClick={() => setNamedMode("auto")}
              >
                Automatic
              </button>
              <button
                className={`named-mode-btn ${namedMode === "manual" ? "active" : ""}`}
                onClick={() => setNamedMode("manual")}
              >
                Manual (Token)
              </button>
            </div>

            {namedMode === "auto" && (
              <div className="auto-setup-section">
                {autoState === "idle" && (
                  <div className="auto-setup-status">
                    <p className="form-hint" style={{ marginTop: 8 }}>
                      Sign in to your Cloudflare account to automatically create and configure a tunnel.
                    </p>
                    <button className="btn btn-cloudflare" onClick={handleCloudflareLogin}>
                      Sign in to Cloudflare
                    </button>
                  </div>
                )}

                {autoState === "checking" && (
                  <div className="auto-setup-status">
                    <div className="spinner" style={{ margin: "16px auto" }} />
                    <p className="form-hint" style={{ textAlign: "center" }}>Checking login status...</p>
                  </div>
                )}

                {autoState === "logging_in" && (
                  <div className="auto-setup-status">
                    <div className="spinner" style={{ margin: "16px auto" }} />
                    <p className="form-hint" style={{ textAlign: "center" }}>
                      Waiting for Cloudflare authentication...
                    </p>
                    <p className="form-hint" style={{ textAlign: "center", marginTop: 4 }}>
                      A browser window should have opened. Complete the login there.
                    </p>
                  </div>
                )}

                {autoState === "logged_in" && (
                  <div>
                    <div className="login-success">
                      <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
                        <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0zm3.41 5.09L7 9.5 4.59 7.09 3.17 8.5 7 12.33l5.83-5.83-1.42-1.41z"/>
                      </svg>
                      Signed in to Cloudflare
                    </div>
                    <div className="form-row">
                      <div className="form-group">
                        <label>Subdomain</label>
                        <input
                          type="text"
                          value={autoSubdomain}
                          onChange={(e) => setAutoSubdomain(e.target.value)}
                          placeholder="app"
                        />
                      </div>
                      <div className="form-group">
                        <label>Domain</label>
                        <input
                          type="text"
                          value={autoDomain}
                          onChange={(e) => setAutoDomain(e.target.value)}
                          placeholder="yourdomain.com"
                        />
                      </div>
                    </div>
                    {autoSubdomain && autoDomain && (
                      <div className="tunnel-url-preview">
                        Your site will be at <strong>https://{autoSubdomain}.{autoDomain}</strong>
                      </div>
                    )}
                  </div>
                )}

                {autoState === "setting_up" && (
                  <div className="auto-setup-status">
                    <div className="spinner" style={{ margin: "16px auto" }} />
                    <p className="form-hint" style={{ textAlign: "center" }}>
                      Creating tunnel and configuring DNS...
                    </p>
                  </div>
                )}

                {autoState === "error" && (
                  <div>
                    {autoError && <div className="form-error" style={{ marginBottom: 12 }}>{autoError}</div>}
                    <button
                      className="btn"
                      onClick={() => {
                        setAutoError(null);
                        setAutoState("idle");
                      }}
                    >
                      Retry
                    </button>
                  </div>
                )}
              </div>
            )}

            {namedMode === "manual" && (
              <div>
                <div className="tunnel-setup-guide">
                  <p className="tunnel-guide-title">First, configure in Cloudflare:</p>
                  <ol>
                    <li>
                      Open <a href="https://one.dash.cloudflare.com" target="_blank" rel="noopener noreferrer">Cloudflare Zero Trust</a> &rarr; <strong>Networks</strong> &rarr; <strong>Tunnels</strong>
                    </li>
                    <li>Create a tunnel (or use existing) &rarr; copy the <strong>token</strong></li>
                    <li>
                      Go to <strong>Public Hostname</strong> tab &rarr; <strong>Add a hostname</strong>
                    </li>
                    <li>
                      Set subdomain to <strong>{namedSubdomain || "<subdomain>"}</strong>,
                      domain to <strong>{namedDomain || "<your domain>"}</strong>
                    </li>
                    <li>
                      Set type to <strong>HTTP</strong>, URL to <strong>localhost:{targetPort}</strong>
                    </li>
                  </ol>
                </div>

                <p className="tunnel-guide-title" style={{ marginTop: 12 }}>Then, fill in below:</p>
                <div className="form-row">
                  <div className="form-group">
                    <label>Subdomain</label>
                    <input
                      type="text"
                      value={namedSubdomain}
                      onChange={(e) => setNamedSubdomain(e.target.value)}
                      placeholder="app"
                    />
                  </div>
                  <div className="form-group">
                    <label>Cloudflare Domain</label>
                    <input
                      type="text"
                      value={namedDomain}
                      onChange={(e) => setNamedDomain(e.target.value)}
                      placeholder="yourdomain.com"
                    />
                  </div>
                </div>
                <div className="form-group">
                  <label>Tunnel Token</label>
                  <input
                    type="password"
                    value={namedToken}
                    onChange={(e) => setNamedToken(e.target.value)}
                    placeholder="eyJhIjoiN..."
                  />
                </div>
                {namedSubdomain && namedDomain && (
                  <div className="tunnel-url-preview">
                    Your site will be at <strong>https://{namedSubdomain}.{namedDomain}</strong>
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {tab === "ssh" && (
          <div className="tunnel-config">
            <div className="form-row">
              <div className="form-group">
                <label>Host</label>
                <input
                  type="text"
                  value={sshHost}
                  onChange={(e) => setSshHost(e.target.value)}
                  placeholder="vps.example.com"
                />
              </div>
              <div className="form-group" style={{ maxWidth: 100 }}>
                <label>Port</label>
                <input
                  type="number"
                  value={sshPort}
                  onChange={(e) => setSshPort(parseInt(e.target.value, 10) || 22)}
                  min="1"
                  max="65535"
                />
              </div>
            </div>
            <div className="form-row">
              <div className="form-group">
                <label>User</label>
                <input
                  type="text"
                  value={sshUser}
                  onChange={(e) => setSshUser(e.target.value)}
                  placeholder="root"
                />
              </div>
              <div className="form-group" style={{ maxWidth: 120 }}>
                <label>Remote Port</label>
                <input
                  type="number"
                  value={sshRemotePort}
                  onChange={(e) => setSshRemotePort(parseInt(e.target.value, 10) || 8080)}
                  min="1"
                  max="65535"
                />
              </div>
            </div>
            <div className="form-group">
              <label>SSH Key Path <span className="form-optional">(optional)</span></label>
              <input
                type="text"
                value={sshKey}
                onChange={(e) => setSshKey(e.target.value)}
                placeholder="~/.ssh/id_rsa"
              />
            </div>
          </div>
        )}

        {error && <div className="form-error">{error}</div>}

        <div className="modal-actions">
          <button className="btn" onClick={onClose}>Cancel</button>
          {tab === "named" && namedMode === "auto" && autoState === "logged_in" ? (
            <button
              className="btn btn-primary"
              onClick={handleAutoSetup}
              disabled={isAutoWorking || !autoCanSetup}
            >
              Configure & Start
            </button>
          ) : (
            <button
              className="btn btn-primary"
              onClick={handleStart}
              disabled={loading || !canStart || (tab === "named" && namedMode === "auto")}
            >
              {loading ? "Starting..." : "Start Sharing"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
