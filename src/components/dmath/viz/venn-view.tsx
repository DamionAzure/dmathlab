"use client";

// Venn diagram renderer (SVG) for the venn Visualization IR.
// Regions are colored; set elements are placed inside their region bands.
// Handles three shapes: binary operations, COMPLEMENT, and single-set display.

import { Shapes } from "lucide-react";
import type { Elem, VennIr } from "@/lib/dmath/types";

const VIEW_W = 620;
const VIEW_H = 410;
const RECT = { x: 12, y: 34, w: 596, h: 356 };
const R = 112;
const AX = 250;
const BX = 372;
const CY = 210;

function elemKey(e: Elem): string {
  return Array.isArray(e) ? `(${e.map(elemKey).join(",")})` : String(e);
}

function elemText(e: Elem): string {
  return Array.isArray(e) ? `(${e.map(elemText).join(", ")})` : String(e);
}

/** Grid placement of n labels inside an x-band. */
function placeElems(elems: Elem[], x0: number, x1: number, y0: number, y1: number) {
  const shown = elems.slice(0, 12);
  const n = shown.length;
  const positions: { x: number; y: number; text: string }[] = [];
  if (n === 0) return positions;
  const cols = Math.min(n, Math.max(1, Math.ceil(Math.sqrt(n * ((x1 - x0) / (y1 - y0))))));
  const rows = Math.ceil(n / cols);
  const bw = (x1 - x0) / (cols + 1);
  const bh = (y1 - y0) / (rows + 1);
  shown.forEach((e, i) => {
    const col = i % cols;
    const row = Math.floor(i / cols);
    positions.push({
      x: x0 + bw * (col + 1),
      y: y0 + bh * (row + 1),
      text: elemText(e),
    });
  });
  return positions;
}

function ElementLabels({ items, extra }: { items: { x: number; y: number; text: string }[]; extra?: number }) {
  return (
    <>
      {items.map((p, i) => (
        <text
          key={i}
          x={p.x}
          y={p.y}
          textAnchor="middle"
          dominantBaseline="central"
          className="fill-zinc-700 font-mono text-[11px] font-medium"
          style={{ paintOrder: "stroke" }}
        >
          {p.text}
        </text>
      ))}
      {extra ? (
        <text
          x={items[items.length - 1]?.x ?? (12 + 596) / 2}
          y={(items[items.length - 1]?.y ?? 210) + 22}
          textAnchor="middle"
          className="fill-zinc-500 font-mono text-[10px] italic"
        >
          +{extra} more
        </text>
      ) : null}
    </>
  );
}

export function VennView({ ir }: { ir: VennIr }) {
  const isComplement = ir.operation === "complement";
  const isDisplay = ir.operation === "display";
  const twoSet = !isComplement && !isDisplay;

  // Region bands (binary layout)
  const bands = {
    left: [AX - R + 14, AX + 32] as const,
    mid: [AX + 44, BX - 44] as const,
    right: [BX - 32, BX + R - 14] as const,
  };

  const leftOnly = placeElems(ir.regions.leftOnly, bands.left[0], bands.left[1], CY - 74, CY + 74);
  const intersection = placeElems(ir.regions.intersection, bands.mid[0], bands.mid[1], CY - 64, CY + 64);
  const rightOnly = placeElems(ir.regions.rightOnly, bands.right[0], bands.right[1], CY - 74, CY + 74);

  // Outside elements: strip along the top and bottom of the universe rect,
  // clear of both circles and of the result banner at the bottom edge.
  const outside = (() => {
    const spots: { x: number; y: number }[] = [];
    const yTop = RECT.y + 18;
    const yBot = RECT.y + RECT.h - 36;
    for (let x = 60; x <= VIEW_W - 60; x += 72) {
      spots.push({ x, y: yTop }, { x, y: yBot });
    }
    return ir.regions.outside.slice(0, 10).map((e, i) => ({
      ...spots[i % spots.length],
      text: elemText(e),
    }));
  })();
  const outsideExtra = Math.max(0, ir.regions.outside.length - 10);

  const labelA = ir.labels[0] ?? "A";
  const labelB = ir.labels[1] ?? "B";

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Shapes className="h-3.5 w-3.5 text-teal-400" />
          {ir.labels.length > 1
            ? `${labelA} ${ir.operation} ${labelB}`
            : labelA}
        </span>
        <span className="rounded-md border border-teal-300 bg-teal-50 px-2 py-1 text-xs font-bold tracking-wide text-teal-800">
          {ir.operation_label.toUpperCase()}
        </span>
        {ir.universe && (
          <span className="font-mono text-xs text-zinc-500">
            U = {JSON.stringify(ir.universe).replace(/,/g, ", ")}
          </span>
        )}
      </div>

      <svg
        viewBox={`0 0 ${VIEW_W} ${VIEW_H}`}
        className="w-full max-w-xl rounded-lg border border-zinc-200 bg-white"
        role="img"
        aria-label={`Venn diagram of ${labelA} ${ir.operation} ${labelB}`}
      >
        {/* Universe */}
        <rect
          x={RECT.x}
          y={RECT.y}
          width={RECT.w}
          height={RECT.h}
          rx={14}
          className="fill-zinc-50 stroke-zinc-300"
          strokeWidth={1.5}
        />
        <text x={RECT.x + 12} y={RECT.y - 10} className="fill-zinc-500 font-mono text-[12px] font-semibold">
          UNIVERSE {ir.universe ? "U" : "(not declared)"}
        </text>

        {twoSet && (
          <>
            <defs>
              <clipPath id="clipA">
                <circle cx={AX} cy={CY} r={R} />
              </clipPath>
            </defs>
            {/* left-only region */}
            <circle cx={AX} cy={CY} r={R} className="fill-emerald-500/25 stroke-emerald-600" strokeWidth={2} />
            {/* right circle */}
            <circle cx={BX} cy={CY} r={R} className="fill-violet-500/25 stroke-violet-600" strokeWidth={2} />
            {/* intersection emphasis */}
            <circle cx={BX} cy={CY} r={R} clipPath="url(#clipA)" className="fill-teal-500/30" />
            <circle cx={AX} cy={CY} r={R} className="fill-none stroke-emerald-600" strokeWidth={2} />

            <text x={AX - 78} y={CY - R - 10} textAnchor="middle" className="fill-emerald-800 font-mono text-[13px] font-bold">
              {labelA}
            </text>
            <text x={BX + 78} y={CY - R - 10} textAnchor="middle" className="fill-violet-800 font-mono text-[13px] font-bold">
              {labelB}
            </text>

            <ElementLabels items={leftOnly} extra={Math.max(0, ir.regions.leftOnly.length - 12)} />
            <ElementLabels items={intersection} extra={Math.max(0, ir.regions.intersection.length - 12)} />
            <ElementLabels items={rightOnly} extra={Math.max(0, ir.regions.rightOnly.length - 12)} />
          </>
        )}

        {isComplement && (
          <>
            {/* shade everything outside A, inside the universe */}
            <path
              fillRule="evenodd"
              className="fill-teal-500/30"
              d={`M ${RECT.x} ${RECT.y} h ${RECT.w} v ${RECT.h} h ${-RECT.w} Z M ${AX} ${CY} m ${-R} 0 a ${R} ${R} 0 1 0 ${2 * R} 0 a ${R} ${R} 0 1 0 ${-2 * R} 0`}
            />
            <circle cx={AX} cy={CY} r={R} className="fill-emerald-500/25 stroke-emerald-600" strokeWidth={2} />
            <text x={AX} y={CY - R - 10} textAnchor="middle" className="fill-emerald-800 font-mono text-[13px] font-bold">
              {labelA}
            </text>
            <ElementLabels items={leftOnly} extra={Math.max(0, ir.regions.leftOnly.length - 12)} />
          </>
        )}

        {isDisplay && (
          <>
            <circle cx={AX} cy={CY} r={R} className="fill-emerald-500/25 stroke-emerald-600" strokeWidth={2} />
            <text x={AX} y={CY - R - 10} textAnchor="middle" className="fill-emerald-800 font-mono text-[13px] font-bold">
              {labelA}
            </text>
            <ElementLabels items={leftOnly} extra={Math.max(0, ir.regions.leftOnly.length - 12)} />
          </>
        )}

        {/* outside elements (binary + complement) */}
        {!isDisplay &&
          outside.map((p, i) => (
            <g key={i}>
              <circle cx={p.x} cy={p.y} r={11} className="fill-zinc-200/80 stroke-zinc-400" />
              <text
                x={p.x}
                y={p.y}
                textAnchor="middle"
                dominantBaseline="central"
                className="fill-zinc-600 font-mono text-[10px]"
              >
                {p.text}
              </text>
            </g>
          ))}
        {outsideExtra > 0 && (
          <text x={VIEW_W - 40} y={RECT.y + 16} textAnchor="end" className="fill-zinc-500 font-mono text-[10px] italic">
            +{outsideExtra} more outside
          </text>
        )}

        {/* result banner inside the SVG */}
        <text x={RECT.x + 12} y={RECT.y + RECT.h - 12} className="fill-zinc-700 font-mono text-[12px] font-semibold">
          {ir.result_display}
        </text>
      </svg>

      {/* Region legend */}
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-zinc-600">
        {twoSet && (
          <>
            <LegendDot cls="bg-emerald-500/40" label={`${labelA} only`} n={ir.regions.leftOnly.length} />
            <LegendDot cls="bg-teal-500/40" label="intersection" n={ir.regions.intersection.length} />
            <LegendDot cls="bg-violet-500/40" label={`${labelB} only`} n={ir.regions.rightOnly.length} />
            <LegendDot cls="bg-zinc-300" label="outside both" n={ir.regions.outside.length} />
          </>
        )}
        {isComplement && (
          <>
            <LegendDot cls="bg-emerald-500/40" label={labelA} n={ir.regions.leftOnly.length} />
            <LegendDot cls="bg-teal-500/40" label={`complement (outside ${labelA})`} n={ir.regions.outside.length} />
          </>
        )}
        {isDisplay && <LegendDot cls="bg-emerald-500/40" label={labelA} n={ir.regions.leftOnly.length} />}
      </div>

      {ir.note && <p className="text-xs italic text-amber-700">{ir.note}</p>}
    </div>
  );
}

function LegendDot({ cls, label, n }: { cls: string; label: string; n: number }) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className={`inline-block h-2.5 w-2.5 rounded-sm ${cls}`} />
      {label} <span className="font-mono text-zinc-400">({n})</span>
    </span>
  );
}
