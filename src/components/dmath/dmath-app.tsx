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
  Keyboard,
  Link2,
  Loader2,
  Printer,
  FileDown,
  Monitor,
  Moon,
  Play,
  Radar,
  Sparkles,
  Sun,
  Trash2,
} from "lucide-react";

import { useTheme } from "@/lib/dmath/theme";
import { SettingsMenu } from "./settings-menu";
import { ReferenceDialog } from "@/components/dmath/reference-dialog";
import { ExportableCard } from "@/components/dmath/export-bar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CodeEditor, type CodeEditorHandle } from "@/components/dmath/code-editor";
import { SymbolBar } from "./symbol-bar";
import { EasyPanel } from "./easy-panel";
import { ConfirmModal, ExampleModal } from "./modals";
import { ShortcutsDialog } from "./shortcuts-dialog";
import { saveOrDownload, sessionToMarkdown } from "@/lib/dmath/export";
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

const SOURCE_KEY = "kaylei.source";

/** Program from a `#p=` share link, else the last autosaved one. */
function loadInitialSource(): string | null {
  try {
    const m = /[#&]p=([^&]+)/.exec(window.location.hash);
    if (m) {
      const bin = atob(m[1].replace(/-/g, "+").replace(/_/g, "/"));
      return new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0)));
    }
  } catch {
    /* bad link: fall through */
  }
  try {
    return localStorage.getItem(SOURCE_KEY);
  } catch {
    return null;
  }
}

function canShare(): boolean {
  return typeof window !== "undefined" && /^https?:$/.test(window.location.protocol);
}

function shareUrl(source: string): string {
  const bytes = new TextEncoder().encode(source);
  let bin = "";
  bytes.forEach((b) => (bin += String.fromCharCode(b)));
  const b64 = btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  return `${window.location.origin}${window.location.pathname}#p=${b64}`;
}

export function DMathApp() {
  const { choice: theme, cycle: cycleTheme } = useTheme();
  // Start from a shared link, else the last session's program, else the default example.
  const [initial] = useState(() => loadInitialSource() ?? DEFAULT_EXAMPLE.code);
  const [source, setSource] = useState(initial);
  const editorRef = useRef<CodeEditorHandle>(null);
  const [easy, setEasy] = useState(() => {
    try {
      return localStorage.getItem("kaylei.mode") !== "code";
    } catch {
      return true;
    }
  });
  const chooseMode = (e: boolean) => {
    setEasy(e);
    try {
      localStorage.setItem("kaylei.mode", e ? "easy" : "code");
    } catch {
      /* ignore */
    }
  };
  const [refOpen, setRefOpen] = useState(false);
  const [keysOpen, setKeysOpen] = useState(false);
  const [welcome, setWelcome] = useState(() => {
    try {
      return localStorage.getItem("kaylei.welcomed") !== "1";
    } catch {
      return false;
    }
  });
  const dismissWelcome = () => {
    setWelcome(false);
    try {
      localStorage.setItem("kaylei.welcomed", "1");
    } catch {
      /* ignore */
    }
  };

  // "?" opens the shortcut list unless you are typing somewhere.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement | null;
      if (e.key === "?" && !(t && (t.tagName === "TEXTAREA" || t.tagName === "INPUT"))) setKeysOpen(true);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const exportSession = async () => {
    const saved = await saveOrDownload("kaylei-session.md", "md", sessionToMarkdown(source, envelope));
    if (saved) setNotice("Session saved as Markdown.");
  };
  const [exampleId, setExampleId] = useState<string>(initial === DEFAULT_EXAMPLE.code ? DEFAULT_EXAMPLE.id : "");
  const [envelope, setEnvelope] = useState<DMathEnvelope | null>(null);
  const [engine, setEngine] = useState<DMathEngine | null>(null);
  const [engineError, setEngineError] = useState<string | null>(null);
  const [running, setRunning] = useState(false);
  const engineRef = useRef<DMathEngine | null>(null);

  // Document (file) state. `dirty` = edited since last open/save/example load.
  const [filePath, setFilePath] = useState<string | null>(null);
  const [savedSource, setSavedSource] = useState(initial);
  const [notice, setNotice] = useState<string | null>(null);
  const dirty = source !== savedSource;

  // Remember the program between visits (best effort).
  useEffect(() => {
    const t = setTimeout(() => {
      try {
        localStorage.setItem(SOURCE_KEY, source);
      } catch {
        /* storage unavailable */
      }
    }, 400);
    return () => clearTimeout(t);
  }, [source]);

  const copyLink = async () => {
    try {
      await navigator.clipboard.writeText(shareUrl(source));
      setNotice("Link copied. Anyone who opens it gets this program.");
    } catch {
      setNotice("Couldn't copy the link from here.");
    }
  };

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

  const run = useCallback(async (text?: string) => {
    const eng = engineRef.current;
    if (!eng) return;
    setRunning(true);
    try {
      setEnvelope(await eng.run(typeof text === "string" ? text : source));
    } catch (e) {
      setEngineError(e instanceof Error ? e.message : String(e));
    } finally {
      setRunning(false);
    }
  }, [source]);

  // Auto-run the default example once the engine is ready.
  useEffect(() => {
    if (engine && envelope === null && !engineError && !easy) {
      run();
    }
  }, [engine, envelope, engineError, run, easy]);

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
  // In-app confirmation (works on Android too, where there is no native dialog).
  const [confirmAsk, setConfirmAsk] = useState<((ok: boolean) => void) | null>(null);
  const confirmDiscardLocal = () => new Promise<boolean>((resolve) => setConfirmAsk(() => resolve));
  const answerConfirm = (ok: boolean) => {
    confirmAsk?.(ok);
    setConfirmAsk(null);
  };
  const [examplesOpen, setExamplesOpen] = useState(false);
  const canReplace = async () => !latest.current.dirty || (await confirmDiscardLocal());

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
      <header className="sticky top-0 z-30 pt-[env(safe-area-inset-top)] border-b border-zinc-200/80 bg-white/80 backdrop-blur-md">
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
              onClick={() => setKeysOpen(true)}
              title="Keyboard shortcuts (?)"
              aria-label="Keyboard shortcuts"
              className="hidden h-7 w-7 items-center justify-center rounded-full border border-zinc-300 text-zinc-600 hover:bg-zinc-100 sm:inline-flex"
            >
              <Keyboard className="h-3.5 w-3.5" />
            </button>
            <SettingsMenu />
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
        {welcome && (
          <div data-print-hide className="mb-4 flex items-start gap-3 rounded-xl border border-[#1C9A93]/30 bg-[#1C9A93]/10 p-4 text-sm text-zinc-800">
            <div className="min-w-0 flex-1 space-y-1">
              <p className="font-heading text-zinc-900">Welcome to Kaylei</p>
              <p>
                Write sets, logic, graphs or number-theory programs on the left and press <strong>Run</strong>. Pick an example to start,
                tap the symbol bar for quick inserts, and open <strong>Reference</strong> for every function.
              </p>
            </div>
            <button onClick={dismissWelcome} className="shrink-0 rounded-md border border-[#1C9A93]/40 bg-white px-3 py-1.5 text-xs text-zinc-700 hover:bg-zinc-50">
              Got it
            </button>
          </div>
        )}

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
                  {easy ? "Easy Mode" : "DSL Editor"}
                  {(filePath || dirty) && (
                    <span className="max-w-[180px] truncate font-mono text-[11px] font-normal text-zinc-500">
                      {fileName(filePath) ?? "Untitled"}
                      {dirty ? " •" : ""}
                    </span>
                  )}
                </CardTitle>
                <div className="ml-auto flex flex-wrap items-center gap-2">
                  <div className="inline-flex rounded-lg border border-zinc-300 bg-zinc-100 p-0.5 text-xs font-semibold" role="group" aria-label="Mode">
                    {[true, false].map((m) => (
                      <button key={String(m)} type="button" onClick={() => chooseMode(m)} aria-pressed={easy === m}
                        className={`rounded-md px-2.5 py-1 ${easy === m ? "bg-white text-emerald-700 shadow-sm" : "text-zinc-600"}`}>
                        {m ? "Easy" : "Code"}
                      </button>
                    ))}
                  </div>
                  {!easy && (
                    <Button variant="outline" size="sm" className="h-8 max-w-[220px] text-xs" onClick={() => setExamplesOpen(true)} aria-label="Load an example program">
                      <span className="truncate">{EXAMPLES.find((e) => e.id === exampleId)?.name ?? "Examples…"}</span>
                    </Button>
                  )}
                </div>
              </CardHeader>
              <CardContent className="space-y-3">
                {easy ? (
                  <EasyPanel
                    running={running}
                    ready={!!engine}
                    onRun={(code) => {
                      setSource(code);
                      setSavedSource(code);
                      setExampleId("");
                      void run(code);
                    }}
                    onEditAsCode={(code) => {
                      setSource(code);
                      setExampleId("");
                      chooseMode(false);
                    }}
                  />
                ) : (
                <>
                <CodeEditor
                  value={source}
                  onChange={setSource}
                  errorLines={errorLines}
                  onRun={() => run()}
                  ref={editorRef}
                />
                <SymbolBar onInsert={(t, back) => editorRef.current?.insert(t, back)} />
                <div className="flex flex-wrap items-center gap-2">
                  <Button
                    onClick={() => run()}
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
                  {canShare() && (
                    <Button variant="outline" size="sm" className="h-9" onClick={copyLink}>
                      <Link2 className="mr-1.5 h-4 w-4" />
                      Copy link
                    </Button>
                  )}
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
                </>
                )}
              </CardContent>
            </Card>

            <Card className="border-zinc-200 shadow-sm">
              <CardHeader className="flex flex-row items-center gap-2 space-y-0 pb-3">
                <CardTitle className="flex items-center gap-2 text-sm font-bold">
                  <IconTile icon={Radar} />
                  Output
                </CardTitle>
                {envelope && (
                  <div data-print-hide className="ml-auto flex items-center gap-1.5">
                    <button
                      type="button"
                      onClick={exportSession}
                      className="inline-flex h-7 items-center gap-1 rounded-full border border-zinc-300 px-2.5 text-xs text-zinc-600 hover:bg-zinc-100"
                      title="Save the program, results and worked solutions as Markdown"
                    >
                      <FileDown className="h-3.5 w-3.5" /> Markdown
                    </button>
                    <button
                      type="button"
                      onClick={() => window.print()}
                      className="inline-flex h-7 items-center gap-1 rounded-full border border-zinc-300 px-2.5 text-xs text-zinc-600 hover:bg-zinc-100"
                      title="Print or save as PDF"
                    >
                      <Printer className="h-3.5 w-3.5" /> PDF
                    </button>
                  </div>
                )}
                {envelope && (
                  <Badge
                    variant="outline"
                    className={`font-mono text-[11px] ${
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
                <OutputPanel envelope={envelope} onJump={(l) => editorRef.current?.jumpToLine(l)} />
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
      <ShortcutsDialog open={keysOpen} onClose={() => setKeysOpen(false)} />
      <ExampleModal open={examplesOpen} currentId={exampleId} onClose={() => setExamplesOpen(false)}
        onPick={(id) => { setExamplesOpen(false); void loadExample(id); }} />
      <ConfirmModal open={confirmAsk !== null} message="You have unsaved changes. Discard them?" confirmLabel="Discard" onResult={answerConfirm} />
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
