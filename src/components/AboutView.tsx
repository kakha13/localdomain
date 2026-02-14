import { open } from "@tauri-apps/plugin-shell";
import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import logoImg from "../assets/logo.png";

export function AboutView() {
  const [version, setVersion] = useState("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  const handleLink = (url: string) => (e: React.MouseEvent) => {
    e.preventDefault();
    open(url);
  };

  return (
    <div className="about-view">
      <div className="about-card">
        <div className="about-header">
          <img src={logoImg} alt="LocalDomain" className="about-logo" />
          <div>
            <h2>LocalDomain</h2>
            <p className="about-version">v{version}</p>
          </div>
        </div>

        <p className="about-description">
          Manage local development domains with custom routing, HTTPS, and public tunnels.
        </p>

        <div className="about-links">
          <a
            href="https://getlocaldomain.com/"
            onClick={handleLink("https://getlocaldomain.com/")}
            className="about-link-row"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <circle cx="12" cy="12" r="10" /><line x1="2" y1="12" x2="22" y2="12" /><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
            </svg>
            <span>getlocaldomain.com</span>
            <svg className="about-link-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="9,18 15,12 9,6" />
            </svg>
          </a>
          <a
            href="mailto:kakhagiorgashvili@gmail.com"
            onClick={handleLink("mailto:kakhagiorgashvili@gmail.com")}
            className="about-link-row"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" /><polyline points="22,6 12,13 2,6" />
            </svg>
            <span>kakhagiorgashvili@gmail.com</span>
            <svg className="about-link-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="9,18 15,12 9,6" />
            </svg>
          </a>
          <a
            href="https://sourceforge.net/p/localdomain/"
            onClick={handleLink("https://sourceforge.net/p/localdomain/")}
            className="about-link-row"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7,10 12,15 17,10" /><line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            <span>SourceForge</span>
            <svg className="about-link-arrow" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="9,18 15,12 9,6" />
            </svg>
          </a>
        </div>

        <a
          href="https://buymeacoffee.com/kakha13"
          onClick={handleLink("https://buymeacoffee.com/kakha13")}
          className="btn-donate"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M18 8h1a4 4 0 010 8h-1"/><path d="M2 8h16v9a4 4 0 01-4 4H6a4 4 0 01-4-4V8z"/><line x1="6" y1="1" x2="6" y2="4"/><line x1="10" y1="1" x2="10" y2="4"/><line x1="14" y1="1" x2="14" y2="4"/>
          </svg>
          Buy me a coffee
        </a>
      </div>
    </div>
  );
}
