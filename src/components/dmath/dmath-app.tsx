"use client";

import kayleiIcon from "@/assets/kaylei-icon.svg";

// Main Kaylei application shell.
// Left: DSL editor + controls + output panel. Right: visualizations + trace.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { LucideIcon } from "lucide-react";
import {
  BookOpen,
  Braces,
  Eraser,
  FlaskConical,
  Loader2,
  Monitor,
  Moon,
  Play,
  Radar,
  Sparkles,
  Sun,
  Trash2,
} from "lucide-react";

import { useTheme } from "@/lib/dmath/theme";
import { ReferenceDialog } from "@/components/dmath/reference-dialog";
import { ExportableCard } from "@/components/dmath/export-bar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { CodeEditor } from "@/components/dmath/code-editor";
import { OutputPanel } from "@/components/dmath/output-panel";
import { TracePanel } from "@/components/dmath/trace-panel";
import { TruthTableView } from "@/components/dmath/viz/truth-table-view";
import { VennView } from "@/components/dmath/viz/venn-view";
import { EquivalenceView } from "@/components/dmath/viz/equivalence-view";
import { DigraphView } from "@/components/dmath/viz/digraph-view";
import { HasseView } from "@/components/dmath/viz/hasse-view";
import { MatrixView } from "@/components/dmath/viz/matrix-view";
import { GraphView } from "@/components/dmath/viz/graph-view";
import { SieveView } from "@/components/dmath/viz/sieve-view";
import { PascalView } from "@/components/dmath/viz/pascal-view";
import { StepsView } from "@/components/dmath/viz/steps-view";
import { loadEngine, type DMathEngine } from "@/lib/dmath/engine";
import {
  confirmDiscard,
  fileName,
  listenDesktop,
  openFileDialog,
  readFile,
  saveFile,
  setDocState,
  startupFile,
  type MenuAction,
  type OpenedFile,
} from "@/lib/dmath/desktop";
import { DEFAULT_EXAMPLE, EXAMPLES, EXAMPLE_GROUPS } from "@/lib/dmath/examples";
import { isPropertyCheckIr, type DMathEnvelope, type VizIr } from "@/lib/dmath/types";

export function DMathApp() {
  const { choice: theme, cycle: cycleTheme } = useTheme();
  const [source, setSource] = useState(DEFAULT_EXAMPLE.code);
  const [refOpen, setRefOpen] = useState(false);
  const [exampleId, setExampleId] = useState<string>(DEFAULT_EXAMPLE.id);
  const [envelope, setEnvelope] = useState<DMathEnvelope | null>(null);
  const [engine, setEngine] = useState<DMathEngine | null>(null);
  const [engineError, setEngineError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const engineRef = useRef<DMathEngine | null>(null);

  // Document (file) state. `dirty` = edited since last open/save/example load.
  const [filePath, setFilePath] = useState<string | null>(null);
  const [savedSource, setSavedSource] = useState(DEFAULT_EXAMPLE.code);
  const [notice, setNotice] = useState<string | null>(null);
  const dirty = source !== savedSource;

  // Load the Rust core once.
  useEffect(() => {
    let alive = true;
    loadEngine()
      .then((eng) => {
        if (!alive) return;
        engineRef.current = eng;
        setEngine(eng);
      })
      .catch((e: unknown) => {
        if (!alive) return;
        setEngineError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      alive = false;
    };
  }, []);

  const run = useCallback(async () => {
    const eng = engineRef.current;
    if (!eng) return;
    setRunning(true);
    try {
      setEnvelope(await eng.run(source));
    } catch (e) {
      setEngineError(e instanceof Error ? e.message : String(e));
    } finally {
      setRunning(false);
    }
  }, [source]);

  // Auto-run the default example once the engine is ready.
  useEffect(() => {
    if (engine && envelope === null && !engineError) {
      run();
    }
  }, [engine, envelope, engineError, run]);

  const errorLines = useMemo(() => {
    if (!envelope) return [];
    return envelope.diagnostics
      .filter((d) => d.severity === "error")
      .map((d) => d.line);
  }, [envelope]);

  // Property-check IR renders in the Output panel; everything else is a view.
  const vizViews = useMemo(
    () => (envelope ? envelope.visualizations.filter((v) => !isPropertyCheckIr(v)) : []),
    [envelope],
  );

  // ── Document actions (File menu, drag-and-drop, buttons) ──────────────────
  const latest = useRef({ source, filePath, dirty });
  latest.current = { source, filePath, dirty };

  /** True if it is fine to replace the current text (saved, or user agreed). */
  const canReplace = async () => !latest.current.dirty || (await confirmDiscard());

  const adopt = (f: OpenedFile) => {
    setSource(f.contents);
    setSavedSource(f.contents);
    setFilePath(f.path);
    setExampleId("");
    setEnvelope(null);
    setNotice(null);
  };

  const fail = (e: unknown) => setNotice(e instanceof Error ? e.message : String(e));

  const newDocument = async () => {
    if (!(await canReplace())) return;
    setSource("");
    setSavedSource("");
    setFilePath(null);
    setExampleId("");
    setEnvelope(null);
    setNotice(null);
  };

  const openDocument = async () => {
    try {
      if (!(await canReplace())) return;
      const f = await openFileDialog();
      if (f) adopt(f);
    } catch (e) {
      fail(e);
    }
  };

  const openPath = async (path: string) => {
    try {
      if (!(await canReplace())) return;
      adopt(await readFile(path));
    } catch (e) {
      fail(e);
    }
  };

  const saveDocument = async (saveAs: boolean) => {
    const { source: text, filePath: current } = latest.current;
    try {
      const path = await saveFile(saveAs ? null : current, text);
      if (path) {
        setFilePath(path);
        setSavedSource(text);
        setNotice(null);
      }
    } catch (e) {
      fail(e);
    }
  };

  const loadExample = async (id: string) => {
    const ex = EXAMPLES.find((e) => e.id === id);
    if (!ex) return;
    if (!(await canReplace())) return;
    setExampleId(id);
    setSource(ex.code);
    setSavedSource(ex.code);
    setFilePath(null);
    setEnvelope(null);
    setNotice(null);
  };

  // Keep the native window title ("• name — Kaylei") in sync.
  useEffect(() => {
    setDocState(fileName(filePath), dirty).catch(() => {});
  }, [filePath, dirty]);

  // Native menu, drag-and-drop and "open with" wiring. Handlers are read
  // through a ref so the listeners are registered once but never stale.
  const handlers = useRef<{ menu(a: MenuAction): void; open(p: string): void }>({
    menu: () => {},
    open: () => {},
  });
  handlers.current = {
    menu: (a) => {
      if (a === "new") void newDocument();
      else if (a === "open") void openDocument();
      else if (a === "save") void saveDocument(false);
      else if (a === "save_as") void saveDocument(true);
      else if (a === "run") void run();
      else if (a === "clear_output") setEnvelope(null);
      else if (a === "clear_editor") setSource("");
      else if (a === "reference") setRefOpen(true);
    },
    open: (p) => void openPath(p),
  };
  useEffect(() => {
    let off: (() => void) | undefined;
    let alive = true;
    listenDesktop({
      onMenu: (a) => handlers.current.menu(a),
      onOpenPath: (p) => handlers.current.open(p),
    })
      .then((u) => (alive ? (off = u) : u()))
      .catch(() => {});
    startupFile()
      .then((f) => alive && f && adopt(f))
      .catch(() => {});
    return () => {
      alive = false;
      off?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex min-h-screen w-full max-w-full flex-col overflow-x-clip bg-zinc-100 text-zinc-900">
      {/* Header */}
      <header className="sticky top-0 z-30 border-b border-zinc-200/80 bg-white/80 backdrop-blur-md">
        <div className="mx-auto flex max-w-[1680px] flex-wrap items-center gap-x-4 gap-y-2 px-4 py-3 sm:px-6">
          <div className="flex items-center gap-3">
            <img src={kayleiIcon} alt="" className="h-9 w-9 shrink-0" />
            <div>
              <h1 className="font-brand text-2xl leading-none text-[#1C9A93]">Kaylei</h1>
            </div>
          </div>
          <div className="ml-auto flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => setRefOpen(true)}
              title="Language reference (F1)"
              className="inline-flex h-7 items-center gap-1 rounded-full border border-zinc-300 px-2.5 text-[11px] font-medium text-zinc-600 hover:bg-zinc-100"
            >
              <BookOpen className="h-3.5 w-3.5" /> Reference
            </button>
            <button
              type="button"
              onClick={cycleTheme}
              title={`Theme: ${theme} (click to change)`}
              aria-label={`Theme: ${theme}. Click to change.`}
              className="inline-flex h-7 w-7 items-center justify-center rounded-full border border-zinc-300 text-zinc-600 hover:bg-zinc-100"
            >
              {theme === "dark" ? <Moon className="h-3.5 w-3.5" /> : theme === "light" ? <Sun className="h-3.5 w-3.5" /> : <Monitor className="h-3.5 w-3.5" />}
            </button>
          </div>
        </div>
      </header>

      {/* Main */}
      <main className="mx-auto w-full max-w-[1680px] flex-1 px-4 py-5 sm:px-6">
        {notice && (
          <div className="mb-4 flex items-start gap-2 rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900">
            <span className="flex-1">{notice}</span>
            <button
              className="text-xs text-amber-700 underline"
              onClick={() => setNotice(null)}
            >
              Dismiss
            </button>
          </div>
        )}

        {engineError && (
          <div className="mb-4 rounded-lg border border-rose-300 bg-rose-50 p-3 text-sm text-rose-800">
            <strong>DMath Core is unavailable:</strong> {engineError}
            <span className="block text-xs text-rose-600">
              The native computation engine could not be reached. Restart the app to retry.
            </span>
          </div>
        )}

        <div className="grid grid-cols-1 gap-5 lg:grid-cols-[minmax(400px,44%)_minmax(0,1fr)]">
          {/* Left column: editor + output */}
          <div className="min-w-0 space-y-5">
            <Card className="border-zinc-200 shadow-sm">
              <CardHeader className="flex flex-row flex-wrap items-center gap-2 space-y-0 pb-3">
                <CardTitle className="flex items-center gap-2 text-sm font-bold">
                  <IconTile icon={Braces} />
                  DSL Editor
                  {(filePath || dirty) && (
                    <span className="max-w-[180px] truncate font-mono text-[11px] font-normal text-zinc-500">
                      {fileName(filePath) ?? "Untitled"}
                      {dirty ? " •" : ""}
                    </span>
                  )}
                </CardTitle>
                <div className="ml-auto flex flex-wrap items-center gap-2">
                  <Select value={exampleId} onValueChange={loadExample}>
                    <SelectTrigger
                      className="h-8 w-[220px] max-w-full text-xs"
                      aria-label="Load an example program"
                    >
                      <SelectValue placeholder="Examples…" />
                    </SelectTrigger>
                    <SelectContent>
                      {EXAMPLE_GROUPS.map((g) => (
                        <SelectGroup key={g}>
                          <SelectLabel className="text-xs">{g}</SelectLabel>
                          {EXAMPLES.filter((e) => e.group === g).map((e) => (
                            <SelectItem key={e.id} value={e.id} className="text-xs">
                              {e.name}
                            </SelectItem>
                          ))}
                        </SelectGroup>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                <CodeEditor
                  value={source}
                  onChange={setSource}
                  errorLines={errorLines}
                  onRun={run}
                />
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    onClick={run}
                    disabled={!engine || running}
                    className="h-9 bg-gradient-to-b from-emerald-500 to-emerald-600 px-4 font-semibold text-white shadow-sm shadow-emerald-600/30 hover:from-emerald-500 hover:to-emerald-700"
                  >
                    {running || !engine ? (
                      <Loader2 className="mr-1.5 h-4 w-4 animate-spin" />
                    ) : (
                      <Play className="mr-1.5 h-4 w-4" />
                    )}
                    Run
                    <kbd className="ml-2 rounded border border-emerald-400/40 bg-emerald-500/20 px-1.5 py-0.5 font-mono text-[10px] font-normal">
                      Ctrl+↵
                    </kbd>
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-9"
                    onClick={() => setSource("")}
                  >
                    <Eraser className="mr-1.5 h-4 w-4" />
                    Clear editor
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="h-9"
                    onClick={() => setEnvelope(null)}
                  >
                    <Trash2 className="mr-1.5 h-4 w-4" />
                    Clear output
                  </Button>
                </div>
              </CardContent>
            </Card>

            <Card className="border-zinc-200 shadow-sm">
              <CardHeader className="flex flex-row items-center gap-2 space-y-0 pb-3">
                <CardTitle className="flex items-center gap-2 text-sm font-bold">
                  <IconTile icon={Radar} />
                  Output
                </CardTitle>
                {envelope && (
                  <Badge
                    variant="outline"
                    className={`ml-auto font-mono text-[11px] ${
                      envelope.ok
                        ? "border-emerald-300 bg-emerald-50 text-emerald-700"
                        : "border-rose-300 bg-rose-50 text-rose-700"
                    }`}
                  >
                    {envelope.ok ? "OK" : "ERRORS"}
                  </Badge>
                )}
              </CardHeader>
              <CardContent>
                <OutputPanel envelope={envelope} />
              </CardContent>
            </Card>
          </div>

          {/* Right column: visualizations + trace */}
          <div className="min-w-0 space-y-5">
            <Card className="border-zinc-200 shadow-sm">
              <CardHeader className="flex flex-row items-center gap-2 space-y-0 pb-3">
                <CardTitle className="flex items-center gap-2 text-sm font-bold">
                  <IconTile icon={Sparkles} />
                  Visualization
                </CardTitle>
              </CardHeader>
              <CardContent>
                {vizViews.length > 0 ? (
                  <div className="space-y-6">
                    {vizViews.map((viz, i) => (
                      <VizCard key={i} viz={viz} />
                    ))}
                  </div>
                ) : (
                  <p className="py-6 text-center text-sm text-zinc-400">
                    Add a{" "}
                    <code className="rounded bg-zinc-100 px-1 font-mono text-xs">VISUALIZE … AS TRUTH_TABLE / VENN / DIGRAPH / HASSE / MATRIX / GRAPH</code>{" "}
                    statement, then run the program.
                  </p>
                )}
              </CardContent>
            </Card>

            <Card className="border-zinc-200 shadow-sm">
              <CardHeader className="flex flex-row items-center gap-2 space-y-0 pb-3">
                <CardTitle className="flex items-center gap-2 text-sm font-bold">
                  <IconTile icon={FlaskConical} />
                  Execution Trace
                </CardTitle>
              </CardHeader>
              <CardContent>
                <TracePanel envelope={envelope} />
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
      <ReferenceDialog open={refOpen} onClose={() => setRefOpen(false)} onTry={(code) => setSource(code)} />

    </div>
  );
}

function IconTile({ icon: Icon }: { icon: LucideIcon }) {
  return (
    <span className="flex h-7 w-7 items-center justify-center rounded-lg bg-emerald-50 ring-1 ring-emerald-200/70">
      <Icon className="h-4 w-4 text-emerald-600" />
    </span>
  );
}

function VizCard({ viz }: { viz: VizIr }) {
  return (
    <ExportableCard viz={viz}>
      {viz.type === "truth_table" && <TruthTableView ir={viz} />}
      {viz.type === "venn" && <VennView ir={viz} />}
      {viz.type === "equivalence_check" && <EquivalenceView ir={viz} />}
      {viz.type === "digraph" && <DigraphView ir={viz} />}
      {viz.type === "hasse" && <HasseView ir={viz} />}
      {viz.type === "relation_matrix" && <MatrixView ir={viz} />}
      {viz.type === "graph_view" && <GraphView ir={viz} />}
      {viz.type === "sieve_grid" && <SieveView ir={viz} />}
      {viz.type === "pascal" && <PascalView ir={viz} />}
      {viz.type === "steps" && <StepsView ir={viz} />}
    </ExportableCard>
  );
}
