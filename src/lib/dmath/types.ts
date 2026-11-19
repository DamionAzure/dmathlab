// TypeScript mirror of the DMath Core Visualization IR (contract: dmath.viz.v1).
// These interfaces match the serde output of the Rust engine exactly.

export type Severity = "info" | "warning" | "error";

export interface DiagnosticJson {
  severity: Severity;
  message: string;
  line: number;
  col: number;
  hint?: string;
}

/** Set elements: numbers, or tuples produced by CROSS (nested arrays). */
// A number, a tuple (array), or a nested set (sent as text such as "{1, 2}").
export type Elem = number | string | Elem[];

export interface OutputEntry {
  kind: "value";
  name?: string;
  expression: string;
  value_type: "Boolean" | "Number" | "Set" | "Graph" | "List" | "Mapping";
  display: string;
  elements?: Elem[];
}

export interface TruthTableIr {
  type: "truth_table";
  expression: string;
  variables: string[];
  rows: Record<string, boolean>[];
  classification: "tautology" | "contradiction" | "contingency";
  classification_note: string;
  row_count: number;
}

export interface VennRegions {
  leftOnly: Elem[];
  intersection: Elem[];
  rightOnly: Elem[];
  outside: Elem[];
}

export interface VennIr {
  type: "venn";
  operation: string;
  operation_label: string;
  labels: string[];
  operands: Record<string, Elem[]>;
  result: Elem[];
  result_display: string;
  universe?: Elem[];
  regions: VennRegions;
  note?: string;
}

export interface EquivalenceIr {
  type: "equivalence_check";
  left: string;
  right: string;
  variables: string[];
  rows: Record<string, boolean>[];
  equivalent: boolean;
}

// ---- Relations (additive, v1.1) -------------------------------------------

export interface DigraphIr {
  type: "digraph";
  expression: string;
  vertices: number[];
  edges: [number, number][];
}

export interface HasseIr {
  type: "hasse";
  expression: string;
  levels: number[][];
  edges: [number, number][];
}

export interface RelationMatrixIr {
  type: "relation_matrix";
  expression: string;
  vertices: number[];
  matrix: number[][];
}

export interface RelationPropertiesIr {
  type: "relation_properties";
  expression: string;
  base_set: number[];
  reflexive: boolean;
  symmetric: boolean;
  antisymmetric: boolean;
  transitive: boolean;
  is_equivalence: boolean;
  is_partial_order: boolean;
  counterexamples: Record<string, string>; // keys among reflexive/symmetric/antisymmetric/transitive
  classes: number[][] | null; // only when is_equivalence
}

// ---- Graph theory (additive, v1.1) ----------------------------------------

export interface GraphPropertiesIr {
  type: "graph_properties";
  expression: string;
  vertex_count: number;
  edge_count: number;
  connected: boolean;
  tree: boolean;
  bipartite: boolean;
  complete: boolean;
  degrees: Record<string, number>;
  notes: string[];
}

export interface GraphViewEdge {
  from: number;
  to: number;
  weight?: number;
  highlight?: "path" | "tree";
}

export interface GraphViewIr {
  type: "graph_view";
  expression: string;
  directed: boolean;
  vertices: number[];
  edges: GraphViewEdge[];
  highlight_path?: number[];
  highlight_visited?: number[];
  colors?: Record<string, number>;
  total_weight?: number;
  note?: string;
}

// ---- Number theory (additive, v1.2) ---------------------------------------

export interface SieveIr {
  type: "sieve_grid";
  expression: string;
  limit: number;
  primes: number[];
  smallest_factor: Record<string, number>; // composite -> smallest prime factor
  prime_count: number;
  note?: string;
}

// ---- Combinatorics (additive, v1.3) ---------------------------------------

export interface PascalIr {
  type: "pascal";
  expression: string;
  rows: number[][]; // rows[i][j] = C(i, j)
  row_count: number;
  highlight?: [number, number]; // (row, col) of a COMB(n, k) origin
  note?: string;
}

export interface StepsTable {
  headers: string[];
  rows: string[][];
}

export interface StepLine {
  text: string;
  tone: "plain" | "head" | "good" | "bad";
}

export interface StepSection {
  kind: string;
  title: string;
  intro?: string;
  steps: StepLine[];
  table?: StepsTable;
  outcome?: string;
  /** Set when this method computed its own answer: does it match the result? */
  agrees?: boolean;
}

/** `VISUALIZE G AS STEPS`: a worked, multi-method solution. */
export interface StepsIr {
  type: "steps";
  expression: string;
  operation: string;
  title: string;
  result: string;
  sections: StepSection[];
  verified: boolean;
}

export type VizIr =
  | TruthTableIr
  | VennIr
  | EquivalenceIr
  | DigraphIr
  | HasseIr
  | RelationMatrixIr
  | RelationPropertiesIr
  | GraphPropertiesIr
  | GraphViewIr
  | SieveIr
  | PascalIr
  | StepsIr;

/** Type guard: property-check IR is rendered as a rich card in the Output panel, not the Visualization tab. */
export function isPropertyCheckIr(
  v: VizIr,
): v is RelationPropertiesIr | GraphPropertiesIr {
  return v.type === "relation_properties" || v.type === "graph_properties";
}

export interface Intermediate {
  expr: string;
  result: string;
  detail?: string;
}

export interface TraceStep {
  step: number;
  statement: string;
  kind:
    | "assignment"
    | "display"
    | "visualize"
    | "check_equivalent"
    | "check_properties";
  status: "ok" | "error";
  summary: string;
  intermediates: Intermediate[];
}

export interface DMathEnvelope {
  contract: string;
  ok: boolean;
  outputs: OutputEntry[];
  visualizations: VizIr[];
  trace: TraceStep[];
  diagnostics: DiagnosticJson[];
}
