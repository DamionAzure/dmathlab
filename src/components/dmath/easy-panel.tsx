"use client";

// Easy Mode: pick an area and a task, fill in a few boxes, press Run. The panel writes
// the Kaylei program for you and shows it, so you can learn the code or edit it.

import { useMemo, useState } from "react";
import { Code2, Loader2, Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { AREAS, FIELDS, GRAPH_DEFAULTS, OPTS, OPT_DEFAULTS, buildAction, type Field, type Values } from "@/lib/dmath/easy";

const input =
  "w-full rounded-md border border-zinc-300 bg-white px-2.5 py-1.5 font-mono text-sm text-zinc-900 outline-none focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500/20";
const chip =
  "rounded-md border border-zinc-300 bg-white px-2 py-1 font-mono text-xs font-semibold text-zinc-700 hover:border-emerald-400 hover:bg-emerald-50";
const LOGIC_KEYS = ["P", "Q", "R", "AND", "OR", "NOT", "XOR", "IMPLIES", "IFF", "(", ")"];

function Label({ children, note }: { children: React.ReactNode; note?: string }) {
  return (
    <div className="mb-1 flex flex-wrap items-baseline gap-x-2">
      <span className="text-xs font-semibold text-zinc-700">{children}</span>
      {note && <span className="text-[11px] text-zinc-500">{note}</span>}
    </div>
  );
}

function Check({ checked, onChange, children }: { checked: boolean; onChange: (v: boolean) => void; children: React.ReactNode }) {
  return (
    <label className="flex cursor-pointer items-center gap-2 text-xs text-zinc-700">
      <input type="checkbox" checked={checked} onChange={(e) => onChange(e.target.checked)} className="h-4 w-4 accent-emerald-600" />
      {children}
    </label>
  );
}

export function EasyPanel({
  running, ready, onRun, onEditAsCode,
}: {
  running: boolean;
  ready: boolean;
  onRun: (code: string) => void;
  onEditAsCode: (code: string) => void;
}) {
  const [areaId, setAreaId] = useState(AREAS[0].id);
  const [actionId, setActionId] = useState<Record<string, string>>({});
  const [vals, setVals] = useState<Values>(() => {
    const v: Values = { ...GRAPH_DEFAULTS };
    for (const f of Object.values(FIELDS)) v[f.id] = f.def;
    return v;
  });
  const [opts, setOpts] = useState<Record<string, boolean>>({ ...OPT_DEFAULTS });
  const [lastFormula, setLastFormula] = useState("f1");

  const area = AREAS.find((a) => a.id === areaId)!;
  const action = area.actions.find((a) => a.id === actionId[areaId]) ?? area.actions[0];
  const set = (id: string, v: string) => setVals((s) => ({ ...s, [id]: v }));
  const built = useMemo(() => buildAction(action, vals, opts), [action, vals, opts]);

  const addKey = (k: string) => {
    const add = (s: string) => (s && k !== ")" ? s.trimEnd() + " " + k : s + k);
    set(lastFormula, add(vals[lastFormula] ?? ""));
  };

  const renderField = (f: Field) => {
    if (f.kind === "graph") {
      return (
        <div key="g" className="space-y-2.5 rounded-lg border border-zinc-200 bg-zinc-50/60 p-2.5">
          <div>
            <Label note="1..6 or 1, 2, 3">Vertices</Label>
            <input className={input} value={vals["g.v"]} onChange={(e) => set("g.v", e.target.value)} spellCheck={false} />
          </div>
          <div>
            <Label note={opts.w ? "(from, to, weight)" : "(from, to)"}>Edges</Label>
            <input className={input} value={vals["g.e"]} onChange={(e) => set("g.e", e.target.value)} spellCheck={false} />
          </div>
        </div>
      );
    }
    const multi = f.kind === "formula";
    return (
      <div key={f.id}>
        <Label note={f.note}>{f.label}</Label>
        <input
          className={input}
          value={vals[f.id]}
          onFocus={() => multi && setLastFormula(f.id)}
          onChange={(e) => set(f.id, e.target.value)}
          spellCheck={false}
          inputMode={f.kind === "int" ? "numeric" : undefined}
        />
      </div>
    );
  };

  const fields = action.fields.map((id) => FIELDS[id]);
  const wide = fields.filter((f) => f.kind === "formula" || f.kind === "pairs" || f.kind === "graph" || f.kind === "expr");
  const narrow = fields.filter((f) => !wide.includes(f));
  const optIds = (action.opts ?? []).filter((o) => OPTS[o]);

  return (
    <div className="space-y-3">
      <div className="grid grid-cols-2 gap-1.5 sm:grid-cols-4">
        {AREAS.map((a) => (
          <button
            key={a.id}
            type="button"
            onClick={() => setAreaId(a.id)}
            className={`rounded-lg border px-2 py-2 text-left transition-colors ${
              areaId === a.id ? "border-emerald-500 bg-emerald-50 text-emerald-900" : "border-zinc-200 bg-white text-zinc-700 hover:border-zinc-300"
            }`}
          >
            <div className="text-xs font-bold">{a.title}</div>
            <div className="text-[11px] leading-tight text-zinc-500">{a.hint}</div>
          </button>
        ))}
      </div>

      <div>
        <Label>What do you want to do?</Label>
        <select
          className={input + " font-sans"}
          value={action.id}
          onChange={(e) => setActionId((s) => ({ ...s, [areaId]: e.target.value }))}
        >
          {area.actions.map((a) => <option key={a.id} value={a.id}>{a.label}</option>)}
        </select>
      </div>

      {wide.map(renderField)}
      {narrow.length > 0 && (
        <div className={`grid gap-2.5 ${narrow.length === 1 ? "grid-cols-1" : "grid-cols-2"}`}>{narrow.map(renderField)}</div>
      )}

      {action.fields.some((id) => FIELDS[id].kind === "formula") && (
        <div className="flex flex-wrap gap-1.5">
          {LOGIC_KEYS.map((k) => (
            <button key={k} type="button" className={chip} onClick={() => addKey(k)}>{k}</button>
          ))}
        </div>
      )}

      {action.fields.includes("g") && (
        <div className="flex flex-wrap gap-x-5 gap-y-1.5">
          {!action.directed && <Check checked={!!opts.dir} onChange={(v) => setOpts((s) => ({ ...s, dir: v }))}>{OPTS.dir.label}</Check>}
          <Check checked={!!opts.w} onChange={(v) => setOpts((s) => ({ ...s, w: v }))}>{OPTS.w.label}</Check>
        </div>
      )}

      {optIds.filter((o) => o !== "dir" && o !== "w").length > 0 && (
        <div className="flex flex-wrap gap-x-5 gap-y-1.5">
          {optIds.filter((o) => o !== "dir" && o !== "w").map((o) => (
            <Check key={o} checked={!!opts[o]} onChange={(v) => setOpts((s) => ({ ...s, [o]: v }))}>{OPTS[o].label}</Check>
          ))}
        </div>
      )}

      {built.error ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-2.5 py-1.5 text-xs text-amber-800">{built.error}</p>
      ) : (
        <details className="rounded-md border border-zinc-200 bg-zinc-50 px-2.5 py-1.5">
          <summary className="cursor-pointer text-xs font-medium text-zinc-600">Show the program this makes</summary>
          <pre className="mt-1.5 overflow-x-auto font-mono text-xs text-zinc-800">{built.code}</pre>
        </details>
      )}

      <div className="flex flex-wrap items-center gap-2">
        <Button
          onClick={() => built.code && onRun(built.code)}
          disabled={!ready || running || !built.code}
          className="h-9 bg-gradient-to-b from-emerald-500 to-emerald-600 px-4 font-semibold text-white shadow-sm shadow-emerald-600/30 hover:from-emerald-500 hover:to-emerald-700"
        >
          {running || !ready ? <Loader2 className="mr-1.5 h-4 w-4 animate-spin" /> : <Play className="mr-1.5 h-4 w-4" />}
          Run
        </Button>
        <Button variant="outline" size="sm" className="h-9" disabled={!built.code} onClick={() => built.code && onEditAsCode(built.code)}>
          <Code2 className="mr-1.5 h-4 w-4" />
          Edit as code
        </Button>
      </div>
    </div>
  );
}
