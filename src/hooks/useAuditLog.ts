import { useState, useEffect, useCallback } from "react";
import type { AuditLogEntry } from "../lib/types";
import * as api from "../lib/api";

export function useAuditLog(pageSize = 50) {
  const [entries, setEntries] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(true);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      const data = await api.getAuditLog(pageSize, 0);
      setEntries(data);
      setOffset(0);
      setHasMore(data.length === pageSize);
    } catch {
      setEntries([]);
    } finally {
      setLoading(false);
    }
  }, [pageSize]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const loadMore = useCallback(async () => {
    const newOffset = offset + pageSize;
    try {
      const data = await api.getAuditLog(pageSize, newOffset);
      setEntries((prev) => [...prev, ...data]);
      setOffset(newOffset);
      setHasMore(data.length === pageSize);
    } catch {
      // ignore
    }
  }, [offset, pageSize]);

  return { entries, loading, hasMore, refresh, loadMore };
}
