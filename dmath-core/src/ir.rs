//! Visualization Intermediate Representation (Visualization IR).
//!
//! This is the stable contract between DMath Core and any frontend: the
//! engine never produces HTML/CSS/SVG itself, only these structured values
//! (serde -> JSON). The `type` discriminator tells the frontend which
//! renderer to use.

use std::collections::BTreeMap;

use serde::Serialize;

pub const CONTRACT_VERSION: &str = "dmath.viz.v1";

#[derive(Debug, Clone, Serialize)]
pub struct Envelope {
    pub contract: &'static str,
    pub ok: bool,
    pub outputs: Vec<OutputEntry>,
    pub visualizations: Vec<VizIr>,
    pub trace: Vec<TraceStep>,
    pub diagnostics: Vec<DiagJson>,
}

impl Envelope {
    pub fn empty() -> Envelope {
        Envelope {
            contract: CONTRACT_VERSION,
            ok: true,
            outputs: Vec::new(),
            visualizations: Vec::new(),
            trace: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputEntry {
    pub kind: &'static str, // "value"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub expression: String,
    pub value_type: &'static str,
    pub display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elements: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum VizIr {
    #[serde(rename = "truth_table")]
    TruthTable(TruthTableIr),
    #[serde(rename = "venn")]
    Venn(VennIr),
    #[serde(rename = "equivalence_check")]
    Equivalence(EquivalenceIr),
    #[serde(rename = "digraph")]
    Digraph(DigraphIr),
    #[serde(rename = "hasse")]
    Hasse(HasseIr),
    #[serde(rename = "relation_matrix")]
    RelationMatrix(RelationMatrixIr),
    #[serde(rename = "relation_properties")]
    RelationProperties(RelationPropertiesIr),
    #[serde(rename = "graph_properties")]
    GraphProperties(GraphPropertiesIr),
    #[serde(rename = "graph_view")]
    GraphView(GraphViewIr),
    #[serde(rename = "sieve_grid")]
    Sieve(SieveIr),
    #[serde(rename = "pascal")]
    Pascal(PascalIr),
    #[serde(rename = "steps")]
    Steps(StepsIr),
}

#[derive(Debug, Clone, Serialize)]
pub struct TruthTableIr {
    pub expression: String,
    pub variables: Vec<String>,
    pub rows: Vec<BTreeMap<String, serde_json::Value>>,
    pub classification: &'static str, // "tautology" | "contradiction" | "contingency"
    pub classification_note: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct VennIr {
    /// "union" | "intersection" | "difference" | "cross"? (no) | "complement" | "display"
    pub operation: String,
    pub operation_label: String,
    pub labels: Vec<String>,
    pub operands: BTreeMap<String, Vec<serde_json::Value>>,
    pub result: Vec<serde_json::Value>,
    pub result_display: String,
    pub universe: Option<Vec<serde_json::Value>>,
    pub regions: VennRegions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VennRegions {
    #[serde(rename = "leftOnly")]
    pub left_only: Vec<serde_json::Value>,
    #[serde(rename = "intersection")]
    pub intersection: Vec<serde_json::Value>,
    #[serde(rename = "rightOnly")]
    pub right_only: Vec<serde_json::Value>,
    #[serde(rename = "outside")]
    pub outside: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EquivalenceIr {
    pub left: String,
    pub right: String,
    pub variables: Vec<String>,
    pub rows: Vec<BTreeMap<String, serde_json::Value>>,
    pub equivalent: bool,
}

/// `VISUALIZE R AS DIGRAPH` on a relation: sorted field vertices and sorted
/// `[from, to]` pairs (self-loops included).
#[derive(Debug, Clone, Serialize)]
pub struct DigraphIr {
    pub expression: String,
    pub vertices: Vec<i64>,
    pub edges: Vec<(i64, i64)>,
}

/// `VISUALIZE R AS HASSE` on a partial order: bottom-first level layers and
/// the cover edges.
#[derive(Debug, Clone, Serialize)]
pub struct HasseIr {
    pub expression: String,
    pub levels: Vec<Vec<i64>>,
    pub edges: Vec<(i64, i64)>,
}

/// `VISUALIZE R AS MATRIX` on a relation: `matrix[i][j] = 1` iff
/// `(vertices[i], vertices[j])` is in the relation.
#[derive(Debug, Clone, Serialize)]
pub struct RelationMatrixIr {
    pub expression: String,
    pub vertices: Vec<i64>,
    pub matrix: Vec<Vec<u8>>,
}

/// `CHECK PROPERTIES R` on a relation.
#[derive(Debug, Clone, Serialize)]
pub struct RelationPropertiesIr {
    pub expression: String,
    /// Sorted field of the relation.
    pub base_set: Vec<i64>,
    pub reflexive: bool,
    pub symmetric: bool,
    pub antisymmetric: bool,
    pub transitive: bool,
    pub is_equivalence: bool,
    pub is_partial_order: bool,
    /// Human-readable counterexample per failed property; always present
    /// (empty object when everything holds).
    pub counterexamples: BTreeMap<String, String>,
    /// Equivalence classes, only when `is_equivalence` (else null).
    pub classes: Option<Vec<Vec<i64>>>,
}

/// `CHECK PROPERTIES G` on a graph.
#[derive(Debug, Clone, Serialize)]
pub struct GraphPropertiesIr {
    pub expression: String,
    pub vertex_count: usize,
    pub edge_count: usize,
    pub connected: bool,
    pub tree: bool,
    pub bipartite: bool,
    pub complete: bool,
    /// Vertex -> degree (JSON keys become strings).
    pub degrees: BTreeMap<i64, i64>,
    pub notes: Vec<String>,
}

/// One edge of a graph view; `weight` / `highlight` are omitted when absent.
#[derive(Debug, Clone, Serialize)]
pub struct GraphEdgeView {
    pub from: i64,
    pub to: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<&'static str>,
}

/// `VISUALIZE G AS GRAPH` (and the automatic view emitted when DISPLAYing a
/// BFS/DFS/SHORTEST_PATH call).
#[derive(Debug, Clone, Serialize)]
pub struct GraphViewIr {
    pub expression: String,
    pub directed: bool,
    pub vertices: Vec<i64>,
    /// Edges in the graph's own edge order.
    pub edges: Vec<GraphEdgeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_path: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_visited: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colors: Option<BTreeMap<i64, i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_weight: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// `VISUALIZE S AS SIEVE` on the result of `SIEVE(n)`: the prime/composite
/// grid from 1 (or 0) to `limit`, plus the smallest prime factor of every
/// composite — the sieve "crosses out" each composite through it.
#[derive(Debug, Clone, Serialize)]
pub struct SieveIr {
    pub expression: String,
    /// The sieve limit `n` (grid shows 0..=limit).
    pub limit: i64,
    /// All primes ≤ `limit`, ascending.
    pub primes: Vec<i64>,
    /// Composite -> smallest prime factor (JSON keys become strings).
    pub smallest_factor: BTreeMap<i64, i64>,
    pub prime_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// `VISUALIZE P AS PASCAL` on the result of `PASCAL(n)`: the first `n`
/// triangle rows, row `i` holding `C(i, 0) … C(i, i)`. When the visualized
/// value came from `COMB(n, k)` the corresponding cell is highlighted.
#[derive(Debug, Clone, Serialize)]
pub struct PascalIr {
    pub expression: String,
    /// Triangle rows, `rows[i][j] = C(i, j)`.
    pub rows: Vec<Vec<i64>>,
    pub row_count: usize,
    /// Optional highlighted cell `(row, col)` for a `COMB(n, k)` origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight: Option<(i64, i64)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// `VISUALIZE G AS STEPS` on a number-theory / combinatorics result: a
/// worked solution. Several independent methods each compute the answer
/// (e.g. GCD by quick prime check, Euclid, factorization, listing divisors);
/// `agrees` records that every method reached the engine's result.
#[derive(Debug, Clone, Serialize)]
pub struct StepsIr {
    pub expression: String,
    /// Function name, e.g. "GCD".
    pub operation: String,
    pub title: String,
    /// The final answer, formatted like `DISPLAY` would.
    pub result: String,
    pub sections: Vec<StepSection>,
    /// True when every section that computed its own answer matches `result`.
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepSection {
    /// Renderer hint: "check" | "euclid" | "factorization" | "listing" | "formula" |
    /// "trial" | "witness" | "rounds" | "multiply" | "verify" | "note".
    pub kind: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intro: Option<String>,
    pub steps: Vec<Step>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<StepTable>,
    /// One-line conclusion of this method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// Set when this method computed its own answer: does it equal `result`?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agrees: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub text: String,
    /// "plain" | "head" | "good" | "bad"
    pub tone: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct StepTable {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceStep {
    pub step: usize,
    pub statement: String,
    pub kind: &'static str, // "assignment" | "display" | "visualize" | "check_equivalent" | "check_properties"
    pub status: &'static str, // "ok" | "error"
    pub summary: String,
    pub intermediates: Vec<Intermediate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Intermediate {
    pub expr: String,
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagJson {
    pub severity: crate::token::Severity,
    pub message: String,
    pub line: usize,
    pub col: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl From<crate::token::Diagnostic> for DiagJson {
    fn from(d: crate::token::Diagnostic) -> Self {
        DiagJson { severity: d.severity, message: d.message, line: d.line, col: d.col, hint: d.hint }
    }
}
