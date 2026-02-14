import { useState, useEffect, useCallback } from "react";

type Theme = "light" | "dark";
type ThemePreference = "light" | "dark" | "system";

const STORAGE_KEY = "localdomain-theme";

function getSystemTheme(): Theme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function getStoredPreference(): ThemePreference {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

function resolveTheme(preference: ThemePreference): Theme {
  return preference === "system" ? getSystemTheme() : preference;
}

function applyTheme(theme: Theme) {
  document.documentElement.setAttribute("data-theme", theme);
}

export function useTheme() {
  const [preference, setPreference] = useState<ThemePreference>(getStoredPreference);
  const [resolvedTheme, setResolvedTheme] = useState<Theme>(() =>
    resolveTheme(getStoredPreference())
  );

  // Apply theme to DOM whenever resolved theme changes
  useEffect(() => {
    applyTheme(resolvedTheme);
  }, [resolvedTheme]);

  // Listen for system theme changes when preference is "system"
  useEffect(() => {
    if (preference !== "system") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      const newTheme = e.matches ? "dark" : "light";
      setResolvedTheme(newTheme);
    };

    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [preference]);

  const setTheme = useCallback((newPreference: ThemePreference) => {
    localStorage.setItem(STORAGE_KEY, newPreference);
    setPreference(newPreference);
    setResolvedTheme(resolveTheme(newPreference));
  }, []);

  const toggle = useCallback(() => {
    // Cycle: if currently light → dark, if currently dark → light
    const next: Theme = resolvedTheme === "light" ? "dark" : "light";
    setTheme(next);
  }, [resolvedTheme, setTheme]);

  return {
    theme: resolvedTheme,
    preference,
    setTheme,
    toggle,
  };
}
