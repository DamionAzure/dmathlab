"use client";

// Output panel: computed values, property-check cards, diagnostics
// (errors / warnings / info).

import {
  AlertTriangle,
  AlertCircle,
  Info,
  Lightbulb,
  Hash,
  Braces,
  List,
  Map,
  Network,
  ToggleLeft,
} from "lucide-react";
import type { DiagnosticJson, DMathEnvelope, Elem } from "@/lib/dmath/types";
import { isPropertyCheckIr } from "@/lib/dmath/types";
import { PropertiesView } from "@/components/dmath/viz/properties-view";

function ElemChip({ e }: { e: Elem }) {
  const text = Array.isArray(e) ? `(${e.map(String).join(", ")})` : String(e);
  return (
    <span className="inline-flex items-center rounded border border-zinc-200 bg-zinc-50 px-1.5 py-0.5 font-mono text-xs text-zinc-700">
      {text}
    </span>
  );
}

function TypeIcon({ type }: { type: string }) {
  if (type === "Set") return <Braces className="h-3.5 w-3.5" />;
  if (type === "Boolean") return <ToggleLeft className="h-3.5 w-3.5" />;
  if (type === "List") return <List className="h-3.5 w-3.5" />;
  if (type === "Mapping") return <Map className="h-3.5 w-3.5" />;
  if (type === "Graph") return <Network className="h-3.5 w-3.5" />;
  return <Hash className="h-3.5 w-3.5" />;
}

const TYPE_STYLE: Record<string, string> = {
  Boolean: "border-emerald-300 bg-emerald-50 text-emerald-700",
  Set: "border-violet-300 bg-violet-50 text-violet-700",
  Number: "border-teal-300 bg-teal-50 text-teal-700",
  List: "border-sky-300 bg-sky-50 text-sky-700",
  Mapping: "border-lime-300 bg-lime-50 text-lime-700",
  Graph: "border-rose-300 bg-rose-50 text-rose-700",
};

function DiagnosticCard({ d, onJump }: { d: DiagnosticJson; onJump?: (line: number) => void }) {
  const style =
    d.severity === "error"
      ? { border: "border-rose-300 bg-rose-50", icon: <AlertCircle className="h-4 w-4 text-rose-600" />, title: "text-rose-900" }
      : d.severity === "warning"
        ? { border: "border-amber-300 bg-amber-50", icon: <AlertTriangle className="h-4 w-4 text-amber-600" />, title: "text-amber-900" }
        : { border: "border-zinc-300 bg-zinc-50", icon: <Info className="h-4 w-4 text-zinc-500" />, title: "text-zinc-700" };

  return (
    <div className={`rounded-lg border p-3 text-sm ${style.border}`}>
      <div className="flex items-start gap-2.5">
        <span className="mt-0.5 shrink-0">{style.icon}</span>
        <div className="min-w-0 flex-1">
          <div className="flex flex-wrap items-center gap-2">
            <span className={`font-medium ${style.title}`}>{d.message}</span>
            <button
              type="button"
              onClick={() => onJump?.(d.line)}
              disabled={!onJump}
              title={onJump ? "Jump to this line in the editor" : undefined}
              className={`rounded border border-zinc-200 bg-white px-1.5 py-0.5 font-mono text-[11px] text-zinc-500 ${onJump ? "underline decoration-dotted underline-offset-2 hover:border-zinc-400 hover:text-zinc-800" : ""}`}
            >
              line {d.line}, col {d.col}
            </button>
            <span className="rounded bg-zinc-100 px-1.5 py-0.5 text-[11px] font-semibold uppercase tracking-wide text-zinc-500">
              {d.severity}
            </span>
          </div>
          {d.hint && (
            <p className="mt-1.5 flex items-start gap-1.5 text-xs italic text-zinc-600">
              <Lightbulb className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
              {d.hint}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

export function OutputPanel({ envelope, onJump }: { envelope: DMathEnvelope | null; onJump?: (line: number) => void }) {
  if (!envelope) {
    return (
      <p className="py-6 text-center text-sm text-zinc-400">
        Run a program to see computed values here.
      </p>
    );
  }

  const errors = envelope.diagnostics.filter((d) => d.severity === "error");
  const others = envelope.diagnostics.filter((d) => d.severity !== "error");
  // Property checks arrive as Visualization IR but are rendered as rich cards
  // in the Output panel, alongside the value cards.
  const propertyChecks = envelope.visualizations.filter(isPropertyCheckIr);

  return (
    <div className="space-y-3">
      {errors.length > 0 && (
        <div className="space-y-2">
          {errors.map((d, i) => (
            <DiagnosticCard key={i} d={d} onJump={onJump} />
          ))}
        </div>
      )}

      {envelope.outputs.length > 0 ? (
        <div className="space-y-2">
          {envelope.outputs.map((o, i) => (
            <div
              key={i}
              className="rounded-lg border border-zinc-200 bg-white p-3 transition-colors hover:border-zinc-300"
            >
              <div className="flex flex-wrap items-center gap-2">
                {o.name && (
                  <span className="rounded bg-zinc-800 px-2 py-0.5 font-mono text-xs font-bold text-emerald-300">
                    {o.name}
                  </span>
                )}
                <span
                  className={`inline-flex items-center gap-1 rounded border px-1.5 py-0.5 text-[11px] font-semibold ${TYPE_STYLE[o.value_type] ?? "border-zinc-300 bg-zinc-50 text-zinc-600"}`}
                >
                  <TypeIcon type={o.value_type} />
                  {o.value_type}
                </span>
                {!o.name && (
                  <span className="truncate font-mono text-xs text-zinc-400" title={o.expression}>
                    {o.expression}
                  </span>
                )}
              </div>
              <div className="mt-2">
                {o.elements ? (
                  <div className="flex flex-wrap gap-1.5">
                    {o.elements.map((e, j) => (
                      <ElemChip key={j} e={e} />
                    ))}
                    {o.elements.length === 0 && (
                      <span className="font-mono text-sm text-zinc-500">{"{}"} (empty)</span>
                    )}
                  </div>
                ) : (
                  <span
                    className={`font-mono text-sm font-semibold ${
                      o.display === "TRUE" ? "text-emerald-700" : o.display === "FALSE" ? "text-rose-700" : "text-zinc-800"
                    }`}
                  >
                    {o.display}
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      ) : (
        errors.length === 0 && propertyChecks.length === 0 && (
          <p className="py-2 text-center text-sm text-zinc-400">
            No DISPLAY statements in this program.
          </p>
        )
      )}

      {propertyChecks.length > 0 && (
        <div className="space-y-2">
          {propertyChecks.map((ir, i) => (
            <PropertiesView key={i} ir={ir} />
          ))}
        </div>
      )}

      {others.length > 0 && (
        <div className="space-y-2">
          {others.map((d, i) => (
            <DiagnosticCard key={i} d={d} onJump={onJump} />
          ))}
        </div>
      )}
    </div>
  );
}
