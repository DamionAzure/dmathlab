"use client";

// Pascal's-triangle renderer for the pascal Visualization IR.
// Rows are centered; every cell equals C(row, col) — the sum of the two
// cells above it. A cell originating from COMB(n, k) is highlighted.

import { Triangle } from "lucide-react";
import type { PascalIr } from "@/lib/dmath/types";

export function PascalView({ ir }: { ir: PascalIr }) {
  const highlighted = ir.highlight;

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Triangle className="h-3.5 w-3.5 text-orange-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-orange-300 bg-orange-50 px-2 py-1 text-xs font-bold tracking-wide text-orange-800">
          PASCAL&apos;S TRIANGLE
        </span>
        <span className="font-mono text-xs text-zinc-500">
          {ir.row_count} rows
        </span>
      </div>

      <div className="overflow-x-auto rounded-lg border border-zinc-200 bg-white p-4">
        <div className="flex flex-col items-center gap-1">
          {ir.rows.map((row, i) => (
            <div key={i} className="flex items-center justify-center gap-1">
              {row.map((value, j) => {
                const isHighlight = highlighted?.[0] === i && highlighted?.[1] === j;
                return (
                  <span
                    key={j}
                    title={`C(${i}, ${j}) = ${value}`}
                    className={`inline-flex min-w-14 justify-center rounded px-2 py-1 font-mono text-xs tabular-nums ${
                      isHighlight
                        ? "bg-amber-100 font-bold text-amber-900 ring-2 ring-amber-400"
                        : "bg-zinc-50 text-zinc-700 hover:bg-orange-50 hover:text-orange-700"
                    }`}
                  >
                    {value}
                  </span>
                );
              })}
            </div>
          ))}
        </div>
      </div>

      {highlighted && (
        <p className="rounded-md bg-amber-50 px-3 py-2 text-xs text-amber-800">
          {ir.note ?? `Highlighted cell: C(${highlighted[0]}, ${highlighted[1]})`} — every entry
          above it feeds into this value.
        </p>
      )}

      <p className="text-xs text-zinc-500">
        Row <span className="font-mono">i</span> holds the binomial coefficients C(i, 0) …
        C(i, i); each interior entry is the sum of the two entries above it, and row sums are the
        powers of two 1, 2, 4, 8, …
      </p>
    </div>
  );
}
