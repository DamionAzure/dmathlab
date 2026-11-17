// Export visualizations as PNG / SVG pictures and tabular results as
// Markdown. Pictures are rendered from the live SVG with its computed styles
// baked in, so the file looks like the screen (without relying on our CSS).

import { invoke } from "@tauri-apps/api/core";
import type { RelationMatrixIr, StepsIr, TruthTableIr } from "./types";

type ExportExt = "png" | "svg" | "md";

function toBase64(bytes: Uint8Array): string {
  let s = "";
  const CH = 0x8000;
  for (let i = 0; i < bytes.length; i += CH) s += String.fromCharCode(...bytes.subarray(i, i + CH));
  return btoa(s);
}

/** Ask where to save, then write the file. Resolves to the saved path or null (cancelled). */
export async function saveExport(suggestedName: string, ext: ExportExt, data: string | Uint8Array): Promise<string | null> {
  const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data;
  return invoke<string | null>("export_file", {
    suggestedName,
    extension: ext,
    dataBase64: toBase64(bytes),
  });
}

export function safeName(text: string, fallback = "dmath"): string {
  const s = text.replace(/[^\w.-]+/g, "_").replace(/^_+|_+$/g, "").slice(0, 48);
  return s || fallback;
}

// ───────────────────────── SVG / PNG ─────────────────────────

const STYLE_PROPS = [
  "fill", "fill-opacity", "stroke", "stroke-width", "stroke-opacity", "stroke-dasharray", "stroke-linecap",
  "stroke-linejoin", "opacity", "font-family", "font-size", "font-weight", "font-style", "letter-spacing",
  "text-anchor", "dominant-baseline", "paint-order", "visibility", "display",
] as const;

/** Clone `svg`, copying each element's computed style inline, and add a background. */
export function standaloneSvg(svg: SVGSVGElement, background: string): { text: string; width: number; height: number } {
  const box = svg.getBoundingClientRect();
  const width = Math.max(1, Math.round(box.width));
  const height = Math.max(1, Math.round(box.height));
  const clone = svg.cloneNode(true) as SVGSVGElement;
  const src = [svg, ...Array.from(svg.querySelectorAll("*"))];
  const dst = [clone, ...Array.from(clone.querySelectorAll("*"))];
  src.forEach((el, i) => {
    const cs = getComputedStyle(el);
    const target = dst[i] as SVGElement;
    for (const p of STYLE_PROPS) {
      const v = cs.getPropertyValue(p);
      if (v) target.style.setProperty(p, v);
    }
    target.removeAttribute("class");
  });
  clone.setAttribute("xmlns", "http://www.w3.org/2000/svg");
  clone.setAttribute("width", String(width));
  clone.setAttribute("height", String(height));
  if (!clone.getAttribute("viewBox")) clone.setAttribute("viewBox", `0 0 ${width} ${height}`);
  const bg = document.createElementNS("http://www.w3.org/2000/svg", "rect");
  const vb = clone.viewBox.baseVal;
  bg.setAttribute("x", String(vb?.x ?? 0));
  bg.setAttribute("y", String(vb?.y ?? 0));
  bg.setAttribute("width", String(vb?.width || width));
  bg.setAttribute("height", String(vb?.height || height));
  bg.setAttribute("fill", background);
  clone.insertBefore(bg, clone.firstChild);
  return { text: `<?xml version="1.0" encoding="UTF-8"?>\n${new XMLSerializer().serializeToString(clone)}`, width, height };
}

/** Rasterize a standalone SVG to PNG bytes at `scale`× resolution. */
export async function svgToPng(svgText: string, width: number, height: number, scale = 2): Promise<Uint8Array> {
  const url = URL.createObjectURL(new Blob([svgText], { type: "image/svg+xml;charset=utf-8" }));
  try {
    const img = new Image();
    img.decoding = "sync";
    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve();
      img.onerror = () => reject(new Error("Could not render the picture."));
      img.src = url;
    });
    const canvas = document.createElement("canvas");
    canvas.width = Math.round(width * scale);
    canvas.height = Math.round(height * scale);
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("No canvas available.");
    ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    const blob = await new Promise<Blob | null>((r) => canvas.toBlob(r, "image/png"));
    if (!blob) throw new Error("Could not encode the PNG.");
    return new Uint8Array(await blob.arrayBuffer());
  } finally {
    URL.revokeObjectURL(url);
  }
}

export function pageBackground(): string {
  return document.documentElement.classList.contains("dark") ? "#18181b" : "#ffffff";
}

// ───────────────────────── Markdown ─────────────────────────

function cell(s: string): string {
  return s.replace(/\|/g, "\\|").replace(/\n/g, " ");
}

function mdTable(headers: string[], rows: string[][]): string {
  const head = `| ${headers.map(cell).join(" | ")} |`;
  const rule = `| ${headers.map(() => "---").join(" | ")} |`;
  return [head, rule, ...rows.map((r) => `| ${r.map(cell).join(" | ")} |`)].join("\n");
}

export function stepsToMarkdown(ir: StepsIr): string {
  const out: string[] = [`# ${ir.expression}`, "", `**Result:** ${ir.result}`, ""];
  ir.sections.forEach((s, i) => {
    out.push(`## ${i + 1}. ${s.title}`, "");
    if (s.intro) out.push(s.intro, "");
    for (const st of s.steps) {
      if (st.tone === "head") {
        if (out[out.length - 1] !== "") out.push("");
        out.push(`**${st.text}**`, "");
      }
      else if (st.tone === "good") out.push(`- ✅ ${st.text}`);
      else if (st.tone === "bad") out.push(`- ❌ ${st.text}`);
      else out.push(`- ${st.text}`);
    }
    if (s.steps.length) out.push("");
    if (s.table) out.push(mdTable(s.table.headers, s.table.rows), "");
    if (s.outcome) out.push(`> **${s.outcome}**${s.agrees === true ? " (matches the engine)" : s.agrees === false ? " (DISAGREES with the engine)" : ""}`, "");
  });
  const methods = ir.sections.filter((s) => s.agrees !== undefined).length;
  out.push("---", "", ir.verified ? (methods > 1 ? `All ${methods} methods agree: ${ir.expression} = ${ir.result}` : `Answer: ${ir.expression} = ${ir.result}`) : "The methods disagree — please report this.");
  return out.join("\n") + "\n";
}

export function truthTableToMarkdown(ir: TruthTableIr): string {
  const rows = ir.rows.map((r) => [...ir.variables.map((v) => (r[v] ? "T" : "F")), r["result"] ? "T" : "F"]);
  return [`# Truth table: ${ir.expression}`, "", mdTable([...ir.variables, "result"], rows), "", `**${ir.classification}** — ${ir.classification_note}`, ""].join("\n");
}

export function matrixToMarkdown(ir: RelationMatrixIr): string {
  const rows = ir.matrix.map((r, i) => [String(ir.vertices[i]), ...r.map(String)]);
  return [`# Relation matrix: ${ir.expression}`, "", mdTable(["", ...ir.vertices.map(String)], rows), ""].join("\n");
}
