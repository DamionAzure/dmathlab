"use client";

// Relation-matrix renderer for the relation_matrix Visualization IR.
// An HTML table: header row/column carry the vertex labels, entry (i, j)
// is tinted when (vi, vj) ∈ R.

import { Grid3x3 } from "lucide-react";
import type { RelationMatrixIr } from "@/lib/dmath/types";

export function MatrixView({ ir }: { ir: RelationMatrixIr }) {
  const n = ir.vertices.length;
  const rows = ir.matrix.length;

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Grid3x3 className="h-3.5 w-3.5 text-violet-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-violet-300 bg-violet-50 px-2 py-1 text-xs font-bold tracking-wide text-violet-800">
          RELATION MATRIX
        </span>
        <span className="font-mono text-xs text-zinc-500">
          {n} × {n}
        </span>
      </div>

      {n === 0 || rows === 0 ? (
        <p className="rounded-lg border border-dashed border-zinc-300 py-8 text-center text-sm text-zinc-400">
          Empty relation — nothing to tabulate.
        </p>
      ) : (
        <div className="overflow-hidden rounded-lg border border-zinc-200">
          <div className="max-h-96 overflow-auto">
            <table className="border-collapse font-mono text-xs">
              <thead>
                <tr>
                  <th className="sticky left-0 top-0 z-20 border-b border-r border-zinc-300 bg-zinc-100 px-3 py-2 text-zinc-400" aria-label="corner cell" />
                  {ir.vertices.map((v) => (
                    <th
                      key={v}
                      className="sticky top-0 z-10 border-b border-zinc-300 bg-zinc-100 px-3 py-2 text-center font-bold text-zinc-700"
                    >
                      {v}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {ir.matrix.slice(0, n).map((row, i) => (
                  <tr key={i} className={i % 2 === 0 ? "bg-white" : "bg-zinc-50/60"}>
                    <th className="sticky left-0 z-10 border-b border-r border-zinc-300 bg-zinc-100 px-3 py-1.5 text-center font-bold text-zinc-700">
                      {ir.vertices[i]}
                    </th>
                    {ir.vertices.map((_, j) => {
                      const cell = row[j] ?? 0;
                      return (
                        <td
                          key={j}
                          className={`px-3 py-1.5 text-center ${
                            cell === 1
                              ? "bg-emerald-100/80 font-bold text-emerald-800"
                              : "text-zinc-300"
                          }`}
                        >
                          {cell === 1 ? 1 : 0}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <p className="text-xs text-zinc-500">
        Entry (i, j) = 1 ⇔ (vᵢ, vⱼ) ∈ R — rows are the first component, columns the second.
      </p>
    </div>
  );
}
