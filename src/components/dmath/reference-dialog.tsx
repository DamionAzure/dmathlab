"use client";

// Help → Language Reference: searchable list of every statement, operator
// and function, each with a runnable example that can be dropped into the editor.

import { useEffect, useMemo, useRef, useState } from "react";
import { Search, X } from "lucide-react";

import { REFERENCE, type RefEntry } from "@/lib/dmath/reference";

interface Props {
  open: boolean;
  onClose(): void;
  /** Replace the editor contents with this program. */
  onTry(code: string): void;
}

function matches(e: RefEntry, q: string): boolean {
  const hay = `${e.name} ${e.syntax} ${e.summary} ${e.returns ?? ""}`.toLowerCase();
  return q.split(/\s+/).every((w) => hay.includes(w));
}

export function ReferenceDialog({ open, onClose, onTry }: Props) {
  const [query, setQuery] = useState("");
  const [active, setActive] = useState("all");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!open) return;
    setQuery("");
    inputRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  const q = query.trim().toLowerCase();
  const sections = useMemo(
    () =>
      REFERENCE.filter((s) => active === "all" || s.id === active)
        .map((s) => ({ ...s, entries: q ? s.entries.filter((e) => matches(e, q)) : s.entries }))
        .filter((s) => s.entries.length > 0),
    [q, active],
  );

  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/40 p-4 pt-[6vh] backdrop-blur-[2px]"
      onMouseDown={(e) => e.target === e.currentTarget && onClose()}
      role="dialog"
      aria-modal="true"
      aria-label="Language reference"
    >
      <div className="flex max-h-[86vh] w-full max-w-3xl flex-col overflow-hidden rounded-2xl border border-zinc-200 bg-white shadow-2xl">
        <div className="flex items-center gap-3 border-b border-zinc-200 px-4 py-3">
          <Search className="h-4 w-4 shrink-0 text-zinc-400" aria-hidden />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search the language: GCD, union, shortest path…"
            className="min-w-0 flex-1 bg-transparent text-sm text-zinc-900 outline-none placeholder:text-zinc-400"
          />
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="rounded-md p-1 text-zinc-500 hover:bg-zinc-100"
          >
            <X className="h-4 w-4" />
          </button>
        </div>

        <div className="flex flex-wrap gap-1.5 border-b border-zinc-200 px-4 py-2">
          {[{ id: "all", title: "All" }, ...REFERENCE].map((s) => (
            <button
              key={s.id}
              type="button"
              onClick={() => setActive(s.id)}
              className={`rounded-full px-2.5 py-0.5 text-xs font-medium ${
                active === s.id ? "bg-zinc-900 text-white" : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200"
              }`}
            >
              {s.title}
            </button>
          ))}
        </div>

        <div className="space-y-6 overflow-y-auto px-4 py-4">
          {sections.length === 0 && <p className="py-8 text-center text-sm text-zinc-500">Nothing matches “{query}”.</p>}
          {sections.map((s) => (
            <section key={s.id}>
              <h3 className="text-sm font-semibold text-zinc-900">{s.title}</h3>
              {s.blurb && !q && <p className="mt-1 text-xs leading-relaxed text-zinc-600">{s.blurb}</p>}
              <div className="mt-2 space-y-2">
                {s.entries.map((e) => (
                  <article key={e.name} className="rounded-lg border border-zinc-200 bg-zinc-50/60 p-3">
                    <div className="flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1">
                      <code className="font-mono text-[13px] font-semibold text-emerald-700">{e.syntax}</code>
                      {e.returns && <span className="text-[11px] text-zinc-500">gives {e.returns}</span>}
                    </div>
                    <p className="mt-1 text-xs leading-relaxed text-zinc-700">{e.summary}</p>
                    <pre className="keep-dark mt-2 overflow-x-auto rounded-md bg-zinc-950 px-3 py-2 font-mono text-[12px] leading-relaxed text-zinc-100">
                      {e.example}
                    </pre>
                    <button
                      type="button"
                      onClick={() => {
                        onTry(e.example);
                        onClose();
                      }}
                      className="mt-2 rounded-md border border-zinc-300 bg-white px-2 py-0.5 text-[11px] font-medium text-zinc-700 hover:bg-zinc-100"
                    >
                      Try it in the editor
                    </button>
                  </article>
                ))}
              </div>
            </section>
          ))}
        </div>
      </div>
    </div>
  );
}
