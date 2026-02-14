import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ServiceStatus } from "../lib/types";
import { useLoading } from "./useLoading";
import * as api from "../lib/api";

export function useServiceStatus(pollIntervalMs = 5000) {
  const [status, setStatus] = useState<ServiceStatus>({
    daemon_running: false,
    caddy_running: false,
    ca_installed: false,
    ca_trusted: false,
    xampp_running: false,
  });
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { track } = useLoading();

  const refresh = useCallback(async () => {
    try {
      const data = await api.getServiceStatus();
      setStatus(data);
    } catch {
      setStatus({
        daemon_running: false,
        caddy_running: false,
        ca_installed: false,
        ca_trusted: false,
        xampp_running: false,
      });
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, pollIntervalMs);
    const unlisten = listen("state-changed", () => { refresh(); });
    return () => {
      clearInterval(interval);
      unlisten.then(fn => fn());
    };
  }, [refresh, pollIntervalMs]);

  const start = useCallback(async () => {
    try {
      setError(null);
      await track(api.startService());
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [refresh, track]);

  const stop = useCallback(async () => {
    try {
      setError(null);
      await track(api.stopService());
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [refresh, track]);

  const installDaemon = useCallback(async () => {
    try {
      setError(null);
      await track(api.installDaemon());
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [refresh, track]);

  const uninstallDaemon = useCallback(async () => {
    try {
      setError(null);
      await track(api.uninstallDaemon());
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [refresh, track]);

  const trustCa = useCallback(async () => {
    try {
      setError(null);
      await track(api.trustCa());
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  }, [refresh, track]);

  const clearError = useCallback(() => setError(null), []);

  return { status, loading, error, refresh, start, stop, installDaemon, uninstallDaemon, trustCa, clearError };
}
