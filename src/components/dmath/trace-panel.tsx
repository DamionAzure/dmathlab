"use client";

// Execution trace: per-statement steps with expandable intermediate
// evaluation entries (spec PL-07 / ST-09).

import { useState } from "react";
import { Check, ChevronDown, ChevronRight, CircleAlert, ListTree } from "lucide-react";
import type { DMathEnvelope } from "@/lib/dmath/types";

const KIND_LABEL: Record<string, string> = {
  assignment: "ASSIGN",
  display: "DISPLAY",
  visualize: "VIZ",
  check_equivalent: "CHECK",
  check_properties: "CHECK PROPERTIES",
};

export function TracePanel({ envelope }: { envelope: DMathEnvelope | null }) {
  const [open, setOpen] = useState<Set<number>>(new Set());

  if (!envelope) {
    return (
      <p className="py-6 text-center text-sm text-zinc-400">
        The execution trace explains how each statement was evaluated.
      </p>
    );
  }

  const toggle = (step: number) => {
    setOpen((prev) => {
      const next = new Set(prev);
      if (next.has(step)) next.delete(step);
      else next.add(step);
      return next;
    });
  };

  return (
    <div className="max-h-[26rem] space-y-1.5 overflow-y-auto pr-1">
      {envelope.trace.map((t) => {
        const hasInter = t.intermediates.length > 0;
        const expanded = open.has(t.step);
        return (
          <div
            key={t.step}
            className={`rounded-lg border p-2.5 ${
              t.status === "error" ? "border-rose-200 bg-rose-50/50" : "border-zinc-200 bg-white"
            }`}
          >
            <div className="flex items-start gap-2.5">
              <span className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-zinc-800 font-mono text-[10px] font-bold text-emerald-300">
                {t.step}
              </span>
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="rounded bg-zinc-100 px-1.5 py-0.5 font-mono text-[10px] font-bold tracking-wide text-zinc-500">
                    {KIND_LABEL[t.kind] ?? t.kind}
                  </span>
                  <code className="truncate font-mono text-xs text-zinc-800" title={t.statement}>
                    {t.statement}
                  </code>
                  {t.status === "error" ? (
                    <CircleAlert className="h-3.5 w-3.5 shrink-0 text-rose-500" />
                  ) : (
                    <Check className="h-3.5 w-3.5 shrink-0 text-emerald-500" />
                  )}
                </div>
                <p className="mt-1 truncate font-mono text-xs text-zinc-500" title={t.summary}>
                  {t.summary}
                </p>

                {hasInter && (
                  <div className="mt-1.5">
                    <button
                      type="button"
                      onClick={() => toggle(t.step)}
                      className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] font-medium text-violet-700 transition-colors hover:bg-violet-50"
                      aria-expanded={expanded}
                    >
                      <ListTree className="h-3.5 w-3.5" />
                      {t.intermediates.length} intermediate step{t.intermediates.length === 1 ? "" : "s"}
                      {expanded ? (
                        <ChevronDown className="h-3 w-3" />
                      ) : (
                        <ChevronRight className="h-3 w-3" />
                      )}
                    </button>
                    {expanded && (
                      <div className="mt-1.5 space-y-1 border-l-2 border-violet-200 pl-2.5">
                        {t.intermediates.map((it, i) => (
                          <div key={i} className="font-mono text-[11px] leading-relaxed">
                            <span className="text-zinc-700">{it.expr}</span>
                            <span className="text-zinc-400"> = </span>
                            <span className="font-semibold text-emerald-700">{it.result}</span>
                            {it.detail && (
                              <span className="text-zinc-400"> · {it.detail}</span>
                            )}
                          </div>
                        ))}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
