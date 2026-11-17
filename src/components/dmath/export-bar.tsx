"use client";

// Hover toolbar on a visualization card: save as PNG / SVG (picture views) or
// Markdown (tables and step-by-step solutions).

import { useRef, useState, type ReactNode } from "react";
import { Check, Download } from "lucide-react";

import type { VizIr } from "@/lib/dmath/types";
import {
  matrixToMarkdown,
  pageBackground,
  safeName,
  saveExport,
  standaloneSvg,
  stepsToMarkdown,
  svgToPng,
  truthTableToMarkdown,
} from "@/lib/dmath/export";

const PICTURE_TYPES = new Set<VizIr["type"]>(["venn", "digraph", "hasse", "graph_view"]);

function baseName(viz: VizIr): string {
  const e = "expression" in viz && typeof viz.expression === "string" ? viz.expression : viz.type;
  return safeName(`${viz.type}-${e}`);
}

export function ExportableCard({ viz, children }: { viz: VizIr; children: ReactNode }) {
  const contentRef = useRef<HTMLDivElement>(null);
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);

  const markdown =
    viz.type === "steps" ? stepsToMarkdown(viz)
    : viz.type === "truth_table" ? truthTableToMarkdown(viz)
    : viz.type === "relation_matrix" ? matrixToMarkdown(viz)
    : null;
  const picture = PICTURE_TYPES.has(viz.type);
  if (!markdown && !picture) return <div className="rounded-xl border border-zinc-200 bg-zinc-50/40 p-3">{children}</div>;

  const report = (ok: boolean, text: string) => {
    setMsg({ ok, text });
    window.setTimeout(() => setMsg(null), 4000);
  };

  const run = async (kind: "png" | "svg" | "md") => {
    try {
      let saved: string | null = null;
      if (kind === "md" && markdown) {
        saved = await saveExport(`${baseName(viz)}.md`, "md", markdown);
      } else {
        // The picture is the largest <svg> in the card (chips carry tiny icon SVGs).
        const svgs = Array.from(contentRef.current?.querySelectorAll("svg") ?? []).filter(
          (n): n is SVGSVGElement => n instanceof SVGSVGElement,
        );
        const area = (n: SVGSVGElement) => {
          const r = n.getBoundingClientRect();
          return r.width * r.height;
        };
        const svg = svgs.sort((a, b) => area(b) - area(a))[0];
        if (!svg || area(svg) < 40 * 40) throw new Error("Nothing to export yet.");
        const { text, width, height } = standaloneSvg(svg, pageBackground());
        saved = kind === "svg"
          ? await saveExport(`${baseName(viz)}.svg`, "svg", text)
          : await saveExport(`${baseName(viz)}.png`, "png", await svgToPng(text, width, height, 2));
      }
      if (saved) report(true, `Saved ${saved.split(/[\\/]/).pop()}`);
    } catch (e) {
      report(false, e instanceof Error ? e.message : String(e));
    }
  };

  const btn =
    "rounded-md px-2 py-0.5 text-[11px] font-medium text-zinc-600 hover:bg-zinc-200/70 focus-visible:bg-zinc-200/70";
  return (
    <div className="group/export relative rounded-xl border border-zinc-200 bg-zinc-50/40 p-3">
      <div
        className={`absolute right-2 top-2 z-10 flex items-center gap-0.5 rounded-lg border border-zinc-200 bg-white/90 px-1 py-0.5 shadow-sm backdrop-blur transition-opacity focus-within:opacity-100 group-hover/export:opacity-100 ${msg ? "opacity-100" : "opacity-0"}`}
      >
        {msg ? (
          <span className={`flex items-center gap-1 px-1.5 text-[11px] font-medium ${msg.ok ? "text-emerald-700" : "text-rose-700"}`}>
            {msg.ok && <Check className="h-3 w-3" />}
            {msg.text}
          </span>
        ) : (
          <>
            <Download className="ml-1 h-3 w-3 text-zinc-400" aria-hidden />
            {picture && (
              <>
                <button type="button" className={btn} onClick={() => void run("png")} title="Save as PNG picture">
                  PNG
                </button>
                <button type="button" className={btn} onClick={() => void run("svg")} title="Save as SVG (vector)">
                  SVG
                </button>
              </>
            )}
            {markdown && (
              <button type="button" className={btn} onClick={() => void run("md")} title="Save as Markdown">
                Markdown
              </button>
            )}
          </>
        )}
      </div>
      <div ref={contentRef}>{children}</div>
    </div>
  );
}
