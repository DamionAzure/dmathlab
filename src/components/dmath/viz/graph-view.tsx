"use client";

// Interactive-ish graph renderer for the graph_view Visualization IR.
// Circular layout; undirected edges are plain lines, directed ones get
// arrowheads. Supports weight pills, path/tree edge highlights (with a glow
// underlay + order badges), visited-order rings, greedy-coloring fills with a
// legend, a total-weight badge and an engine note caption.

import { useId } from "react";
import { Waypoints } from "lucide-react";
import type { GraphViewIr } from "@/lib/dmath/types";
import { BEND, edgeGeometry, loopGeometry, tipExtra } from "@/lib/dmath/arrows";
import { ArrowMarker } from "@/components/dmath/viz/arrow-marker";

interface Pt {
  x: number;
  y: number;
}

const VIEW_W = 640;
const VIEW_H = 480;
const CX = VIEW_W / 2;
const CY = VIEW_H / 2 + 6;
const NODE_R = 18;

// Fixed qualitative palette (no blues/indigos): emerald, amber, rose, violet, cyan, lime.
const PALETTE = [
  { fill: "#059669", stroke: "#065f46" }, // emerald-600
  { fill: "#d97706", stroke: "#92400e" }, // amber-600
  { fill: "#e11d48", stroke: "#9f1239" }, // rose-600
  { fill: "#7c3aed", stroke: "#5b21b6" }, // violet-600
  { fill: "#0e7490", stroke: "#155e75" }, // cyan-700
  { fill: "#4d7c0f", stroke: "#3f6212" }, // lime-700
];

const EDGE_COLOR = "#a1a1aa"; // zinc-400
const PATH_COLOR = "#f59e0b"; // amber-500
const TREE_COLOR = "#10b981"; // emerald-500
const VISIT_COLOR = "#8b5cf6"; // violet-500
const ARROW_SIZE = 11;
const ARROW_SIZE_HI = 14; // highlighted (path/tree) edges are thicker, so their heads are too

function norm(x: number, y: number): Pt {
  const len = Math.hypot(x, y);
  return len < 1e-9 ? { x: 0, y: -1 } : { x: x / len, y: y / len };
}

function pairKey(a: number, b: number): string {
  return a < b ? `${a}-${b}` : `${b}-${a}`;
}

function edgeKey(a: number, b: number): string {
  return `${a}->${b}`;
}

function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return `${n}${s[(v - 20) % 10] ?? s[v] ?? s[0]}`;
}

/** Circular layout: first vertex at the top, clockwise; a lone vertex is centered. */
function circleLayout(vertices: number[]): Map<number, Pt> {
  const pos = new Map<number, Pt>();
  const n = vertices.length;
  const radius = n === 2 ? 115 : Math.min(185, 70 + 19 * n);
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

export function GraphView({ ir }: { ir: GraphViewIr }) {
  const uid = useId().replace(/[^a-zA-Z0-9]/g, "");
  const mArrow = `gv-arrow-${uid}`;
  const mArrowPath = `gv-arrow-path-${uid}`;
  const mArrowTree = `gv-arrow-tree-${uid}`;

  const pos = circleLayout(ir.vertices);

  // Resolve per-edge highlight classes ("path" beats "tree").
  const pathPairs = new Set<string>();
  if (ir.highlight_path && ir.highlight_path.length > 1) {
    for (let i = 0; i + 1 < ir.highlight_path.length; i++) {
      const a = ir.highlight_path[i];
      const b = ir.highlight_path[i + 1];
      pathPairs.add(ir.directed ? edgeKey(a, b) : pairKey(a, b));
    }
  }

  interface DrawnEdge {
    key: string;
    d: string;
    cls: "path" | "tree" | null;
    weight?: number;
    mid: Pt;
  }

  const loopVertices = new Set(ir.edges.filter((e) => e.from === e.to).map((e) => e.from));
  const directedSet = new Set(ir.edges.map((e) => edgeKey(e.from, e.to)));
  const drawn: DrawnEdge[] = [];
  const seen = new Set<string>();
  for (const e of ir.edges) {
    const p1 = pos.get(e.from);
    const p2 = pos.get(e.to);
    if (!p1 || !p2) continue;
    const key = ir.directed ? edgeKey(e.from, e.to) : pairKey(e.from, e.to);
    const dedupeKey = `${key}|${e.weight ?? ""}`;
    if (seen.has(dedupeKey)) continue;
    seen.add(dedupeKey);

    const cls0: "path" | "tree" | null =
      e.highlight === "path" || pathPairs.has(ir.directed ? edgeKey(e.from, e.to) : pairKey(e.from, e.to))
        ? "path"
        : e.highlight === "tree"
          ? "tree"
          : null;
    const headSize = cls0 ? ARROW_SIZE_HI : ARROW_SIZE;
    const extra = ir.directed ? tipExtra(headSize) : 0;
    let geom: { d: string; mid: Pt } | null;
    if (e.from === e.to) {
      // Self-loop (the undirected view draws it too, just without a head).
      geom = loopGeometry(p1, norm(p1.x - CX, p1.y - CY), NODE_R, 12, 2, extra);
    } else {
      // Reciprocal directed edges bow to their own right so both stay visible.
      const recip = ir.directed && directedSet.has(edgeKey(e.to, e.from));
      geom = edgeGeometry(p1, p2, NODE_R, NODE_R, { bend: recip ? BEND : 0, gap: 2, endExtra: extra });
    }
    if (!geom) continue;
    const edgeMatchKey = ir.directed ? edgeKey(e.from, e.to) : pairKey(e.from, e.to);
    const cls: "path" | "tree" | null =
      e.highlight === "path" || pathPairs.has(edgeMatchKey)
        ? "path"
        : e.highlight === "tree"
          ? "tree"
          : null;
    drawn.push({ key: dedupeKey, d: geom.d, cls, weight: e.weight, mid: geom.mid });
  }

  const hasWeights = ir.edges.some((e) => e.weight !== undefined);
  const hasPath = drawn.some((d) => d.cls === "path");
  const hasTree = drawn.some((d) => d.cls === "tree");
  const visited = ir.highlight_visited ?? [];

  // Vertex color fills (1-based color indices; cycle through the palette).
  const colorOf = (v: number): (typeof PALETTE)[number] | null => {
    if (!ir.colors) return null;
    const c = ir.colors[String(v)];
    if (c === undefined) return null;
    return PALETTE[((c - 1) % PALETTE.length + PALETTE.length) % PALETTE.length];
  };
  const colorCount = ir.colors
    ? Object.values(ir.colors).reduce((m, c) => Math.max(m, c), 0)
    : 0;

  // Outward radial direction per vertex (for badge placement).
  const outward = (p: Pt): Pt => norm(p.x - CX, p.y - CY);

  // Path order badges: first occurrence of each vertex wins.
  const pathBadges: { v: number; label: string; x: number; y: number; w: number }[] = [];
  if (ir.highlight_path) {
    const used = new Set<number>();
    ir.highlight_path.forEach((v, i) => {
      if (used.has(v)) return;
      used.add(v);
      const p = pos.get(v);
      if (!p) return;
      let d = outward(p);
      // A self-loop occupies the outward side: slide the badge along the node instead.
      if (loopVertices.has(v)) d = { x: -d.y, y: d.x };
      const label = ordinal(i + 1);
      const w = 14 + label.length * 5.8;
      pathBadges.push({ v, label, x: p.x + d.x * (NODE_R + 13), y: p.y + d.y * (NODE_R + 13), w });
    });
  }

  // Visited-order badges (small index, placed inward so they never clash with path badges).
  const visitBadges: { v: number; index: number; x: number; y: number }[] = [];
  visited.forEach((v, i) => {
    if (visitBadges.some((b) => b.v === v)) return;
    const p = pos.get(v);
    if (!p) return;
    const d = outward(p);
    visitBadges.push({ v, index: i + 1, x: p.x - d.x * (NODE_R + 12), y: p.y - d.y * (NODE_R + 12) });
  });

  const markerFor = (cls: DrawnEdge["cls"]): string | undefined => {
    if (!ir.directed) return undefined;
    if (cls === "path") return `url(#${mArrowPath})`;
    if (cls === "tree") return `url(#${mArrowTree})`;
    return `url(#${mArrow})`;
  };

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Waypoints className="h-3.5 w-3.5 text-rose-400" />
          {ir.expression}
        </span>
        <span className="rounded-md border border-rose-300 bg-rose-50 px-2 py-1 text-xs font-bold tracking-wide text-rose-800">
          {ir.directed ? "DIRECTED GRAPH" : "GRAPH"}
        </span>
        <span className="font-mono text-xs text-zinc-500">
          |V| = {ir.vertices.length} · |E| = {ir.edges.length}
        </span>
        {ir.total_weight !== undefined && (
          <span className="rounded-md border border-amber-300 bg-amber-50 px-2 py-1 font-mono text-xs font-bold text-amber-800">
            total weight {ir.total_weight}
          </span>
        )}
      </div>

      {ir.vertices.length === 0 ? (
        <p className="rounded-lg border border-dashed border-zinc-300 py-8 text-center text-sm text-zinc-400">
          Empty graph — no vertices to draw.
        </p>
      ) : (
        <svg
          viewBox={`0 0 ${VIEW_W} ${VIEW_H}`}
          className="w-full max-w-xl rounded-lg border border-zinc-200 bg-white"
          role="img"
          aria-label={`Graph view of ${ir.expression}`}
        >
          <defs>
            <ArrowMarker id={mArrow} color={ir.directed ? "#71717a" : EDGE_COLOR} size={ARROW_SIZE} />
            <ArrowMarker id={mArrowPath} color={PATH_COLOR} size={ARROW_SIZE_HI} />
            <ArrowMarker id={mArrowTree} color={TREE_COLOR} size={ARROW_SIZE_HI} />
          </defs>

          {/* glow underlay for highlighted paths */}
          {hasPath &&
            drawn
              .filter((d) => d.cls === "path")
              .map((d) => (
                <path
                  key={`glow-${d.key}`}
                  d={d.d}
                  fill="none"
                  stroke={PATH_COLOR}
                  strokeOpacity={0.22}
                  strokeWidth={11}
                  strokeLinecap="round"
                />
              ))}

          {/* edges */}
          {drawn.map((d) => (
            <path
              key={d.key}
              d={d.d}
              fill="none"
              strokeLinecap="round"
              stroke={d.cls === "path" ? PATH_COLOR : d.cls === "tree" ? TREE_COLOR : ir.directed ? "#71717a" : EDGE_COLOR}
              strokeWidth={d.cls ? 3.25 : 1.75}
              markerEnd={markerFor(d.cls)}
            />
          ))}

          {/* weight pills */}
          {hasWeights &&
            drawn
              .filter((d) => d.weight !== undefined)
              .map((d) => {
                const text = String(d.weight);
                const w = 12 + text.length * 6.4;
                return (
                  <g key={`w-${d.key}`}>
                    <rect
                      x={d.mid.x - w / 2}
                      y={d.mid.y - 8}
                      width={w}
                      height={16}
                      rx={8}
                      className="fill-white stroke-zinc-300"
                      strokeWidth={1}
                    />
                    <text
                      x={d.mid.x}
                      y={d.mid.y}
                      textAnchor="middle"
                      dominantBaseline="central"
                      className="fill-zinc-700 font-mono text-[9.5px] font-semibold"
                    >
                      {text}
                    </text>
                  </g>
                );
              })}

          {/* visited-order rings */}
          {visitBadges.map((b) => {
            const p = pos.get(b.v);
            if (!p) return null;
            return (
              <circle
                key={`ring-${b.v}`}
                cx={p.x}
                cy={p.y}
                r={NODE_R + 4}
                fill="none"
                stroke={VISIT_COLOR}
                strokeWidth={2}
              />
            );
          })}

          {/* vertices */}
          {ir.vertices.map((v) => {
            const p = pos.get(v);
            if (!p) return null;
            const c = colorOf(v);
            return (
              <g key={v}>
                <circle
                  cx={p.x}
                  cy={p.y}
                  r={NODE_R}
                  fill={c ? c.fill : "#059669"}
                  stroke={c ? c.stroke : "#065f46"}
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

          {/* path-order badges */}
          {pathBadges.map((b) => (
            <g key={`path-${b.v}`}>
              <rect
                x={b.x - b.w / 2}
                y={b.y - 7.5}
                width={b.w}
                height={15}
                rx={7.5}
                fill="#fef3c7"
                stroke={PATH_COLOR}
                strokeWidth={1}
              />
              <text
                x={b.x}
                y={b.y}
                textAnchor="middle"
                dominantBaseline="central"
                className="fill-amber-800 font-mono text-[9px] font-bold"
              >
                {b.label}
              </text>
            </g>
          ))}

          {/* visited-order badges */}
          {visitBadges.map((b) => (
            <g key={`visit-${b.v}`}>
              <circle cx={b.x} cy={b.y} r={7.5} fill="#ede9fe" stroke={VISIT_COLOR} strokeWidth={1} />
              <text
                x={b.x}
                y={b.y}
                textAnchor="middle"
                dominantBaseline="central"
                className="fill-violet-700 font-mono text-[9px] font-bold"
              >
                {b.index}
              </text>
            </g>
          ))}
        </svg>
      )}

      {/* legend */}
      {(colorCount > 0 || hasPath || hasTree || visited.length > 0) && (
        <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-zinc-600">
          {Array.from({ length: colorCount }, (_, i) => {
            const c = PALETTE[i % PALETTE.length];
            return (
              <span key={i} className="inline-flex items-center gap-1.5">
                <span className="inline-block h-2.5 w-2.5 rounded-sm" style={{ backgroundColor: c.fill }} />
                color {i + 1}
              </span>
            );
          })}
          {hasPath && (
            <span className="inline-flex items-center gap-1.5">
              <span className="inline-block h-0.5 w-4 rounded" style={{ backgroundColor: PATH_COLOR }} />
              shortest path
            </span>
          )}
          {hasTree && (
            <span className="inline-flex items-center gap-1.5">
              <span className="inline-block h-0.5 w-4 rounded" style={{ backgroundColor: TREE_COLOR }} />
              tree edges (MST)
            </span>
          )}
          {visited.length > 0 && (
            <span className="inline-flex items-center gap-1.5">
              <span className="inline-block h-2.5 w-2.5 rounded-full border-2" style={{ borderColor: VISIT_COLOR }} />
              visit order
            </span>
          )}
        </div>
      )}

      {ir.note && <p className="text-xs italic text-amber-700">{ir.note}</p>}
    </div>
  );
}
