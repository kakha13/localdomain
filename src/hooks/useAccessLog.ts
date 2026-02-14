import { useState, useEffect, useCallback, useRef } from "react";
import type { AccessLogEntry } from "../lib/types";
import * as api from "../lib/api";

export function useAccessLog(domain: string, limit = 100, intervalMs = 2000) {
  const [entries, setEntries] = useState<AccessLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetch = useCallback(async () => {
    try {
      const data = await api.getAccessLog(domain, limit);
      setEntries(data);
    } catch {
      // ignore polling errors
    } finally {
      setLoading(false);
    }
  }, [domain, limit]);

  useEffect(() => {
    setLoading(true);
    fetch();
    timerRef.current = setInterval(fetch, intervalMs);
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
  }, [fetch, intervalMs]);

  const clear = useCallback(async () => {
    await api.clearAccessLog(domain);
    setEntries([]);
  }, [domain]);

  return { entries, loading, clear, refresh: fetch };
}
