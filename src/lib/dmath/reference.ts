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
    id: "arithmetic",
    title: "Arithmetic, comparisons & ranges",
    blurb:
      "Whole-number arithmetic with the usual precedence: ^ binds tightest (and groups right to left), then * / %, then + -. / is floor division and % takes the sign of the right-hand side, so -7 / 2 is -4 and -7 % 3 is 2. Results must fit in 64 bits.",
    entries: [
      { name: "+ - * / % ^", syntax: "a + b", returns: "Number", summary: "Add, subtract, multiply, floor-divide, remainder and power. Dividing by zero or a negative exponent is an error.", example: "DISPLAY 2 + 3 * 4\nDISPLAY 2 ^ 10\nDISPLAY -7 / 2, -7 % 3" },
      { name: "< > <= >= == !=", syntax: "a < b", returns: "Boolean", summary: "Compare two numbers. == and != also compare two sets. Combine them with AND / OR.", example: "x = 7\nDISPLAY x > 1 AND x < 10\nDISPLAY 2 + 2 == 4" },
      { name: "Range {a..b}", syntax: "{1..10}", returns: "Set", summary: "Every whole number from a to b inside a set literal; mix with other items like {1, 5..8, 20}. Up to 10,000 elements.", example: "S = {1..10}\nDISPLAY SUM(S)\nDISPLAY SIZE({1, 5..8, 20})" },
      { name: "IN", syntax: "x IN A", returns: "Boolean", summary: "Is x an element of A? x is a number, or a set when A holds sets (such as a power set).", example: "DISPLAY 3 IN {1..5}\nDISPLAY {1, 2} IN POWERSET({1, 2, 3})" },
      { name: "PSUBSET", syntax: "A PSUBSET B", returns: "Boolean", summary: "Proper subset: A is inside B and B has something extra.", example: "DISPLAY {1} PSUBSET {1, 2}\nDISPLAY {1, 2} PSUBSET {1, 2}" },
      { name: "MIN", syntax: "MIN(A)", returns: "Number", summary: "The smallest element of a set of numbers (an empty set is an error).", example: "DISPLAY MIN({4, 9, 2})" },
      { name: "MAX", syntax: "MAX(A)", returns: "Number", summary: "The largest element of a set of numbers.", example: "DISPLAY MAX({4, 9, 2})" },
      { name: "SUM", syntax: "SUM(A)", returns: "Number", summary: "Adds up the elements of a set of numbers; the sum of {} is 0.", example: "DISPLAY SUM({1..100})" },
      { name: "IS_EMPTY", syntax: "IS_EMPTY(A)", returns: "Boolean", summary: "TRUE when the set has no elements.", example: "DISPLAY IS_EMPTY({1} INTERSECT {2})" },
    ],
  },
  {
    id: "logic",
    title: "Logic & sets",
    blurb:
      "Operators bind from loosest to tightest: IFF, IMPLIES, OR / NOR, XOR, AND / NAND, NOT, then comparisons (SUBSET, PSUBSET, EQUALS, IN, <, >, <=, >=, ==, !=), then UNION / DIFFERENCE / SYMDIFF, INTERSECT, CROSS, then + -, * / %, unary minus and ^. Use parentheses to be explicit.",
    entries: [
      { name: "AND", syntax: "P AND Q", returns: "Boolean", summary: "TRUE only when both sides are TRUE.", example: "DISPLAY TRUE AND FALSE" },
      { name: "OR", syntax: "P OR Q", returns: "Boolean", summary: "TRUE when at least one side is TRUE.", example: "DISPLAY TRUE OR FALSE" },
      { name: "NOT", syntax: "NOT P", returns: "Boolean", summary: "Flips TRUE and FALSE.", example: "DISPLAY NOT TRUE" },
      { name: "IMPLIES", syntax: "P IMPLIES Q", returns: "Boolean", summary: "FALSE only when P is TRUE and Q is FALSE.", example: "DISPLAY TRUE IMPLIES FALSE" },
      { name: "IFF", syntax: "P IFF Q", returns: "Boolean", summary: "TRUE when both sides have the same value.", example: "DISPLAY TRUE IFF TRUE" },
      { name: "XOR", syntax: "P XOR Q", returns: "Boolean", summary: "Exclusive or: TRUE when exactly one side is TRUE.", example: "DISPLAY TRUE XOR FALSE" },
      { name: "NAND", syntax: "P NAND Q", returns: "Boolean", summary: "NOT (P AND Q): FALSE only when both sides are TRUE.", example: "DISPLAY TRUE NAND TRUE" },
      { name: "NOR", syntax: "P NOR Q", returns: "Boolean", summary: "NOT (P OR Q): TRUE only when both sides are FALSE.", example: "DISPLAY FALSE NOR FALSE" },
      { name: "UNION", syntax: "A UNION B", returns: "Set", summary: "Everything in A or B.", example: "DISPLAY {1,2} UNION {2,3}" },
      { name: "INTERSECT", syntax: "A INTERSECT B", returns: "Set", summary: "Only what is in both.", example: "DISPLAY {1,2} INTERSECT {2,3}" },
      { name: "DIFFERENCE", syntax: "A DIFFERENCE B", returns: "Set", summary: "What is in A but not in B.", example: "DISPLAY {1,2,3} DIFFERENCE {2}" },
      { name: "SYMDIFF", syntax: "A SYMDIFF B", returns: "Set", summary: "Symmetric difference: what is in exactly one of the two sets (the set version of XOR).", example: "DISPLAY {1,2,3} SYMDIFF {3,4}" },
      { name: "COMPLEMENT", syntax: "COMPLEMENT A", returns: "Set", summary: "Everything in UNIVERSE that is not in A. Declare UNIVERSE first.", example: "UNIVERSE = {1,2,3,4}\nDISPLAY COMPLEMENT {1,2}" },
      { name: "CROSS", syntax: "A CROSS B", returns: "Set of pairs", summary: "The Cartesian product: every pair (a, b) with a in A and b in B.", example: "DISPLAY {1,2} CROSS {3,4}" },
      { name: "SUBSET", syntax: "A SUBSET B", returns: "Boolean", summary: "TRUE when every element of A is also in B.", example: "DISPLAY {1,2} SUBSET {1,2,3}" },
      { name: "EQUALS", syntax: "A EQUALS B", returns: "Boolean", summary: "TRUE when the two sets have exactly the same elements.", example: "DISPLAY {1,2} EQUALS {2,1}" },
      { name: "SIZE", syntax: "SIZE(A)", returns: "Number", summary: "The cardinality |A|: how many distinct elements the set has.", example: "DISPLAY SIZE({4,4,7,9})" },
      { name: "POWERSET", syntax: "POWERSET(A)", returns: "Set of sets", summary: "Every subset of A, from {} up to A itself (2^n of them, for up to 10 elements). Subsets are written as nested sets, and you can store the result like any set.", example: "P = POWERSET({1,2,3})\nDISPLAY P\nDISPLAY SIZE(P)\nDISPLAY CONTAINS(P, {1,3})" },
      { name: "CONTAINS", syntax: "CONTAINS(A, x)", returns: "Boolean", summary: "Is x an element of A? x is a number, or a set when A holds sets (such as a power set).", example: "DISPLAY CONTAINS({1,2,3}, 2)" },
      { name: "IS_DISJOINT", syntax: "IS_DISJOINT(A, B)", returns: "Boolean", summary: "TRUE when A and B share no element.", example: "DISPLAY IS_DISJOINT({1,2}, {3,4})" },
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
      { name: "AS MATRIX", syntax: "VISUALIZE R AS MATRIX", summary: "The 0/1 relation matrix. Give it a graph and you get the adjacency matrix instead (1 where an edge joins the row vertex to the column vertex).", example: REL + "\nVISUALIZE R AS MATRIX" },
      { name: "AS MATRIX (graph)", syntax: "VISUALIZE G AS MATRIX", summary: "The adjacency matrix of a graph. Undirected graphs give a symmetric matrix; isolated vertices keep their row and column.", example: G6 + "\nVISUALIZE G AS MATRIX" },
      { name: "AS GRAPH", syntax: "VISUALIZE G AS GRAPH", summary: "Draw a graph, or the result of MST(G) (the chosen edges are highlighted).", example: G6 + "\nVISUALIZE G AS GRAPH" },
      { name: "AS SIEVE", syntax: "VISUALIZE S AS SIEVE", summary: "The Sieve of Eratosthenes as a grid of crossed-out numbers. S must come from SIEVE(n).", example: "S = SIEVE(50)\nVISUALIZE S AS SIEVE" },
      { name: "AS PASCAL", syntax: "VISUALIZE P AS PASCAL", summary: "Pascal's triangle. P comes from PASCAL(n), or from COMB(n, k) to highlight that cell.", example: "P = PASCAL(8)\nVISUALIZE P AS PASCAL" },
      { name: "AS STEPS", syntax: "VISUALIZE X AS STEPS", summary: "A worked, step-by-step solution of a call, solved by several methods that must all agree. Works for GCD, LCM, IS_PRIME, FACTORS, DIVISORS, TOTIENT, SIEVE, FACT, PERM, COMB, EGCD, MODINV, MODPOW, BFS, DFS, SHORTEST_PATH, MST, COLOR, IS_BIPARTITE, IS_CONNECTED, TOPO_SORT, TRANSITIVE_CLOSURE, REFLEXIVE_CLOSURE, SYMMETRIC_CLOSURE, POWERSET, the set operators (UNION, INTERSECT, DIFFERENCE, SYMDIFF) and SIZE of a union (inclusion–exclusion).", example: "G = GCD(48, 18)\nVISUALIZE G AS STEPS" },
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
    id: "functions",
    title: "Functions & equivalence classes",
    blurb: "A function is a relation where every input has exactly one output, written as a set of pairs like F = {(1, 4), (2, 5)}.",
    entries: [
      { name: "DOMAIN", syntax: "DOMAIN(R)", returns: "Set", summary: "Every first component of the pairs.", example: "R = {(1,4), (2,5), (3,5)}\nDISPLAY DOMAIN(R)" },
      { name: "RANGE", syntax: "RANGE(R)", returns: "Set", summary: "Every second component of the pairs (the outputs that are actually used).", example: "R = {(1,4), (2,5), (3,5)}\nDISPLAY RANGE(R)" },
      { name: "IMAGE", syntax: "IMAGE(R, S)", returns: "Set", summary: "Everything the inputs in S map to.", example: "R = {(1,4), (2,5), (3,5)}\nDISPLAY IMAGE(R, {2, 3})" },
      { name: "IS_FUNCTION", syntax: "IS_FUNCTION(R, A)", returns: "Boolean", summary: "TRUE when every element of A has exactly one output and nothing outside A is used.", example: "F = {(1,4), (2,5), (3,6)}\nDISPLAY IS_FUNCTION(F, {1,2,3})" },
      { name: "IS_INJECTIVE", syntax: "IS_INJECTIVE(R)", returns: "Boolean", summary: "One-to-one: a function where no two inputs share an output.", example: "DISPLAY IS_INJECTIVE({(1,4), (2,4)})" },
      { name: "IS_SURJECTIVE", syntax: "IS_SURJECTIVE(R, B)", returns: "Boolean", summary: "Onto: a function whose outputs are exactly the codomain B.", example: "F = {(1,4), (2,5), (3,5)}\nDISPLAY IS_SURJECTIVE(F, {4, 5})" },
      { name: "IS_BIJECTIVE", syntax: "IS_BIJECTIVE(R, B)", returns: "Boolean", summary: "Both one-to-one and onto B.", example: "F = {(1,4), (2,5), (3,6)}\nDISPLAY IS_BIJECTIVE(F, {4, 5, 6})" },
      { name: "CLASSES", syntax: "CLASSES(R)", returns: "Set of sets", summary: "The equivalence classes of an equivalence relation: a partition written as nested sets.", example: "R = {(1,1), (2,2), (3,3), (1,2), (2,1)}\nDISPLAY CLASSES(R)" },
      { name: "CLASS_OF", syntax: "CLASS_OF(R, x)", returns: "Set", summary: "The equivalence class [x]: everything related to x.", example: "R = {(1,1), (2,2), (3,3), (1,2), (2,1)}\nDISPLAY CLASS_OF(R, 2)" },
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
      { name: "TOPO_SORT", syntax: "TOPO_SORT(D)", returns: "List", summary: "A topological order of a directed graph: every arrow points forward. Ties go to the smallest vertex. A cycle gives an error naming the stuck vertices.", example: "D = DIGRAPH {1,2,3,4,5} {(5,2), (5,1), (2,3), (3,4), (1,4)}\nDISPLAY TOPO_SORT(D)" },
      { name: "IS_DAG", syntax: "IS_DAG(D)", returns: "Boolean", summary: "Is a directed graph free of cycles?", example: "D = DIGRAPH {1,2,3,4,5} {(5,2), (5,1), (2,3), (3,4), (1,4)}\nDISPLAY IS_DAG(D)" },
      { name: "COMPONENTS", syntax: "COMPONENTS(G)", returns: "Mapping", summary: "Which connected piece each vertex belongs to, numbered 1, 2, … from the smallest vertex (directions ignored).", example: "G = GRAPH {1,2,3,4,5,6} {(1,2), (2,3), (4,5)}\nDISPLAY COMPONENTS(G)" },
      { name: "IS_EULERIAN", syntax: "IS_EULERIAN(G)", returns: "Boolean", summary: "Can you walk every edge exactly once and end where you began? Needs all edges in one piece and every vertex balanced (even degree, or in = out for a digraph).", example: "G = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}\nDISPLAY IS_EULERIAN(G)" },
      { name: "GRAPH_COMPLEMENT", syntax: "GRAPH_COMPLEMENT(G)", returns: "Graph", summary: "The graph with an edge exactly where G has none (no loops, weights dropped).", example: G6 + "\nH = GRAPH_COMPLEMENT(G)\nDISPLAY EDGES(H)" },
      { name: "DIAMETER", syntax: "DIAMETER(G)", returns: "Number", summary: "The longest shortest path, counted in edges. Undefined (an error) if the graph is not connected.", example: G6 + "\nDISPLAY DIAMETER(G)" },
      { name: "BRIDGES", syntax: "BRIDGES(G)", returns: "Set", summary: "Edges whose removal disconnects part of an undirected graph.", example: "G = GRAPH {1,2,3,4,5} {(1,2), (2,3), (1,3), (3,4), (4,5)}\nDISPLAY BRIDGES(G)" },
      { name: "CUT_VERTICES", syntax: "CUT_VERTICES(G)", returns: "Set", summary: "Vertices whose removal disconnects part of an undirected graph (articulation points).", example: "G = GRAPH {1,2,3,4,5} {(1,2), (2,3), (1,3), (3,4), (4,5)}\nDISPLAY CUT_VERTICES(G)" },
      { name: "IS_HAMILTONIAN", syntax: "IS_HAMILTONIAN(G)", returns: "Boolean", summary: "Is there a cycle through every vertex exactly once? Searches graphs of up to 20 vertices.", example: "G = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4), (4,1), (1,3)}\nDISPLAY IS_HAMILTONIAN(G)" },
      { name: "CHROMATIC_NUMBER", syntax: "CHROMATIC_NUMBER(G)", returns: "Number", summary: "The fewest colors that give every edge two different colors. Exact, unlike COLOR which is greedy.", example: "G = GRAPH {1,2,3,4,5} {(1,2), (2,3), (3,4), (4,5), (5,1)}\nDISPLAY CHROMATIC_NUMBER(G)\nDISPLAY CHROMATIC_NUMBER(GRAPH {1,2,3,4} {(1,2), (2,3), (3,4)})" },
      { name: "DEGREE_SEQUENCE", syntax: "DEGREE_SEQUENCE(G)", returns: "List", summary: "All vertex degrees, largest first.", example: "G = GRAPH {1,2,3,4} {(1,2), (1,3), (1,4), (2,3)}\nDISPLAY DEGREE_SEQUENCE(G)" },
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
    id: "modular",
    title: "Modular arithmetic",
    blurb: "Clock arithmetic: numbers wrap around at m. These are the tools behind RSA, hashing and calendar maths.",
    entries: [
      { name: "MOD", syntax: "MOD(a, m)", returns: "Number", summary: "The remainder of a divided by m, always from 0 to m−1 (so MOD(−7, 5) is 3, not −2).", example: "DISPLAY MOD(-7, 5)" },
      { name: "MODPOW", syntax: "MODPOW(base, exponent, m)", returns: "Number", summary: "base to the power exponent, reduced mod m, by square-and-multiply. Fast even for huge exponents; the exponent must be 0 or more.", example: "A = MODPOW(7, 222, 1000000007)\nDISPLAY A\nVISUALIZE A AS STEPS" },
      { name: "MODINV", syntax: "MODINV(a, m)", returns: "Number", summary: "The number x with a·x ≡ 1 (mod m). It exists only when GCD(a, m) = 1; otherwise you get an error that says so.", example: "A = MODINV(3, 7)\nDISPLAY A\nVISUALIZE A AS STEPS" },
      { name: "EGCD", syntax: "EGCD(a, b)", returns: "List", summary: "Extended Euclid. Gives [g, x, y] where g = GCD(a, b) and a·x + b·y = g (Bézout's identity).", example: "A = EGCD(240, 46)\nDISPLAY A\nVISUALIZE A AS STEPS" },
      { name: "CRT", syntax: "CRT(a1, m1, a2, m2)", returns: "List", summary: "Chinese remainder theorem. Gives [x, M]: x ≡ a1 (mod m1) and x ≡ a2 (mod m2) exactly when x ≡ x (mod M). Moduli that share a factor are fine if the two conditions agree.", example: "DISPLAY CRT(2, 3, 3, 5)" },
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
      { name: "FIB", syntax: "FIB(n)", returns: "Number", summary: "The n-th Fibonacci number with F(0) = 0 and F(1) = 1. n can be at most 92.", example: "DISPLAY FIB(10)" },
      { name: "CATALAN", syntax: "CATALAN(n)", returns: "Number", summary: "The n-th Catalan number C(2n, n) / (n + 1): balanced bracket strings, binary trees, polygon triangulations. n up to 35.", example: "DISPLAY CATALAN(5)" },
      { name: "DERANGE", syntax: "DERANGE(n)", returns: "Number", summary: "Derangements: ways to rearrange n items so that none stays in its original place. n up to 20.", example: "DISPLAY DERANGE(4)" },
      { name: "SIGMA", syntax: "SIGMA(n)", returns: "Number", summary: "σ(n): the sum of all positive divisors of n. A number is perfect when SIGMA(n) is twice n.", example: "DISPLAY SIGMA(12)\nDISPLAY SIGMA(28) == 2 * 28" },
      { name: "ORDER_MOD", syntax: "ORDER_MOD(a, m)", returns: "Number", summary: "The multiplicative order: the least k ≥ 1 with a^k ≡ 1 (mod m). Needs GCD(a, m) = 1.", example: "DISPLAY ORDER_MOD(3, 7)\nDISPLAY MODPOW(3, ORDER_MOD(3, 7), 7)" },
      { name: "TO_BASE", syntax: "TO_BASE(n, base)", returns: "List", summary: "The digits of n in a base from 2 to 36, most significant first (a digit above 9 stays a number, so 255 in base 16 is [15, 15]).", example: "DISPLAY TO_BASE(10, 2)\nDISPLAY TO_BASE(255, 16)" },
      { name: "FROM_BASE", syntax: "FROM_BASE(digits, base)", returns: "Number", summary: "Reads digits in a base. Pass a list from TO_BASE, or a plain number whose decimal digits are the digits (FROM_BASE(1010, 2) is 10).", example: "DISPLAY FROM_BASE(1010, 2)\nDISPLAY FROM_BASE(TO_BASE(255, 16), 16)" },
      { name: "PARTITIONS", syntax: "PARTITIONS(n)", returns: "Number", summary: "p(n): ways to write n as a sum of positive whole numbers, order ignored (4 = 4, 3+1, 2+2, 2+1+1, 1+1+1+1, so p(4) = 5). Works up to n = 405.", example: "DISPLAY PARTITIONS(4)\nDISPLAY PARTITIONS(100)" },
      { name: "BELL", syntax: "BELL(n)", returns: "Number", summary: "Bell number: ways to split n items into non-empty groups. It adds up the STIRLING2 numbers for every group count. Works up to n = 25.", example: "DISPLAY BELL(5)\nDISPLAY BELL(5) == STIRLING2(5,1) + STIRLING2(5,2) + STIRLING2(5,3) + STIRLING2(5,4) + STIRLING2(5,5)" },
      { name: "STIRLING2", syntax: "STIRLING2(n, k)", returns: "Number", summary: "Stirling number of the second kind: ways to split n items into k non-empty groups. n up to 25.", example: "DISPLAY STIRLING2(5, 2)" },
    ],
  },
];
