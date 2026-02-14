import { useEffect, useState, type ReactNode } from 'react'

type ViewId = 'domains' | 'settings' | 'audit' | 'about' | 'xampp'

const VIEWS: ViewId[] = ['domains', 'settings', 'audit', 'about', 'xampp']
const AUTO_MS = 5000

const DOMAIN_SAMPLES = [
  { name: 'dev.local', target: '127.0.0.1:3000', icon: 'monitor' as const, enabled: true },
  { name: 'domain.local', target: 'hosts only', icon: 'monitor' as const, enabled: true },
  { name: 'kakha13.local', target: '/Applications/XAMPP/xamppfiles/...', icon: 'xampp' as const, enabled: true },
  { name: 'shorty.local', target: '127.0.0.1:8501', icon: 'monitor' as const, enabled: false },
]

export default function AppPreviewInteractive() {
  const [activeIndex, setActiveIndex] = useState(0)
  const [autoplay, setAutoplay] = useState(true)

  const activeView = VIEWS[activeIndex]

  useEffect(() => {
    if (!autoplay) return
    const id = window.setTimeout(() => {
      setActiveIndex((prev) => (prev + 1) % VIEWS.length)
    }, AUTO_MS)
    return () => window.clearTimeout(id)
  }, [activeIndex, autoplay])

  const goTo = (index: number, byUser = false) => {
    if (byUser) setAutoplay(false)
    setActiveIndex((index + VIEWS.length) % VIEWS.length)
  }

  return (
    <div
      className="ld-app-clone"
      role="region"
      aria-label="LocalDomain application preview"
      onClickCapture={() => {
        if (autoplay) setAutoplay(false)
      }}
    >
      <div className="ld-app-titlebar">
        <span className="ld-app-dot"></span>
        <span className="ld-app-dot"></span>
        <span className="ld-app-dot"></span>
        <span className="ld-app-titlebar-text">LocalDomain</span>
      </div>

      <div className="ld-app-body">
        <aside className="ld-app-sidebar">
          <div className="ld-app-sidebar-brand">
            <div className="ld-app-logo-wrap">
              <img src="/logo.svg" alt="LocalDomain" className="ld-app-logo" />
            </div>
            <span className="ld-app-brand-title">LocalDomain</span>
          </div>

          <ul className="ld-app-sidebar-nav">
            <li>
              <button className={activeView === 'domains' ? 'active' : ''} onClick={() => goTo(0, true)}>
                <GlobeIcon />
                <span>Domains</span>
              </button>
            </li>
            <li>
              <button className={activeView === 'settings' ? 'active' : ''} onClick={() => goTo(1, true)}>
                <SettingsIcon />
                <span>Settings</span>
              </button>
            </li>
            <li>
              <button className={activeView === 'audit' ? 'active' : ''} onClick={() => goTo(2, true)}>
                <ListIcon />
                <span>Audit Log</span>
              </button>
            </li>
            <li>
              <button className={activeView === 'about' ? 'active' : ''} onClick={() => goTo(3, true)}>
                <InfoIcon />
                <span>About</span>
              </button>
            </li>
            <li>
              <div className="ld-app-nav-section">Services</div>
              <button className={activeView === 'xampp' ? 'active' : ''} onClick={() => goTo(4, true)}>
                <XamppIcon size={18} />
                <span>XAMPP</span>
              </button>
            </li>
          </ul>

          <div className="ld-app-sidebar-footer">
            <button>
              <ChevronLeftIcon />
              <span>Collapse</span>
            </button>
            <button>
              <SunIcon />
              <span>Light Mode</span>
            </button>
          </div>
        </aside>

        <main className="ld-app-main-content">
          <div className="ld-app-header-bar">
            <div className="ld-app-header-left">
              <h3>{titleFor(activeView)}</h3>
              {activeView === 'domains' && (
                <div className="ld-app-header-subtitle">
                  <span className="ld-app-status-dot"></span>
                  <span>SYSTEM ACTIVE</span>
                </div>
              )}
            </div>

            {activeView === 'domains' && (
              <div className="ld-app-header-right">
                <div className="ld-app-search">
                  <SearchIcon />
                  <span>Search domains...</span>
                </div>
                <div className="ld-app-filter">
                  <button className="active">All</button>
                  <button>Active</button>
                  <button>Inactive</button>
                </div>
                <button className="ld-app-add-btn" aria-label="Add domain">
                  <PlusIcon />
                </button>
              </div>
            )}
          </div>

          <div className="ld-app-view-content">
            {activeView === 'domains' && <DomainsView />}
            {activeView === 'settings' && <SettingsView />}
            {activeView === 'audit' && <AuditView />}
            {activeView === 'about' && <AboutView />}
            {activeView === 'xampp' && <XamppView />}
          </div>

          <div className="ld-app-status-bar-wrapper">
            <div className="ld-app-status-bar">
              <div className="ld-app-status-indicators">
                <span className="ld-app-status-dot"></span>
                <span>Daemon: Running</span>
                <span className="ld-app-status-dot"></span>
                <span>Caddy: Running</span>
                <span className="ld-app-status-dot"></span>
                <span>CA Trusted</span>
              </div>
            </div>
          </div>

          <div className="ld-app-slider-controls">
            <button className="ld-app-slider-arrow" onClick={() => goTo(activeIndex - 1, true)} aria-label="Previous page">
              <ChevronLeftIcon />
            </button>
            <div className="ld-app-slider-dots" aria-label="Preview pages">
              {VIEWS.map((view, i) => (
                <button
                  key={view}
                  className={`ld-app-slider-dot ${activeIndex === i ? 'active' : ''}`}
                  onClick={() => goTo(i, true)}
                  aria-label={`Open ${titleFor(view)}`}
                />
              ))}
            </div>
            <button className="ld-app-slider-arrow" onClick={() => goTo(activeIndex + 1, true)} aria-label="Next page">
              <ChevronRightIcon />
            </button>
            <div className="ld-app-slider-label">{autoplay ? 'Auto slider on' : 'Auto slider stopped'}</div>
            {autoplay && <div key={activeView} className="ld-app-slider-progress" />}
          </div>
        </main>
      </div>
    </div>
  )
}

function titleFor(view: ViewId) {
  if (view === 'domains') return 'Domains'
  if (view === 'settings') return 'Settings'
  if (view === 'audit') return 'Audit Log'
  if (view === 'about') return 'About'
  return 'XAMPP'
}

function DomainsView() {
  return (
    <div className="ld-app-domain-list">
      <div className="ld-app-domain-grid">
        {DOMAIN_SAMPLES.map((domain) => (
          <DomainCard key={domain.name} {...domain} />
        ))}
        <button className="ld-app-add-domain-card">
          <div className="ld-app-add-domain-icon">
            <PlusIcon size={22} />
          </div>
          <span>Configure a new domain</span>
        </button>
      </div>
    </div>
  )
}

function SettingsView() {
  return (
    <div className="settings-view">
      <section className="settings-section">
        <h3>General</h3>
        <div className="form-group">
          <label className="checkbox-label">
            <input type="checkbox" checked readOnly />
            Start on boot
          </label>
        </div>
        <div className="form-row">
          <div className="form-group">
            <label>HTTP Port</label>
            <input type="number" value={80} readOnly />
          </div>
          <div className="form-group">
            <label>HTTPS Port</label>
            <input type="number" value={443} readOnly />
          </div>
        </div>
        <button className="btn btn-primary">Save Settings</button>
      </section>

      <section className="settings-section">
        <h3>Service</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">
              Proxy server
              <span className="status-badge status-badge-active">Running</span>
            </div>
            <p className="settings-service-desc">The reverse proxy handles domain routing on your machine.</p>
          </div>
          <button className="btn btn-sm">Stop</button>
        </div>
      </section>

      <section className="settings-section">
        <h3>Certificate Authority</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">
              Root CA
              <span className="status-badge status-badge-active">Trusted</span>
            </div>
            <p className="settings-service-desc">The root certificate is used to sign HTTPS certificates for your local domains.</p>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h3>Tunnels</h3>
        <div className="form-group">
          <label>Cloudflare Tunnel Token <span className="form-optional">(optional)</span></label>
          <input type="password" value="eyJhIjoiN..." readOnly />
        </div>
        <p className="form-hint">Used for Named Tunnels. Get your token from the Cloudflare Zero Trust dashboard.</p>
        <div className="form-row">
          <div className="form-group">
            <label>Default SSH Host <span className="form-optional">(optional)</span></label>
            <input type="text" value="vps.example.com" readOnly />
          </div>
          <div className="form-group">
            <label>Default SSH User <span className="form-optional">(optional)</span></label>
            <input type="text" value="root" readOnly />
          </div>
        </div>
        <div className="form-group">
          <label>Default SSH Key Path <span className="form-optional">(optional)</span></label>
          <input type="text" value="~/.ssh/id_rsa" readOnly />
        </div>
      </section>

      <section className="settings-section settings-danger-zone">
        <h3>Danger Zone</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">Uninstall service</div>
            <p className="settings-service-desc">Removes the background service, stops all proxying, and cleans up hosts entries.</p>
          </div>
          <button className="btn btn-sm btn-danger">Uninstall</button>
        </div>
      </section>
    </div>
  )
}

function AuditView() {
  return (
    <div className="audit-log-view">
      <div className="audit-log-header">
        <button className="btn btn-sm btn-danger">Clear All</button>
      </div>
      <div className="audit-list">
        <AuditEntry badgeClass="audit-badge-created" badge="Created" detail="dev.local" time="2m ago" />
        <AuditEntry badgeClass="audit-badge-updated" badge="Updated" detail="shorty.local" time="8m ago" />
        <AuditEntry badgeClass="audit-badge-enabled" badge="Enabled" detail="kakha13.local" time="21m ago" />
        <AuditEntry badgeClass="audit-badge-disabled" badge="Disabled" detail="domain.local" time="1h ago" />
      </div>
      <button className="btn audit-load-more">Load More</button>
    </div>
  )
}

function AuditEntry({ badgeClass, badge, detail, time }: { badgeClass: string; badge: string; detail: string; time: string }) {
  return (
    <div className="audit-entry">
      <span className={`audit-badge ${badgeClass}`}>{badge}</span>
      <span className="audit-entry-detail">{detail}</span>
      <span className="audit-entry-time">{time}</span>
    </div>
  )
}

function AboutView() {
  return (
    <div className="about-view">
      <div className="about-card">
        <div className="about-header">
          <img src="/logo.svg" alt="LocalDomain" className="about-logo" />
          <div>
            <h2>LocalDomain</h2>
            <p className="about-version">v0.1.2</p>
          </div>
        </div>
        <p className="about-description">Manage local development domains with custom routing, HTTPS, and public tunnels.</p>

        <div className="about-links">
          <a className="about-link-row" href="https://getlocaldomain.com" target="_blank" rel="noreferrer">
            <GlobeIconSmall />
            <span>getlocaldomain.com</span>
            <ChevronRightSmall />
          </a>
          <a className="about-link-row" href="mailto:kakhagiorgashvili@gmail.com">
            <MailIconSmall />
            <span>kakhagiorgashvili@gmail.com</span>
            <ChevronRightSmall />
          </a>
          <a className="about-link-row" href="https://sourceforge.net/p/localdomain/" target="_blank" rel="noreferrer">
            <DownloadIconSmall />
            <span>SourceForge</span>
            <ChevronRightSmall />
          </a>
        </div>

        <a className="btn-donate" href="https://buymeacoffee.com/kakha13" target="_blank" rel="noreferrer">
          <CoffeeIconSmall />
          Buy me a coffee
        </a>
      </div>
    </div>
  )
}

function XamppView() {
  return (
    <div className="settings-view">
      <h2>XAMPP</h2>

      <section className="settings-section">
        <h3>Apache</h3>
        <div className="settings-service-row">
          <div>
            <div className="settings-service-label">
              Apache Server
              <span className="status-badge status-badge-active">Running</span>
            </div>
            <p className="settings-service-desc">XAMPP Apache serves PHP sites via VirtualHost configuration.</p>
          </div>
          <button className="btn btn-sm">Stop</button>
        </div>
      </section>

      <section className="settings-section">
        <h3>Configuration</h3>
        <div className="form-group">
          <label>XAMPP Path</label>
          <div className="form-row">
            <input type="text" value="/Applications/XAMPP/xamppfiles" readOnly style={{ flex: 1 }} />
            <button className="btn btn-sm">Auto-Detect</button>
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h3>VHost Scanner</h3>
        <p className="form-hint">Scan httpd-vhosts.conf for existing VirtualHost entries to import as domains.</p>
        <button className="btn btn-sm">Scan VHosts</button>
        <div className="scanned-vhosts">
          <div className="scanned-vhosts-list">
            <label className="checkbox-label scanned-vhost-item">
              <input type="checkbox" checked readOnly />
              <span className="scanned-vhost-info">
                <span className="scanned-vhost-name">mysite.local</span>
                <span className="scanned-vhost-root">/Applications/XAMPP/htdocs/mysite</span>
              </span>
            </label>
            <label className="checkbox-label scanned-vhost-item">
              <input type="checkbox" checked readOnly />
              <span className="scanned-vhost-info">
                <span className="scanned-vhost-name">api.local</span>
                <span className="scanned-vhost-root">/Applications/XAMPP/htdocs/api</span>
              </span>
            </label>
          </div>
          <div className="scanned-vhosts-actions">
            <button className="btn btn-sm btn-primary">Import 2 Selected</button>
            <button className="btn btn-sm">Cancel</button>
          </div>
        </div>
      </section>
    </div>
  )
}

function DomainCard({ name, target, icon, enabled }: { name: string; target: string; icon: 'monitor' | 'xampp'; enabled: boolean }) {
  return (
    <div className="ld-app-domain-card">
      <div className="ld-app-domain-card-header">
        <div className="ld-app-domain-card-left">
          <div className="ld-app-domain-card-icon">
            {icon === 'xampp' ? <XamppIcon size={20} /> : <MonitorIcon />}
          </div>
          <div className="ld-app-domain-card-info">
            <div className="ld-app-domain-card-name">{name}</div>
            <div className="ld-app-domain-card-target">{target}</div>
          </div>
        </div>

        <label className="ld-app-toggle-switch">
          <input type="checkbox" checked={enabled} readOnly />
          <span className="ld-app-toggle-slider" />
        </label>
      </div>

      <div className="ld-app-domain-card-meta">
        <div className="ld-app-domain-card-actions">
          <ActionIcon>
            <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
            <polyline points="15 3 21 3 21 9" />
            <line x1="10" y1="14" x2="21" y2="3" />
          </ActionIcon>
          <ActionIcon>
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
          </ActionIcon>
          <ActionIcon>
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4 12.5-12.5z" />
          </ActionIcon>
          <ActionIcon>
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </ActionIcon>
          <ActionIcon>
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </ActionIcon>
        </div>
        <button className="ld-app-card-remove-btn" aria-label="Delete domain">
          <TrashIcon />
        </button>
      </div>
    </div>
  )
}

function ActionIcon({ children }: { children: ReactNode }) {
  return (
    <button className="ld-app-card-action-btn" aria-label="Domain action">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        {children}
      </svg>
    </button>
  )
}

function GlobeIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
    </svg>
  )
}

function GlobeIconSmall() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
    </svg>
  )
}

function SettingsIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="3" />
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
    </svg>
  )
}

function ListIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
    </svg>
  )
}

function InfoIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="12" y1="16" x2="12" y2="12" />
      <line x1="12" y1="8" x2="12.01" y2="8" />
    </svg>
  )
}

function SearchIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
  )
}

function PlusIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  )
}

function MonitorIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
      <line x1="8" y1="21" x2="16" y2="21" />
      <line x1="12" y1="17" x2="12" y2="21" />
    </svg>
  )
}

function TrashIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
      <path d="M10 11v6" />
      <path d="M14 11v6" />
      <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
    </svg>
  )
}

function ChevronLeftIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="15 18 9 12 15 6" />
    </svg>
  )
}

function ChevronRightIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="9 18 15 12 9 6" />
    </svg>
  )
}

function ChevronRightSmall() {
  return (
    <svg className="about-link-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="9 18 15 12 9 6" />
    </svg>
  )
}

function SunIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="5" />
      <line x1="12" y1="1" x2="12" y2="3" />
      <line x1="12" y1="21" x2="12" y2="23" />
      <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
      <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
      <line x1="1" y1="12" x2="3" y2="12" />
      <line x1="21" y1="12" x2="23" y2="12" />
      <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
      <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
    </svg>
  )
}

function MailIconSmall() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
      <polyline points="22,6 12,13 2,6" />
    </svg>
  )
}

function DownloadIconSmall() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" y1="15" x2="12" y2="3" />
    </svg>
  )
}

function CoffeeIconSmall() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M18 8h1a4 4 0 0 1 0 8h-1" />
      <path d="M2 8h16v9a4 4 0 0 1-4 4H6a4 4 0 0 1-4-4V8z" />
      <line x1="6" y1="1" x2="6" y2="4" />
      <line x1="10" y1="1" x2="10" y2="4" />
      <line x1="14" y1="1" x2="14" y2="4" />
    </svg>
  )
}

function XamppIcon({ size }: { size: number }) {
  return (
    <svg width={size} height={size} className="ld-app-xampp-icon" viewBox="0 0 24 24" fill="currentColor" stroke="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M16.792,11.923c0.113,0.043,0.226,0.079,0.334,0.128c0.45,0.203,0.715,0.553,0.748,1.044 c0.041,0.634,0.044,1.271,0.002,1.905c-0.049,0.732-0.725,1.292-1.483,1.271c-0.735-0.021-1.369-0.62-1.397-1.341 c-0.017-0.441-0.003-0.884-0.006-1.326c-0.001-0.239-0.003-0.242-0.245-0.243c-1.363-0.001-2.726,0.008-4.089-0.003 c-0.888-0.007-1.421,0.482-1.471,1.46c-0.019,0.38-0.1,0.727-0.357,1.018c-0.397,0.451-0.898,0.601-1.472,0.466 c-0.554-0.131-0.867-0.522-1.035-1.048c-0.117-0.367-0.056-0.737,0.012-1.094c0.341-1.797,1.366-3.006,3.125-3.555 c0.357-0.112,0.731-0.166,1.105-0.166c0.94,0.001,1.881,0.001,2.821-0.001c0.128,0,0.257-0.012,0.385-0.021 c0.702-0.051,1.166-0.511,1.22-1.352c0.004-0.064,0-0.129,0.001-0.193c0.011-0.788,0.605-1.396,1.393-1.425 c0.787-0.029,1.438,0.527,1.493,1.318c0.076,1.083-0.265,2.046-0.913,2.907C16.903,11.751,16.819,11.816,16.792,11.923z M8.249,10.436c-0.258-0.008-0.571,0.018-0.882-0.035c-0.536-0.09-0.876-0.39-1.02-0.916C6.19,8.912,6.25,8.388,6.698,7.96 C7.154,7.526,7.694,7.4,8.285,7.645c0.52,0.216,0.859,0.731,0.89,1.293C9.2,9.382,9.178,9.828,9.182,10.272 c0.001,0.116-0.043,0.167-0.161,0.165C8.781,10.434,8.542,10.436,8.249,10.436z M21.682,0H2.318C1.102,0,0.116,0.986,0.116,2.202 v19.317c0,1.37,1.111,2.481,2.481,2.481h18.807c1.37,0,2.481-1.111,2.481-2.481V2.202C23.884,0.986,22.898,0,21.682,0z M20.125,12.473c0.519,0.804,0.733,1.69,0.677,2.657c-0.108,1.886-1.413,3.474-3.25,3.916c-2.585,0.623-4.566-0.923-5.233-2.794 c-0.109-0.304-0.16-0.622-0.224-0.985c-0.068,0.414-0.115,0.789-0.264,1.134c-0.697,1.617-1.884,2.603-3.665,2.799 c-2.104,0.232-4.048-1.067-4.632-3.084c-0.25-0.863-0.175-1.747-0.068-2.625c0.08-0.653,0.321-1.268,0.632-1.848 c0.057-0.106,0.057-0.184-0.01-0.285c-0.561-0.845-0.779-1.777-0.7-2.784C3.43,8.035,3.56,7.52,3.805,7.038 C4.52,5.626,6.09,4.427,8.193,4.626c1.849,0.175,3.562,1.77,3.83,3.564c0.013,0.09,0.039,0.178,0.068,0.311 c0.044-0.241,0.076-0.439,0.118-0.636c0.344-1.63,1.94-3.335,4.201-3.357c2.292-0.021,3.99,1.776,4.31,3.446 c0.17,0.888,0.089,1.776-0.103,2.663c-0.112,0.517-0.31,1.008-0.524,1.492C20.034,12.245,20.043,12.345,20.125,12.473z" />
    </svg>
  )
}
