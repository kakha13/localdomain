import React, { useState } from 'react'

interface FeatureHeroCardProps {
  variant: 'xampp' | 'ssl' | 'tunnel' | 'domain'
  title: string
  description: string
}

const icons = {
  xampp: (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
      <polyline points="10 9 9 9 8 9" />
    </svg>
  ),
  ssl: (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
      <path d="M7 11V7a5 5 0 0 1 10 0v4" />
    </svg>
  ),
  tunnel: (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="10" />
      <line x1="2" y1="12" x2="22" y2="12" />
      <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
    </svg>
  ),
  domain: (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="2" />
      <path d="M12 2v4" />
      <path d="M12 18v4" />
      <path d="M4.93 4.93l2.83 2.83" />
      <path d="M16.24 16.24l2.83 2.83" />
      <path d="M2 12h4" />
      <path d="M18 12h4" />
      <path d="M4.93 19.07l2.83-2.83" />
      <path d="M16.24 7.76l2.83-2.83" />
    </svg>
  ),
}

const previewContent = {
  xampp: (
    <div style={{ padding: '16px', background: 'var(--ld-bg-tertiary)', borderRadius: '8px', height: '100%' }}>
      <div style={{ fontSize: '11px', fontWeight: 600, color: 'var(--ld-text-secondary)', marginBottom: '10px' }}>Scanning XAMPP VHosts...</div>
      <div style={{ height: '6px', background: 'var(--ld-border)', borderRadius: '3px', overflow: 'hidden', marginBottom: '10px' }}>
        <div style={{ height: '100%', width: '60%', background: 'var(--ld-accent)', borderRadius: '3px' }} />
      </div>
      <div style={{ fontSize: '11px', color: 'var(--ld-text-secondary)' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', padding: '2px 0' }}>
          <span style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--ld-success)' }} />
          mysite.local → /htdocs/mysite
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px', padding: '2px 0' }}>
          <span style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--ld-success)' }} />
          api.local → /htdocs/api
        </div>
      </div>
    </div>
  ),
  ssl: (
    <div style={{ padding: '16px', background: 'var(--ld-bg-tertiary)', borderRadius: '8px', height: '100%' }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', padding: '8px 12px', background: 'var(--ld-bg-elevated)', borderRadius: '6px', marginBottom: '10px', border: '1px solid var(--ld-border)' }}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--ld-success)" strokeWidth="2">
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
        <span style={{ fontSize: '11px', fontFamily: 'monospace', color: 'var(--ld-text-primary)' }}>https://myapp.local</span>
      </div>
      <div style={{ fontSize: '10px', color: 'var(--ld-text-secondary)', display: 'flex', flexDirection: 'column', gap: '4px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <span>Certificate:</span>
          <span style={{ color: 'var(--ld-success)' }}>Valid</span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <span>Issuer:</span>
          <span>LocalDomain CA</span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <span>Expires:</span>
          <span>Feb 2027</span>
        </div>
      </div>
      <div style={{ marginTop: '8px', display: 'inline-flex', alignItems: 'center', gap: '4px', padding: '3px 8px', background: 'var(--ld-badge-bg)', borderRadius: '4px', fontSize: '9px', fontWeight: 600, color: 'var(--ld-badge-text)' }}>
        Trusted
      </div>
    </div>
  ),
  tunnel: (
    <div style={{ padding: '16px', background: 'var(--ld-bg-tertiary)', borderRadius: '8px', height: '100%' }}>
      <div style={{ fontSize: '12px', fontWeight: 600, color: 'var(--ld-text-primary)', marginBottom: '8px' }}>myproject.local</div>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '4px', marginBottom: '8px', color: 'var(--ld-text-tertiary)' }}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <polyline points="19 12 12 19 5 12" />
        </svg>
      </div>
      <div style={{ padding: '8px 10px', background: 'var(--ld-bg-elevated)', borderRadius: '6px', border: '1px solid var(--ld-border)', marginBottom: '8px' }}>
        <span style={{ fontSize: '10px', color: 'var(--ld-accent)', fontFamily: 'monospace' }}>abc123.trycloudflare.com</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <span style={{ fontSize: '10px', padding: '2px 6px', background: 'var(--ld-bg-elevated)', borderRadius: '4px', color: 'var(--ld-text-secondary)' }}>Copy</span>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px', fontSize: '10px', color: 'var(--ld-success)', fontWeight: 500 }}>
          <span style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--ld-success)' }} />
          Live
        </div>
      </div>
    </div>
  ),
  domain: (
    <div style={{ padding: '16px', background: 'var(--ld-bg-tertiary)', borderRadius: '8px', height: '100%' }}>
      <div style={{ fontSize: '12px', fontWeight: 600, color: 'var(--ld-text-secondary)', marginBottom: '10px' }}>+ Add Domain</div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
          <label style={{ fontSize: '10px', fontWeight: 600, color: 'var(--ld-text-secondary)' }}>Domain</label>
          <input type="text" defaultValue="project.test" readOnly style={{ padding: '6px 8px', fontSize: '11px', background: 'var(--ld-bg-elevated)', border: '1px solid var(--ld-border)', borderRadius: '4px', color: 'var(--ld-text-primary)' }} />
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
          <label style={{ fontSize: '10px', fontWeight: 600, color: 'var(--ld-text-secondary)' }}>Target</label>
          <input type="text" defaultValue="127.0.0.1:3000" readOnly style={{ padding: '6px 8px', fontSize: '11px', background: 'var(--ld-bg-elevated)', border: '1px solid var(--ld-border)', borderRadius: '4px', color: 'var(--ld-text-primary)' }} />
        </div>
        <div style={{ display: 'flex', gap: '8px', marginTop: '4px' }}>
          <button style={{ flex: 1, padding: '6px', fontSize: '11px', background: 'var(--ld-bg-elevated)', border: '1px solid var(--ld-border)', borderRadius: '4px', color: 'var(--ld-text-secondary)', cursor: 'default' }}>Cancel</button>
          <button style={{ flex: 1, padding: '6px', fontSize: '11px', background: 'var(--ld-accent)', border: 'none', borderRadius: '4px', color: 'white', fontWeight: 600, cursor: 'default' }}>Create</button>
        </div>
      </div>
    </div>
  ),
}

export default function FeatureHeroCard({ variant, title, description }: FeatureHeroCardProps) {
  const [isHovered, setIsHovered] = useState(false)

  return (
    <div
      className={`ld-hero-feature-card ${variant}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onFocus={() => setIsHovered(true)}
      onBlur={() => setIsHovered(false)}
      tabIndex={0}
      role="article"
      aria-label={`${title} feature`}
    >
      <div className={`ld-hero-feature-icon ${variant === 'ssl' ? 'ld-icon-color-change' : ''} ${variant === 'xampp' ? 'ld-icon-pulse' : ''} ${variant === 'tunnel' ? 'ld-icon-connection' : ''} ${variant === 'domain' ? 'ld-icon-dns' : ''}`}>
        {icons[variant]}
      </div>
      <h3>{title}</h3>
      <p>{description}</p>
      <div className={`ld-hero-feature-screenshot ${isHovered ? 'visible' : ''}`}>
        {previewContent[variant]}
      </div>
    </div>
  )
}
