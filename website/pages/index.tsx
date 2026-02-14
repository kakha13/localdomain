import React, { useEffect, useState } from 'react'
import Head from 'next/head'
import AppPreviewInteractive from '../components/AppPreviewInteractive'
import FeatureHeroCard from '../components/FeatureHeroCard'

const STORAGE_KEY = 'localdomain-landing-theme'

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export default function LandingPage() {
  const [theme, setTheme] = useState<'light' | 'dark'>('light')

  useEffect(() => {
    const stored = localStorage.getItem(STORAGE_KEY)
    const initial = (stored === 'light' || stored === 'dark') ? stored : getSystemTheme()
    setTheme(initial)
    document.documentElement.setAttribute('data-theme', initial)

    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = (e: MediaQueryListEvent) => {
      if (!localStorage.getItem(STORAGE_KEY)) {
        const t = e.matches ? 'dark' : 'light'
        setTheme(t)
        document.documentElement.setAttribute('data-theme', t)
      }
    }
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])

  const toggleTheme = () => {
    const next = theme === 'light' ? 'dark' : 'light'
    setTheme(next)
    localStorage.setItem(STORAGE_KEY, next)
    document.documentElement.setAttribute('data-theme', next)
  }

  return (
    <div className="landing-page">
      <Head>
        <title>LocalDomain - Free Local Domain Manager for Developers</title>
        <meta name="description" content="Manage local domains like domain.local with HTTPS, reverse proxy, public tunnels, and system tray control. Free desktop app for macOS, Windows, and Linux." />
        <meta property="og:title" content="LocalDomain - Free Local Domain Manager for Developers" />
        <meta property="og:description" content="Manage local domains like domain.local with HTTPS, reverse proxy, public tunnels, and system tray control. Free desktop app for macOS, Windows, and Linux." />
        <meta property="og:image" content="https://getlocaldomain.com/cover-image.webp" />
        <meta property="og:type" content="website" />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content="LocalDomain - Free Local Domain Manager for Developers" />
        <meta name="twitter:description" content="Manage local domains like domain.local with HTTPS, reverse proxy, public tunnels, and system tray control. Free desktop app for macOS, Windows, and Linux." />
        <meta name="twitter:image" content="https://getlocaldomain.com/cover-image.webp" />
      </Head>

      {/* Navigation */}
      <nav className="ld-nav">
        <div className="ld-nav-inner">
          <a href="/" className="ld-nav-brand">
            <img src="/logo.svg" alt="LocalDomain" width="24" height="24" />
            LocalDomain
          </a>
          <ul className="ld-nav-links">
            <li><a href="#features">Features</a></li>
            <li><a href="#how-it-works">How It Works</a></li>
            <li><a href="/docs">Docs</a></li>
            <li><a href="#support">Support</a></li>
          </ul>
          <div className="ld-nav-right">
            <a href="https://github.com/kakha13/localdomain" target="_blank" rel="noopener noreferrer" className="ld-github-link" aria-label="GitHub repository">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
            </a>
            <button className="ld-theme-btn" onClick={toggleTheme} aria-label="Toggle theme">
              {theme === 'dark' ? (
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="5"/>
                  <line x1="12" y1="1" x2="12" y2="3"/>
                  <line x1="12" y1="21" x2="12" y2="23"/>
                  <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                  <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                  <line x1="1" y1="12" x2="3" y2="12"/>
                  <line x1="21" y1="12" x2="23" y2="12"/>
                  <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                  <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                </svg>
              ) : (
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
              )}
            </button>
            <a href="mailto:kakhagiorgashvili@gmail.com" className="ld-contact-btn">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
                <polyline points="22,6 12,13 2,6"/>
              </svg>
              Contact
            </a>
          </div>
        </div>
      </nav>

      {/* Hero */}
      <section className="ld-hero">
        <div className="ld-hero-tags" aria-hidden="true">
          <span className="ld-htag" style={{ top: '8%', left: '5%', animationDelay: '0s', animationDuration: '18s' }}>HTTPS</span>
          <span className="ld-htag" style={{ top: '22%', right: '8%', animationDelay: '2s', animationDuration: '22s' }}>Reverse Proxy</span>
          <span className="ld-htag" style={{ top: '45%', left: '2%', animationDelay: '1s', animationDuration: '20s' }}>Wildcard</span>
          <span className="ld-htag" style={{ top: '65%', right: '3%', animationDelay: '3s', animationDuration: '19s' }}>Public Tunnels</span>
          <span className="ld-htag" style={{ top: '80%', left: '10%', animationDelay: '0.5s', animationDuration: '24s' }}>TLS Certs</span>
          <span className="ld-htag" style={{ top: '15%', left: '18%', animationDelay: '4s', animationDuration: '21s' }}>Port Routing</span>
          <span className="ld-htag" style={{ top: '55%', right: '12%', animationDelay: '1.5s', animationDuration: '17s' }}>Cloudflare</span>
          <span className="ld-htag" style={{ top: '35%', left: '8%', animationDelay: '2.5s', animationDuration: '23s' }}>Hosts File</span>
          <span className="ld-htag" style={{ top: '75%', right: '15%', animationDelay: '3.5s', animationDuration: '20s' }}>Daemon</span>
          <span className="ld-htag" style={{ top: '10%', right: '20%', animationDelay: '0.8s', animationDuration: '25s' }}>Cross-Platform</span>
          <span className="ld-htag" style={{ top: '90%', left: '22%', animationDelay: '1.2s', animationDuration: '19s' }}>Share</span>
          <span className="ld-htag" style={{ top: '50%', left: '15%', animationDelay: '4.5s', animationDuration: '22s' }}>Local Domains</span>
          <span className="ld-htag" style={{ top: '30%', right: '5%', animationDelay: '2.8s', animationDuration: '18s' }}>HTTPS</span>
          <span className="ld-htag" style={{ top: '70%', left: '3%', animationDelay: '1.8s', animationDuration: '21s' }}>Tunnels</span>
          <span className="ld-htag" style={{ top: '85%', right: '10%', animationDelay: '3.2s', animationDuration: '24s' }}>Proxy</span>
          <span className="ld-htag" style={{ top: '20%', left: '12%', animationDelay: '0.3s', animationDuration: '20s' }}>Wildcard</span>
          <span className="ld-htag" style={{ top: '40%', right: '18%', animationDelay: '2.2s', animationDuration: '23s' }}>Certs</span>
          <span className="ld-htag" style={{ top: '60%', left: '20%', animationDelay: '3.8s', animationDuration: '17s' }}>Routing</span>
        </div>

        <h1 className="ld-animate-in ld-delay-1">Local domains,<br/>zero hassle.</h1>
        <p className="ld-animate-in ld-delay-2">
          Map custom domains like <strong>domain.local</strong> to your local dev servers.
          HTTPS that just works, reverse proxy, public tunnels, and a system tray for quick control. All from a clean native app.
        </p>
        <div className="ld-hero-actions ld-animate-in ld-delay-3">
          <a href="https://sourceforge.net/projects/localdomain/files/v0.1.2/LocalDomain_0.1.2_aarch64.dmg" className="ld-btn ld-btn-mac">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/>
            </svg>
            Download for macOS
          </a>
          <a href="https://sourceforge.net/projects/localdomain/files/v0.1.2/LocalDomain_0.1.2_x64_en-US.msi" className="ld-btn ld-btn-windows">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
            Download for Windows
          </a>
          <a href="https://sourceforge.net/projects/localdomain/files/v0.1.2/LocalDomain_0.1.2_amd64.deb" className="ld-btn ld-btn-linux">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <line x1="4" y1="20" x2="20" y2="20"/>
              <polyline points="7 17 12 12 17 17"/>
              <line x1="12" y1="12" x2="12" y2="4"/>
            </svg>
            Linux (amd64.deb)
          </a>
        </div>
        <div className="ld-hero-donate ld-animate-in ld-delay-4">
          <span>Open source and always will be.</span>
          <a href="https://buymeacoffee.com/kakha13" target="_blank" rel="noopener noreferrer" className="ld-hero-donate-link">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2L15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26z"/>
            </svg>
            Buy Me Tokens
          </a>
        </div>
      </section>

      {/* App Preview - Interactive Demo */}
      <section className="ld-preview-section ld-animate-in ld-delay-4">
        <div style={{ textAlign: 'center', marginBottom: '24px' }}>
          <h2 style={{ fontSize: 'clamp(24px, 4vw, 32px)', fontWeight: 800, marginBottom: '8px' }}>See It In Action</h2>
          <p style={{ color: 'var(--ld-text-secondary)', fontSize: '16px' }}>Watch how LocalDomain streamlines your local development workflow</p>
        </div>
        <AppPreviewInteractive />
      </section>

      {/* Hero Feature Cards */}
      <section className="ld-features ld-hero-section" style={{ paddingTop: '40px' }}>
        <div className="ld-section-label">Core Features</div>
        <h2 className="ld-section-title" style={{ marginBottom: '40px' }}>Everything you need</h2>
        <div className="ld-hero-features-grid">
          <FeatureHeroCard
            variant="xampp"
            title="Import XAMPP VirtualHosts"
            description="Scan and import existing Apache VirtualHosts in one click"
          />
          <FeatureHeroCard
            variant="ssl"
            title="HTTPS Without Warnings"
            description="Auto-generated certificates trusted by your system"
          />
          <FeatureHeroCard
            variant="tunnel"
            title="Share Instantly via Cloudflare"
            description="Create public URLs for local sites in seconds"
          />
          <FeatureHeroCard
            variant="domain"
            title="Custom Local Domains"
            description="Create *.local domains with wildcard support"
          />
        </div>
      </section>

      {/* Features - Standard Cards */}
      <section className="ld-features" id="features" style={{ paddingTop: '20px' }}>
        <div className="ld-section-label">And More</div>
        <h2 className="ld-section-title">Powerful features<br/>for every workflow</h2>
        <p className="ld-section-desc">
          Even more tools to streamline your local development experience.
        </p>
        <div className="ld-features-grid">
          <FeatureCard
            icon={<><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></>}
            title="Reverse Proxy"
            desc={<>Route any custom domain to a specific local port. Point <strong>domain.local</strong> to <strong>localhost:3000</strong> and <strong>api.domain.local</strong> to <strong>localhost:8080</strong>.</>}
          />
          <FeatureCard
            icon={<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>}
            title="Wildcard Domains"
            desc={<>Enable wildcard support so <strong>*.domain.local</strong> resolves automatically. Perfect for multi-tenant apps or microservices.</>}
          />
          <FeatureCard
            icon={<><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></>}
            title="Background Service"
            desc="A lightweight daemon runs in the background managing your hosts file, proxy server, and TLS certificates. Optionally starts on boot."
          />
          <FeatureCard
            icon={<><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="16" y1="13" x2="8" y2="13"/><line x1="16" y1="17" x2="8" y2="17"/><polyline points="10 9 9 9 8 9"/></>}
            title="Audit Log"
            desc="Every change is recorded. See exactly when a domain was added, modified, or removed. Full transparency for your local setup."
          />
          <FeatureCard
            icon={<><rect x="4" y="4" width="16" height="16" rx="2"/><line x1="8" y1="2" x2="8" y2="4"/><line x1="16" y1="2" x2="16" y2="4"/><line x1="8" y1="20" x2="8" y2="22"/><line x1="16" y1="20" x2="16" y2="22"/><circle cx="12" cy="12" r="2"/></>}
            title="System Tray"
            desc="Control everything from the system tray without opening the window. Toggle domains, start or stop services, and see live status at a glance."
          />
          <FeatureCard
            icon={<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>}
            title="Access Log"
            desc="Watch HTTP requests hit your domains in real time. See method, path, status code, and timing for each request - like a built-in dev tools network tab."
          />
        </div>
      </section>

      {/* How it Works */}
      <section className="ld-how-it-works" id="how-it-works">
        <div className="ld-section-label">How It Works</div>
        <h2 className="ld-section-title">Up and running<br/>in three steps</h2>
        <p className="ld-section-desc">
          No terminal commands. No config files. Just a clean app.
        </p>
        <div className="ld-steps">
          <div className="ld-step">
            <div className="ld-step-number">1</div>
            <div className="ld-step-content">
              <h3>Install &amp; launch</h3>
              <p>Download LocalDomain and open it. The setup wizard will ask for your admin password to install a lightweight background service that manages your hosts file and proxy.</p>
            </div>
          </div>
          <div className="ld-step">
            <div className="ld-step-number">2</div>
            <div className="ld-step-content">
              <h3>Add a domain</h3>
              <p>Click &quot;Add Domain&quot; and type the domain you want. Choose the target port, pick HTTP, HTTPS, or both, and optionally enable wildcard.</p>
              <div className="ld-step-code">
                <span className="dim">Domain:</span> domain.local<br/>
                <span className="dim">Target:</span> 127.0.0.1:<span className="highlight">3000</span><br/>
                <span className="dim">Protocol:</span> HTTPS<br/>
                <span className="dim">Wildcard:</span> Off
              </div>
            </div>
          </div>
          <div className="ld-step">
            <div className="ld-step-number">3</div>
            <div className="ld-step-content">
              <h3>Open in your browser</h3>
              <p>That&apos;s it. Navigate to <strong>https://domain.local</strong> in any browser and it just works - valid certificate, no warnings, routed to your local dev server.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Sponsors */}
      <section className="ld-sponsors">
        <div className="ld-section-label">Sponsors</div>
        <h2 className="ld-section-title">Supported by</h2>
        <div className="ld-sponsors-grid">
          <a href="https://any.ge/" target="_blank" rel="noopener noreferrer" className="ld-sponsor-card">
            <img src="/sponsors/anyge.png" alt="Any.ge" className="ld-sponsor-logo" />
            <span className="ld-sponsor-name">Any.ge</span>
            <span className="ld-sponsor-desc">Community project catalog</span>
          </a>
          <a href="https://qartvelo.com/" target="_blank" rel="noopener noreferrer" className="ld-sponsor-card">
            <img src="/sponsors/qartvelo.png" alt="Qartvelo.com" className="ld-sponsor-logo ld-tint-white" />
            <span className="ld-sponsor-name">Qartvelo.com</span>
            <span className="ld-sponsor-desc">Free online web tools</span>
          </a>
        </div>
      </section>

      {/* CTA / Support */}
      <section className="ld-cta" id="support">
        <div className="ld-cta-box">
          <h2>Open source, forever free.</h2>
          <p>
            This project is open source and always will be. It was vibe-coded from start to finish. If it saves you time, buy me some tokens to keep the vibes going.
          </p>
          <a href="https://buymeacoffee.com/kakha13" target="_blank" rel="noopener noreferrer" className="ld-btn ld-btn-coffee">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M12 2L15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26z"/>
            </svg>
            Buy Me Tokens
          </a>
        </div>
      </section>

      {/* Footer */}
      <footer className="ld-footer">
        <div className="ld-footer-inner">
          <div className="ld-footer-left">
            LocalDomain {'\u2014'} Free local domain manager for developers.
          </div>
          <ul className="ld-footer-links">
            <li><a href="/docs">Docs</a></li>
            <li><a href="https://github.com/kakha13/localdomain" target="_blank" rel="noopener noreferrer">GitHub</a></li>
            <li><a href="mailto:kakhagiorgashvili@gmail.com">Contact</a></li>
            <li><a href="https://buymeacoffee.com/kakha13" target="_blank" rel="noopener noreferrer">Support</a></li>
          </ul>
        </div>
      </footer>
    </div>
  )
}

function PreviewCard({ name, target, enabled }: { name: string; target: string; enabled: boolean }) {
  return (
    <div className={`ld-preview-card${enabled ? '' : ' disabled-card'}`}>
      <div className="ld-preview-card-header">
        <div className="ld-preview-card-left">
          <div className="ld-preview-card-icon">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
          </div>
          <div className="ld-preview-card-info">
            <span className="ld-preview-card-name">{name}</span>
            <span className="ld-preview-card-target">{target}</span>
          </div>
        </div>
        <span className={`ld-preview-toggle ${enabled ? 'on' : 'off'}`}></span>
      </div>
    </div>
  )
}

function FeatureCard({ icon, title, desc }: { icon: React.ReactNode; title: string; desc: React.ReactNode }) {
  return (
    <div className="ld-feature-card">
      <div className="ld-feature-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          {icon}
        </svg>
      </div>
      <h3>{title}</h3>
      <p>{desc}</p>
    </div>
  )
}
