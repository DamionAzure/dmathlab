"use client";

// DSL editor: line numbers + syntax highlighting overlay + error line marks.
//
// Implementation: a transparent <textarea> stacked on a <pre> that renders
// the same text with token colors. Both share identical font metrics so the
// text and highlight stay in register. The gutter is a third layer synced by
// scrollTop.

import { forwardRef, useEffect, useImperativeHandle, useMemo, useRef } from "react";
import { tokenizeSource } from "@/lib/dmath/highlight";

interface CodeEditorProps {
  value: string;
  onChange: (v: string) => void;
  errorLines: number[];
  onRun: () => void;
  readOnly?: boolean;
}

/** Methods the page can call on the editor (symbol bar, click-to-jump errors). */
export interface CodeEditorHandle {
  /** Insert text at the cursor (replacing any selection); `caretBack` moves the cursor left afterwards. */
  insert(text: string, caretBack?: number): void;
  /** Select a 1-based line and scroll it into view. */
  jumpToLine(line: number): void;
  focus(): void;
}

export const CodeEditor = forwardRef<CodeEditorHandle, CodeEditorProps>(function CodeEditor(
  { value, onChange, errorLines, onRun },
  ref,
) {
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

  useImperativeHandle(ref, () => ({
    insert(text, caretBack = 0) {
      const ta = taRef.current;
      if (!ta) return;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      onChange(value.slice(0, start) + text + value.slice(end));
      const pos = start + text.length - caretBack;
      ta.focus();
      // React re-renders the controlled value first, which resets the caret; set ours after that.
      requestAnimationFrame(() =>
        requestAnimationFrame(() => {
          ta.selectionStart = ta.selectionEnd = pos;
        }),
      );
    },
    jumpToLine(line) {
      const ta = taRef.current;
      if (!ta) return;
      const all = value.split("\n");
      const idx = Math.min(Math.max(line, 1), all.length) - 1;
      const start = all.slice(0, idx).reduce((n, l) => n + l.length + 1, 0);
      ta.focus();
      ta.setSelectionRange(start, start + all[idx].length);
      ta.scrollTop = Math.max(0, idx * 20 - ta.clientHeight / 3); // 20px = leading-5
      syncScroll();
    },
    focus() {
      taRef.current?.focus();
    },
  }));

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      onRun();
      return;
    }
    // Ctrl/Cmd + / toggles "# " comments on the selected lines.
    if ((e.ctrlKey || e.metaKey) && e.key === "/") {
      e.preventDefault();
      const ta = e.currentTarget;
      const a = value.lastIndexOf("\n", ta.selectionStart - 1) + 1;
      let b = value.indexOf("\n", ta.selectionEnd);
      if (b === -1) b = value.length;
      const block = value.slice(a, b).split("\n");
      const allCommented = block.every((l) => l.trim() === "" || /^\s*#/.test(l));
      const out = block.map((l) =>
        l.trim() === "" ? l : allCommented ? l.replace(/^(\s*)# ?/, "$1") : l.replace(/^(\s*)/, "$1# "),
      );
      onChange(value.slice(0, a) + out.join("\n") + value.slice(b));
      requestAnimationFrame(() => ta.setSelectionRange(a, a + out.join("\n").length));
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
    <div className="keep-dark relative flex h-72 w-full min-w-0 max-w-full overflow-hidden rounded-xl sm:h-80 lg:h-96 border border-zinc-800 bg-zinc-950 font-mono shadow-[inset_0_1px_0_rgb(255_255_255/0.05)] text-[13px] leading-5">
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
          autoCapitalize="none"
          autoCorrect="off"
          autoComplete="off"
          aria-label="DMath program editor"
          className="absolute inset-0 h-full w-full resize-none overflow-auto whitespace-pre [overflow-wrap:normal] [-webkit-text-size-adjust:100%] bg-transparent px-3 py-3 text-transparent caret-emerald-300 outline-none selection:bg-emerald-400/30"
        />
      </div>
    </div>
  );
});
