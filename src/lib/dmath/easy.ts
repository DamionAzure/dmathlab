// Easy Mode: a catalogue of tasks. Each task lists the boxes it needs and turns the
// answers into a Kaylei program, so nothing has to be typed as code.

export type Kind = "formula" | "set" | "pairs" | "int" | "expr" | "graph";
export interface Field { id: string; label: string; kind: Kind; note?: string; def: string }
export interface OptDef { id: string; label: string }
export type Values = Record<string, string>;
export type Built = { code: string; error?: undefined } | { code?: undefined; error: string };

export interface Action {
  id: string;
  label: string;
  fields: string[]; // ids from FIELDS
  opts?: string[]; // ids from OPTS
  /** Return program lines. Throw EasyError for a friendly message. */
  build: (v: Values, o: Record<string, boolean>) => string[];
  directed?: boolean; // graph is always directed (hides the toggle)
}
export interface Area { id: string; title: string; hint: string; actions: Action[] }

class EasyError extends Error {}
const fail = (m: string): never => { throw new EasyError(m); };

// ---- parsing helpers -------------------------------------------------------
const KEYWORDS = new Set(["AND", "OR", "NOT", "XOR", "NAND", "NOR", "IMPLIES", "IFF", "TRUE", "FALSE"]);

export function formulaVars(f: string): string[] {
  const out: string[] = [];
  for (const m of f.matchAll(/[A-Za-z_][A-Za-z0-9_]*/g)) {
    if (!KEYWORDS.has(m[0].toUpperCase()) && !out.includes(m[0])) out.push(m[0]);
  }
  return out;
}

function formula(v: Values, id: string, label: string): string {
  const f = (v[id] ?? "").trim();
  if (!f) fail(`Type a formula for ${label}, for example P AND Q.`);
  if (/[^A-Za-z0-9_\s()]/.test(f)) fail(`${label}: use letters, AND / OR / NOT / IMPLIES / IFF / XOR and brackets only.`);
  const n = formulaVars(f).length;
  if (n === 0) fail(`${label}: add at least one variable such as P or Q.`);
  if (n > 8) fail(`${label}: use at most 8 variables.`);
  return f;
}

function setLit(v: Values, id: string, label: string): string {
  const parts = (v[id] ?? "").split(/[\s,;]+/).filter(Boolean);
  if (parts.length === 0) fail(`${label}: type at least one whole number, for example 1, 2, 3 or 1..5.`);
  for (const p of parts) {
    if (!/^-?\d+$/.test(p) && !/^-?\d+\.\.-?\d+$/.test(p)) fail(`${label}: “${p}” is not a whole number or a range like 1..5.`);
  }
  return `{${parts.join(", ")}}`;
}

function pairsLit(v: Values, id: string, label: string): string {
  const t = v[id] ?? "";
  if (/[^\d\s,()\-;>]/.test(t)) fail(`${label}: use whole numbers in pairs, like (1,2), (2,3).`);
  const nums = t.match(/\d+/g) ?? [];
  if (nums.length === 0) fail(`${label}: type at least one pair, for example (1,2), (2,3).`);
  if (nums.length % 2 !== 0) fail(`${label}: every pair needs two numbers, but there is an odd count.`);
  const out: string[] = [];
  for (let i = 0; i < nums.length; i += 2) out.push(`(${nums[i]},${nums[i + 1]})`);
  return `{${out.join(", ")}}`;
}

function int(v: Values, id: string, label: string): string {
  const t = (v[id] ?? "").trim();
  if (!/^-?\d+$/.test(t)) fail(`${label}: type a whole number, like 12.`);
  return t;
}

function expr(v: Values, id: string): string {
  const t = (v[id] ?? "").trim();
  if (!t) fail("Type a calculation, for example (7 + 5) * 3.");
  if (/[^\d\s+\-*/%^()]/.test(t)) fail("Use whole numbers and + - * / % ^ ( ) only.");
  return t;
}

/** Graph literal from the three graph boxes. `forceDir` makes it a digraph. */
function graph(v: Values, o: Record<string, boolean>, forceDir = false): string {
  const vs = (v["g.v"] ?? "").split(/[\s,;]+/).filter(Boolean);
  if (vs.length === 0) fail("Vertices: type at least one vertex, for example 1..6 or 1, 2, 3.");
  const list: string[] = [];
  for (const t of vs) {
    const r = /^(\d+)\.\.(\d+)$/.exec(t);
    if (r) {
      const [lo, hi] = [Number(r[1]), Number(r[2])];
      if (hi < lo || hi - lo > 200) fail("Vertices: a range like 1..6 must go upward and stay under 200.");
      for (let i = lo; i <= hi; i++) list.push(String(i));
    } else if (/^\d+$/.test(t)) list.push(t);
    else fail(`Vertices: “${t}” is not a whole number or a range like 1..6.`);
  }
  const verts = `{${list.join(", ")}}`;
  const weighted = !!o.w;
  const nums = (v["g.e"] ?? "").match(/\d+/g) ?? [];
  if (/[^\d\s,()\-;>]/.test(v["g.e"] ?? "")) fail("Edges: use whole numbers in brackets, like (1,2), (2,3).");
  const k = weighted ? 3 : 2;
  if (nums.length % k !== 0) fail(`Edges: every edge needs ${k} numbers${weighted ? " (from, to, weight)" : ""}, like ${weighted ? "(1,2,7)" : "(1,2)"}.`);
  const edges: string[] = [];
  for (let i = 0; i < nums.length; i += k) edges.push(`(${nums.slice(i, i + k).join(",")})`);
  return `${forceDir || o.dir ? "DIGRAPH" : "GRAPH"} ${verts} {${edges.join(", ")}}`;
}

// ---- boxes and options -----------------------------------------------------
export const FIELDS: Record<string, Field> = {
  f1: { id: "f1", label: "Formula", kind: "formula", note: "use letters like P, Q, R", def: "P IMPLIES Q" },
  f2: { id: "f2", label: "Formula 2", kind: "formula", def: "NOT P OR Q" },
  A: { id: "A", label: "Set A", kind: "set", note: "1, 2, 3 or 1..5", def: "1..5" },
  B: { id: "B", label: "Set B", kind: "set", def: "4, 5, 6" },
  U: { id: "U", label: "Universe (all elements)", kind: "set", def: "1..10" },
  x: { id: "x", label: "Number x", kind: "int", def: "3" },
  R: { id: "R", label: "Relation R", kind: "pairs", note: "pairs like (1,2), (2,3)", def: "(1,1), (2,2), (3,3), (1,2), (2,1)" },
  S: { id: "S", label: "Second relation S", kind: "pairs", def: "(1,2), (2,3)" },
  F: { id: "F", label: "Function F", kind: "pairs", note: "each pair is (input, output)", def: "(1,4), (2,5), (3,6)" },
  dom: { id: "dom", label: "Inputs (domain)", kind: "set", def: "1, 2, 3" },
  cod: { id: "cod", label: "Possible outputs (codomain)", kind: "set", def: "4, 5, 6" },
  sub: { id: "sub", label: "Subset of the inputs", kind: "set", def: "2, 3" },
  g: { id: "g", label: "Graph", kind: "graph", def: "" },
  a: { id: "a", label: "From vertex", kind: "int", def: "1" },
  b: { id: "b", label: "To vertex", kind: "int", def: "5" },
  n: { id: "n", label: "n", kind: "int", def: "36" },
  cn: { id: "cn", label: "n", kind: "int", note: "keep it small, e.g. 0 to 20", def: "5" },
  ck: { id: "ck", label: "k", kind: "int", def: "2" },
  m: { id: "m", label: "Modulus m", kind: "int", def: "7" },
  p: { id: "p", label: "Number a", kind: "int", def: "48" },
  q: { id: "q", label: "Number b", kind: "int", def: "18" },
  e: { id: "e", label: "Exponent", kind: "int", def: "222" },
  a2: { id: "a2", label: "a₂", kind: "int", def: "3" },
  m2: { id: "m2", label: "m₂", kind: "int", def: "5" },
  base: { id: "base", label: "Base", kind: "int", def: "2" },
  digits: { id: "digits", label: "Digits written as one number", kind: "int", note: "base 10 or less, e.g. 1010", def: "1010" },
  calc: { id: "calc", label: "Calculation", kind: "expr", note: "+ - * / % ^ and brackets", def: "(7 + 5) * 3" },
};
export const GRAPH_DEFAULTS: Values = { "g.v": "1..6", "g.e": "(1,2), (1,3), (2,4), (3,4), (4,5), (5,6)" };

export const OPTS: Record<string, OptDef> = {
  venn: { id: "venn", label: "Draw a Venn diagram" },
  steps: { id: "steps", label: "Show the worked solution" },
  dir: { id: "dir", label: "Edges have a direction" },
  w: { id: "w", label: "Edges have weights" },
};
export const OPT_DEFAULTS: Record<string, boolean> = { venn: true, steps: false, dir: false, w: false };

// ---- builders --------------------------------------------------------------
const show = (call: string, steps: boolean, stepsOk = true): string[] =>
  steps && stepsOk ? [`X = ${call}`, "DISPLAY X", "VISUALIZE X AS STEPS"] : [`DISPLAY ${call}`];

const setOp = (id: string, label: string, op: string): Action => ({
  id, label, fields: ["A", "B"], opts: ["venn", "steps"],
  build: (v, o) => {
    const e = `A ${op} B`;
    const l = [`A = ${setLit(v, "A", "Set A")}`, `B = ${setLit(v, "B", "Set B")}`, `DISPLAY ${e}`];
    if (o.venn) l.push(`VISUALIZE ${e} AS VENN`);
    if (o.steps) l.push(`C = ${e}`, "VISUALIZE C AS STEPS");
    return l;
  },
});
const setTest = (id: string, label: string, op: string): Action => ({
  id, label, fields: ["A", "B"],
  build: (v) => [`A = ${setLit(v, "A", "Set A")}`, `B = ${setLit(v, "B", "Set B")}`, `DISPLAY A ${op} B`],
});
const oneSet = (id: string, label: string, fn: string): Action => ({
  id, label, fields: ["A"], build: (v) => [`A = ${setLit(v, "A", "Set A")}`, `DISPLAY ${fn}(A)`],
});
const relAct = (id: string, label: string, tail: string[], opts?: string[]): Action => ({
  id, label, fields: ["R"], opts, build: (v) => [`R = ${pairsLit(v, "R", "Relation R")}`, ...tail],
});
const closure = (id: string, label: string, fn: string): Action => ({
  id, label, fields: ["R"], opts: ["steps"],
  build: (v, o) => [`R = ${pairsLit(v, "R", "Relation R")}`, `C = ${fn}(R)`, "DISPLAY C", ...(o.steps ? ["VISUALIZE C AS STEPS"] : [])],
});

const gAct = (id: string, label: string, fields: string[], f: (v: Values, o: Record<string, boolean>) => string[], extra: { opts?: string[]; directed?: boolean } = {}): Action => ({
  id, label, fields: ["g", ...fields], opts: ["dir", "w", ...(extra.opts ?? [])].filter((x) => !(extra.directed && x === "dir")), directed: extra.directed,
  build: (v, o) => [`G = ${graph(v, o, extra.directed)}`, ...f(v, o)],
});
const gShow = (id: string, label: string, call: string, stepsOk = false): Action =>
  gAct(id, label, [], (_v, o) => show(call, !!o.steps, stepsOk), { opts: stepsOk ? ["steps"] : [] });

export const AREAS: Area[] = [
  {
    id: "logic", title: "Logic", hint: "Truth tables, equivalence",
    actions: [
      { id: "table", label: "Truth table (tautology or not?)", fields: ["f1"], build: (v) => {
        const f = formula(v, "f1", "Formula");
        return [...formulaVars(f).map((x) => `${x} = TRUE`), `RESULT = ${f}`, "DISPLAY RESULT", "VISUALIZE RESULT AS TRUTH_TABLE"];
      } },
      { id: "equiv", label: "Are two formulas equivalent?", fields: ["f1", "f2"], build: (v) => {
        const a = formula(v, "f1", "Formula 1"), b = formula(v, "f2", "Formula 2");
        return [...[...new Set([...formulaVars(a), ...formulaVars(b)])].map((x) => `${x} = TRUE`), `CHECK EQUIVALENT ${a}, ${b}`];
      } },
    ],
  },
  {
    id: "sets", title: "Sets", hint: "Operations, Venn, counting",
    actions: [
      setOp("union", "Union  A ∪ B", "UNION"),
      setOp("inter", "Intersection  A ∩ B", "INTERSECT"),
      setOp("diff", "Difference  A − B", "DIFFERENCE"),
      setOp("sym", "Symmetric difference  A △ B", "SYMDIFF"),
      { id: "cross", label: "Cartesian product  A × B", fields: ["A", "B"], build: (v) => [`A = ${setLit(v, "A", "Set A")}`, `B = ${setLit(v, "B", "Set B")}`, "DISPLAY A CROSS B"] },
      { id: "incl", label: "Size of a union (inclusion–exclusion)", fields: ["A", "B"], build: (v) => [`A = ${setLit(v, "A", "Set A")}`, `B = ${setLit(v, "B", "Set B")}`, "N = SIZE(A UNION B)", "DISPLAY N", "VISUALIZE N AS STEPS"] },
      setTest("subset", "Is A a subset of B?", "SUBSET"),
      setTest("psubset", "Is A a proper subset of B?", "PSUBSET"),
      setTest("equals", "Are A and B equal?", "EQUALS"),
      { id: "disj", label: "Are A and B disjoint?", fields: ["A", "B"], build: (v) => [`A = ${setLit(v, "A", "Set A")}`, `B = ${setLit(v, "B", "Set B")}`, "DISPLAY IS_DISJOINT(A, B)"] },
      { id: "power", label: "Power set of A", fields: ["A"], opts: ["steps"], build: (v, o) => [`A = ${setLit(v, "A", "Set A")}`, "P = POWERSET(A)", "DISPLAY P", ...(o.steps ? ["VISUALIZE P AS STEPS"] : [])] },
      { id: "comp", label: "Complement of A", fields: ["A", "U"], build: (v) => [`UNIVERSE = ${setLit(v, "U", "Universe")}`, `A = ${setLit(v, "A", "Set A")}`, "DISPLAY COMPLEMENT A"] },
      { id: "member", label: "Is x in A?", fields: ["A", "x"], build: (v) => [`A = ${setLit(v, "A", "Set A")}`, `DISPLAY ${int(v, "x", "x")} IN A`] },
      oneSet("size", "How many elements (size)?", "SIZE"),
      oneSet("min", "Smallest element", "MIN"),
      oneSet("max", "Largest element", "MAX"),
      oneSet("sum", "Sum of the elements", "SUM"),
      oneSet("empty", "Is A empty?", "IS_EMPTY"),
    ],
  },
  {
    id: "relations", title: "Relations", hint: "Properties, closures, diagrams",
    actions: [
      relAct("props", "Check its properties", ["CHECK PROPERTIES R"]),
      relAct("matrix", "Show the matrix", ["VISUALIZE R AS MATRIX"]),
      relAct("digraph", "Draw the arrows (digraph)", ["VISUALIZE R AS DIGRAPH"]),
      relAct("hasse", "Draw the Hasse diagram (partial order)", ["VISUALIZE R AS HASSE"]),
      closure("refl", "Reflexive closure", "REFLEXIVE_CLOSURE"),
      closure("symm", "Symmetric closure", "SYMMETRIC_CLOSURE"),
      closure("trans", "Transitive closure", "TRANSITIVE_CLOSURE"),
      relAct("inv", "Inverse relation", ["DISPLAY INVERSE(R)"]),
      relAct("classes", "Equivalence classes", ["DISPLAY CLASSES(R)"]),
      { id: "classof", label: "Equivalence class of x", fields: ["R", "x"], build: (v) => [`R = ${pairsLit(v, "R", "Relation R")}`, `DISPLAY CLASS_OF(R, ${int(v, "x", "x")})`] },
      { id: "compose", label: "Compose R then S", fields: ["R", "S"], build: (v) => [`R = ${pairsLit(v, "R", "Relation R")}`, `S = ${pairsLit(v, "S", "Second relation S")}`, "DISPLAY COMPOSE(R, S)"] },
    ],
  },
  {
    id: "functions", title: "Functions", hint: "One-to-one? Onto?",
    actions: [
      { id: "check", label: "Is it a function? One-to-one? Onto?", fields: ["F", "dom", "cod"], build: (v) => [
        `F = ${pairsLit(v, "F", "Function F")}`, `A = ${setLit(v, "dom", "Inputs (domain)")}`, `B = ${setLit(v, "cod", "Possible outputs (codomain)")}`,
        "DISPLAY IS_FUNCTION(F, A)", "DISPLAY IS_INJECTIVE(F)", "DISPLAY IS_SURJECTIVE(F, B)", "DISPLAY IS_BIJECTIVE(F, B)"] },
      { id: "dr", label: "Domain and range", fields: ["F"], build: (v) => [`F = ${pairsLit(v, "F", "Function F")}`, "DISPLAY DOMAIN(F), RANGE(F)"] },
      { id: "image", label: "Image of some inputs", fields: ["F", "sub"], build: (v) => [`F = ${pairsLit(v, "F", "Function F")}`, `S = ${setLit(v, "sub", "Subset of the inputs")}`, "DISPLAY IMAGE(F, S)"] },
    ],
  },
  {
    id: "graphs", title: "Graphs", hint: "Paths, trees, colouring",
    actions: [
      gAct("draw", "Draw the graph", [], () => ["VISUALIZE G AS GRAPH"]),
      gAct("gmatrix", "Adjacency matrix", [], () => ["VISUALIZE G AS MATRIX"]),
      gAct("props", "Properties (connected, tree, bipartite…)", [], () => ["CHECK PROPERTIES G"]),
      gAct("degseq", "Degrees of all vertices", [], () => ["DISPLAY DEGREE_SEQUENCE(G)"]),
      gAct("deg", "Degree and neighbours of a vertex", ["a"], (v) => [`DISPLAY DEGREE(G, ${int(v, "a", "Vertex")})`, `DISPLAY NEIGHBORS(G, ${int(v, "a", "Vertex")})`]),
      gAct("adj", "Are two vertices adjacent?", ["a", "b"], (v) => [`DISPLAY ADJACENT(G, ${int(v, "a", "From vertex")}, ${int(v, "b", "To vertex")})`]),
      gAct("bfs", "Breadth-first search (BFS)", ["a"], (v, o) => show(`BFS(G, ${int(v, "a", "Start vertex")})`, !!o.steps), { opts: ["steps"] }),
      gAct("dfs", "Depth-first search (DFS)", ["a"], (v, o) => show(`DFS(G, ${int(v, "a", "Start vertex")})`, !!o.steps), { opts: ["steps"] }),
      gAct("path", "Shortest path", ["a", "b"], (v, o) => show(`SHORTEST_PATH(G, ${int(v, "a", "From vertex")}, ${int(v, "b", "To vertex")})`, !!o.steps), { opts: ["steps"] }),
      gAct("mst", "Minimum spanning tree (weighted)", [], (_v, o) => ["T = MST(G)", "DISPLAY EDGES(T)", "VISUALIZE T AS GRAPH", ...(o.steps ? ["VISUALIZE T AS STEPS"] : [])], { opts: ["steps"] }),
      gAct("color", "Colour the vertices", [], (_v, o) => [...show("COLOR(G)", !!o.steps), "DISPLAY CHROMATIC_NUMBER(G)"], { opts: ["steps"] }),
      gAct("comp", "Connected components", [], () => ["DISPLAY COMPONENTS(G)", "DISPLAY IS_CONNECTED(G)"]),
      gAct("shape", "Tree? Bipartite? Complete?", [], () => ["DISPLAY IS_TREE(G)", "DISPLAY IS_BIPARTITE(G)", "DISPLAY IS_COMPLETE(G)"]),
      gAct("euler", "Eulerian or Hamiltonian?", [], () => ["DISPLAY IS_EULERIAN(G)", "DISPLAY IS_HAMILTONIAN(G)"]),
      gAct("diam", "Diameter, bridges, cut vertices", [], () => ["DISPLAY DIAMETER(G)", "DISPLAY BRIDGES(G)", "DISPLAY CUT_VERTICES(G)"]),
      gAct("gcomp", "Complement graph", [], () => ["H = GRAPH_COMPLEMENT(G)", "DISPLAY EDGES(H)", "VISUALIZE H AS GRAPH"]),
      gAct("topo", "Topological order (directed, no cycles)", [], (_v, o) => [...show("TOPO_SORT(G)", !!o.steps), "DISPLAY IS_DAG(G)"], { directed: true, opts: ["steps"] }),
    ],
  },
  {
    id: "numbers", title: "Numbers", hint: "Primes, gcd, modular, bases",
    actions: [
      { id: "calc", label: "Calculator", fields: ["calc"], build: (v) => [`DISPLAY ${expr(v, "calc")}`] },
      { id: "prime", label: "Is n prime?", fields: ["n"], opts: ["steps"], build: (v, o) => show(`IS_PRIME(${int(v, "n", "n")})`, !!o.steps) },
      { id: "fact", label: "Prime factors of n", fields: ["n"], opts: ["steps"], build: (v, o) => show(`FACTORS(${int(v, "n", "n")})`, !!o.steps) },
      { id: "div", label: "Divisors of n", fields: ["n"], opts: ["steps"], build: (v, o) => show(`DIVISORS(${int(v, "n", "n")})`, !!o.steps) },
      { id: "tot", label: "Totient φ(n) and sum of divisors σ(n)", fields: ["n"], opts: ["steps"], build: (v, o) => [...show(`TOTIENT(${int(v, "n", "n")})`, !!o.steps), `DISPLAY SIGMA(${int(v, "n", "n")})`] },
      { id: "gcd", label: "GCD and LCM", fields: ["p", "q"], opts: ["steps"], build: (v, o) => [...show(`GCD(${int(v, "p", "a")}, ${int(v, "q", "b")})`, !!o.steps), `DISPLAY LCM(${int(v, "p", "a")}, ${int(v, "q", "b")})`] },
      { id: "egcd", label: "Extended GCD (Bézout)", fields: ["p", "q"], opts: ["steps"], build: (v, o) => show(`EGCD(${int(v, "p", "a")}, ${int(v, "q", "b")})`, !!o.steps) },
      { id: "sieve", label: "Sieve of Eratosthenes", fields: ["n"], build: (v) => [`S = SIEVE(${int(v, "n", "n")})`, "DISPLAY S", "VISUALIZE S AS SIEVE"] },
      { id: "mod", label: "Remainder  a mod m", fields: ["p", "m"], build: (v) => [`DISPLAY MOD(${int(v, "p", "a")}, ${int(v, "m", "m")})`] },
      { id: "modpow", label: "Power mod m  (a^e mod m)", fields: ["p", "e", "m"], opts: ["steps"], build: (v, o) => show(`MODPOW(${int(v, "p", "a")}, ${int(v, "e", "Exponent")}, ${int(v, "m", "m")})`, !!o.steps) },
      { id: "modinv", label: "Inverse mod m", fields: ["p", "m"], opts: ["steps"], build: (v, o) => show(`MODINV(${int(v, "p", "a")}, ${int(v, "m", "m")})`, !!o.steps) },
      { id: "order", label: "Order of a mod m", fields: ["p", "m"], build: (v) => [`DISPLAY ORDER_MOD(${int(v, "p", "a")}, ${int(v, "m", "m")})`] },
      { id: "crt", label: "Chinese remainder theorem", fields: ["p", "m", "a2", "m2"], build: (v) => [`DISPLAY CRT(${int(v, "p", "a")}, ${int(v, "m", "m")}, ${int(v, "a2", "a₂")}, ${int(v, "m2", "m₂")})`] },
      { id: "tobase", label: "Write n in another base", fields: ["n", "base"], build: (v) => [`DISPLAY TO_BASE(${int(v, "n", "n")}, ${int(v, "base", "Base")})`] },
      { id: "frombase", label: "Read digits in a base", fields: ["digits", "base"], build: (v) => [`DISPLAY FROM_BASE(${int(v, "digits", "Digits")}, ${int(v, "base", "Base")})`] },
    ],
  },
  {
    id: "counting", title: "Counting", hint: "Factorials, combinations, sequences",
    actions: [
      { id: "fact", label: "Factorial n!", fields: ["cn"], opts: ["steps"], build: (v, o) => show(`FACT(${int(v, "cn", "n")})`, !!o.steps) },
      { id: "perm", label: "Permutations P(n, k)", fields: ["cn", "ck"], opts: ["steps"], build: (v, o) => show(`PERM(${int(v, "cn", "n")}, ${int(v, "ck", "k")})`, !!o.steps) },
      { id: "comb", label: "Combinations C(n, k)", fields: ["cn", "ck"], opts: ["steps"], build: (v, o) => show(`COMB(${int(v, "cn", "n")}, ${int(v, "ck", "k")})`, !!o.steps) },
      { id: "pascal", label: "Pascal's triangle", fields: ["cn"], build: (v) => [`P = PASCAL(${int(v, "cn", "Rows")})`, "DISPLAY P", "VISUALIZE P AS PASCAL"] },
      { id: "fib", label: "Fibonacci number", fields: ["cn"], build: (v) => [`DISPLAY FIB(${int(v, "cn", "n")})`] },
      { id: "cat", label: "Catalan number", fields: ["cn"], build: (v) => [`DISPLAY CATALAN(${int(v, "cn", "n")})`] },
      { id: "der", label: "Derangements", fields: ["cn"], build: (v) => [`DISPLAY DERANGE(${int(v, "cn", "n")})`] },
      { id: "st2", label: "Stirling number of the 2nd kind", fields: ["cn", "ck"], build: (v) => [`DISPLAY STIRLING2(${int(v, "cn", "n")}, ${int(v, "ck", "k")})`] },
      { id: "part", label: "Integer partitions", fields: ["cn"], build: (v) => [`DISPLAY PARTITIONS(${int(v, "cn", "n")})`] },
      { id: "bell", label: "Bell number", fields: ["cn"], build: (v) => [`DISPLAY BELL(${int(v, "cn", "n")})`] },
    ],
  },
];

export function buildAction(action: Action, v: Values, o: Record<string, boolean>): Built {
  try {
    return { code: action.build(v, o).join("\n") };
  } catch (e) {
    if (e instanceof EasyError) return { error: e.message };
    throw e;
  }
}
