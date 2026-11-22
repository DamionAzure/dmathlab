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
    id: "logic-gates",
    group: "Propositional Logic",
    name: "XOR, NAND and NOR",
    description: "The gates beyond AND / OR / NOT, plus a De Morgan proof",
    code: `# XOR is "exactly one is TRUE"; NAND and NOR are NOT (AND) and NOT (OR)
P = TRUE
Q = FALSE

DISPLAY P XOR Q
DISPLAY P NAND Q
DISPLAY P NOR Q

GATE = P XOR Q
VISUALIZE GATE AS TRUTH_TABLE

# De Morgan: NAND is the same as NOT P OR NOT Q
CHECK EQUIVALENT P NAND Q, NOT P OR NOT Q`,
  },
  {
    id: "sets-symdiff",
    group: "Set Theory",
    name: "Symmetric Difference and Size",
    description: "SYMDIFF, SIZE, CONTAINS and IS_DISJOINT on two sets",
    code: `# SYMDIFF keeps what is in exactly one of the two sets
A = {1, 2, 3, 4}
B = {3, 4, 5}

C = A SYMDIFF B
DISPLAY C
DISPLAY SIZE(C)
DISPLAY CONTAINS(C, 5)
DISPLAY IS_DISJOINT(A, B)
VISUALIZE C AS VENN`,
  },
  {
    id: "sets-arith",
    group: "Set Theory",
    name: "Arithmetic, Ranges and Comparisons",
    description: "{1..n}, SUM / MIN / MAX, IN, PSUBSET and numeric tests",
    code: `# Ranges build big sets in one go
S = {1..10}
EVENS = {2, 4, 6, 8, 10}

DISPLAY SUM(S), MIN(S), MAX(S)
DISPLAY 4 IN EVENS, 5 IN EVENS
DISPLAY EVENS PSUBSET S

# Arithmetic: ^ first, then * / %, then + -
n = SUM(S)
DISPLAY n / 10, n % 10, 2 ^ 10 - 1
DISPLAY n > 50 AND n <= 55`,
  },
  {
    id: "sets-powerset",
    group: "Set Theory",
    name: "Power Set",
    description: "All subsets of a set, with a 2^n check",
    code: `# The power set holds every subset, from {} up to the set itself
A = {1, 2, 3}
P = POWERSET(A)

DISPLAY P
DISPLAY SIZE(P)

# P is a normal set of sets, so you can ask questions about it
DISPLAY CONTAINS(P, {1, 3})
DISPLAY CONTAINS(P, {4})

# Nested set literals work too
DISPLAY P EQUALS {{}, {1}, {2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}`,
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
    id: "modular-arithmetic",
    group: "Number Theory",
    name: "Modular arithmetic — inverses & CRT",
    description: "MOD, MODPOW, MODINV, EGCD and the Chinese remainder theorem",
    code: `# Clock arithmetic: results wrap around at the modulus.
DISPLAY MOD(-7, 5)

# Fast powers: 7^222 mod a big prime, without ever building 7^222.
DISPLAY MODPOW(7, 222, 1000000007)

# Inverse: 3 · 5 = 15 ≡ 1 (mod 7) — with the full working
I = MODINV(3, 7)
DISPLAY I
VISUALIZE I AS STEPS

# Bezout: 240·(-9) + 46·47 = 2 = GCD(240, 46)
E = EGCD(240, 46)
DISPLAY E
VISUALIZE E AS STEPS

# "x leaves 2 when divided by 3 and 3 when divided by 5"
DISPLAY CRT(2, 3, 3, 5)`,
  },
  {
    id: "counting-sequences",
    group: "Combinatorics",
    name: "Counting — Fibonacci, Catalan, derangements",
    description: "FIB, CATALAN, DERANGE and STIRLING2",
    code: `DISPLAY FIB(10)          # 55
DISPLAY CATALAN(5)       # 42 ways to balance 5 pairs of brackets
DISPLAY DERANGE(4)       # 9 ways to hand back 4 hats, nobody gets their own
DISPLAY STIRLING2(5, 2)  # 15 ways to split 5 people into 2 teams`,
  },
  {
    id: "numbers-bases",
    group: "Number Theory",
    name: "Bases, Divisor Sums and Orders",
    description: "TO_BASE, FROM_BASE, SIGMA, ORDER_MOD",
    code: `# Writing numbers in other bases
DISPLAY TO_BASE(10, 2)
DISPLAY TO_BASE(255, 16)
DISPLAY FROM_BASE(1010, 2)
DISPLAY FROM_BASE(TO_BASE(2026, 8), 8)

# Sum of divisors: perfect numbers satisfy SIGMA(n) = 2n
DISPLAY SIGMA(6) == 12
DISPLAY SIGMA(28) == 56

# How long until powers of 3 wrap around to 1 (mod 7)?
DISPLAY ORDER_MOD(3, 7)
DISPLAY MODPOW(3, ORDER_MOD(3, 7), 7)`,
  },
  {
    id: "counting-partitions",
    group: "Combinatorics",
    name: "Partitions and Bell Numbers",
    description: "PARTITIONS, BELL and inclusion–exclusion",
    code: `DISPLAY PARTITIONS(4)    # 5: 4, 3+1, 2+2, 2+1+1, 1+1+1+1
DISPLAY PARTITIONS(100)
DISPLAY BELL(4)          # 15 ways to split 4 items into groups

# Bell numbers add up Stirling numbers of the second kind
DISPLAY BELL(4) == STIRLING2(4,1) + STIRLING2(4,2) + STIRLING2(4,3) + STIRLING2(4,4)

# Inclusion–exclusion, with a worked solution
A = {1, 2, 3, 4}
B = {3, 4, 5}
C = {4, 5, 6}
N = SIZE(A UNION B UNION C)
DISPLAY N
VISUALIZE N AS STEPS`,
  },
  {
    id: "graphs-structure",
    group: "Graph Theory",
    name: "Graph Structure — bridges, diameter, coloring",
    description: "Complement, diameter, bridges, cut vertices, Hamiltonian, chromatic number and the adjacency matrix",
    code: `# Two triangles joined by a single edge
G = GRAPH {1,2,3,4,5,6} {(1,2), (2,3), (1,3), (3,4), (4,5), (5,6), (4,6)}

DISPLAY DIAMETER(G)
DISPLAY BRIDGES(G)
DISPLAY CUT_VERTICES(G)
DISPLAY CHROMATIC_NUMBER(G)
DISPLAY IS_HAMILTONIAN(G)

H = GRAPH_COMPLEMENT(G)
DISPLAY SIZE(EDGES(H))

VISUALIZE G AS MATRIX`,
  },
  {
    id: "steps-sets-relations",
    group: "Set Theory",
    name: "Worked Solutions — sets, orders and closures",
    description: "AS STEPS for SYMDIFF, POWERSET, TOPO_SORT and a transitive closure",
    code: `A = {1, 2, 3, 4}
B = {3, 4, 5}
C = A SYMDIFF B
VISUALIZE C AS STEPS

P = POWERSET({1, 2, 3})
VISUALIZE P AS STEPS

D = DIGRAPH {1,2,3,4,5} {(1,2), (1,3), (2,4), (3,4), (4,5)}
T = TOPO_SORT(D)
VISUALIZE T AS STEPS

R = {(1,2), (2,3), (3,4)}
TC = TRANSITIVE_CLOSURE(R)
VISUALIZE TC AS STEPS`,
  },
  {
    id: "graphs-dag",
    group: "Graph Theory",
    name: "Graphs — ordering tasks & components",
    description: "TOPO_SORT, IS_DAG, COMPONENTS, IS_EULERIAN",
    code: `# Task 5 must come before 2 and 1, and so on.
D = DIGRAPH {1,2,3,4,5} {(5,2), (5,1), (2,3), (3,4), (1,4)}
DISPLAY IS_DAG(D)
DISPLAY TOPO_SORT(D)

# Which pieces is a network made of?
G = GRAPH {1,2,3,4,5,6} {(1,2), (2,3), (4,5)}
DISPLAY COMPONENTS(G)

# A triangle can be walked edge-by-edge and closed up.
T = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}
DISPLAY IS_EULERIAN(T)
DISPLAY DEGREE_SEQUENCE(T)`,
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
    id: "relations-functions",
    group: "Relations",
    name: "Functions and Equivalence Classes",
    description: "DOMAIN, RANGE, injective / surjective / bijective, CLASSES",
    code: `# A function is a relation with one output per input
F = {(1, 4), (2, 5), (3, 6)}
DISPLAY DOMAIN(F), RANGE(F)
DISPLAY IS_FUNCTION(F, {1, 2, 3})
DISPLAY IS_INJECTIVE(F)
DISPLAY IS_SURJECTIVE(F, {4, 5, 6})
DISPLAY IS_BIJECTIVE(F, {4, 5, 6})

# Two inputs share an output, so G is not one-to-one
G = {(1, 4), (2, 4)}
DISPLAY IS_INJECTIVE(G)

# Equivalence classes split a set into a partition
R = {(1,1), (2,2), (3,3), (1,2), (2,1)}
DISPLAY CLASSES(R)
DISPLAY CLASS_OF(R, 2)`,
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
