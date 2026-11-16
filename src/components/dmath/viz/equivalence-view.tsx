"use client";

// Logical equivalence check renderer for the equivalence_check IR.

import { Link2, Unlink2, GitCompareArrows } from "lucide-react";
import type { EquivalenceIr } from "@/lib/dmath/types";

function Bit({ v }: { v: boolean }) {
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

export function EquivalenceView({ ir }: { ir: EquivalenceIr }) {
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <GitCompareArrows className="h-3.5 w-3.5 text-violet-400" />
          {ir.left} &nbsp;≡?&nbsp; {ir.right}
        </span>
        <span
          className={`inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs font-bold tracking-wide ${
            ir.equivalent
              ? "border-emerald-300 bg-emerald-100 text-emerald-800"
              : "border-rose-300 bg-rose-100 text-rose-800"
          }`}
        >
          {ir.equivalent ? <Link2 className="h-4 w-4" /> : <Unlink2 className="h-4 w-4" />}
          {ir.equivalent ? "LOGICALLY EQUIVALENT" : "NOT EQUIVALENT"}
        </span>
      </div>

      <div className="overflow-hidden rounded-lg border border-zinc-200">
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
                  {ir.left}
                </th>
                <th className="border-b border-zinc-300 px-3 py-2 text-center font-mono text-xs font-bold text-violet-700">
                  {ir.right}
                </th>
              </tr>
            </thead>
            <tbody>
              {ir.rows.map((row, i) => {
                const mismatch = row["left"] !== row["right"];
                return (
                  <tr
                    key={i}
                    className={`${mismatch ? "bg-rose-50" : i % 2 === 0 ? "bg-white" : "bg-zinc-50/60"}`}
                  >
                    {ir.variables.map((v) => (
                      <td key={v} className="px-3 py-1.5 text-center">
                        <Bit v={row[v]} />
                      </td>
                    ))}
                    <td className="px-3 py-1.5 text-center">
                      <Bit v={row["left"]} />
                    </td>
                    <td className="px-3 py-1.5 text-center">
                      <Bit v={row["right"]} />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      <p className="text-xs text-zinc-500">
        {ir.equivalent
          ? "Both expressions evaluate to the same value for every assignment of the variables."
          : "The highlighted rows show assignments where the two expressions disagree."}
      </p>
    </div>
  );
}
