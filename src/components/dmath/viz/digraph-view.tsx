"use client";

// Directed-graph (digraph) renderer for the digraph Visualization IR.
// Vertices sit on a circle; pairs are slightly curved directed arcs with
// arrowheads; self-loops are small tangent circles pointing outward.

import { useId } from "react";
import { Network } from "lucide-react";
import type { DigraphIr } from "@/lib/dmath/types";
import { BEND, edgeGeometry, loopGeometry, tipExtra, type Pt } from "@/lib/dmath/arrows";
import { ArrowMarker } from "@/components/dmath/viz/arrow-marker";

const VIEW_W = 620;
const VIEW_H = 470;
const CX = VIEW_W / 2;
const CY = VIEW_H / 2 + 8;
const NODE_R = 17;
const LOOP_R = 11;
const ARROW_SIZE = 11;
const EDGE_COLOR = "#71717a"; // zinc-500 (line and head share one colour)

function norm(x: number, y: number): Pt {
  const len = Math.hypot(x, y);
  return len < 1e-9 ? { x: 0, y: -1 } : { x: x / len, y: y / len };
}

/** Circular layout: first vertex at the top, clockwise; a lone vertex is centered. */
function circleLayout(vertices: number[]): Map<number, Pt> {
  const pos = new Map<number, Pt>();
  const n = vertices.length;
  const radius = n === 2 ? 110 : Math.min(180, 68 + 18.5 * n);
  vertices.forEach((v, i) => {
    if (n === 1) {
      pos.set(v, { x: CX, y: CY });
      return;
    }
    const a = -Math.PI / 2 + (2 * Math.PI * i) / n;
    pos.set(v, { x: CX + radius * Math.cos(a), y: CY + radius * Math.sin(a) });
  });
  return pos;
}

export function DigraphView({ ir }: { ir: DigraphIr }) {
  const uid = useId().replace(/[^a-zA-Z0-9]/g, "");
  const arrowId = `dg-arrow-${uid}`;

  const pos = circleLayout(ir.vertices);
  const edgeSet = new Set(ir.edges.map(([a, b]) => `${a}>${b}`));
  const seen = new Set<string>();
  const paths: string[] = [];
  for (const [a, b] of ir.edges) {
    const key = `${a}>${b}`;
    if (seen.has(key)) continue;
    seen.add(key);
    const p1 = pos.get(a);
    const p2 = pos.get(b);
    if (!p1 || !p2) continue;
    if (a === b) {
      // Loop opens away from the layout centre.
      paths.push(loopGeometry(p1, norm(p1.x - CX, p1.y - CY), NODE_R, LOOP_R, 2, tipExtra(ARROW_SIZE)).d);
    } else {
      // Straight, unless the reverse edge exists too: then each bows to its own right.
      const bend = edgeSet.has(`${b}>${a}`) ? BEND : 0;
      const g = edgeGeometry(p1, p2, NODE_R, NODE_R, { bend, gap: 2, endExtra: tipExtra(ARROW_SIZE) });
      if (g) paths.push(g.d);
    }
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Network className="h-3.5 w-3.5 text-sky-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-sky-300 bg-sky-50 px-2 py-1 text-xs font-bold tracking-wide text-sky-800">
          DIGRAPH
        </span>
        <span className="font-mono text-xs text-zinc-500">
          |V| = {ir.vertices.length} · |E| = {ir.edges.length}
        </span>
      </div>

      {ir.vertices.length === 0 ? (
        <p className="rounded-lg border border-dashed border-zinc-300 py-8 text-center text-sm text-zinc-400">
          Empty relation — no vertices to draw.
        </p>
      ) : (
        <svg
          viewBox={`0 0 ${VIEW_W} ${VIEW_H}`}
          className="w-full max-w-xl rounded-lg border border-zinc-200 bg-white"
          role="img"
          aria-label={`Digraph of ${ir.expression}`}
        >
          <defs>
            <ArrowMarker id={arrowId} color={EDGE_COLOR} size={ARROW_SIZE} />
          </defs>

          {/* directed edges and self-loops */}
          {paths.map((d, i) => (
            <path
              key={i}
              d={d}
              fill="none"
              stroke={EDGE_COLOR}
              strokeWidth={1.6}
              strokeLinecap="round"
              markerEnd={`url(#${arrowId})`}
            />
          ))}

          {/* vertices */}
          {ir.vertices.map((v) => {
            const p = pos.get(v);
            if (!p) return null;
            return (
              <g key={v}>
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
                  className="fill-white font-mono text-[12px] font-bold"
                >
                  {v}
                </text>
              </g>
            );
          })}
        </svg>
      )}

      <p className="text-xs text-zinc-500">
        Arrowheads show pair order — an arrow u → v means (u, v) ∈ R; a loop on u means (u, u) ∈ R.
      </p>
    </div>
  );
}
