import { useState } from "react";
import type { View } from "../lib/types";
import type { ServiceStatus } from "../lib/types";
import { useTheme } from "../hooks/useTheme";
import { useLoading } from "../hooks/useLoading";
import { ServiceStatusBar } from "./ServiceStatusBar";
import {
  GlobeIcon,
  SettingsIcon,
  ListIcon,
  InfoIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  SunIcon,
  MoonIcon,
  SearchIcon,
  PlusIcon,
  XamppIcon,
} from "./Icons";
import logoImg from "../assets/logo.png";

const VIEW_TITLES: Record<View, string> = {
  domains: "Domains",
  xampp: "XAMPP",
  settings: "Settings",
  audit: "Audit Log",
  inspect: "Inspect",
  about: "About",
};

type NavSection = {
  label?: string;
  items: { view: View; label: string; icon: React.ReactNode }[];
};

const NAV_SECTIONS: NavSection[] = [
  {
    items: [
      { view: "domains", label: "Domains", icon: <GlobeIcon /> },
      { view: "settings", label: "Settings", icon: <SettingsIcon /> },
      { view: "audit", label: "Audit Log", icon: <ListIcon /> },
      { view: "about", label: "About", icon: <InfoIcon /> },
    ],
  },
  {
    label: "Services",
    items: [{ view: "xampp", label: "XAMPP", icon: <XamppIcon /> }],
  },
];

export type DomainFilter = "all" | "active" | "inactive";

interface LayoutProps {
  currentView: View;
  onViewChange: (view: View) => void;
  status: ServiceStatus;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onAddDomain: () => void;
  domainFilter: DomainFilter;
  onDomainFilterChange: (filter: DomainFilter) => void;
  children: React.ReactNode;
}

export function Layout({
  currentView,
  onViewChange,
  status,
  searchQuery,
  onSearchChange,
  onAddDomain,
  domainFilter,
  onDomainFilterChange,
  children,
}: LayoutProps) {
  const isActive = status.daemon_running && status.caddy_running;
  const { theme, toggle } = useTheme();
  const { isLoading, progress } = useLoading();
  const [collapsed, setCollapsed] = useState(false);

  return (
    <div className="app-layout">
      {/* Sidebar */}
      <nav className={`sidebar ${collapsed ? "collapsed" : ""}`}>
        <div className="sidebar-brand">
          <img src={logoImg} alt="LocalDomain" className="sidebar-logo" />
          <span className="sidebar-brand-title">LocalDomain</span>
        </div>

        <ul className="sidebar-nav">
          {NAV_SECTIONS.map((section, si) => (
            <li key={si}>
              {section.label && (
                <div className="nav-section-label">
                  <span>{section.label}</span>
                </div>
              )}
              {section.items.map((item) => (
                <button
                  key={item.view}
                  className={currentView === item.view ? "active" : ""}
                  onClick={() => onViewChange(item.view)}
                >
                  {item.icon}
                  <span className="nav-label">{item.label}</span>
                </button>
              ))}
            </li>
          ))}
        </ul>

        <div className="sidebar-footer">
          <button
            className="sidebar-footer-btn"
            onClick={() => setCollapsed((c) => !c)}
          >
            {collapsed ? <ChevronRightIcon /> : <ChevronLeftIcon />}
            <span className="sidebar-footer-label">Collapse</span>
          </button>
          <button className="sidebar-footer-btn" onClick={toggle}>
            {theme === "dark" ? <SunIcon /> : <MoonIcon />}
            <span className="sidebar-footer-label">
              {theme === "dark" ? "Light Mode" : "Dark Mode"}
            </span>
          </button>
        </div>
      </nav>

      {/* Main Content */}
      <main className="main-content">
        {/* Global loading bar */}
        <div className={`global-loading-bar ${isLoading ? "active" : ""}`}>
          <div
            className="global-loading-bar-fill"
            style={{ transform: `scaleX(${Math.max(0, Math.min(progress, 100)) / 100})` }}
          />
        </div>

        {/* Header Bar — hidden on inspect view (inspector has its own header) */}
        {currentView !== "inspect" && <div className="header-bar">
          <div className="header-left">
            <h1 className="header-title">{VIEW_TITLES[currentView]}</h1>
            {currentView === "domains" && (
              <div className="header-subtitle">
                <span className={`status-dot ${isActive ? "green" : "red"}`} />
                {isActive ? "SYSTEM ACTIVE" : "SYSTEM INACTIVE"}
              </div>
            )}
          </div>
          {currentView === "domains" && (
            <div className="header-right">
              <div className="header-search">
                <SearchIcon />
                <input
                  type="text"
                  placeholder="Search domains..."
                  value={searchQuery}
                  onChange={(e) => onSearchChange(e.target.value)}
                />
              </div>
              <div className="header-filter">
                <button
                  className={`header-filter-btn ${domainFilter === "all" ? "active" : ""}`}
                  onClick={() => onDomainFilterChange("all")}
                  title="Show all domains"
                >
                  All
                </button>
                <button
                  className={`header-filter-btn ${domainFilter === "active" ? "active" : ""}`}
                  onClick={() => onDomainFilterChange("active")}
                  title="Show active domains"
                >
                  {/* <span className="filter-dot green" /> */}
                  Active
                </button>
                <button
                  className={`header-filter-btn ${domainFilter === "inactive" ? "active" : ""}`}
                  onClick={() => onDomainFilterChange("inactive")}
                  title="Show inactive domains"
                >
                  {/* <span className="filter-dot orange" /> */}
                  Inactive
                </button>
              </div>
              <button className="btn btn-primary btn-icon" onClick={onAddDomain} title="Add Domain">
                <PlusIcon />
              </button>
            </div>
          )}
        </div>}

        <div className="main-content-scroll">{children}</div>
        <ServiceStatusBar onNavigate={onViewChange} />
      </main>
    </div>
  );
}
