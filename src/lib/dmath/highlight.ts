// Lightweight tokenizer for DMath syntax highlighting in the editor.
// Mirrors the real lexer's keyword set (case-insensitive keywords,
// case-sensitive identifiers) but is purely cosmetic.

export interface Token {
  text: string;
  cls: string;
}

const LOGIC_OPS = new Set(["AND", "OR", "NOT", "XOR", "NAND", "NOR", "IMPLIES", "IFF"]);
const SET_OPS = new Set(["UNION", "INTERSECT", "SYMDIFF", "DIFFERENCE", "COMPLEMENT", "CROSS", "SUBSET", "EQUALS"]);
const STMT_KW = new Set([
  "DISPLAY",
  "VISUALIZE",
  "AS",
  "CHECK",
  "EQUIVALENT",
  "PROPERTIES",
  // Visualization targets (… AS DIGRAPH / HASSE / MATRIX / GRAPH)
  "DIGRAPH",
  "HASSE",
  "MATRIX",
  "STEPS",
  "GRAPH",
]);
// Builtin functions of the Relations & Graph Theory domains (v1.1).
const BUILTIN_FN = new Set([
  "INVERSE",
  "COMPOSE",
  "TRANSITIVE_CLOSURE",
  "REFLEXIVE_CLOSURE",
  "SYMMETRIC_CLOSURE",
  "IS_REFLEXIVE",
  "IS_SYMMETRIC",
  "IS_ANTISYMMETRIC",
  "IS_TRANSITIVE",
  "IS_EQUIVALENCE",
  "IS_PARTIAL_ORDER",
  "VERTICES",
  "EDGES",
  "DEGREE",
  "NEIGHBORS",
  "ADJACENT",
  "BFS",
  "DFS",
  "SHORTEST_PATH",
  "IS_CONNECTED",
  "IS_TREE",
  "IS_BIPARTITE",
  "IS_COMPLETE",
  "COLOR",
  "MST",
  // Number theory & combinatorics (v1.2 / v1.3)
  "IS_PRIME",
  "FACTORS",
  "DIVISORS",
  "GCD",
  "LCM",
  "TOTIENT",
  "SIEVE",
  "FACT",
  "PERM",
  "COMB",
  "PASCAL",
  // Modular arithmetic, more counting, more graphs
  "MOD",
  "MODPOW",
  "MODINV",
  "EGCD",
  "CRT",
  "FIB",
  "CATALAN",
  "DERANGE",
  "STIRLING2",
  "TOPO_SORT",
  "IS_DAG",
  "COMPONENTS",
  "IS_EULERIAN",
  "DEGREE_SEQUENCE",
  "SIZE",
  "POWERSET",
  "CONTAINS",
  "IS_DISJOINT",
]);
const LITERALS = new Set(["TRUE", "FALSE"]);

const CLS = {
  logic: "text-violet-300 font-semibold",
  set: "text-teal-300 font-semibold",
  stmt: "text-amber-300",
  literal: "text-emerald-300 font-semibold",
  universe: "text-orange-300 font-semibold",
  fn: "text-sky-300 font-semibold",
  number: "text-orange-300",
  ident: "text-zinc-100",
  punct: "text-zinc-500",
  comment: "text-zinc-500 italic",
};

function wordClass(word: string): string {
  const u = word.toUpperCase();
  if (u === "UNIVERSE") return CLS.universe;
  if (LITERALS.has(u)) return CLS.literal;
  if (LOGIC_OPS.has(u)) return CLS.logic;
  if (SET_OPS.has(u)) return CLS.set;
  if (STMT_KW.has(u)) return CLS.stmt;
  if (BUILTIN_FN.has(u)) return CLS.fn;
  return CLS.ident;
}

/** Tokenize one line of DMath source into styled spans. */
export function tokenizeLine(line: string): Token[] {
  const toks: Token[] = [];
  let i = 0;
  const n = line.length;

  while (i < n) {
    const c = line[i];

    // Comments to end of line
    if (c === "#" || (c === "/" && line[i + 1] === "/")) {
      toks.push({ text: line.slice(i), cls: CLS.comment });
      break;
    }
    // Whitespace
    if (c === " " || c === "\t") {
      let j = i;
      while (j < n && (line[j] === " " || line[j] === "\t")) j++;
      toks.push({ text: line.slice(i, j), cls: "" });
      i = j;
      continue;
    }
    // Words (identifiers / keywords)
    if (/[A-Za-z_]/.test(c)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(line[j])) j++;
      toks.push({ text: line.slice(i, j), cls: wordClass(line.slice(i, j)) });
      i = j;
      continue;
    }
    // Numbers
    if (/[0-9]/.test(c)) {
      let j = i;
      while (j < n && /[0-9]/.test(line[j])) j++;
      toks.push({ text: line.slice(i, j), cls: CLS.number });
      i = j;
      continue;
    }
    // Punctuation
    toks.push({ text: c, cls: CLS.punct });
    i++;
  }
  return toks;
}

export function tokenizeSource(source: string): Token[][] {
  return source.split("\n").map(tokenizeLine);
}
