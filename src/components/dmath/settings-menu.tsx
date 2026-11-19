// Small popover for interface preferences (currently: font).

import { useEffect, useRef, useState } from "react";
import { Check, Type } from "lucide-react";
import { FONT_OPTIONS, useFont } from "@/lib/dmath/fonts";

export function SettingsMenu() {
  const { font, setFont } = useFont();
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent | TouchEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && setOpen(false);
    document.addEventListener("mousedown", onDown);
    document.addEventListener("touchstart", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("touchstart", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        title="Font"
        aria-label="Choose font"
        aria-expanded={open}
        className="inline-flex h-7 w-7 items-center justify-center rounded-full border border-zinc-300 text-zinc-600 hover:bg-zinc-100"
      >
        <Type className="h-3.5 w-3.5" />
      </button>
      {open && (
        <div
          role="menu"
          className="absolute right-0 top-9 z-40 w-64 max-w-[calc(100vw-2rem)] rounded-xl border border-zinc-200 bg-white p-1.5 shadow-lg"
        >
          <p className="px-2 pb-1 pt-1 text-[11px] font-medium uppercase tracking-wide text-zinc-400">Interface font</p>
          {FONT_OPTIONS.map((o) => (
            <button
              key={o.id}
              type="button"
              role="menuitemradio"
              aria-checked={font === o.id}
              onClick={() => {
                setFont(o.id);
                setOpen(false);
              }}
              className="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left hover:bg-zinc-100"
            >
              <span className="min-w-0 flex-1">
                <span className="block text-base leading-tight text-zinc-900" style={{ fontFamily: o.family, fontWeight: 400 }}>
                  {o.label} <span className="text-zinc-500">∪ ∈ →</span>
                </span>
                <span className="block text-[11px] text-zinc-500" style={{ fontFamily: "Inter, system-ui, sans-serif" }}>
                  {o.hint}
                </span>
              </span>
              {font === o.id && <Check className="h-4 w-4 shrink-0 text-[#1C9A93]" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
