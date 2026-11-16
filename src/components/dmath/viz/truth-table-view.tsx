"use client";

// Truth table renderer for the truth_table Visualization IR.

import { CheckCircle2, XCircle, MinusCircle, Table2 } from "lucide-react";
import type { TruthTableIr } from "@/lib/dmath/types";

const CLASS_STYLE: Record<
  TruthTableIr["classification"],
  { label: string; badge: string; icon: React.ReactNode; ring: string }
> = {
  tautology: {
    label: "TAUTOLOGY",
    badge: "bg-emerald-100 text-emerald-800 border-emerald-300",
    icon: <CheckCircle2 className="h-4 w-4" />,
    ring: "border-emerald-300 bg-emerald-50",
  },
  contradiction: {
    label: "CONTRADICTION",
    badge: "bg-rose-100 text-rose-800 border-rose-300",
    icon: <XCircle className="h-4 w-4" />,
    ring: "border-rose-300 bg-rose-50",
  },
  contingency: {
    label: "CONTINGENCY",
    badge: "bg-amber-100 text-amber-800 border-amber-300",
    icon: <MinusCircle className="h-4 w-4" />,
    ring: "border-amber-300 bg-amber-50",
  },
};

function Bit({ v }: { v: boolean | undefined }) {
  if (v === undefined) return <span className="text-zinc-400">—</span>;
  return (
    <span
      className={`inline-block min-w-14 rounded px-2 py-0.5 text-center text-xs font-semibold ${
        v ? "bg-emerald-100 text-emerald-700" : "bg-rose-100 text-rose-700"
      }`}
    >
      {v ? "TRUE" : "FALSE"}
    </span>
  );
}

export function TruthTableView({ ir }: { ir: TruthTableIr }) {
  const style = CLASS_STYLE[ir.classification];
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Table2 className="h-3.5 w-3.5 text-emerald-400" />
          {ir.expression}
        </span>
        <span
          className={`inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs font-bold tracking-wide ${style.badge}`}
        >
          {style.icon}
          {style.label}
        </span>
        <span className="text-xs text-zinc-500">
          {ir.row_count} row{ir.row_count === 1 ? "" : "s"}
        </span>
      </div>

      <div className={`overflow-hidden rounded-lg border ${style.ring}`}>
        <div className="max-h-96 overflow-auto">
          <table className="w-full border-collapse text-sm">
            <thead className="sticky top-0 z-10 bg-zinc-100">
              <tr>
                {ir.variables.map((v) => (
                  <th
                    key={v}
                    className="border-b border-zinc-300 px-3 py-2 text-center font-mono text-xs font-bold text-zinc-700"
                  >
                    {v}
                  </th>
                ))}
                <th className="border-b border-zinc-300 px-3 py-2 text-center font-mono text-xs font-bold text-emerald-700">
                  {ir.expression}
                </th>
              </tr>
            </thead>
            <tbody>
              {ir.rows.map((row, i) => (
                <tr key={i} className={i % 2 === 0 ? "bg-white" : "bg-zinc-50/60"}>
                  {ir.variables.map((v) => (
                    <td key={v} className="px-3 py-1.5 text-center">
                      <Bit v={row[v]} />
                    </td>
                  ))}
                  <td className="px-3 py-1.5 text-center font-semibold">
                    <Bit v={row["result"]} />
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <p className="text-xs text-zinc-500">{ir.classification_note}</p>
    </div>
  );
}
