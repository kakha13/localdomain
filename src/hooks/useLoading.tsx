import { createContext, useContext, useState, useCallback, useRef, useEffect, type ReactNode } from "react";

const MIN_VISIBLE_MS = 500;
const START_PROGRESS = 10;
const MAX_ACTIVE_PROGRESS = 90;
const CREEP_INTERVAL_MS = 120;
const FINISH_ANIM_MS = 180;

interface LoadingContextValue {
  /** Whether the loading bar should be visible */
  isLoading: boolean;
  /** Current loading progress (0-100) */
  progress: number;
  /** Wrap an async action to track it globally */
  track: <T>(promise: Promise<T>) => Promise<T>;
}

const LoadingContext = createContext<LoadingContextValue>({
  isLoading: false,
  progress: 0,
  track: (p) => p,
});

export function LoadingProvider({ children }: { children: ReactNode }) {
  const [visible, setVisible] = useState(false);
  const [progress, setProgress] = useState(0);
  const countRef = useRef(0);
  const showTimeRef = useRef(0);
  const hideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const finishTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const creepTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const clearTimers = useCallback(() => {
    if (hideTimerRef.current) {
      clearTimeout(hideTimerRef.current);
      hideTimerRef.current = null;
    }
    if (finishTimerRef.current) {
      clearTimeout(finishTimerRef.current);
      finishTimerRef.current = null;
    }
    if (creepTimerRef.current) {
      clearInterval(creepTimerRef.current);
      creepTimerRef.current = null;
    }
  }, []);

  // Cleanup timer on unmount
  useEffect(() => {
    return () => {
      clearTimers();
    };
  }, [clearTimers]);

  const startCreep = useCallback(() => {
    if (creepTimerRef.current) return;
    creepTimerRef.current = setInterval(() => {
      setProgress((current) => {
        if (current >= MAX_ACTIVE_PROGRESS) return current;
        const remaining = MAX_ACTIVE_PROGRESS - current;
        return Math.min(MAX_ACTIVE_PROGRESS, current + Math.max(1, remaining * 0.14));
      });
    }, CREEP_INTERVAL_MS);
  }, []);

  const showLoading = useCallback(() => {
    clearTimers();
    showTimeRef.current = Date.now();
    setProgress(START_PROGRESS);
    setVisible(true);
    startCreep();
  }, [clearTimers, startCreep]);

  const finishLoading = useCallback(() => {
    if (countRef.current > 0) return; // still busy
    if (creepTimerRef.current) {
      clearInterval(creepTimerRef.current);
      creepTimerRef.current = null;
    }

    const elapsed = Date.now() - showTimeRef.current;
    const remaining = Math.max(0, MIN_VISIBLE_MS - elapsed);

    if (hideTimerRef.current) clearTimeout(hideTimerRef.current);
    hideTimerRef.current = setTimeout(() => {
      hideTimerRef.current = null;
      if (countRef.current > 0) return;

      setProgress(100);
      if (finishTimerRef.current) clearTimeout(finishTimerRef.current);
      finishTimerRef.current = setTimeout(() => {
        finishTimerRef.current = null;
        if (countRef.current === 0) {
          setVisible(false);
          setProgress(0);
        }
      }, FINISH_ANIM_MS);
    }, remaining);
  }, []);

  const track = useCallback(async <T,>(promise: Promise<T>): Promise<T> => {
    countRef.current += 1;
    if (countRef.current === 1) {
      showLoading();
    }
    try {
      return await promise;
    } finally {
      countRef.current -= 1;
      finishLoading();
    }
  }, [finishLoading, showLoading]);

  return (
    <LoadingContext.Provider value={{ isLoading: visible, progress, track }}>
      {children}
    </LoadingContext.Provider>
  );
}

export function useLoading() {
  return useContext(LoadingContext);
}
