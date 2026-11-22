"use client";

// In-app dialogs. They work the same on desktop and on Android (the native
// dialog commands do not exist on mobile, which used to make example loading fail).

import { useEffect } from "react";
import { X } from "lucide-react";
import { EXAMPLES, EXAMPLE_GROUPS } from "@/lib/dmath/examples";

function Shell({ title, onClose, children }: { title: string; onClose: () => void; children: React.ReactNode }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);
  return (
    <div
      className="fixed inset-0 z-50 flex items-end justify-center bg-black/40 p-3 backdrop-blur-[2px] sm:items-center"
      onClick={(e) => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      <div className="flex max-h-[85vh] w-full max-w-lg flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl">
        <div className="flex items-center justify-between border-b border-zinc-200 px-4 py-3">
          <h2 className="text-sm font-bold text-zinc-900">{title}</h2>
          <button type="button" onClick={onClose} aria-label="Close" className="rounded-md p-1 text-zinc-500 hover:bg-zinc-100">
            <X className="h-4 w-4" />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

export function ExampleModal({ open, currentId, onPick, onClose }: {
  open: boolean;
  currentId: string;
  onPick: (id: string) => void;
  onClose: () => void;
}) {
  if (!open) return null;
  return (
    <Shell title="Choose an example" onClose={onClose}>
      <div className="overflow-y-auto p-2">
        {EXAMPLE_GROUPS.map((g) => (
          <div key={g} className="mb-2">
            <div className="px-2 py-1 text-[11px] font-bold uppercase tracking-wide text-zinc-500">{g}</div>
            {EXAMPLES.filter((e) => e.group === g).map((e) => (
              <button
                key={e.id}
                type="button"
                onClick={() => onPick(e.id)}
                className={`block w-full rounded-lg px-3 py-2 text-left hover:bg-emerald-50 ${e.id === currentId ? "bg-emerald-50" : ""}`}
              >
                <div className="text-sm font-semibold text-zinc-900">{e.name}</div>
                <div className="text-xs text-zinc-500">{e.description}</div>
              </button>
            ))}
          </div>
        ))}
      </div>
    </Shell>
  );
}

export function ConfirmModal({ open, message, confirmLabel, onResult }: {
  open: boolean;
  message: string;
  confirmLabel: string;
  onResult: (ok: boolean) => void;
}) {
  if (!open) return null;
  return (
    <Shell title="Are you sure?" onClose={() => onResult(false)}>
      <div className="space-y-4 p-4">
        <p className="text-sm text-zinc-700">{message}</p>
        <div className="flex justify-end gap-2">
          <button type="button" onClick={() => onResult(false)} className="h-9 rounded-md border border-zinc-300 px-3 text-sm font-medium text-zinc-700 hover:bg-zinc-50">
            Cancel
          </button>
          <button type="button" onClick={() => onResult(true)} className="h-9 rounded-md bg-rose-600 px-3 text-sm font-semibold text-white hover:bg-rose-700">
            {confirmLabel}
          </button>
        </div>
      </div>
    </Shell>
  );
}
