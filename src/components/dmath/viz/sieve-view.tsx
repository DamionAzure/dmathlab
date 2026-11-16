"use client";

// Sieve-of-Eratosthenes grid renderer for the sieve_grid Visualization IR.
// One cell per number 0..=limit: primes are solid emerald, composites are
// struck through and tinted by their smallest prime factor (the sieve
// "crosses out" each composite through one of its prime divisors).

import { Eraser } from "lucide-react";
import type { SieveIr } from "@/lib/dmath/types";

/** Cells actually rendered (the IR keeps every number up to the limit). */
const MAX_RENDERED_CELLS = 2000;

/** Deterministic tint per smallest-prime-factor, so 2's multiples always
 * share one color, 3's another, and so on. */
const SPF_TINTS: Array<{ bg: string; dot: string; label: string }> = [
  { bg: "bg-rose-100 text-rose-400", dot: "bg-rose-400", label: "multiple of 2" },
  { bg: "bg-amber-100 text-amber-400", dot: "bg-amber-400", label: "multiple of 3" },
  { bg: "bg-sky-100 text-sky-400", dot: "bg-sky-400", label: "multiple of 5" },
  { bg: "bg-violet-100 text-violet-400", dot: "bg-violet-400", label: "multiple of 7" },
  { bg: "bg-fuchsia-100 text-fuchsia-400", dot: "bg-fuchsia-400", label: "multiple of 11" },
  { bg: "bg-cyan-100 text-cyan-400", dot: "bg-cyan-400", label: "multiple of 13" },
];

function tintFor(spf: number) {
  switch (spf) {
    case 2: return SPF_TINTS[0];
    case 3: return SPF_TINTS[1];
    case 5: return SPF_TINTS[2];
    case 7: return SPF_TINTS[3];
    case 11: return SPF_TINTS[4];
    case 13: return SPF_TINTS[5];
    default: return SPF_TINTS[spf % SPF_TINTS.length];
  }
}

export function SieveView({ ir }: { ir: SieveIr }) {
  const truncated = ir.limit + 1 > MAX_RENDERED_CELLS;
  const shown = truncated ? MAX_RENDERED_CELLS : ir.limit + 1;
  const primeSet = new Set(ir.primes);

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Eraser className="h-3.5 w-3.5 text-amber-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 text-xs font-bold tracking-wide text-amber-800">
          SIEVE OF ERATOSTHENES
        </span>
        <span className="font-mono text-xs text-zinc-500">
          {ir.prime_count} primes ≤ {ir.limit}
        </span>
      </div>

      <div className="flex flex-wrap gap-1 rounded-lg border border-zinc-200 bg-white p-3">
        {Array.from({ length: shown }, (_, n) => {
          const isPrime = primeSet.has(n);
          const spf = ir.smallest_factor[String(n)];
          const tint = spf !== undefined ? tintFor(spf) : undefined;
          return (
            <span
              key={n}
              title={
                n < 2
                  ? `${n} — neither prime nor composite`
                  : isPrime
                    ? `${n} — prime`
                    : `${n} = ${spf} × ${n / spf} — crossed out by ${spf}`
              }
              className={`inline-flex h-8 w-9 items-center justify-center rounded font-mono text-[11px] leading-none ${
                isPrime
                  ? "bg-emerald-500 font-bold text-white"
                  : n < 2
                    ? "border border-dashed border-zinc-300 text-zinc-300"
                    : `line-through decoration-2 ${tint?.bg ?? "bg-zinc-100 text-zinc-400"}`
              }`}
            >
              {n}
            </span>
          );
        })}
      </div>

      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-zinc-500">
        <span className="inline-flex items-center gap-1.5">
          <span className="inline-block h-3 w-4 rounded bg-emerald-500" /> prime
        </span>
        {SPF_TINTS.slice(0, 4).map((t) => (
          <span key={t.label} className="inline-flex items-center gap-1.5">
            <span className={`inline-block h-2.5 w-2.5 rounded-full ${t.dot}`} /> {t.label}
          </span>
        ))}
        <span className="inline-flex items-center gap-1.5">
          <span className="inline-block h-3 w-4 rounded border border-dashed border-zinc-300" />
          0 and 1 (neither)
        </span>
      </div>

      {truncated && (
        <p className="rounded-md bg-amber-50 px-3 py-2 text-xs text-amber-800">
          Grid shows the first {shown} numbers up to {ir.limit}; all {ir.prime_count} primes are
          listed in the Output panel.
        </p>
      )}

      <p className="text-xs text-zinc-500">
        The sieve walks p = 2, 3, 5, … and crosses out every multiple starting at p² — whatever
        survives unmarked is prime. Hover a cell to see its smallest prime factor.
      </p>
    </div>
  );
}
