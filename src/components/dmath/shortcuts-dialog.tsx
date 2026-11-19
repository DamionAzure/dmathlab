"use client";

// A small cheat sheet of keyboard shortcuts. Opens from the header button or with "?".

import { useEffect } from "react";
import { X } from "lucide-react";

const MOD = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘" : "Ctrl";

const ROWS: { keys: string[]; what: string }[] = [
  { keys: [MOD, "Enter"], what: "Run the program" },
  { keys: [MOD, "/"], what: "Comment or uncomment the selected lines" },
  { keys: ["Tab"], what: "Indent by two spaces" },
  { keys: ["?"], what: "Show this list (when not typing in the editor)" },
  { keys: ["Esc"], what: "Close a menu or dialog" },
];

export function ShortcutsDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/40 p-4 pt-[10vh] backdrop-blur-[2px]"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
      aria-label="Keyboard shortcuts"
    >
      <div className="w-full max-w-md overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl">
        <div className="flex items-center justify-between border-b border-zinc-200 px-4 py-3">
          <h2 className="font-heading text-zinc-900">Keyboard shortcuts</h2>
          <button onClick={onClose} aria-label="Close" className="rounded p-1 text-zinc-500 hover:bg-zinc-100">
            <X className="h-4 w-4" />
          </button>
        </div>
        <ul className="divide-y divide-zinc-100 px-4 py-1">
          {ROWS.map((r) => (
            <li key={r.what} className="flex items-center justify-between gap-3 py-2.5 text-sm text-zinc-700">
              <span>{r.what}</span>
              <span className="flex shrink-0 gap-1">
                {r.keys.map((k) => (
                  <kbd key={k} className="rounded border border-zinc-300 bg-zinc-50 px-1.5 py-0.5 font-mono text-xs text-zinc-700">
                    {k}
                  </kbd>
                ))}
              </span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
