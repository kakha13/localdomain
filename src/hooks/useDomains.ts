import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { Domain, CreateDomainRequest, UpdateDomainRequest } from "../lib/types";
import { useLoading } from "./useLoading";
import * as api from "../lib/api";

export function useDomains() {
  const [domains, setDomains] = useState<Domain[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { track } = useLoading();

  const refresh = useCallback(async () => {
    try {
      setError(null);
      const data = await api.listDomains();
      setDomains(data);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = listen("state-changed", () => { refresh(); });
    return () => { unlisten.then(fn => fn()); };
  }, [refresh]);

  const create = useCallback(
    async (request: CreateDomainRequest) => {
      try {
        setError(null);
        await track(api.createDomain(request));
        await refresh();
      } catch (e) {
        setError(String(e));
        throw e;
      }
    },
    [refresh, track]
  );

  const update = useCallback(
    async (request: UpdateDomainRequest) => {
      try {
        setError(null);
        await track(api.updateDomain(request));
        await refresh();
      } catch (e) {
        setError(String(e));
        throw e;
      }
    },
    [refresh, track]
  );

  const remove = useCallback(
    async (id: string) => {
      try {
        setError(null);
        await track(api.deleteDomain(id));
        await refresh();
      } catch (e) {
        setError(String(e));
        throw e;
      }
    },
    [refresh, track]
  );

  const toggle = useCallback(
    async (id: string, enabled: boolean) => {
      try {
        setError(null);
        await track(api.toggleDomain(id, enabled));
        await refresh();
      } catch (e) {
        setError(String(e));
        throw e;
      }
    },
    [refresh, track]
  );

  return { domains, loading, error, refresh, create, update, remove, toggle };
}
