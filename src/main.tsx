import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { LoadingProvider } from "./hooks/useLoading";
import "./styles/global.css";

// Apply saved theme immediately to prevent flash
const stored = localStorage.getItem("localdomain-theme");
const theme =
  stored === "light" || stored === "dark"
    ? stored
    : window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
document.documentElement.setAttribute("data-theme", theme);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <LoadingProvider>
      <App />
    </LoadingProvider>
  </React.StrictMode>
);
