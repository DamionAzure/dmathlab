// A scrolling row of one-tap inserts, mainly for phones where braces and
// keywords are slow to type. Buttons never steal focus, so the keyboard stays up.

interface Sym {
  label: string;
  insert: string;
  back?: number; // move the cursor back this far after inserting
  title?: string;
}

const SYMBOLS: Sym[] = [
  { label: "{ }", insert: "{}", back: 1, title: "Braces" },
  { label: "( )", insert: "()", back: 1, title: "Parentheses" },
  { label: ",", insert: ", " },
  { label: "=", insert: " = " },
  { label: "UNION", insert: " UNION ", title: "∪" },
  { label: "INTERSECT", insert: " INTERSECT ", title: "∩" },
  { label: "DIFFERENCE", insert: " DIFFERENCE ", title: "∖" },
  { label: "SYMDIFF", insert: " SYMDIFF ", title: "△" },
  { label: "COMPLEMENT", insert: "COMPLEMENT ", title: "complement" },
  { label: "SUBSET", insert: " SUBSET ", title: "⊆" },
  { label: "PSUBSET", insert: " PSUBSET ", title: "⊂" },
  { label: "IN", insert: " IN ", title: "∈" },
  { label: "..", insert: "..", title: "Range, e.g. {1..10}" },
  { label: "+ − × ÷", insert: " + ", title: "Arithmetic: + - * / % ^" },
  { label: "< >", insert: " < ", title: "Comparisons: < > <= >= == !=" },
  { label: "AND", insert: " AND ", title: "∧" },
  { label: "OR", insert: " OR ", title: "∨" },
  { label: "NOT", insert: "NOT ", title: "¬" },
  { label: "IMPLIES", insert: " IMPLIES ", title: "→" },
  { label: "IFF", insert: " IFF ", title: "↔" },
  { label: "XOR", insert: " XOR ", title: "⊕" },
  { label: "NAND", insert: " NAND ", title: "⊼" },
  { label: "NOR", insert: " NOR ", title: "⊽" },
  { label: "DISPLAY", insert: "DISPLAY " },
  { label: "VISUALIZE", insert: "VISUALIZE " },
  { label: "AS", insert: " AS " },
  { label: "#", insert: "# ", title: "Comment" },
];

export function SymbolBar({ onInsert }: { onInsert: (text: string, back?: number) => void }) {
  return (
    <div
      role="toolbar"
      aria-label="Insert symbols"
      className="-mx-1 flex gap-1.5 overflow-x-auto px-1 pb-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden"
    >
      {SYMBOLS.map((s) => (
        <button
          key={s.label}
          type="button"
          title={s.title}
          // keep focus (and the on-screen keyboard) in the editor
          onMouseDown={(e) => e.preventDefault()}
          onClick={() => onInsert(s.insert, s.back)}
          className="h-8 shrink-0 rounded-md border border-zinc-300 bg-white px-2.5 font-mono text-xs text-zinc-700 active:bg-zinc-200"
        >
          {s.label}
        </button>
      ))}
    </div>
  );
}
