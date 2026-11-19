//! Runtime values and the mathematical set element type.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

/// An element of a set: either a number or an ordered tuple produced by a
/// Cartesian product or a set literal. Ordering is total, so sets can live in
/// a `BTreeSet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetElem {
    Num(i64),
    Tuple(Vec<SetElem>),
    /// A set used as an element (for example a member of a power set). The
    /// elements are kept sorted and unique so equal sets compare equal.
    Set(Vec<SetElem>),
}

impl SetElem {
    fn rank(&self) -> u8 {
        match self {
            SetElem::Num(_) => 0,
            SetElem::Tuple(_) => 1,
            SetElem::Set(_) => 2,
        }
    }
}

impl PartialOrd for SetElem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Numbers by value, tuples lexicographically, and nested sets by size first
/// and then lexicographically, so a power set reads {}, {1}, {2}, {1, 2}, ...
impl Ord for SetElem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (SetElem::Num(a), SetElem::Num(b)) => a.cmp(b),
            (SetElem::Tuple(a), SetElem::Tuple(b)) => a.cmp(b),
            (SetElem::Set(a), SetElem::Set(b)) => {
                a.len().cmp(&b.len()).then_with(|| a.cmp(b))
            }
            _ => match self.rank().cmp(&other.rank()) {
                Ordering::Equal => Ordering::Equal,
                o => o,
            },
        }
    }
}

impl std::fmt::Display for SetElem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetElem::Num(n) => write!(f, "{}", n),
            SetElem::Tuple(items) => {
                let inner: Vec<String> = items.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", inner.join(", "))
            }
            SetElem::Set(items) => {
                let inner: Vec<String> = items.iter().map(|e| e.to_string()).collect();
                write!(f, "{{{}}}", inner.join(", "))
            }
        }
    }
}

impl SetElem {
    /// JSON encoding used by the Visualization IR: numbers stay numbers,
    /// tuples become arrays.
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            SetElem::Num(n) => json!(n),
            SetElem::Tuple(items) => {
                serde_json::Value::Array(items.iter().map(|e| e.to_json()).collect())
            }
            // Nested sets travel as their text, e.g. "{1, 2}", so the interface
            // never mistakes them for tuples (which are JSON arrays).
            SetElem::Set(_) => json!(self.to_string()),
        }
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, SetElem::Tuple(_))
    }
}

/// Convert a set of elements to the JSON array form.
pub fn elems_to_json<'a>(elems: impl IntoIterator<Item = &'a SetElem>) -> Vec<serde_json::Value> {
    elems.into_iter().map(|e| e.to_json()).collect()
}

/// Render a set in canonical mathematical notation: `{1, 2, 3}`.
pub fn set_to_string(s: &BTreeSet<SetElem>) -> String {
    if s.is_empty() {
        return "{}".to_string();
    }
    let inner: Vec<String> = s.iter().map(|e| e.to_string()).collect();
    format!("{{{}}}", inner.join(", "))
}

/// Render a plain set of integers (`{1, 2, 3}`); used in hints.
pub fn int_set_to_string(s: &BTreeSet<i64>) -> String {
    if s.is_empty() {
        return "{}".to_string();
    }
    let inner: Vec<String> = s.iter().map(|v| v.to_string()).collect();
    format!("{{{}}}", inner.join(", "))
}

/// One edge of a graph. For undirected graphs edges are stored normalized as
/// `(min, max)`, so `(2, 1)` and `(1, 2)` are the same edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub from: i64,
    pub to: i64,
    /// `None` for unweighted edges (they behave as weight 1 where a weight is
    /// needed, e.g. Dijkstra / MST).
    pub weight: Option<i64>,
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.weight {
            Some(w) => write!(f, "({}, {}, {})", self.from, self.to, w),
            None => write!(f, "({}, {})", self.from, self.to),
        }
    }
}

/// Where a graph value came from. MST results remember their total weight so
/// the graph view can highlight the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphOrigin {
    Literal,
    Mst { total_weight: i64 },
}

/// A graph value: directed or undirected, with a vertex set and an ordered
/// edge list (deduplicated, undirected edges normalized to `(min, max)`).
#[derive(Debug, Clone, PartialEq)]
pub struct GraphVal {
    pub directed: bool,
    pub vertices: BTreeSet<i64>,
    pub edges: Vec<Edge>,
    pub origin: Option<GraphOrigin>,
}

impl GraphVal {
    /// Build a graph from raw literal data: deduplicates edges (for
    /// undirected graphs on the normalized `(min, max)` key, keeping the
    /// first weight seen) and tags the result as user-written.
    pub fn build(directed: bool, vertices: BTreeSet<i64>, edges: Vec<Edge>) -> GraphVal {
        let mut seen: BTreeSet<(i64, i64)> = BTreeSet::new();
        let mut out: Vec<Edge> = Vec::new();
        for e in edges {
            let (f, t) = if directed { (e.from, e.to) } else { (e.from.min(e.to), e.from.max(e.to)) };
            if seen.insert((f, t)) {
                out.push(Edge { from: f, to: t, weight: e.weight });
            }
        }
        GraphVal { directed, vertices, edges: out, origin: Some(GraphOrigin::Literal) }
    }
}

/// Render a single edge the way set elements render: `(1, 2)` or `(1, 2, 7)`.
pub fn edge_to_elem(e: &Edge) -> SetElem {
    SetElem::Tuple(vec![SetElem::Num(e.from), SetElem::Num(e.to)])
}

/// A runtime value of the DSL.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(i64),
    Set(BTreeSet<SetElem>),
    Graph(GraphVal),
    /// Ordered traversal result (BFS, DFS, shortest paths).
    List(Vec<SetElem>),
    /// Vertex -> color-index assignment produced by COLOR.
    Mapping(BTreeMap<i64, i64>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "Boolean",
            Value::Number(_) => "Number",
            Value::Set(_) => "Set",
            Value::Graph(_) => "Graph",
            Value::List(_) => "List",
            Value::Mapping(_) => "Mapping",
        }
    }

    pub fn display(&self) -> String {
        match self {
            Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Set(s) => set_to_string(s),
            Value::Graph(g) => graph_to_string(g),
            Value::List(items) => {
                let inner: Vec<String> = items.iter().map(|e| e.to_string()).collect();
                format!("[{}]", inner.join(", "))
            }
            Value::Mapping(m) => {
                if m.is_empty() {
                    return "{}".to_string();
                }
                let inner: Vec<String> =
                    m.iter().map(|(k, v)| format!("{} -> {}", k, v)).collect();
                format!("{{{}}}", inner.join(", "))
            }
        }
    }
}

/// Canonical display of a graph value, e.g.
/// `Graph(V = {1, 2, 3}, E = {(1, 2), (2, 3)})`. Weighted edges render as
/// `(1, 2, 7)`; directed graphs use the word `Digraph`; MST results append
/// `, total weight = N` inside the parentheses.
pub fn graph_to_string(g: &GraphVal) -> String {
    let v_inner: Vec<String> = g.vertices.iter().map(|v| v.to_string()).collect();
    let e_inner: Vec<String> = g.edges.iter().map(|e| e.to_string()).collect();
    let mut s = format!(
        "{}(V = {{{}}}, E = {{{}}}",
        if g.directed { "Digraph" } else { "Graph" },
        v_inner.join(", "),
        e_inner.join(", ")
    );
    if let Some(GraphOrigin::Mst { total_weight }) = &g.origin {
        s.push_str(&format!(", total weight = {}", total_weight));
    }
    s.push(')');
    s
}

/// Static types used by the semantic analyzer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    Bool,
    Number,
    Set,
    Graph,
    List,
    Mapping,
}

impl DType {
    pub fn name(&self) -> &'static str {
        match self {
            DType::Bool => "Boolean",
            DType::Number => "Number",
            DType::Set => "Set",
            DType::Graph => "Graph",
            DType::List => "List",
            DType::Mapping => "Mapping",
        }
    }
}
