// Interface font preference. The wordmark always stays in Xylo Mirage; this
// switches the headings and controls. Remembered in localStorage (best effort)
// and applied as a `data-font` attribute on <html>.

import { useCallback, useEffect, useState } from "react";

export type FontChoice = "kaylei" | "clean" | "bold" | "compact";

export const FONT_OPTIONS: { id: FontChoice; label: string; hint: string; family: string }[] = [
  { id: "kaylei", label: "Kaylei", hint: "Rounded and friendly (default)", family: '"Parafigure", "Inter", sans-serif' },
  { id: "clean", label: "Clean", hint: "Plain system-style text", family: '"Inter", system-ui, sans-serif' },
  { id: "bold", label: "Bold", hint: "Chunky headings, easy buttons", family: '"Themerity Labs", "Inter", sans-serif' },
  { id: "compact", label: "Compact", hint: "Narrow; fits more on screen", family: '"Paramache", "Inter", sans-serif' },
];

const KEY = "kaylei.font";
const IDS = FONT_OPTIONS.map((o) => o.id);

function read(): FontChoice {
  try {
    const v = localStorage.getItem(KEY) as FontChoice | null;
    if (v && IDS.includes(v)) return v;
  } catch {
    /* storage unavailable */
  }
  return "kaylei";
}

export function applyFont(choice: FontChoice): void {
  document.documentElement.dataset.font = choice;
}

/** Call once before the first render so the first paint uses the saved font. */
export function initFont(): void {
  applyFont(read());
}

export function useFont(): { font: FontChoice; setFont: (f: FontChoice) => void } {
  const [font, setFontState] = useState<FontChoice>(read);
  useEffect(() => {
    applyFont(font);
    try {
      localStorage.setItem(KEY, font);
    } catch {
      /* ignore */
    }
  }, [font]);
  const setFont = useCallback((f: FontChoice) => setFontState(f), []);
  return { font, setFont };
}
