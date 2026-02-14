import { useState, useEffect } from "react";
import type { View } from "./lib/types";
import { Layout, type DomainFilter } from "./components/Layout";
import { SetupScreen } from "./components/SetupScreen";
import { DomainList } from "./components/DomainList";
import { SettingsView } from "./components/SettingsView";
import { AuditLogView } from "./components/AuditLogView";
import { RequestInspector } from "./components/RequestInspector";
import { AboutView } from "./components/AboutView";
import { XamppView } from "./components/XamppView";
import { useServiceStatus } from "./hooks/useServiceStatus";

function App() {
  const [currentView, setCurrentView] = useState<View>("domains");
  const [inspectTarget, setInspectTarget] = useState<{ id: string; name: string; accessLog: boolean } | null>(null);
  const [showSetup, setShowSetup] = useState<boolean | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [domainFilter, setDomainFilter] = useState<DomainFilter>("all");
  const [addTrigger, setAddTrigger] = useState(0);
  const { status, loading, refresh, start, stop, uninstallDaemon, trustCa } =
    useServiceStatus();

  // Decide initial view: show setup only if daemon isn't running on first load
  useEffect(() => {
    if (!loading && showSetup === null) {
      setShowSetup(!status.daemon_running);
    }
  }, [loading, showSetup, status.daemon_running]);

  if (loading || showSetup === null) {
    return (
      <div className="app-loading">
        <div className="spinner" />
      </div>
    );
  }

  if (showSetup) {
    return (
      <SetupScreen
        onComplete={() => {
          refresh();
          setShowSetup(false);
        }}
        onSkip={() => setShowSetup(false)}
      />
    );
  }

  const handleInspect = (id: string, name: string, accessLog: boolean) => {
    setInspectTarget({ id, name, accessLog });
    setCurrentView("inspect");
  };

  const handleViewChange = (view: View) => {
    setCurrentView(view);
    if (view !== "inspect") {
      setInspectTarget(null);
    }
  };

  return (
    <Layout
      currentView={currentView}
      onViewChange={handleViewChange}
      status={status}
      searchQuery={searchQuery}
      onSearchChange={setSearchQuery}
      domainFilter={domainFilter}
      onDomainFilterChange={setDomainFilter}
      onAddDomain={() => setAddTrigger((n) => n + 1)}
    >
      {currentView === "domains" && (
        <DomainList
          onInspect={handleInspect}
          searchQuery={searchQuery}
          domainFilter={domainFilter}
          addTrigger={addTrigger}
        />
      )}
      {currentView === "inspect" && inspectTarget && (
        <RequestInspector
          domainId={inspectTarget.id}
          domain={inspectTarget.name}
          initialAccessLog={inspectTarget.accessLog}
          onBack={() => handleViewChange("domains")}
        />
      )}
      {currentView === "settings" && (
        <SettingsView
          status={status}
          onStart={start}
          onStop={stop}
          onTrustCa={trustCa}
          onUninstall={async () => {
            await uninstallDaemon();
            await refresh();
            setShowSetup(true);
            setCurrentView("domains");
          }}
        />
      )}
      {currentView === "xampp" && <XamppView status={status} />}
      {currentView === "audit" && <AuditLogView />}
      {currentView === "about" && <AboutView />}
    </Layout>
  );
}

export default App;
