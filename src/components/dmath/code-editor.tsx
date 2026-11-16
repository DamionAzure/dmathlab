"use client";

// DSL editor: line numbers + syntax highlighting overlay + error line marks.
//
// Implementation: a transparent <textarea> stacked on a <pre> that renders
// the same text with token colors. Both share identical font metrics so the
// text and highlight stay in register. The gutter is a third layer synced by
// scrollTop.

import { useEffect, useMemo, useRef } from "react";
import { tokenizeSource } from "@/lib/dmath/highlight";

interface CodeEditorProps {
  value: string;
  onChange: (v: string) => void;
  errorLines: number[];
  onRun: () => void;
  readOnly?: boolean;
}

export function CodeEditor({ value, onChange, errorLines, onRun }: CodeEditorProps) {
  const taRef = useRef<HTMLTextAreaElement>(null);
  const preRef = useRef<HTMLPreElement>(null);
  const gutterRef = useRef<HTMLDivElement>(null);

  const lines = useMemo(() => tokenizeSource(value), [value]);
  const lineCount = useMemo(() => value.split("\n").length, [value]);
  const errorSet = useMemo(() => new Set(errorLines), [errorLines]);

  const syncScroll = () => {
    const ta = taRef.current;
    if (!ta) return;
    if (preRef.current) {
      preRef.current.scrollTop = ta.scrollTop;
      preRef.current.scrollLeft = ta.scrollLeft;
    }
    if (gutterRef.current) {
      gutterRef.current.scrollTop = ta.scrollTop;
    }
  };

  useEffect(() => {
    syncScroll();
  }, [value]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      onRun();
      return;
    }
    if (e.key === "Tab") {
      e.preventDefault();
      const ta = e.currentTarget;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      const next = value.slice(0, start) + "  " + value.slice(end);
      onChange(next);
      requestAnimationFrame(() => {
        ta.selectionStart = ta.selectionEnd = start + 2;
      });
    }
  };

  return (
    <div className="relative flex overflow-hidden rounded-xl border border-zinc-800 bg-zinc-950 font-mono shadow-[inset_0_1px_0_rgb(255_255_255/0.05)] text-[13px] leading-5">
      {/* Line-number gutter */}
      <div
        ref={gutterRef}
        aria-hidden
        className="w-11 shrink-0 select-none overflow-hidden border-r border-zinc-800 bg-zinc-950 py-3 text-right"
      >
        {Array.from({ length: lineCount }, (_, i) => (
          <div
            key={i}
            className={`px-2 ${errorSet.has(i + 1) ? "bg-rose-500/20 text-rose-300" : "text-zinc-600"}`}
          >
            {i + 1}
          </div>
        ))}
      </div>

      {/* Code area */}
      <div className="relative min-w-0 flex-1">
        <pre
          ref={preRef}
          aria-hidden
          className="pointer-events-none absolute inset-0 overflow-hidden whitespace-pre px-3 py-3"
        >
          {lines.map((toks, i) => (
            <div
              key={i}
              className={`min-h-5 ${errorSet.has(i + 1) ? "rounded bg-rose-500/10" : ""}`}
            >
              {toks.length === 0 ? (
                <span> </span>
              ) : (
                toks.map((t, j) => (
                  <span key={j} className={t.cls}>
                    {t.text}
                  </span>
                ))
              )}
            </div>
          ))}
          {/* trailing spacer keeps last line visible while typing */}
          <div className="min-h-5" />
        </pre>
        <textarea
          ref={taRef}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onScroll={syncScroll}
          onKeyDown={handleKeyDown}
          spellCheck={false}
          aria-label="DMath program editor"
          className="absolute inset-0 h-full w-full resize-none overflow-auto whitespace-pre bg-transparent px-3 py-3 text-transparent caret-emerald-300 outline-none selection:bg-emerald-400/30"
        />
      </div>
    </div>
  );
}
