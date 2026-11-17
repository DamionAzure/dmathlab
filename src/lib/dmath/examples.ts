// Example DMath programs, including the exact examples from the project spec.

export interface DMathExample {
  id: string;
  group:
    | "Propositional Logic"
    | "Set Theory"
    | "Type & Error Handling"
    | "Relations"
    | "Graph Theory"
    | "Number Theory"
    | "Combinatorics";
  name: string;
  description: string;
  code: string;
}

export const EXAMPLES: DMathExample[] = [
  {
    id: "sets-basics",
    group: "Set Theory",
    name: "Set Operations — spec §2.2",
    description: "Union, intersection, difference with a Venn diagram",
    code: `# Spec section 2.2 — Set Theory Workbench
A = {1, 2, 3, 4}
B = {3, 4, 5, 6}

C = A UNION B
D = A INTERSECT B
E = A DIFFERENCE B

DISPLAY C
DISPLAY D
DISPLAY E

VISUALIZE C AS VENN`,
  },
  {
    id: "logic-truth-table",
    group: "Propositional Logic",
    name: "Truth Table — spec §2.1",
    description: "P AND Q with full truth table and classification",
    code: `# Spec section 2.1 — Propositional Logic Workbench
P = TRUE
Q = FALSE

RESULT = P AND Q

DISPLAY RESULT
VISUALIZE RESULT AS TRUTH_TABLE`,
  },
  {
    id: "logic-tautology",
    group: "Propositional Logic",
    name: "Tautology vs Contradiction",
    description: "Law of excluded middle and its negation",
    code: `# Two classic classifications
P = TRUE
Q = TRUE

TAUTOLOGY = P OR NOT P
CONTRADICTION = P AND NOT P

DISPLAY TAUTOLOGY
DISPLAY CONTRADICTION

VISUALIZE TAUTOLOGY AS TRUTH_TABLE
VISUALIZE CONTRADICTION AS TRUTH_TABLE`,
  },
  {
    id: "logic-iff",
    group: "Propositional Logic",
    name: "Biconditional Identity",
    description: "P IFF Q equals (P IMPLIES Q) AND (Q IMPLIES P)",
    code: `# The biconditional is exactly mutual implication
P = TRUE
Q = FALSE

R = (P IFF Q) IFF ((P IMPLIES Q) AND (Q IMPLIES P))

DISPLAY R
VISUALIZE R AS TRUTH_TABLE`,
  },
  {
    id: "logic-equiv",
    group: "Propositional Logic",
    name: "Logical Equivalence — spec §3.3",
    description: "CHECK EQUIVALENT P IMPLIES Q, NOT P OR Q",
    code: `# Material implication, verified symbolically
P = TRUE
Q = FALSE

A = P IMPLIES Q
B = NOT P OR Q

DISPLAY A
DISPLAY B

CHECK EQUIVALENT A, B`,
  },
  {
    id: "sets-nested",
    group: "Set Theory",
    name: "Nested Set Expression — spec §3.3",
    description: "Composed operations with intermediate steps",
    code: `# Composed set expression, evaluated step by step
A = {1, 2, 3, 4}
B = {3, 4, 5, 6}

RESULT = (A UNION B) INTERSECT {3, 4, 5}

DISPLAY RESULT
VISUALIZE RESULT AS VENN`,
  },
  {
    id: "sets-complement",
    group: "Set Theory",
    name: "Complement & Universal Set",
    description: "COMPLEMENT relative to a declared UNIVERSE",
    code: `# Complement is taken relative to the universal set
UNIVERSE = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}
A = {2, 4, 6, 8}

C = COMPLEMENT A

DISPLAY C
VISUALIZE C AS VENN`,
  },
  {
    id: "sets-cartesian",
    group: "Set Theory",
    name: "Cartesian Product, Subset & Equality",
    description: "CROSS produces ordered pairs; SUBSET / EQUALS checks",
    code: `# Cartesian product yields ordered pairs
A = {1, 2}
B = {3, 4}
P = A CROSS B
DISPLAY P

# Subset and equality are Boolean-valued
X = {1, 2, 3}
Y = {1, 2, 3, 4}
S = X SUBSET Y
Q = X EQUALS Y
DISPLAY S
DISPLAY Q`,
  },
  {
    id: "mixed",
    group: "Set Theory",
    name: "Sets inside Logic",
    description: "Set comparisons feed propositional connectives",
    code: `# Set comparisons produce Booleans usable in logic
A = {1, 2}
B = {1, 2, 3}
C = {3, 4, 5}

R = (A SUBSET B) AND NOT (A SUBSET C)

DISPLAY R
VISUALIZE R AS TRUTH_TABLE`,
  },
  {
    id: "number-theory",
    group: "Number Theory",
    name: "Number Theory — primes & factorization",
    description: "Miller–Rabin, prime factors, divisors, GCD / LCM, totient",
    code: `# Number theory: primality, factorization, arithmetic functions.
P1 = IS_PRIME(97)
P2 = IS_PRIME(1)
DISPLAY P1
DISPLAY P2

# 360 = 2^3 · 3^2 · 5 (trial division + Pollard rho inside)
F = FACTORS(360)
DISPLAY F

D = DIVISORS(12)
DISPLAY D

G = GCD(48, 18)
L = LCM(4, 6)
DISPLAY G
DISPLAY L

# Euler's totient: 36 = 2^2 · 3^2 -> φ(36) = 36 · (1/2) · (2/3) = 12
T = TOTIENT(36)
DISPLAY T

# The classic sieve, visualized: composites crossed out by their
# smallest prime factor, primes left standing.
S = SIEVE(50)
DISPLAY S
VISUALIZE S AS SIEVE`,
  },
  {
    id: "number-theory-steps",
    group: "Number Theory",
    name: "Number Theory — step by step",
    description: "Worked solutions: GCD, LCM, primality, totient, factors, divisors, sieve",
    code: `# VISUALIZE ... AS STEPS shows the working, using several methods
# that must all agree.

# GCD: prime check -> Euclidean algorithm -> prime factors -> divisor lists
G = GCD(48, 18)
VISUALIZE G AS STEPS

# A prime against a number it does not divide: GCD is 1 straight away
H = GCD(17, 52)
VISUALIZE H AS STEPS

L = LCM(12, 18)
VISUALIZE L AS STEPS

P = IS_PRIME(97)
VISUALIZE P AS STEPS

T = TOTIENT(36)
VISUALIZE T AS STEPS

F = FACTORS(360)
VISUALIZE F AS STEPS`,
  },
  {
    id: "combinatorics",
    group: "Combinatorics",
    name: "Combinatorics — counting & Pascal",
    description: "Factorial, permutations, combinations, Pascal's triangle",
    code: `# Counting: factorial, arrangements, choices.
A = FACT(5)
DISPLAY A

# License plates: 26 · 25 · 24 arrangements
B = PERM(26, 3)
DISPLAY B

# Poker hands: C(52, 5) = 2 598 960
C = COMB(52, 5)
DISPLAY C

# Pascal's triangle — each entry sums the two above it.
P = PASCAL(8)
DISPLAY P
VISUALIZE P AS PASCAL

# A combination is a cell in the triangle: C(8, 3) is highlighted.
V = COMB(8, 3)
DISPLAY V
VISUALIZE V AS PASCAL`,
  },
  {
    id: "errors",
    group: "Type & Error Handling",
    name: "Type & Syntax Errors — spec §3.4",
    description: "Type safety, undefined variables, error recovery",
    code: `# Type error: sets do not support AND (spec section 3.4)
A = {1, 2, 3}
P = TRUE

RESULT = A AND P
DISPLAY RESULT

# Undefined variable: TYP0 was never declared
X = TYP0 OR TRUE

# The program keeps running after errors
B = {2, 3}
C = A INTERSECT B
DISPLAY C`,
  },
  {
    id: "relations-basics",
    group: "Relations",
    name: "Relations — properties & closures",
    description: "CHECK PROPERTIES, Warshall closure, matrix & Hasse views",
    code: `# Relations — a relation is a set of ordered pairs.
R = {(1,1), (1,2), (2,2), (2,3), (3,3), (3,4), (4,4)}
CHECK PROPERTIES R
VISUALIZE R AS DIGRAPH

# Transitive closure (Warshall) completes the order.
TC = TRANSITIVE_CLOSURE(R)
DISPLAY TC
VISUALIZE TC AS MATRIX
CHECK PROPERTIES TC

# TC is now a partial order: draw its Hasse diagram.
VISUALIZE TC AS HASSE`,
  },
  {
    id: "relations-equivalence",
    group: "Relations",
    name: "Relations — equivalence classes",
    description: "Congruence mod 3, equivalence classes, symmetric closure",
    code: `# "Same remainder mod 3" on {0, 1, 2, 3, 4, 5}
M = {(0,0), (0,3), (3,0), (3,3),
     (1,1), (1,4), (4,1), (4,4),
     (2,2), (2,5), (5,2), (5,5)}
CHECK PROPERTIES M
VISUALIZE M AS DIGRAPH

# Closures can repair a relation step by step.
RT = {(1,2), (2,2), (3,3)}
S = SYMMETRIC_CLOSURE(RT)
DISPLAY S
CHECK PROPERTIES S`,
  },
  {
    id: "graphs-basics",
    group: "Graph Theory",
    name: "Graphs — traversal & structure",
    description: "BFS / DFS, shortest path, degree, connectivity checks",
    code: `# Undirected graph: traversal, paths, structure checks.
G = GRAPH {1,2,3,4,5,6} {(1,2), (1,3), (2,4), (3,4), (4,5), (5,6)}
VISUALIZE G AS GRAPH

DISPLAY DEGREE(G, 4)
DISPLAY NEIGHBORS(G, 4)
DISPLAY ADJACENT(G, 2, 4)

ORDER = BFS(G, 1)
DISPLAY ORDER
WALK = DFS(G, 1)
DISPLAY WALK
P = SHORTEST_PATH(G, 1, 6)
DISPLAY P

DISPLAY IS_CONNECTED(G)
DISPLAY IS_TREE(G)
DISPLAY IS_BIPARTITE(G)
CHECK PROPERTIES G`,
  },
  {
    id: "graphs-weighted",
    group: "Graph Theory",
    name: "Graphs — Dijkstra, MST & coloring",
    description: "Weighted shortest path, Kruskal MST, greedy coloring",
    code: `# Weighted graphs: shortest path, minimum spanning tree, coloring.
W = GRAPH {1,2,3,4,5} {(1,2,7), (1,3,3), (3,2,2), (2,4,2), (3,4,8), (4,5,3)}
VISUALIZE W AS GRAPH

P = SHORTEST_PATH(W, 1, 4)
DISPLAY P

T = MST(W)
DISPLAY T
VISUALIZE T AS GRAPH

# Greedy vertex coloring: triangle needs 3, a path needs 2.
K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}
DISPLAY COLOR(K)
PATH4 = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4)}
DISPLAY COLOR(PATH4)`,
  },
  {
    id: "graphs-steps",
    group: "Graph Theory",
    name: "Graphs — step by step",
    description: "Worked solutions: BFS, DFS, Dijkstra, Kruskal & Prim, coloring, bipartite",
    code: `# VISUALIZE ... AS STEPS shows the algorithm round by round.
G = GRAPH {1,2,3,4,5,6} {(1,2), (1,3), (2,4), (3,4), (4,5), (5,6)}
P = BFS(G, 1)
VISUALIZE P AS STEPS

D = DFS(G, 1)
VISUALIZE D AS STEPS

W = GRAPH {1,2,3,4,5} {(1,2,7), (1,3,3), (3,2,2), (2,4,2), (3,4,8), (4,5,3)}
S = SHORTEST_PATH(W, 1, 5)
VISUALIZE S AS STEPS

# Kruskal, cross-checked against Prim
T = MST(W)
VISUALIZE T AS STEPS

C = COLOR(G)
VISUALIZE C AS STEPS

# A triangle is not bipartite: the steps show the odd cycle
K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}
B = IS_BIPARTITE(K)
VISUALIZE B AS STEPS`,
  },
];

/** Display order of example groups in the editor's example picker. */
export const EXAMPLE_GROUPS: DMathExample["group"][] = [
  "Propositional Logic",
  "Set Theory",
  "Relations",
  "Graph Theory",
  "Number Theory",
  "Combinatorics",
  "Type & Error Handling",
];

export const DEFAULT_EXAMPLE = EXAMPLES[0];
