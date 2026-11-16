"use client";

// Rich property-check cards for the relation_properties and graph_properties
// IR types (emitted by CHECK PROPERTIES). Rendered in the Output panel next
// to the regular value cards.

import { AlertTriangle, Check, Network, ShieldCheck, X } from "lucide-react";
import type { GraphPropertiesIr, RelationPropertiesIr } from "@/lib/dmath/types";

function PropBadge({ label, ok }: { label: string; ok: boolean }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-xs font-semibold ${
        ok
          ? "border-emerald-300 bg-emerald-50 text-emerald-800"
          : "border-rose-300 bg-rose-50 text-rose-700"
      }`}
    >
      {ok ? <Check className="h-3.5 w-3.5" /> : <X className="h-3.5 w-3.5" />}
      {label}
    </span>
  );
}

function Counterexample({ text }: { text: string }) {
  return (
    <p className="flex items-start gap-1.5 rounded-md border border-amber-200 bg-amber-50/70 px-2 py-1 text-xs text-amber-800">
      <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0 text-amber-500" />
      <span>{text}</span>
    </p>
  );
}

const REL_CHECKS: { key: "reflexive" | "symmetric" | "antisymmetric" | "transitive"; label: string }[] = [
  { key: "reflexive", label: "Reflexive" },
  { key: "symmetric", label: "Symmetric" },
  { key: "antisymmetric", label: "Antisymmetric" },
  { key: "transitive", label: "Transitive" },
];

function RelationProps({ ir }: { ir: RelationPropertiesIr }) {
  const counterexamples = REL_CHECKS.flatMap((c) => {
    const text = ir.counterexamples[c.key];
    return text ? [{ key: c.key, text }] : [];
  });

  return (
    <div className="space-y-2.5">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <ShieldCheck className="h-3.5 w-3.5 text-teal-400" />
          {ir.expression}
        </span>
        {ir.is_equivalence && (
          <span className="rounded-md border border-emerald-300 bg-emerald-100 px-2 py-1 text-xs font-bold tracking-wide text-emerald-800">
            EQUIVALENCE RELATION
          </span>
        )}
        {ir.is_partial_order && (
          <span className="rounded-md border border-violet-300 bg-violet-100 px-2 py-1 text-xs font-bold tracking-wide text-violet-800">
            PARTIAL ORDER
          </span>
        )}
      </div>

      <div className="flex flex-wrap gap-1.5">
        {REL_CHECKS.map((c) => (
          <PropBadge key={c.key} label={c.label} ok={ir[c.key]} />
        ))}
      </div>

      {counterexamples.length > 0 && (
        <div className="space-y-1">
          {counterexamples.map((c) => (
            <Counterexample key={c.key} text={c.text} />
          ))}
        </div>
      )}

      {ir.classes && ir.classes.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5">
          <span className="text-xs font-medium text-zinc-500">Equivalence classes:</span>
          {ir.classes.map((cls, i) => (
            <span
              key={i}
              className="rounded-md border border-violet-300 bg-violet-50 px-2 py-0.5 font-mono text-xs font-semibold text-violet-800"
            >
              {`{${cls.join(", ")}}`}
            </span>
          ))}
        </div>
      )}

      <p className="font-mono text-xs text-zinc-400">
        base set = {`{${ir.base_set.join(", ")}}`}
      </p>
    </div>
  );
}

function GraphProps({ ir }: { ir: GraphPropertiesIr }) {
  const vertices = Object.keys(ir.degrees)
    .map(Number)
    .sort((a, b) => a - b);

  return (
    <div className="space-y-2.5">
      <div className="flex flex-wrap items-center gap-2">
        <span className="inline-flex items-center gap-1.5 rounded-md bg-zinc-800 px-2 py-1 font-mono text-xs text-zinc-100">
          <Network className="h-3.5 w-3.5 text-rose-400" />
          {ir.expression}
        </span>
        <span className="font-mono text-xs text-zinc-500">
          |V| = {ir.vertex_count} · |E| = {ir.edge_count}
        </span>
      </div>

      <div className="flex flex-wrap gap-1.5">
        <PropBadge label="Connected" ok={ir.connected} />
        <PropBadge label="Tree" ok={ir.tree} />
        <PropBadge label="Bipartite" ok={ir.bipartite} />
        <PropBadge label="Complete" ok={ir.complete} />
      </div>

      {vertices.length > 0 && (
        <div className="overflow-hidden rounded-lg border border-zinc-200">
          <table className="w-full max-w-56 border-collapse text-xs">
            <thead>
              <tr className="bg-zinc-100">
                <th className="border-b border-zinc-300 px-3 py-1.5 text-left font-semibold text-zinc-600">vertex</th>
                <th className="border-b border-zinc-300 px-3 py-1.5 text-right font-semibold text-zinc-600">degree</th>
              </tr>
            </thead>
            <tbody className="font-mono">
              {vertices.map((v) => (
                <tr key={v} className="bg-white">
                  <td className="border-b border-zinc-100 px-3 py-1 text-zinc-700">{v}</td>
                  <td className="border-b border-zinc-100 px-3 py-1 text-right font-semibold text-zinc-800">
                    {ir.degrees[String(v)]}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {ir.notes.length > 0 && (
        <ul className="space-y-1 text-xs text-zinc-600">
          {ir.notes.map((n, i) => (
            <li key={i} className="flex items-start gap-1.5">
              <span className="mt-1.5 h-1 w-1 shrink-0 rounded-full bg-zinc-400" />
              {n}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export function PropertiesView({ ir }: { ir: RelationPropertiesIr | GraphPropertiesIr }) {
  return (
    <div className="rounded-lg border border-zinc-200 bg-white p-3 transition-colors hover:border-zinc-300">
      {ir.type === "relation_properties" ? <RelationProps ir={ir} /> : <GraphProps ir={ir} />}
    </div>
  );
}
