"use client";

// Worked-solution renderer for the `steps` Visualization IR.
// Each section is one independent *method*; methods that compute their own
// answer carry an "agrees" flag, and the footer states whether they all match.

import { CheckCircle2, ListOrdered, TriangleAlert, XCircle } from "lucide-react";
import type { StepLine, StepSection, StepsIr } from "@/lib/dmath/types";

const TONE: Record<StepLine["tone"], string> = {
  plain: "text-zinc-700",
  head: "mt-2 text-[11px] font-bold uppercase tracking-wider text-zinc-500 first:mt-0",
  good: "font-medium text-emerald-700",
  bad: "font-medium text-rose-700",
};

function Table({ headers, rows }: { headers: string[]; rows: string[][] }) {
  return (
    <div className="mt-2 overflow-x-auto rounded-lg border border-zinc-200">
      <table className="w-full border-collapse font-mono text-[12px] tabular-nums">
        <thead>
          <tr className="bg-zinc-50 text-left text-[11px] uppercase tracking-wide text-zinc-500">
            {headers.map((h, i) => (
              <th key={i} className="whitespace-nowrap border-b border-zinc-200 px-2.5 py-1.5 font-semibold">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r, i) => {
            const last = i === rows.length - 1;
            return (
              <tr key={i} className={last ? "bg-emerald-50/60" : i % 2 ? "bg-zinc-50/40" : ""}>
                {r.map((c, j) => (
                  <td key={j} className="whitespace-nowrap border-b border-zinc-100 px-2.5 py-1 text-zinc-800 last:border-b-0">
                    {c}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function Section({ s, index }: { s: StepSection; index: number }) {
  return (
    <details open className="group rounded-xl border border-zinc-200 bg-white shadow-sm">
      <summary className="flex cursor-pointer list-none items-center gap-3 px-3.5 py-2.5 [&::-webkit-details-marker]:hidden">
        <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-zinc-900 text-[11px] font-bold text-white">
          {index}
        </span>
        <span className="flex-1 text-sm font-semibold text-zinc-900">{s.title}</span>
        {s.agrees === true && (
          <span className="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-0.5 text-[11px] font-semibold text-emerald-700 ring-1 ring-emerald-200">
            <CheckCircle2 className="h-3.5 w-3.5" /> matches
          </span>
        )}
        {s.agrees === false && (
          <span className="inline-flex items-center gap-1 rounded-full bg-rose-50 px-2 py-0.5 text-[11px] font-semibold text-rose-700 ring-1 ring-rose-200">
            <XCircle className="h-3.5 w-3.5" /> differs
          </span>
        )}
        <span className="text-xs text-zinc-400 transition-transform group-open:rotate-90">▶</span>
      </summary>
      <div className="space-y-1.5 border-t border-zinc-100 px-3.5 pb-3.5 pt-3">
        {s.intro && <p className="text-[12.5px] leading-relaxed text-zinc-500">{s.intro}</p>}
        {s.steps.length > 0 && (
          <ul className="space-y-1 font-mono text-[12.5px] leading-relaxed">
            {s.steps.map((st, i) => (
              <li key={i} className={TONE[st.tone]}>
                {st.text}
              </li>
            ))}
          </ul>
        )}
        {s.table && <Table headers={s.table.headers} rows={s.table.rows} />}
        {s.outcome && (
          <div className="mt-2 inline-block rounded-md border border-emerald-200 bg-emerald-50 px-2.5 py-1 font-mono text-[12.5px] font-semibold text-emerald-800">
            → {s.outcome}
          </div>
        )}
      </div>
    </details>
  );
}

export function StepsView({ ir }: { ir: StepsIr }) {
  const methods = ir.sections.filter((s) => s.agrees !== undefined).length;
  const display = ir.result.length > 60 ? `${ir.result.slice(0, 60)}…` : ir.result;
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <ListOrdered className="h-3.5 w-3.5 text-emerald-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-emerald-300 bg-emerald-50 px-2 py-1 text-xs font-bold tracking-wide text-emerald-800">
          STEP BY STEP
        </span>
        <span className="font-mono text-xs text-zinc-500">{ir.title}</span>
      </div>

      <div className="space-y-2.5">
        {ir.sections.map((s, i) => (
          <Section key={i} s={s} index={i + 1} />
        ))}
      </div>

      {ir.verified ? (
        <div className="flex items-start gap-2 rounded-xl border border-emerald-300 bg-emerald-50 px-3.5 py-2.5 text-sm text-emerald-900">
          <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0 text-emerald-600" />
          <div>
            <span className="font-semibold">
              {methods > 1 ? `All ${methods} methods agree: ` : "Answer: "}
            </span>
            <span className="break-all font-mono font-bold">
              {ir.expression} = {display}
            </span>
          </div>
        </div>
      ) : (
        <div className="flex items-start gap-2 rounded-xl border border-rose-300 bg-rose-50 px-3.5 py-2.5 text-sm text-rose-900">
          <TriangleAlert className="mt-0.5 h-4 w-4 shrink-0 text-rose-600" />
          <div>
            <span className="font-semibold">The methods disagree.</span> This is a bug in the explanation. Please report it with the
            numbers you used. The engine's answer is <span className="font-mono font-bold">{display}</span>.
          </div>
        </div>
      )}
    </div>
  );
}
