import { createRoot } from "react-dom/client";

import "@fontsource-variable/inter";
import "@fontsource-variable/jetbrains-mono";
import "./styles.css";
import { DMathApp } from "./components/dmath/dmath-app";
import { ErrorBoundary } from "./components/error-boundary";

createRoot(document.getElementById("root")!).render(
  <ErrorBoundary label="DMath Lab hit an unexpected UI error.">
    <DMathApp />
  </ErrorBoundary>,
);
