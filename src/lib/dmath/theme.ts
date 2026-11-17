// Light / dark / follow-the-system theme. The choice is remembered in
// localStorage (best effort) and applied as a `dark` class on <html>.

import { useCallback, useEffect, useState } from "react";

export type ThemeChoice = "system" | "light" | "dark";

const KEY = "dmath.theme";

function readChoice(): ThemeChoice {
  try {
    const v = localStorage.getItem(KEY);
    if (v === "light" || v === "dark" || v === "system") return v;
  } catch {
    /* storage unavailable: fall through */
  }
  return "system";
}

function systemPrefersDark(): boolean {
  return typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: dark)").matches === true;
}

export function applyTheme(choice: ThemeChoice): void {
  const dark = choice === "dark" || (choice === "system" && systemPrefersDark());
  document.documentElement.classList.toggle("dark", dark);
}

/** Call once before the first render so there is no light flash. */
export function initTheme(): void {
  applyTheme(readChoice());
}

const ORDER: ThemeChoice[] = ["system", "light", "dark"];

export function useTheme(): { choice: ThemeChoice; cycle: () => void } {
  const [choice, setChoice] = useState<ThemeChoice>(readChoice);

  useEffect(() => {
    applyTheme(choice);
    try {
      localStorage.setItem(KEY, choice);
    } catch {
      /* ignore */
    }
    if (choice !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const on = () => applyTheme("system");
    mq.addEventListener("change", on);
    return () => mq.removeEventListener("change", on);
  }, [choice]);

  const cycle = useCallback(() => setChoice((c) => ORDER[(ORDER.indexOf(c) + 1) % ORDER.length]), []);
  return { choice, cycle };
}
