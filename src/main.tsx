import { createRoot } from "react-dom/client";

import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./styles.css";
import { DMathApp } from "./components/dmath/dmath-app";
import { ErrorBoundary } from "./components/error-boundary";
import { initTheme } from "./lib/dmath/theme";

initTheme();

createRoot(document.getElementById("root")!).render(
  <ErrorBoundary label="Kaylei hit an unexpected UI error.">
    <DMathApp />
  </ErrorBoundary>,
);
