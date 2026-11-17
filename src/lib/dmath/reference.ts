// Language reference shown in Help → Language Reference. Every `example` is
// a complete program; `npm run check:reference` runs them all through the
// real engine so this page can never document something that doesn't work.

export interface RefEntry {
  /** Keyword / function name, searched and highlighted. */
  name: string;
  /** How it is written. */
  syntax: string;
  /** What it gives back, when that is not obvious. */
  returns?: string;
  summary: string;
  /** A complete, runnable program. */
  example: string;
}

export interface RefSection {
  id: string;
  title: string;
  blurb?: string;
  entries: RefEntry[];
}

const G6 = "G = GRAPH {1,2,3,4,5,6} {(1,2), (1,3), (2,4), (3,4), (4,5), (5,6)}";
const W5 = "W = GRAPH {1,2,3,4,5} {(1,2,7), (1,3,3), (3,2,2), (2,4,2), (3,4,8), (4,5,3)}";
const REL = "R = {(1,1), (1,2), (2,2), (2,3), (3,3)}";

export const REFERENCE: RefSection[] = [
  {
    id: "basics",
    title: "The basics",
    blurb:
      "A program is a list of statements, one per line (or separated by ;). Keywords are not case-sensitive. A # starts a comment that runs to the end of the line. Numbers are whole numbers; write negatives with a minus sign.",
    entries: [
      { name: "Assignment", syntax: "NAME = expression", summary: "Give a value a name. Names can be used in later lines and can be reassigned.", example: "A = {1, 2, 3}\nB = A UNION {9}\nDISPLAY B" },
      { name: "DISPLAY", syntax: "DISPLAY expression [, expression …]", summary: "Show one or more values in the Output panel.", example: "A = {1, 2}\nDISPLAY A, GCD(12, 18)" },
      { name: "Set literal", syntax: "{ 1, 2, 3 }", returns: "Set", summary: "A set of whole numbers. Duplicates collapse. Pairs such as (1, 2) inside the braces make a relation.", example: "S = {3, 1, 2, 3}\nDISPLAY S" },
      { name: "Boolean", syntax: "TRUE   FALSE", returns: "Boolean", summary: "The two truth values.", example: "P = TRUE\nDISPLAY P" },
      { name: "UNIVERSE", syntax: "UNIVERSE = { … }", summary: "Declare the universal set. COMPLEMENT is taken relative to it, and VENN draws the outside region.", example: "UNIVERSE = {1, 2, 3, 4, 5}\nA = {1, 2}\nC = COMPLEMENT A\nDISPLAY C" },
      { name: "GRAPH", syntax: "GRAPH {vertices} {edges}", returns: "Graph", summary: "An undirected graph. An edge is (a, b), or (a, b, weight) for a weighted graph.", example: "G = GRAPH {1,2,3} {(1,2), (2,3,5)}\nDISPLAY VERTICES(G)" },
      { name: "DIGRAPH (literal)", syntax: "DIGRAPH {vertices} {edges}", returns: "Graph", summary: "A directed graph: (a, b) is an arrow from a to b.", example: "D = DIGRAPH {1,2,3} {(1,2), (2,3), (3,1)}\nDISPLAY EDGES(D)" },
    ],
  },
  {
    id: "logic",
    title: "Logic & sets",
    blurb:
      "Operators bind from loosest to tightest: IFF, IMPLIES, OR, AND, NOT, then SUBSET / EQUALS, then UNION / DIFFERENCE, INTERSECT, CROSS. Use parentheses to be explicit.",
    entries: [
      { name: "AND", syntax: "P AND Q", returns: "Boolean", summary: "TRUE only when both sides are TRUE.", example: "DISPLAY TRUE AND FALSE" },
      { name: "OR", syntax: "P OR Q", returns: "Boolean", summary: "TRUE when at least one side is TRUE.", example: "DISPLAY TRUE OR FALSE" },
      { name: "NOT", syntax: "NOT P", returns: "Boolean", summary: "Flips TRUE and FALSE.", example: "DISPLAY NOT TRUE" },
      { name: "IMPLIES", syntax: "P IMPLIES Q", returns: "Boolean", summary: "FALSE only when P is TRUE and Q is FALSE.", example: "DISPLAY TRUE IMPLIES FALSE" },
      { name: "IFF", syntax: "P IFF Q", returns: "Boolean", summary: "TRUE when both sides have the same value.", example: "DISPLAY TRUE IFF TRUE" },
      { name: "UNION", syntax: "A UNION B", returns: "Set", summary: "Everything in A or B.", example: "DISPLAY {1,2} UNION {2,3}" },
      { name: "INTERSECT", syntax: "A INTERSECT B", returns: "Set", summary: "Only what is in both.", example: "DISPLAY {1,2} INTERSECT {2,3}" },
      { name: "DIFFERENCE", syntax: "A DIFFERENCE B", returns: "Set", summary: "What is in A but not in B.", example: "DISPLAY {1,2,3} DIFFERENCE {2}" },
      { name: "COMPLEMENT", syntax: "COMPLEMENT A", returns: "Set", summary: "Everything in UNIVERSE that is not in A. Declare UNIVERSE first.", example: "UNIVERSE = {1,2,3,4}\nDISPLAY COMPLEMENT {1,2}" },
      { name: "CROSS", syntax: "A CROSS B", returns: "Set of pairs", summary: "The Cartesian product: every pair (a, b) with a in A and b in B.", example: "DISPLAY {1,2} CROSS {3,4}" },
      { name: "SUBSET", syntax: "A SUBSET B", returns: "Boolean", summary: "TRUE when every element of A is also in B.", example: "DISPLAY {1,2} SUBSET {1,2,3}" },
      { name: "EQUALS", syntax: "A EQUALS B", returns: "Boolean", summary: "TRUE when the two sets have exactly the same elements.", example: "DISPLAY {1,2} EQUALS {2,1}" },
    ],
  },
  {
    id: "statements",
    title: "Checking & visualizing",
    entries: [
      { name: "CHECK EQUIVALENT", syntax: "CHECK EQUIVALENT expr1, expr2", summary: "Decide whether two logical expressions always have the same value, by trying every combination of the variables.", example: "P = TRUE\nQ = FALSE\nA = P IMPLIES Q\nB = NOT P OR Q\nCHECK EQUIVALENT A, B" },
      { name: "CHECK PROPERTIES", syntax: "CHECK PROPERTIES R", summary: "For a relation: reflexive, symmetric, antisymmetric, transitive, equivalence relation, partial order, each with a counterexample. For a graph: connected, tree, bipartite, complete, and more.", example: REL + "\nCHECK PROPERTIES R" },
      { name: "AS TRUTH_TABLE", syntax: "VISUALIZE expr AS TRUTH_TABLE", summary: "The full truth table of a logical expression, labelled tautology, contradiction or contingency.", example: "P = TRUE\nQ = FALSE\nR = P AND Q\nVISUALIZE R AS TRUTH_TABLE" },
      { name: "AS VENN", syntax: "VISUALIZE setExpr AS VENN", summary: "A Venn diagram of a set operation (UNION, INTERSECT, DIFFERENCE, COMPLEMENT, …).", example: "A = {1,2,3}\nB = {3,4}\nC = A UNION B\nVISUALIZE C AS VENN" },
      { name: "AS DIGRAPH", syntax: "VISUALIZE R AS DIGRAPH", summary: "A relation drawn as arrows between elements.", example: REL + "\nVISUALIZE R AS DIGRAPH" },
      { name: "AS HASSE", syntax: "VISUALIZE R AS HASSE", summary: "The Hasse diagram of a partial order (needs reflexive, antisymmetric and transitive).", example: "R = {(1,1), (2,2), (3,3), (1,2), (1,3)}\nVISUALIZE R AS HASSE" },
      { name: "AS MATRIX", syntax: "VISUALIZE R AS MATRIX", summary: "The 0/1 relation matrix.", example: REL + "\nVISUALIZE R AS MATRIX" },
      { name: "AS GRAPH", syntax: "VISUALIZE G AS GRAPH", summary: "Draw a graph, or the result of MST(G) (the chosen edges are highlighted).", example: G6 + "\nVISUALIZE G AS GRAPH" },
      { name: "AS SIEVE", syntax: "VISUALIZE S AS SIEVE", summary: "The Sieve of Eratosthenes as a grid of crossed-out numbers. S must come from SIEVE(n).", example: "S = SIEVE(50)\nVISUALIZE S AS SIEVE" },
      { name: "AS PASCAL", syntax: "VISUALIZE P AS PASCAL", summary: "Pascal's triangle. P comes from PASCAL(n), or from COMB(n, k) to highlight that cell.", example: "P = PASCAL(8)\nVISUALIZE P AS PASCAL" },
      { name: "AS STEPS", syntax: "VISUALIZE X AS STEPS", summary: "A worked, step-by-step solution of a call, solved by several methods that must all agree. Works for GCD, LCM, IS_PRIME, FACTORS, DIVISORS, TOTIENT, SIEVE, FACT, PERM, COMB, BFS, DFS, SHORTEST_PATH, MST, COLOR, IS_BIPARTITE and IS_CONNECTED.", example: "G = GCD(48, 18)\nVISUALIZE G AS STEPS" },
    ],
  },
  {
    id: "relations",
    title: "Relations",
    blurb: "A relation is a set of ordered pairs such as {(1,2), (2,3)}.",
    entries: [
      { name: "INVERSE", syntax: "INVERSE(R)", returns: "Set", summary: "Flip every pair: (a, b) becomes (b, a).", example: "DISPLAY INVERSE({(1,2), (2,3)})" },
      { name: "COMPOSE", syntax: "COMPOSE(R, S)", returns: "Set", summary: "(a, c) is in the result when (a, b) is in R and (b, c) is in S for some b.", example: "DISPLAY COMPOSE({(1,2)}, {(2,3)})" },
      { name: "TRANSITIVE_CLOSURE", syntax: "TRANSITIVE_CLOSURE(R)", returns: "Set", summary: "The smallest transitive relation containing R (Warshall's algorithm).", example: "DISPLAY TRANSITIVE_CLOSURE({(1,2), (2,3)})" },
      { name: "REFLEXIVE_CLOSURE", syntax: "REFLEXIVE_CLOSURE(R)", returns: "Set", summary: "R plus (a, a) for every element a it mentions.", example: "DISPLAY REFLEXIVE_CLOSURE({(1,2)})" },
      { name: "SYMMETRIC_CLOSURE", syntax: "SYMMETRIC_CLOSURE(R)", returns: "Set", summary: "R plus the reverse of every pair.", example: "DISPLAY SYMMETRIC_CLOSURE({(1,2)})" },
      { name: "IS_REFLEXIVE", syntax: "IS_REFLEXIVE(R)", returns: "Boolean", summary: "Is (a, a) in R for every element a?", example: "DISPLAY IS_REFLEXIVE({(1,1), (2,2)})" },
      { name: "IS_SYMMETRIC", syntax: "IS_SYMMETRIC(R)", returns: "Boolean", summary: "Whenever (a, b) is in R, is (b, a) too?", example: "DISPLAY IS_SYMMETRIC({(1,2), (2,1)})" },
      { name: "IS_ANTISYMMETRIC", syntax: "IS_ANTISYMMETRIC(R)", returns: "Boolean", summary: "If both (a, b) and (b, a) are in R, must a equal b?", example: "DISPLAY IS_ANTISYMMETRIC({(1,2), (2,3)})" },
      { name: "IS_TRANSITIVE", syntax: "IS_TRANSITIVE(R)", returns: "Boolean", summary: "Whenever (a, b) and (b, c) are in R, is (a, c)?", example: "DISPLAY IS_TRANSITIVE({(1,2), (2,3), (1,3)})" },
      { name: "IS_EQUIVALENCE", syntax: "IS_EQUIVALENCE(R)", returns: "Boolean", summary: "Reflexive, symmetric and transitive.", example: "DISPLAY IS_EQUIVALENCE({(1,1), (2,2), (1,2), (2,1)})" },
      { name: "IS_PARTIAL_ORDER", syntax: "IS_PARTIAL_ORDER(R)", returns: "Boolean", summary: "Reflexive, antisymmetric and transitive.", example: "DISPLAY IS_PARTIAL_ORDER({(1,1), (2,2), (1,2)})" },
    ],
  },
  {
    id: "graphs",
    title: "Graphs",
    blurb: "Vertices are whole numbers. Neighbors are always tried in ascending order, so results are repeatable.",
    entries: [
      { name: "VERTICES", syntax: "VERTICES(G)", returns: "Set", summary: "The vertex set.", example: G6 + "\nDISPLAY VERTICES(G)" },
      { name: "EDGES", syntax: "EDGES(G)", returns: "Set", summary: "The edges, as pairs.", example: G6 + "\nDISPLAY EDGES(G)" },
      { name: "DEGREE", syntax: "DEGREE(G, v)", returns: "Number", summary: "How many edge-ends touch v (a loop counts twice; in a digraph, in + out).", example: G6 + "\nDISPLAY DEGREE(G, 4)" },
      { name: "NEIGHBORS", syntax: "NEIGHBORS(G, v)", returns: "List", summary: "The vertices next to v, ascending (successors for a digraph).", example: G6 + "\nDISPLAY NEIGHBORS(G, 4)" },
      { name: "ADJACENT", syntax: "ADJACENT(G, a, b)", returns: "Boolean", summary: "Is there an edge between a and b? Direction matters in a digraph.", example: G6 + "\nDISPLAY ADJACENT(G, 1, 2)" },
      { name: "BFS", syntax: "BFS(G, start)", returns: "List", summary: "Breadth-first visit order: nearest vertices first, using a queue.", example: G6 + "\nDISPLAY BFS(G, 1)" },
      { name: "DFS", syntax: "DFS(G, start)", returns: "List", summary: "Depth-first visit order: dive as deep as possible, then back up.", example: G6 + "\nDISPLAY DFS(G, 1)" },
      { name: "SHORTEST_PATH", syntax: "SHORTEST_PATH(G, from, to)", returns: "List", summary: "The cheapest path. Uses Dijkstra's algorithm when any edge has a weight (weights must not be negative), plain BFS otherwise. An empty list means no path.", example: W5 + "\nDISPLAY SHORTEST_PATH(W, 1, 5)" },
      { name: "IS_CONNECTED", syntax: "IS_CONNECTED(G)", returns: "Boolean", summary: "Can every vertex reach every other (directions ignored)?", example: G6 + "\nDISPLAY IS_CONNECTED(G)" },
      { name: "IS_TREE", syntax: "IS_TREE(G)", returns: "Boolean", summary: "Connected with exactly one fewer edge than vertices.", example: G6 + "\nDISPLAY IS_TREE(G)" },
      { name: "IS_BIPARTITE", syntax: "IS_BIPARTITE(G)", returns: "Boolean", summary: "Can the vertices be split into two teams with every edge going between teams?", example: G6 + "\nDISPLAY IS_BIPARTITE(G)" },
      { name: "IS_COMPLETE", syntax: "IS_COMPLETE(G)", returns: "Boolean", summary: "Is every pair of vertices joined?", example: "K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}\nDISPLAY IS_COMPLETE(K)" },
      { name: "COLOR", syntax: "COLOR(G)", returns: "Mapping", summary: "Greedy colouring: vertices in ascending order each get the smallest colour number their neighbours don't use.", example: G6 + "\nDISPLAY COLOR(G)" },
      { name: "MST", syntax: "MST(G)", returns: "Graph", summary: "A minimum spanning tree (Kruskal's algorithm). The graph must be connected and have no negative weights.", example: W5 + "\nT = MST(W)\nVISUALIZE T AS GRAPH" },
    ],
  },
  {
    id: "number-theory",
    title: "Number theory",
    entries: [
      { name: "IS_PRIME", syntax: "IS_PRIME(n)", returns: "Boolean", summary: "Is n prime? Exact for every 64-bit number.", example: "DISPLAY IS_PRIME(97)" },
      { name: "FACTORS", syntax: "FACTORS(n)", returns: "List", summary: "Prime factorization, with repeats: FACTORS(12) is [2, 2, 3]. n must be positive.", example: "DISPLAY FACTORS(360)" },
      { name: "DIVISORS", syntax: "DIVISORS(n)", returns: "List", summary: "Every positive divisor of n, ascending.", example: "DISPLAY DIVISORS(36)" },
      { name: "GCD", syntax: "GCD(a, b)", returns: "Number", summary: "Greatest common divisor.", example: "DISPLAY GCD(48, 18)" },
      { name: "LCM", syntax: "LCM(a, b)", returns: "Number", summary: "Least common multiple.", example: "DISPLAY LCM(12, 18)" },
      { name: "TOTIENT", syntax: "TOTIENT(n)", returns: "Number", summary: "Euler's φ: how many numbers from 1 to n share no factor with n.", example: "DISPLAY TOTIENT(36)" },
      { name: "SIEVE", syntax: "SIEVE(n)", returns: "List", summary: "All primes up to n (n up to 100 000), by the Sieve of Eratosthenes.", example: "DISPLAY SIEVE(30)" },
    ],
  },
  {
    id: "combinatorics",
    title: "Counting",
    entries: [
      { name: "FACT", syntax: "FACT(n)", returns: "Number", summary: "n! = n × (n−1) × … × 1. n can be at most 20 (21! no longer fits in a 64-bit number).", example: "DISPLAY FACT(10)" },
      { name: "PERM", syntax: "PERM(n, k)", returns: "Number", summary: "Ordered selections: how many ways to arrange k of n things.", example: "DISPLAY PERM(5, 2)" },
      { name: "COMB", syntax: "COMB(n, k)", returns: "Number", summary: "Unordered selections: n choose k.", example: "DISPLAY COMB(5, 2)" },
      { name: "PASCAL", syntax: "PASCAL(n)", returns: "List", summary: "The first n rows of Pascal's triangle (n up to 25).", example: "DISPLAY PASCAL(5)" },
    ],
  },
];
