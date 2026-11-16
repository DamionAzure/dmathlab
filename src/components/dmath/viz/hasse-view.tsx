"use client";

// Hasse diagram renderer for the hasse Visualization IR.
// levels[0] is drawn at the bottom, levels rise upward; cover edges are
// straight lines between node boundaries. Only meaningful for partial orders.

import { useId } from "react";
import { Layers } from "lucide-react";
import type { HasseIr } from "@/lib/dmath/types";

interface Pt {
  x: number;
  y: number;
}

const NODE_R = 14;
const COL_GAP = 64;
const ROW_GAP = 78;

function pairKey(a: number, b: number): string {
  return a < b ? `${a}-${b}` : `${b}-${a}`;
}

export function HasseView({ ir }: { ir: HasseIr }) {
  const uid = useId().replace(/[^a-zA-Z0-9]/g, "");
  const glowId = `hasse-glow-${uid}`;

  const levels = ir.levels;
  const nLevels = levels.length;
  const vertexCount = levels.reduce((acc, l) => acc + l.length, 0);

  // Layout: responsive viewBox sized by the widest level and the level count.
  const maxPerLevel = Math.max(1, ...levels.map((l) => l.length));
  const W = Math.max(380, maxPerLevel * COL_GAP + 90);
  const H = Math.max(210, (nLevels - 1) * ROW_GAP + 130);
  const bottomY = H - 62;

  const pos = new Map<number, Pt>();
  levels.forEach((level, li) => {
    const y = bottomY - li * ROW_GAP;
    level.forEach((v, j) => {
      pos.set(v, { x: W / 2 + (j - (level.length - 1) / 2) * COL_GAP, y });
    });
  });

  // Deduplicate cover edges defensively.
  const seen = new Set<string>();
  const segments: { x1: number; y1: number; x2: number; y2: number }[] = [];
  for (const [a, b] of ir.edges) {
    const key = pairKey(a, b);
    if (seen.has(key)) continue;
    seen.add(key);
    const p1 = pos.get(a);
    const p2 = pos.get(b);
    if (!p1 || !p2) continue;
    const len = Math.hypot(p2.x - p1.x, p2.y - p1.y);
    if (len < 1e-9) continue;
    const t = { x: (p2.x - p1.x) / len, y: (p2.y - p1.y) / len };
    segments.push({
      x1: p1.x + t.x * (NODE_R + 2),
      y1: p1.y + t.y * (NODE_R + 2),
      x2: p2.x - t.x * (NODE_R + 2),
      y2: p2.y - t.y * (NODE_R + 2),
    });
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Layers className="h-3.5 w-3.5 text-amber-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 text-xs font-bold tracking-wide text-amber-800">
          HASSE DIAGRAM
        </span>
        <span className="font-mono text-xs text-zinc-500">
          {vertexCount} element{vertexCount === 1 ? "" : "s"} · {nLevels} level{nLevels === 1 ? "" : "s"}
        </span>
      </div>

      {vertexCount === 0 ? (
        <p className="rounded-lg border border-dashed border-zinc-300 py-8 text-center text-sm text-zinc-400">
          Empty relation — nothing to rank.
        </p>
      ) : (
        <svg
          viewBox={`0 0 ${W} ${H}`}
          className="w-full max-w-xl rounded-lg border border-zinc-200 bg-white"
          role="img"
          aria-label={`Hasse diagram of ${ir.expression}`}
        >
          <defs>
            <radialGradient id={glowId} cx="50%" cy="100%" r="90%">
              <stop offset="0%" stopColor="#fbbf24" stopOpacity="0.14" />
              <stop offset="100%" stopColor="#fbbf24" stopOpacity="0" />
            </radialGradient>
          </defs>
          <rect x="0" y="0" width={W} height={H} fill={`url(#${glowId})`} />

          {/* level guides (subtle, under everything) */}
          {levels.map((_, li) => (
            <line
              key={li}
              x1={24}
              y1={bottomY - li * ROW_GAP}
              x2={W - 24}
              y2={bottomY - li * ROW_GAP}
              className="stroke-zinc-100"
              strokeWidth={1}
            />
          ))}

          {/* cover edges */}
          {segments.map((s, i) => (
            <line
              key={i}
              x1={s.x1}
              y1={s.y1}
              x2={s.x2}
              y2={s.y2}
              className="stroke-zinc-400"
              strokeWidth={1.6}
            />
          ))}

          {/* nodes */}
          {levels.flatMap((level, li) =>
            level.map((v, j) => {
              const p = pos.get(v);
              if (!p) return null;
              return (
                <g key={`${li}-${j}`}>
                  <circle
                    cx={p.x}
                    cy={p.y}
                    r={NODE_R}
                    className="fill-emerald-600 stroke-emerald-800"
                    strokeWidth={1.5}
                  />
                  <text
                    x={p.x}
                    y={p.y}
                    textAnchor="middle"
                    dominantBaseline="central"
                    className="fill-white font-mono text-[11px] font-bold"
                  >
                    {v}
                  </text>
                </g>
              );
            }),
          )}
        </svg>
      )}

      <p className="text-xs text-zinc-500">
        Cover relations only — transitive edges are omitted; the lowest level is drawn at the bottom.
      </p>
    </div>
  );
}
