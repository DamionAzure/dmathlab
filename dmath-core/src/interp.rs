//! Interpreter: evaluates checked statements and produces outputs,
//! visualizations, execution traces and runtime diagnostics.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::algo;
use crate::ast::{BinOp, Expr, SetItem, Stmt, UnOp, VizKind};
use crate::ir::{
    DiagJson, DigraphIr, Envelope, EquivalenceIr, GraphEdgeView, GraphPropertiesIr, GraphViewIr,
    HasseIr, Intermediate, OutputEntry, PascalIr, RelationMatrixIr, RelationPropertiesIr, SieveIr,
    TraceStep, TruthTableIr, VennIr, VennRegions, VizIr, CONTRACT_VERSION,
};
use crate::pretty::{ellipsize, pretty};
use crate::semantic::{describe_operand, CheckedStmt, SymTable};
use crate::token::{Builtin, Diagnostic, Pos};
use crate::value::{
    edge_to_elem, elems_to_json, int_set_to_string, set_to_string, DType, Edge, GraphOrigin,
    GraphVal, SetElem, Value,
};

/// Cap for elements rendered per Venn region (the full result is still
/// reported in `result` / `result_display`).
const MAX_VENN_ELEMENTS_PER_REGION: usize = 100;
/// Maximum number of intermediate evaluations recorded per statement.
const MAX_INTERMEDIATES: usize = 200;
/// Maximum size of a Cartesian product result.
const MAX_CROSS_PRODUCT: usize = 100_000;

#[derive(Debug, Clone)]
enum Provenance {
    Literal,
    BinaryOp {
        op: BinOp,
        left_desc: String,
        right_desc: String,
        left: BTreeSet<SetElem>,
        right: BTreeSet<SetElem>,
    },
    Complement {
        arg_desc: String,
        arg: BTreeSet<SetElem>,
    },
}

pub struct StmtOutcome {
    pub outputs: Vec<OutputEntry>,
    pub visualizations: Vec<VizIr>,
    pub warnings: Vec<Diagnostic>,
    pub summary: String,
    pub intermediates: Vec<Intermediate>,
}

pub struct Interpreter {
    env: HashMap<String, Value>,
    provenance: HashMap<String, Provenance>,
    /// Source graph of COLOR results, so `VISUALIZE C AS GRAPH` can render
    /// the colored graph (a Mapping alone does not know its graph).
    coloring_graphs: HashMap<String, GraphVal>,
    /// The sieve limit `n` behind each `SIEVE(n)` result, so
    /// `VISUALIZE S AS SIEVE` can rebuild the grid.
    sieve_limits: HashMap<String, i64>,
    /// The row count behind each `PASCAL(n)` result, so
    /// `VISUALIZE P AS PASCAL` can rebuild the triangle.
    pascal_rows: HashMap<String, i64>,
    /// The `(n, k)` behind each `COMB(n, k)` result, so
    /// `VISUALIZE C AS PASCAL` can highlight the C(n, k) cell.
    comb_cells: HashMap<String, (i64, i64)>,
    /// The number-theory / combinatorics call (and its evaluated integer
    /// arguments) behind each variable, so `VISUALIZE G AS STEPS` can show
    /// the worked solution.
    step_calls: HashMap<String, (Builtin, Vec<i64>)>,
    /// Same for graph algorithms (BFS, MST, …): the call plus a snapshot of
    /// the graph it ran on.
    graph_step_calls: HashMap<String, (Builtin, GraphVal, Vec<i64>, String)>,
    /// The BFS / DFS / SHORTEST_PATH calls that produced each variable, so
    /// `DISPLAY P` can emit the highlighted traversal view.
    traversals: HashMap<String, Expr>,
    universe: Option<BTreeSet<SetElem>>,
    intermediates: Vec<Intermediate>,
}

type IResult<T> = Result<T, Diagnostic>;

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            env: HashMap::new(),
            provenance: HashMap::new(),
            coloring_graphs: HashMap::new(),
            sieve_limits: HashMap::new(),
            pascal_rows: HashMap::new(),
            comb_cells: HashMap::new(),
            step_calls: HashMap::new(),
            graph_step_calls: HashMap::new(),
            traversals: HashMap::new(),
            universe: None,
            intermediates: Vec::new(),
        }
    }

    /// Execute one checked statement.
    pub fn exec(&mut self, stmt: &CheckedStmt) -> IResult<StmtOutcome> {
        self.intermediates.clear();
        match stmt {
            CheckedStmt::Assign { name, dtype: _, expr } => self.exec_assign(name, expr),
            CheckedStmt::Display { items } => self.exec_display(items),
            CheckedStmt::Visualize { expr, dtype: _, kind, free_bool_vars } => {
                match kind {
                    VizKind::TruthTable => self.exec_truth_table(expr, free_bool_vars),
                    VizKind::Venn => self.exec_venn(expr),
                    VizKind::Digraph => self.exec_relation_viz(expr, RelationViz::Digraph),
                    VizKind::Hasse => self.exec_relation_viz(expr, RelationViz::Hasse),
                    VizKind::Matrix => self.exec_relation_viz(expr, RelationViz::Matrix),
                    VizKind::Graph => self.exec_graph_view(expr),
                    VizKind::Sieve => self.exec_sieve(expr),
                    VizKind::Pascal => self.exec_pascal(expr),
                    VizKind::Steps => self.exec_steps(expr),
                }
            }
            CheckedStmt::CheckEquivalent { left, right, vars } => {
                self.exec_equivalence(left, right, vars)
            }
            CheckedStmt::CheckProperties { expr } => self.exec_check_properties(expr),
        }
    }

    // ----- assignments -------------------------------------------------

    fn exec_assign(&mut self, name: &str, expr: &Expr) -> IResult<StmtOutcome> {
        let value = self.eval(expr, None)?;

        // Remember how this set was produced so Venn diagrams can show the
        // operands (spec: visualization derived from the actual computation).
        let prov = match expr {
            Expr::Binary { op, lhs, rhs, .. } if op.is_set_op() => {
                let lv = self.eval(lhs, None)?;
                let rv = self.eval(rhs, None)?;
                let (ls, rs) = match (&lv, &rv) {
                    (Value::Set(l), Value::Set(r)) => (l.clone(), r.clone()),
                    _ => unreachable!("semantic analysis guarantees sets here"),
                };
                Provenance::BinaryOp {
                    op: *op,
                    left_desc: pretty(lhs),
                    right_desc: pretty(rhs),
                    left: ls,
                    right: rs,
                }
            }
            Expr::Unary { op: UnOp::Complement, arg, .. } => {
                let lv = self.eval(arg, None)?;
                if let Value::Set(s) = lv {
                    Provenance::Complement { arg_desc: pretty(arg), arg: s }
                } else {
                    Provenance::Literal
                }
            }
            Expr::Var { name: src, .. } => {
                self.provenance.get(src).cloned().unwrap_or(Provenance::Literal)
            }
            _ => Provenance::Literal,
        };

        let summary = format!("{} = {}", name, value.display());

        // Track the universal set.
        if name.eq_ignore_ascii_case("UNIVERSE") {
            match &value {
                Value::Set(s) => self.universe = Some(s.clone()),
                _ => self.universe = None,
            }
        }

        // Remember the source graph of COLOR results so that
        // `VISUALIZE C AS GRAPH` can render the colored graph later, and the
        // traversal calls that produced a variable so `DISPLAY P` can draw
        // the highlighted walk.
        match expr {
            Expr::Call { func: Builtin::Color, args, .. } => {
                if let Some(gexpr) = args.first() {
                    if let Ok(Value::Graph(g)) = self.eval(gexpr, None) {
                        self.coloring_graphs.insert(name.to_string(), g);
                    }
                }
            }
            Expr::Var { name: src, .. } => {
                if let Some(g) = self.coloring_graphs.get(src).cloned() {
                    self.coloring_graphs.insert(name.to_string(), g);
                } else {
                    self.coloring_graphs.remove(name);
                }
            }
            _ => {
                self.coloring_graphs.remove(name);
            }
        }
        match expr {
            Expr::Call { func: Builtin::Bfs | Builtin::Dfs | Builtin::ShortestPath, .. } => {
                self.traversals.insert(name.to_string(), expr.clone());
            }
            Expr::Var { name: src, .. } => {
                match self.traversals.get(src).cloned() {
                    Some(call) => {
                        self.traversals.insert(name.to_string(), call);
                    }
                    None => {
                        self.traversals.remove(name);
                    }
                }
            }
            _ => {
                self.traversals.remove(name);
            }
        }

        // Track the provenance of SIEVE / PASCAL / COMB results so the
        // corresponding VISUALIZE ... AS SIEVE / PASCAL views can be rebuilt.
        self.track_number_provenance(name, expr);

        self.provenance.insert(name.to_string(), prov);
        self.env.insert(name.to_string(), value);

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: Vec::new(),
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    // ----- number-theory provenance ---------------------------------------

    /// Remember where SIEVE / PASCAL / COMB results came from, with the same
    /// variable-to-variable propagation the traversal views use. Direct call
    /// expressions are handled inside `exec_sieve` / `exec_pascal`; this map
    /// covers `S = SIEVE(30)` then `VISUALIZE S AS SIEVE`.
    fn track_number_provenance(&mut self, name: &str, expr: &Expr) {
        self.track_step_call(name, expr);
        match expr {
            Expr::Call { func: Builtin::Sieve, args, .. } => {
                self.comb_cells.remove(name);
                self.pascal_rows.remove(name);
                if let Some(Expr::Num { v, .. }) = args.first() {
                    self.sieve_limits.insert(name.to_string(), *v);
                } else if let Some(first) = args.first() {
                    // Non-literal argument: evaluate a copy to remember n.
                    if let Ok(Value::Number(n)) = self.eval(first, None) {
                        self.sieve_limits.insert(name.to_string(), n);
                    }
                }
            }
            Expr::Call { func: Builtin::Pascal, args, .. } => {
                self.sieve_limits.remove(name);
                self.comb_cells.remove(name);
                if let Some(Expr::Num { v, .. }) = args.first() {
                    self.pascal_rows.insert(name.to_string(), *v);
                } else if let Some(first) = args.first() {
                    if let Ok(Value::Number(n)) = self.eval(first, None) {
                        self.pascal_rows.insert(name.to_string(), n);
                    }
                }
            }
            Expr::Call { func: Builtin::Comb, args, .. } => {
                self.sieve_limits.remove(name);
                self.pascal_rows.remove(name);
                if let Some(cell) = self.eval_two_numbers(args) {
                    self.comb_cells.insert(name.to_string(), cell);
                }
            }
            Expr::Var { name: src, .. } => {
                if let Some(n) = self.sieve_limits.get(src).copied() {
                    self.sieve_limits.insert(name.to_string(), n);
                    self.pascal_rows.remove(name);
                    self.comb_cells.remove(name);
                } else if let Some(n) = self.pascal_rows.get(src).copied() {
                    self.pascal_rows.insert(name.to_string(), n);
                    self.sieve_limits.remove(name);
                    self.comb_cells.remove(name);
                } else if let Some(cell) = self.comb_cells.get(src).copied() {
                    self.comb_cells.insert(name.to_string(), cell);
                    self.sieve_limits.remove(name);
                    self.pascal_rows.remove(name);
                } else {
                    self.sieve_limits.remove(name);
                    self.pascal_rows.remove(name);
                    self.comb_cells.remove(name);
                }
            }
            _ => {
                self.sieve_limits.remove(name);
                self.pascal_rows.remove(name);
                self.comb_cells.remove(name);
            }
        }
    }

    /// Remember which steppable call produced `name` (for `AS STEPS`).
    fn track_step_call(&mut self, name: &str, expr: &Expr) {
        self.step_calls.remove(name);
        self.graph_step_calls.remove(name);
        match expr {
            Expr::Call { func, args, .. } if crate::explain::supports(*func) => {
                if let Some(vals) = self.eval_args_quietly(args) {
                    self.step_calls.insert(name.to_string(), (*func, vals));
                }
            }
            Expr::Call { func, args, .. } if crate::explain_graph::supports(*func) => {
                if let Some((g, vals)) = self.eval_graph_call_quietly(args) {
                    self.graph_step_calls.insert(name.to_string(), (*func, g, vals, pretty(expr)));
                }
            }
            Expr::Var { name: src, .. } => {
                if let Some(c) = self.step_calls.get(src).cloned() {
                    self.step_calls.insert(name.to_string(), c);
                }
                if let Some(c) = self.graph_step_calls.get(src).cloned() {
                    self.graph_step_calls.insert(name.to_string(), c);
                }
            }
            _ => {}
        }
    }

    /// Evaluate `(graph, vertex, …)` arguments without leaving trace entries.
    fn eval_graph_call_quietly(&mut self, args: &[Expr]) -> Option<(GraphVal, Vec<i64>)> {
        let mark = self.intermediates.len();
        let mut graph = None;
        let mut nums = Vec::new();
        let mut ok = true;
        for (i, a) in args.iter().enumerate() {
            match self.eval(a, None) {
                Ok(Value::Graph(g)) if i == 0 => graph = Some(g),
                Ok(Value::Number(n)) if i > 0 => nums.push(n),
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        self.intermediates.truncate(mark);
        if ok { graph.map(|g| (g, nums)) } else { None }
    }

    /// Evaluate integer arguments without leaving trace entries behind.
    fn eval_args_quietly(&mut self, args: &[Expr]) -> Option<Vec<i64>> {
        let mark = self.intermediates.len();
        let mut out = Vec::with_capacity(args.len());
        let mut ok = true;
        for a in args {
            match self.eval(a, None) {
                Ok(Value::Number(n)) => out.push(n),
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        self.intermediates.truncate(mark);
        if ok {
            Some(out)
        } else {
            None
        }
    }

    /// `VISUALIZE G AS STEPS` — a worked, multi-method solution for the
    /// number-theory / combinatorics call behind `G` (direct or via a variable).
    fn exec_steps(&mut self, expr: &Expr) -> IResult<StmtOutcome> {
        let pos = expr.pos();
        // Graph algorithms (BFS, DFS, SHORTEST_PATH, MST, COLOR, …).
        let graph_call: Option<(Builtin, GraphVal, Vec<i64>, String)> = match expr {
            Expr::Call { func, args, .. } if crate::explain_graph::supports(*func) => {
                match self.eval_graph_call_quietly(args) {
                    Some((g, vals)) => {
                        let text = pretty(expr);
                        Some((*func, g, vals, text))
                    }
                    None => {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!("'{}' needs a graph first, then whole-number vertices.", func.name()),
                        ))
                    }
                }
            }
            Expr::Var { name, .. } if self.graph_step_calls.contains_key(name) => {
                Some(self.graph_step_calls[name].clone())
            }
            _ => None,
        };
        if let Some((func, g, vals, text)) = graph_call {
            let ir = crate::explain_graph::explain_graph(func, &g, &vals, text)
                .map_err(|m| Diagnostic::error(pos.0, pos.1, m))?;
            let summary = format!(
                "Worked solution for {} ({} method{}{})",
                ir.expression,
                ir.sections.iter().filter(|s| s.agrees.is_some()).count(),
                if ir.sections.iter().filter(|s| s.agrees.is_some()).count() == 1 { "" } else { "s" },
                if ir.verified { ", verified" } else { ", DISAGREEMENT — please report" }
            );
            return Ok(StmtOutcome {
                outputs: Vec::new(),
                visualizations: vec![VizIr::Steps(ir)],
                warnings: Vec::new(),
                summary,
                intermediates: std::mem::take(&mut self.intermediates),
            });
        }
        let (func, args) = match expr {
            Expr::Call { func, args, .. } if crate::explain::supports(*func) => {
                let mark = self.intermediates.len();
                let mut vals = Vec::new();
                for a in args {
                    match self.eval(a, None)? {
                        Value::Number(n) => vals.push(n),
                        other => return Err(defensive_type_error(func.name(), &other, &pos)),
                    }
                }
                self.intermediates.truncate(mark);
                (*func, vals)
            }
            Expr::Var { name, .. } if self.step_calls.contains_key(name) => self.step_calls[name].clone(),
            _ => {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "'STEPS' needs the result of IS_PRIME, FACTORS, DIVISORS, GCD, LCM, TOTIENT, SIEVE, FACT, PERM, COMB, BFS, DFS, SHORTEST_PATH, MST, COLOR, IS_BIPARTITE or IS_CONNECTED, but '{}' is not one.",
                        pretty(expr)
                    ),
                )
                .with_hint("Try G = GCD(48, 18), then VISUALIZE G AS STEPS."));
            }
        };
        let call_text = format!("{}({})", func.name(), args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", "));
        let ir = crate::explain::explain(func, &args, call_text)
            .map_err(|m| Diagnostic::error(pos.0, pos.1, m))?;
        let summary = format!(
            "Worked solution for {} = {} ({} methods{})",
            ir.expression,
            if ir.result.len() > 40 { format!("{}…", ir.result.chars().take(40).collect::<String>()) } else { ir.result.clone() },
            ir.sections.iter().filter(|s| s.agrees.is_some()).count(),
            if ir.verified { ", all agree" } else { ", DISAGREEMENT — please report" }
        );
        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![VizIr::Steps(ir)],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    /// Evaluate two expressions as numbers, for `COMB(n, k)` provenance.
    fn eval_two_numbers(&mut self, args: &[Expr]) -> Option<(i64, i64)> {
        if args.len() != 2 {
            return None;
        }
        let a = match self.eval(&args[0], None) {
            Ok(Value::Number(n)) => n,
            _ => return None,
        };
        let b = match self.eval(&args[1], None) {
            Ok(Value::Number(n)) => n,
            _ => return None,
        };
        Some((a, b))
    }

    // ----- display ------------------------------------------------------

    fn exec_display(&mut self, items: &[(Expr, DType)]) -> IResult<StmtOutcome> {
        let mut outputs = Vec::new();
        let mut visualizations = Vec::new();
        let mut parts = Vec::new();
        for (expr, _) in items {
            let value = self.eval(expr, None)?;
            let elements = match &value {
                Value::Set(s) => Some(elems_to_json(s.iter())),
                Value::List(items) => Some(elems_to_json(items.iter())),
                _ => None,
            };
            let name = match expr {
                Expr::Var { name, .. } => Some(name.clone()),
                _ => None,
            };
            parts.push(format!(
                "{} = {}",
                pretty(expr),
                value.display()
            ));
            outputs.push(OutputEntry {
                kind: "value",
                name,
                expression: pretty(expr),
                value_type: value.type_name(),
                display: value.display(),
                elements,
            });
        }
        // A DISPLAY of BFS / DFS / SHORTEST_PATH (directly, or through a
        // variable holding the result) also emits a graph view with the
        // traversal highlighted, so students see the walk on the graph.
        for (expr, _) in items {
            let call: Option<Expr> = match expr {
                Expr::Call { func: Builtin::Bfs | Builtin::Dfs | Builtin::ShortestPath, .. } => {
                    Some(expr.clone())
                }
                Expr::Var { name, .. } => self.traversals.get(name).cloned(),
                _ => None,
            };
            if let Some(call) = &call {
                if let Some(viz) = self.auto_graph_view(call)? {
                    visualizations.push(viz);
                }
            }
        }

        let summary = parts.join(";  ");
        Ok(StmtOutcome {
            outputs,
            visualizations,
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    // ----- truth tables ---------------------------------------------------

    fn exec_truth_table(
        &mut self,
        expr: &Expr,
        vars: &[String],
    ) -> IResult<StmtOutcome> {
        let expression = pretty(expr);
        let mut rows: Vec<BTreeMap<String, serde_json::Value>> = Vec::new();
        let n = vars.len();

        if n == 0 {
            // A closed Boolean expression: evaluate it once.
            let v = self.eval(expr, None)?;
            let result = matches!(v, Value::Bool(true));
            let mut row = BTreeMap::new();
            row.insert("result".to_string(), serde_json::Value::Bool(result));
            rows.push(row);
        } else {
            // Enumerate assignments from all-TRUE down to all-FALSE, so the
            // first variable (most significant bit) reads TRUE, TRUE, ..., FALSE.
            let total = 1usize << n;
            for mask in (0..total).rev() {
                let mut overrides: HashMap<String, bool> = HashMap::new();
                let mut row: BTreeMap<String, serde_json::Value> = BTreeMap::new();
                for (idx, var) in vars.iter().enumerate() {
                    let bit = (mask >> (n - 1 - idx)) & 1 == 1;
                    overrides.insert(var.clone(), bit);
                    row.insert(var.clone(), serde_json::Value::Bool(bit));
                }
                let v = self.eval(expr, Some(&overrides))?;
                let result = match v {
                    Value::Bool(b) => b,
                    _ => unreachable!("semantic analysis guarantees Boolean here"),
                };
                row.insert("result".to_string(), serde_json::Value::Bool(result));
                rows.push(row);
            }
        }

        let all_true = rows.iter().all(|r| r["result"] == serde_json::Value::Bool(true));
        let all_false = rows.iter().all(|r| r["result"] == serde_json::Value::Bool(false));
        let (classification, note): (&'static str, String) = if all_true {
            ("tautology", "Every assignment evaluates to TRUE.".to_string())
        } else if all_false {
            ("contradiction", "Every assignment evaluates to FALSE.".to_string())
        } else {
            ("contingency", "Some assignments are TRUE and some are FALSE.".to_string())
        };

        let viz = VizIr::TruthTable(TruthTableIr {
            expression: expression.clone(),
            variables: vars.to_vec(),
            row_count: rows.len(),
            rows,
            classification,
            classification_note: note,
        });

        let row_count_display = if n == 0 { 1 } else { 1usize << n };
        let summary = format!(
            "Generated truth table for {} ({} row{}, {})",
            ellipsize(&expression, 40),
            row_count_display,
            if row_count_display == 1 { "" } else { "s" },
            classification
        );

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    // ----- set visualization ----------------------------------------------

    fn exec_venn(&mut self, expr: &Expr) -> IResult<StmtOutcome> {
        // Reject non-numeric sets: Venn regions hold numeric elements.
        check_venn_numeric(expr, self)?;

        let value = self.eval(expr, None)?;
        let s = match &value {
            Value::Set(s) => s.clone(),
            _ => unreachable!("semantic analysis guarantees a set here"),
        };
        let result_display = set_to_string(&s);

        // Determine the provenance (which operation produced this set).
        let prov = match expr {
            Expr::Binary { op, lhs, rhs, .. } if op.is_set_op() => {
                let lv = self.eval(lhs, None)?;
                let rv = self.eval(rhs, None)?;
                let (ls, rs) = match (&lv, &rv) {
                    (Value::Set(l), Value::Set(r)) => (l.clone(), r.clone()),
                    _ => unreachable!(),
                };
                Provenance::BinaryOp {
                    op: *op,
                    left_desc: pretty(lhs),
                    right_desc: pretty(rhs),
                    left: ls,
                    right: rs,
                }
            }
            Expr::Unary { op: UnOp::Complement, arg, .. } => {
                let lv = self.eval(arg, None)?;
                if let Value::Set(a) = lv {
                    Provenance::Complement { arg_desc: pretty(arg), arg: a }
                } else {
                    Provenance::Literal
                }
            }
            Expr::Var { name, .. } => {
                self.provenance.get(name).cloned().unwrap_or(Provenance::Literal)
            }
            _ => Provenance::Literal,
        };

        let universe = self.universe.clone();
        let universe_json = universe.as_ref().map(|u| elems_to_json(u.iter()));

        let mut note: Option<String> = None;
        let (operation, operation_label, labels, operands, regions) = match &prov {
            Provenance::BinaryOp { op, left_desc, right_desc, left, right } => {
                let intersection: BTreeSet<SetElem> =
                    left.intersection(right).cloned().collect();
                let left_only: BTreeSet<SetElem> =
                    left.difference(right).cloned().collect();
                let right_only: BTreeSet<SetElem> =
                    right.difference(left).cloned().collect();
                let outside: BTreeSet<SetElem> = match &universe {
                    Some(u) => u
                        .difference(&left.union(right).cloned().collect())
                        .cloned()
                        .collect(),
                    None => BTreeSet::new(),
                };
                if universe.is_none() {
                    note = Some(
                        "No universal set declared; the outside region is not shown. Declare UNIVERSE = { ... } to display it."
                            .to_string(),
                    );
                }
                (
                    op_symbol(op).to_string(),
                    op_label(op).to_string(),
                    vec![left_desc.clone(), right_desc.clone()],
                    {
                        let mut m = BTreeMap::new();
                        m.insert(left_desc.clone(), elems_to_json(left.iter()));
                        m.insert(right_desc.clone(), elems_to_json(right.iter()));
                        m
                    },
                    VennRegions {
                        left_only: elems_to_json(left_only.iter()),
                        intersection: elems_to_json(intersection.iter()),
                        right_only: elems_to_json(right_only.iter()),
                        outside: elems_to_json(outside.iter()),
                    },
                )
            }
            Provenance::Complement { arg_desc, arg } => {
                let outside: BTreeSet<SetElem> = match &universe {
                    Some(u) => u.difference(arg).cloned().collect(),
                    None => BTreeSet::new(),
                };
                if universe.is_none() {
                    note = Some("No universal set declared; the complement cannot be shaded. Declare UNIVERSE = { ... }.".to_string());
                }
                (
                    "complement".to_string(),
                    "COMPLEMENT".to_string(),
                    vec![arg_desc.clone(), "UNIVERSE".to_string()],
                    {
                        let mut m = BTreeMap::new();
                        if let Some(u) = &universe {
                            m.insert("UNIVERSE".to_string(), elems_to_json(u.iter()));
                        }
                        m.insert(arg_desc.clone(), elems_to_json(arg.iter()));
                        m
                    },
                    VennRegions {
                        left_only: elems_to_json(arg.iter()),
                        intersection: Vec::new(),
                        right_only: Vec::new(),
                        outside: elems_to_json(outside.iter()),
                    },
                )
            }
            _ => {
                let desc = match expr {
                    Expr::Var { name, .. } => name.clone(),
                    other => pretty(other),
                };
                (
                    "display".to_string(),
                    "DISPLAY".to_string(),
                    vec![desc.clone()],
                    {
                        let mut m = BTreeMap::new();
                        m.insert(desc.clone(), elems_to_json(s.iter()));
                        m
                    },
                    VennRegions {
                        left_only: elems_to_json(s.iter()),
                        intersection: Vec::new(),
                        right_only: Vec::new(),
                        outside: match &universe {
                            Some(u) => elems_to_json(u.difference(&s)),
                            None => Vec::new(),
                        },
                    },
                )
            }
        };

        // Truncate very large regions for rendering, keep the math intact.
        let regions = truncate_regions(regions);
        if regions_truncated(&regions) {
            note = Some(note.unwrap_or_default()
                + " Large regions are truncated for display; the mathematical result below is complete.");
        }

        let viz = VizIr::Venn(VennIr {
            operation,
            operation_label,
            labels,
            operands,
            result: elems_to_json(s.iter()),
            result_display,
            universe: universe_json,
            regions,
            note,
        });

        let summary = format!("Generated Venn diagram for {}", ellipsize(&pretty(expr), 40));

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    // ----- equivalence checks ----------------------------------------------

    fn exec_equivalence(
        &mut self,
        left: &Expr,
        right: &Expr,
        vars: &[String],
    ) -> IResult<StmtOutcome> {
        let n = vars.len();
        let mut rows: Vec<BTreeMap<String, serde_json::Value>> = Vec::new();
        let mut equivalent = true;

        let total = 1usize << n;
        for mask in (0..total).rev() {
            let mut overrides: HashMap<String, bool> = HashMap::new();
            let mut row: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            for (idx, var) in vars.iter().enumerate() {
                let bit = (mask >> (n.max(1) - 1 - idx)) & 1 == 1;
                overrides.insert(var.clone(), bit);
                row.insert(var.clone(), serde_json::Value::Bool(bit));
            }
            let lv = self.eval(left, Some(&overrides))?;
            let rv = self.eval(right, Some(&overrides))?;
            let lb = matches!(lv, Value::Bool(true));
            let rb = matches!(rv, Value::Bool(true));
            row.insert("left".to_string(), serde_json::Value::Bool(lb));
            row.insert("right".to_string(), serde_json::Value::Bool(rb));
            if lb != rb {
                equivalent = false;
            }
            rows.push(row);
        }

        let viz = VizIr::Equivalence(EquivalenceIr {
            left: pretty(left),
            right: pretty(right),
            variables: vars.to_vec(),
            rows,
            equivalent,
        });

        let summary = format!(
            "CHECK EQUIVALENT {} , {}  =>  {}",
            ellipsize(&pretty(left), 32),
            ellipsize(&pretty(right), 32),
            if equivalent { "EQUIVALENT" } else { "NOT EQUIVALENT" }
        );

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    // ----- relations & graphs: visualizations -------------------------------

    fn exec_relation_viz(&mut self, expr: &Expr, kind: RelationViz) -> IResult<StmtOutcome> {
        let value = self.eval(expr, None)?;
        let expression = pretty(expr);
        let what = describe_operand(expr);
        let pos = expr.pos();
        let s = match &value {
            Value::Set(s) => s,
            other => return Err(defensive_type_error(kind.name(), other, &pos)),
        };
        let pairs = relation_pairs(s, kind.name(), &what, pos)?;

        let (viz, summary) = match kind {
            RelationViz::Digraph => {
                let vertices = algo::field(&pairs);
                let (nv, ne) = (vertices.len(), pairs.len());
                let viz = VizIr::Digraph(DigraphIr {
                    expression: expression.clone(),
                    vertices,
                    edges: pairs,
                });
                let summary = format!(
                    "Generated digraph for {} ({} vertices, {} edges)",
                    ellipsize(&expression, 40),
                    nv, ne
                );
                (viz, summary)
            }
            RelationViz::Matrix => {
                let vertices = algo::field(&pairs);
                let set: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
                let n = vertices.len();
                let matrix: Vec<Vec<u8>> = vertices
                    .iter()
                    .map(|&i| {
                        vertices.iter().map(|&j| u8::from(set.contains(&(i, j)))).collect()
                    })
                    .collect();
                let viz = VizIr::RelationMatrix(RelationMatrixIr {
                    expression: expression.clone(),
                    vertices,
                    matrix,
                });
                let summary = format!(
                    "Generated relation matrix for {} ({} × {})",
                    ellipsize(&expression, 40),
                    n, n
                );
                (viz, summary)
            }
            RelationViz::Hasse => {
                if let Some(x) = algo::first_reflexive_violation(&pairs) {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!(
                            "HASSE requires a partial order, but '{}' is not reflexive; missing ({}, {}).",
                            what, x, x
                        ),
                    )
                    .with_hint(format!("Apply REFLEXIVE_CLOSURE({}) first.", what)));
                }
                if let Some((a, b)) = algo::first_antisymmetric_violation(&pairs) {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!(
                            "HASSE requires a partial order, but '{}' is not antisymmetric, e.g. both ({}, {}) and ({}, {}) are present.",
                            what, a, b, b, a
                        ),
                    )
                    .with_hint(
                        "A Hasse diagram draws an order; a pair and its inverse cannot both hold.",
                    ));
                }
                if let Some((a, b, c)) = algo::first_transitive_violation(&pairs) {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!(
                            "HASSE requires a partial order, but '{}' is not transitive: ({}, {}) and ({}, {}) hold, but ({}, {}) is missing.",
                            what, a, b, b, c, a, c
                        ),
                    )
                    .with_hint(format!("Apply TRANSITIVE_CLOSURE({}) first.", what)));
                }
                let (levels, edges) = algo::hasse_layout(&pairs);
                let (nl, ne) = (levels.len(), edges.len());
                let viz = VizIr::Hasse(HasseIr { expression: expression.clone(), levels, edges });
                let summary = format!(
                    "Generated Hasse diagram for {} ({} levels, {} cover edges)",
                    ellipsize(&expression, 40),
                    nl, ne
                );
                (viz, summary)
            }
        };

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    fn exec_graph_view(&mut self, expr: &Expr) -> IResult<StmtOutcome> {
        let value = self.eval(expr, None)?;
        let expression = pretty(expr);

        let (viz, summary) = match &value {
            Value::Graph(g) => {
                let (tw, note, tree) = match g.origin {
                    Some(GraphOrigin::Mst { total_weight }) => (
                        Some(total_weight),
                        Some(format!(
                            "Minimum spanning tree: {} edges, total weight {}.",
                            g.edges.len(),
                            total_weight
                        )),
                        true,
                    ),
                    _ => (None, None, false),
                };
                let v = graph_view_ir(
                    g,
                    expression.clone(),
                    None,
                    None,
                    None,
                    tw,
                    note,
                    tree,
                );
                let summary = format!(
                    "Generated graph view for {} ({} vertices, {} edges)",
                    ellipsize(&expression, 40),
                    g.vertices.len(),
                    g.edges.len()
                );
                (v, summary)
            }
            Value::Mapping(m) => {
                let g = self.coloring_graph_for(expr)?;
                let k = m.values().max().copied().unwrap_or(-1) + 1;
                let note =
                    Some(format!("Greedy coloring uses {} color{}.", k, if k == 1 { "" } else { "s" }));
                let v = graph_view_ir(
                    &g,
                    expression.clone(),
                    None,
                    None,
                    Some(m.clone()),
                    None,
                    note,
                    false,
                );
                let summary = format!(
                    "Generated graph view for {} with {} colors",
                    ellipsize(&expression, 40),
                    k
                );
                (v, summary)
            }
            other => return Err(defensive_type_error("GRAPH", other, &expr.pos())),
        };

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    /// The graph behind a coloring value (COLOR call or a variable that
    /// holds one).
    fn coloring_graph_for(&mut self, expr: &Expr) -> IResult<GraphVal> {
        match expr {
            Expr::Call { func: Builtin::Color, args, pos } => {
                let v = self.eval(&args[0], None)?;
                match v {
                    Value::Graph(g) => Ok(g),
                    other => Err(defensive_type_error("COLOR", &other, pos)),
                }
            }
            Expr::Var { name, pos } => self.coloring_graphs.get(name).cloned().ok_or_else(|| {
                Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!("'{}' does not hold a coloring produced by COLOR.", name),
                )
                .with_hint("Write C = COLOR(G) first, then VISUALIZE C AS GRAPH.")
            }),
            _ => Err(Diagnostic::error(
                expr.pos().0,
                expr.pos().1,
                "Cannot locate the graph for this coloring.",
            )
            .with_hint("Assign the coloring to a variable first: C = COLOR(G).")),
        }
    }

    fn exec_check_properties(&mut self, expr: &Expr) -> IResult<StmtOutcome> {
        let value = self.eval(expr, None)?;
        let expression = pretty(expr);
        let what = describe_operand(expr);
        let pos = expr.pos();

        let (viz, summary) = match &value {
            Value::Set(s) => {
                let pairs = relation_pairs(s, "CHECK PROPERTIES", &what, pos)?;
                let reflexive = algo::is_reflexive(&pairs);
                let symmetric = algo::is_symmetric(&pairs);
                let antisymmetric = algo::is_antisymmetric(&pairs);
                let transitive = algo::is_transitive(&pairs);
                let is_equivalence = reflexive && symmetric && transitive;
                let is_partial_order = reflexive && antisymmetric && transitive;

                let mut counterexamples = BTreeMap::new();
                if let Some(x) = algo::first_reflexive_violation(&pairs) {
                    counterexamples
                        .insert("reflexive".to_string(), format!("({}, {}) is missing.", x, x));
                }
                if !symmetric {
                    let set: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
                    if let Some((a, b)) = pairs
                        .iter()
                        .copied()
                        .find(|&(a, b)| a != b && !set.contains(&(b, a)))
                    {
                        counterexamples.insert(
                            "symmetric".to_string(),
                            format!("({}, {}) holds, but ({}, {}) is missing.", a, b, b, a),
                        );
                    }
                }
                if let Some((a, b)) = algo::first_antisymmetric_violation(&pairs) {
                    counterexamples.insert(
                        "antisymmetric".to_string(),
                        format!("both ({}, {}) and ({}, {}) are present.", a, b, b, a),
                    );
                }
                if let Some((a, b, c)) = algo::first_transitive_violation(&pairs) {
                    counterexamples.insert(
                        "transitive".to_string(),
                        format!(
                            "({}, {}) and ({}, {}) hold, but ({}, {}) is missing.",
                            a, b, b, c, a, c
                        ),
                    );
                }
                let classes =
                    if is_equivalence { Some(algo::equivalence_classes(&pairs)) } else { None };
                let class_count = classes.as_ref().map(|c| c.len()).unwrap_or(0);

                let suffix = if is_equivalence {
                    format!("; equivalence relation ({} classes)", class_count)
                } else if is_partial_order {
                    "; partial order".to_string()
                } else {
                    String::new()
                };
                let summary = format!(
                    "{}: reflexive {}, symmetric {}, antisymmetric {}, transitive {}{}",
                    what,
                    yn(reflexive),
                    yn(symmetric),
                    yn(antisymmetric),
                    yn(transitive),
                    suffix
                );
                let viz = VizIr::RelationProperties(RelationPropertiesIr {
                    expression: expression.clone(),
                    base_set: algo::field(&pairs),
                    reflexive,
                    symmetric,
                    antisymmetric,
                    transitive,
                    is_equivalence,
                    is_partial_order,
                    counterexamples,
                    classes,
                });
                (viz, summary)
            }
            Value::Graph(g) => {
                let connected = algo::is_connected(g);
                let tree = algo::is_tree(g);
                let bipartite = algo::is_bipartite(g);
                let complete = algo::is_complete(g);
                let degrees: BTreeMap<i64, i64> =
                    g.vertices.iter().map(|&v| (v, algo::degree(g, v))).collect();
                let mut notes = Vec::new();
                if !connected {
                    let start = *g.vertices.iter().next().unwrap_or(&0);
                    let n = algo::unreachable_count(g);
                    notes.push(format!(
                        "The graph is not connected: {} {} unreachable from vertex {}.",
                        n,
                        if n == 1 { "vertex is" } else { "vertices are" },
                        start
                    ));
                }
                if !bipartite {
                    notes.push(
                        "The graph contains an odd cycle, so it is not bipartite.".to_string(),
                    );
                }
                let summary = format!(
                    "{}: {} vertices, {} edges; connected {}, tree {}, bipartite {}, complete {}",
                    what,
                    g.vertices.len(),
                    g.edges.len(),
                    yn(connected),
                    yn(tree),
                    yn(bipartite),
                    yn(complete)
                );
                let viz = VizIr::GraphProperties(GraphPropertiesIr {
                    expression: expression.clone(),
                    vertex_count: g.vertices.len(),
                    edge_count: g.edges.len(),
                    connected,
                    tree,
                    bipartite,
                    complete,
                    degrees,
                    notes,
                });
                (viz, summary)
            }
            other => {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "'CHECK PROPERTIES' expects a relation or a graph, but this expression has type {}.",
                        other.type_name()
                    ),
                ))
            }
        };

        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    // ----- built-in function calls ------------------------------------------

    fn eval_call(
        &mut self,
        func: Builtin,
        args: &[Expr],
        call: &Expr,
        overrides: Option<&HashMap<String, bool>>,
        pos: &Pos,
    ) -> IResult<Value> {
        let mut vals: Vec<Value> = Vec::with_capacity(args.len());
        for a in args {
            vals.push(self.eval(a, overrides)?);
        }
        let what0 = describe_operand(&args[0]);

        let (value, detail): (Value, Option<String>) = match func {
            // ----- relations -----
            Builtin::Inverse => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (set_from_pairs(algo::inverse(&p)), None)
            }
            Builtin::Compose => {
                let what1 = describe_operand(&args[1]);
                let r = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                let s = self.relation_arg(&vals[1], func.name(), &what1, *pos)?;
                (
                    set_from_pairs(algo::compose(&r, &s)),
                    Some("applies the left relation first".to_string()),
                )
            }
            Builtin::TransitiveClosure => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (
                    set_from_pairs(algo::transitive_closure(&p)),
                    Some("Warshall algorithm".to_string()),
                )
            }
            Builtin::ReflexiveClosure => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (set_from_pairs(algo::reflexive_closure(&p)), None)
            }
            Builtin::SymmetricClosure => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (set_from_pairs(algo::symmetric_closure(&p)), None)
            }
            Builtin::IsReflexive => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_reflexive(&p)), None)
            }
            Builtin::IsSymmetric => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_symmetric(&p)), None)
            }
            Builtin::IsAntisymmetric => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_antisymmetric(&p)), None)
            }
            Builtin::IsTransitive => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_transitive(&p)), None)
            }
            Builtin::IsEquivalence => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_equivalence(&p)), None)
            }
            Builtin::IsPartialOrder => {
                let p = self.relation_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_partial_order(&p)), None)
            }
            // ----- graph theory -----
            Builtin::Vertices => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (
                    Value::Set(g.vertices.iter().map(|&v| SetElem::Num(v)).collect()),
                    None,
                )
            }
            Builtin::Edges => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Set(g.edges.iter().map(edge_to_elem).collect()), None)
            }
            Builtin::Degree => {
                let (g, v, _) = self.graph_and_vertex(&vals, args, func.name(), *pos)?;
                (Value::Number(algo::degree(&g, v)), None)
            }
            Builtin::Neighbors => {
                let (g, v, _) = self.graph_and_vertex(&vals, args, func.name(), *pos)?;
                (
                    Value::List(algo::neighbors(&g, v).into_iter().map(SetElem::Num).collect()),
                    None,
                )
            }
            Builtin::Adjacent => {
                let (g, a, b) = self.graph_and_two_vertices(&vals, args, func.name(), *pos)?;
                (Value::Bool(algo::adjacent(&g, a, b)), None)
            }
            Builtin::Bfs => {
                let (g, v, what) = self.graph_and_vertex(&vals, args, func.name(), *pos)?;
                (
                    Value::List(algo::bfs(&g, v).into_iter().map(SetElem::Num).collect()),
                    Some(format!("neighbors of {} visited in ascending order", what)),
                )
            }
            Builtin::Dfs => {
                let (g, v, _) = self.graph_and_vertex(&vals, args, func.name(), *pos)?;
                (
                    Value::List(algo::dfs(&g, v).into_iter().map(SetElem::Num).collect()),
                    Some("preorder, ascending neighbors".to_string()),
                )
            }
            Builtin::ShortestPath => {
                let (g, from, to) = self.graph_and_two_vertices(&vals, args, func.name(), *pos)?;
                let (path, weighted) =
                    algo::shortest_path(&g, from, to).map_err(|e| algo_error(e, pos, &what0))?;
                let detail =
                    if weighted { "Dijkstra (weighted)" } else { "BFS (unweighted)" };
                (
                    Value::List(path.into_iter().map(SetElem::Num).collect()),
                    Some(detail.to_string()),
                )
            }
            Builtin::IsConnected => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_connected(&g)), None)
            }
            Builtin::IsTree => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_tree(&g)), None)
            }
            Builtin::IsBipartite => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_bipartite(&g)), None)
            }
            Builtin::IsComplete => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (Value::Bool(algo::is_complete(&g)), None)
            }
            Builtin::Color => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (
                    Value::Mapping(algo::greedy_color(&g)),
                    Some("smallest available color per vertex".to_string()),
                )
            }
            Builtin::Mst => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                let t = algo::kruskal(&g).map_err(|e| algo_error(e, pos, &what0))?;
                (Value::Graph(t), Some("Kruskal".to_string()))
            }
            // ----- number theory -----
            Builtin::IsPrime => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                let b = i64_to_u64(n, pos)?;
                (Value::Bool(algo::is_prime(b)), Some("Miller–Rabin primality test".to_string()))
            }
            Builtin::Factors => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                let m = positive_int(n, func.name(), &what0, *pos)?;
                let fs = algo::prime_factors(m);
                (
                    list_value(fs.iter().map(|&f| f as i64)),
                    Some("trial division + Pollard rho".to_string()),
                )
            }
            Builtin::Divisors => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                let m = positive_int(n, func.name(), &what0, *pos)?;
                (
                    list_value(algo::divisors(m as i64)),
                    Some("every product of the prime powers".to_string()),
                )
            }
            Builtin::Gcd => {
                let (a, b) = self.two_number_args(&vals, func.name(), *pos)?;
                (Value::Number(algo::gcd(a, b)), Some("Euclidean algorithm".to_string()))
            }
            Builtin::Lcm => {
                let (a, b) = self.two_number_args(&vals, func.name(), *pos)?;
                match algo::lcm_checked(a, b) {
                    Some(v) => (
                        Value::Number(v),
                        Some("LCM(a, b) = |a·b| / GCD(a, b)".to_string()),
                    ),
                    None => {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!("LCM({}, {}) exceeds the 64-bit integer range.", a, b),
                        )
                        .with_hint("Try smaller inputs: the result must fit in 64 bits."));
                    }
                }
            }
            Builtin::Totient => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                let m = positive_int(n, func.name(), &what0, *pos)?;
                (
                    Value::Number(algo::totient(m as u64) as i64),
                    Some("Euler's product formula φ(n) = n · Π (1 − 1/p)".to_string()),
                )
            }
            Builtin::Sieve => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                if n < 0 {
                    return Err(range_error(func.name(), n, "non-negative", pos));
                }
                if n > algo::SIEVE_MAX_N {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!(
                            "SIEVE requires n ≤ {}, but got {}.",
                            algo::SIEVE_MAX_N, n
                        ),
                    )
                    .with_hint("Larger sieves would not fit the interactive grid view."));
                }
                (
                    list_value(algo::sieve(n as u64).into_iter().map(|p| p as i64)),
                    Some("sieve of Eratosthenes".to_string()),
                )
            }
            // ----- combinatorics -----
            Builtin::Size => match &vals[0] {
                Value::Set(s) => (
                    Value::Number(s.len() as i64),
                    Some("|S| = the number of distinct elements".to_string()),
                ),
                other => return Err(defensive_type_error(func.name(), other, pos)),
            },
            Builtin::Contains => {
                let x = match &vals[1] {
                    Value::Number(n) => SetElem::Num(*n),
                    Value::Set(s) => SetElem::Set(s.iter().cloned().collect()),
                    other => return Err(defensive_type_error(func.name(), other, pos)),
                };
                match &vals[0] {
                    Value::Set(s) => (
                        Value::Bool(s.contains(&x)),
                        Some("x ∈ S: the element is a number, or a set when S holds sets (a power set)".to_string()),
                    ),
                    other => return Err(defensive_type_error(func.name(), other, pos)),
                }
            }
            Builtin::Powerset => match &vals[0] {
                Value::Set(s) => {
                    if s.len() > algo::MAX_POWERSET_ELEMS {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "POWERSET supports sets of at most {} elements, but '{}' has {}.",
                                algo::MAX_POWERSET_ELEMS,
                                what0,
                                s.len()
                            ),
                        )
                        .with_hint(format!(
                            "A set of n elements has 2^n subsets ({} elements would give {}).",
                            s.len(),
                            1u128 << s.len().min(100)
                        )));
                    }
                    (
                        Value::Set(algo::powerset(s)),
                        Some(format!("2^{} = {} subsets, from {{}} up to the set itself", s.len(), 1u64 << s.len())),
                    )
                }
                other => return Err(defensive_type_error(func.name(), other, pos)),
            },
            Builtin::IsDisjoint => match (&vals[0], &vals[1]) {
                (Value::Set(a), Value::Set(b)) => (
                    Value::Bool(a.is_disjoint(b)),
                    Some("TRUE when A INTERSECT B is empty".to_string()),
                ),
                (other, _) => return Err(defensive_type_error(func.name(), other, pos)),
            },
            Builtin::Fact => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                if n < 0 {
                    return Err(range_error(func.name(), n, "non-negative", pos));
                }
                match algo::factorial_checked(n) {
                    Some(v) => (Value::Number(v), Some("n! = 1 · 2 · … · n".to_string())),
                    None => {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "FACT requires n ≤ {}, but got {}.",
                                algo::FACT_MAX_N, n
                            ),
                        )
                        .with_hint(
                            "21! = 51 090 942 171 709 440 000 already exceeds the 64-bit range.",
                        ));
                    }
                }
            }
            Builtin::Perm => {
                let (n, k) = self.two_number_args(&vals, func.name(), *pos)?;
                if n < 0 || k < 0 {
                    return Err(range_error(func.name(), n.min(k), "non-negative", pos));
                }
                if k > n {
                    return Err(k_exceeds_n_error("PERM", n, k, pos));
                }
                match algo::perm_checked(n, k) {
                    Some(v) => (
                        Value::Number(v),
                        Some("P(n, k) = n! / (n − k)!, computed iteratively".to_string()),
                    ),
                    None => {
                        return Err(overflow_error("PERM", n, k, pos));
                    }
                }
            }
            Builtin::Comb => {
                let (n, k) = self.two_number_args(&vals, func.name(), *pos)?;
                if n < 0 || k < 0 {
                    return Err(range_error(func.name(), n.min(k), "non-negative", pos));
                }
                if k > n {
                    return Err(k_exceeds_n_error("COMB", n, k, pos));
                }
                match algo::comb_checked(n, k) {
                    Some(v) => (
                        Value::Number(v),
                        Some("C(n, k) = n! / (k! (n − k)!), computed iteratively".to_string()),
                    ),
                    None => {
                        return Err(overflow_error("COMB", n, k, pos));
                    }
                }
            }
            Builtin::Pascal => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                if n < 1 {
                    return Err(range_error(func.name(), n, "at least 1", pos));
                }
                if n > algo::PASCAL_MAX_ROWS {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!(
                            "PASCAL requires n ≤ {} rows, but got {}.",
                            algo::PASCAL_MAX_ROWS, n
                        ),
                    )
                    .with_hint("Larger triangles would not fit the interactive view."));
                }
                (
                    pascal_value(n as usize),
                    Some("row i holds C(i, 0) … C(i, i)".to_string()),
                )
            }
            // ----- modular arithmetic -----
            Builtin::Mod => {
                let (a, m) = self.two_number_args(&vals, func.name(), *pos)?;
                if m < 1 {
                    return Err(range_error(func.name(), m, "a positive modulus m, but the modulus was", pos));
                }
                (Value::Number(algo::modulo(a, m)), Some("remainder in 0 … m−1".to_string()))
            }
            Builtin::ModPow => {
                let base = self.number_arg(&vals[0], func.name(), "", *pos)?;
                let exp = self.number_arg(&vals[1], func.name(), "", *pos)?;
                let m = self.number_arg(&vals[2], func.name(), "", *pos)?;
                if m < 1 {
                    return Err(range_error(func.name(), m, "a positive modulus m, but the modulus was", pos));
                }
                if exp < 0 {
                    return Err(range_error(func.name(), exp, "a non-negative exponent, but the exponent was", pos)
                        .with_hint("For a negative power, use MODINV first."));
                }
                (Value::Number(algo::modpow(base, exp as u64, m)), Some("square-and-multiply".to_string()))
            }
            Builtin::ModInv => {
                let (a, m) = self.two_number_args(&vals, func.name(), *pos)?;
                if m < 2 {
                    return Err(range_error(func.name(), m, "a modulus of at least 2, but the modulus was", pos));
                }
                match algo::modinv(a, m) {
                    Some(x) => (Value::Number(x), Some(format!("{}·{} ≡ 1 (mod {})", a, x, m))),
                    None => {
                        let g = algo::gcd(a, m);
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!("{} has no inverse modulo {} because GCD({}, {}) = {}, not 1.", a, m, a, m, g),
                        )
                        .with_hint("An inverse exists only when the two numbers share no factor."));
                    }
                }
            }
            Builtin::Egcd => {
                let (a, b) = self.two_number_args(&vals, func.name(), *pos)?;
                let (g, x, y) = algo::egcd(a, b);
                (
                    list_value([g, x, y]),
                    Some(format!("[g, x, y] with {}·x + {}·y = g", a, b)),
                )
            }
            Builtin::Crt => {
                let a1 = self.number_arg(&vals[0], func.name(), "", *pos)?;
                let m1 = self.number_arg(&vals[1], func.name(), "", *pos)?;
                let a2 = self.number_arg(&vals[2], func.name(), "", *pos)?;
                let m2 = self.number_arg(&vals[3], func.name(), "", *pos)?;
                for m in [m1, m2] {
                    if m < 1 {
                        return Err(range_error(func.name(), m, "positive moduli, but a modulus was", pos));
                    }
                }
                match algo::crt(a1, m1, a2, m2) {
                    Ok((x, lcm)) => (
                        list_value([x, lcm]),
                        Some(format!("[x, M]: x ≡ {} (mod {}) and x ≡ {} (mod {}) exactly when x ≡ {} (mod {})", a1, m1, a2, m2, x, lcm)),
                    ),
                    Err(msg) => {
                        return Err(Diagnostic::error(pos.0, pos.1, msg)
                            .with_hint("With coprime moduli there is always exactly one solution."));
                    }
                }
            }
            // ----- more counting -----
            Builtin::Fib => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                if n < 0 || n > algo::FIB_MAX_N {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!("FIB requires n from 0 to {}, but got {}.", algo::FIB_MAX_N, n),
                    )
                    .with_hint(format!("FIB({}) is the largest Fibonacci number that fits in a 64-bit number.", algo::FIB_MAX_N)));
                }
                (Value::Number(algo::fib(n)), Some("F(0) = 0, F(1) = 1, F(n) = F(n−1) + F(n−2)".to_string()))
            }
            Builtin::Catalan => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                if n < 0 || n > algo::CATALAN_MAX_N {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!("CATALAN requires n from 0 to {}, but got {}.", algo::CATALAN_MAX_N, n),
                    ));
                }
                match algo::catalan_checked(n) {
                    Some(v) => (Value::Number(v), Some("C(n) = C(2n, n) / (n + 1)".to_string())),
                    None => return Err(Diagnostic::error(pos.0, pos.1, "CATALAN result does not fit in a 64-bit number.")),
                }
            }
            Builtin::Derange => {
                let n = self.number_arg(&vals[0], func.name(), &what0, *pos)?;
                if n < 0 || n > algo::DERANGE_MAX_N {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!("DERANGE requires n from 0 to {}, but got {}.", algo::DERANGE_MAX_N, n),
                    ));
                }
                match algo::derange_checked(n) {
                    Some(v) => (Value::Number(v), Some("arrangements of n items where none stays in place".to_string())),
                    None => return Err(Diagnostic::error(pos.0, pos.1, "DERANGE result does not fit in a 64-bit number.")),
                }
            }
            Builtin::Stirling2 => {
                let (n, k) = self.two_number_args(&vals, func.name(), *pos)?;
                if n < 0 || k < 0 {
                    return Err(range_error(func.name(), n.min(k), "non-negative", pos));
                }
                if n > algo::STIRLING_MAX_N {
                    return Err(Diagnostic::error(
                        pos.0,
                        pos.1,
                        format!("STIRLING2 requires n ≤ {}, but got {}.", algo::STIRLING_MAX_N, n),
                    ));
                }
                if k > n {
                    return Err(k_exceeds_n_error("STIRLING2", n, k, pos));
                }
                match algo::stirling2_checked(n, k) {
                    Some(v) => (Value::Number(v), Some("ways to split n items into k non-empty groups".to_string())),
                    None => return Err(Diagnostic::error(pos.0, pos.1, "STIRLING2 result does not fit in a 64-bit number.")),
                }
            }
            // ----- more graph tools -----
            Builtin::TopoSort => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                if !g.directed {
                    return Err(needs_digraph("TOPO_SORT", pos));
                }
                match algo::topo_sort(&g) {
                    Ok(order) => (list_value(order), Some("Kahn's algorithm, smallest available vertex first".to_string())),
                    Err(stuck) => {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!("{} has a cycle, so it has no topological order.", what0),
                        )
                        .with_hint(format!(
                            "Vertices on or behind a cycle: {}.",
                            stuck.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
                        )));
                    }
                }
            }
            Builtin::IsDag => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                if !g.directed {
                    return Err(needs_digraph("IS_DAG", pos));
                }
                (Value::Bool(algo::topo_sort(&g).is_ok()), Some("a directed graph with no cycle".to_string()))
            }
            Builtin::Components => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (
                    Value::Mapping(algo::components(&g)),
                    Some("vertex → component number (directions ignored)".to_string()),
                )
            }
            Builtin::IsEulerian => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (
                    Value::Bool(algo::is_eulerian(&g)),
                    Some("one connected piece of edges with every vertex balanced (even degree, or in = out)".to_string()),
                )
            }
            Builtin::DegreeSequence => {
                let g = self.graph_arg(&vals[0], func.name(), &what0, *pos)?;
                (list_value(algo::degree_sequence(&g)), Some("degrees, largest first".to_string()))
            }
        };

        self.record(Intermediate { expr: pretty(call), result: value.display(), detail });
        Ok(value)
    }

    // ----- argument helpers -------------------------------------------------

    fn relation_arg(&self, v: &Value, who: &str, what: &str, pos: Pos) -> IResult<Vec<(i64, i64)>> {
        match v {
            Value::Set(s) => relation_pairs(s, who, what, pos),
            other => Err(defensive_type_error(who, other, &pos)),
        }
    }

    fn graph_arg(&self, v: &Value, who: &str, _what: &str, pos: Pos) -> IResult<GraphVal> {
        match v {
            Value::Graph(g) => Ok(g.clone()),
            other => Err(defensive_type_error(who, other, &pos)),
        }
    }

    /// A plain integer argument (number theory / combinatorics domain).
    fn number_arg(&self, v: &Value, who: &str, _what: &str, pos: Pos) -> IResult<i64> {
        match v {
            Value::Number(n) => Ok(*n),
            other => Err(defensive_type_error(who, other, &pos)),
        }
    }

    /// Two integer arguments (GCD / LCM / PERM / COMB).
    fn two_number_args(&self, vals: &[Value], who: &str, pos: Pos) -> IResult<(i64, i64)> {
        let a = self.number_arg(&vals[0], who, "", pos)?;
        let b = self.number_arg(&vals[1], who, "", pos)?;
        Ok((a, b))
    }

    /// A graph plus one vertex argument, checked for membership.
    fn graph_and_vertex(
        &self,
        vals: &[Value],
        args: &[Expr],
        who: &str,
        pos: Pos,
    ) -> IResult<(GraphVal, i64, String)> {
        let what = describe_operand(&args[0]);
        let g = self.graph_arg(&vals[0], who, &what, pos)?;
        let v = match &vals[1] {
            Value::Number(x) => *x,
            other => return Err(defensive_type_error(who, other, &pos)),
        };
        self.check_vertex(&g, v, &what, pos)?;
        Ok((g, v, what))
    }

    /// A graph plus two vertex arguments, both checked for membership.
    fn graph_and_two_vertices(
        &self,
        vals: &[Value],
        args: &[Expr],
        who: &str,
        pos: Pos,
    ) -> IResult<(GraphVal, i64, i64)> {
        let what = describe_operand(&args[0]);
        let g = self.graph_arg(&vals[0], who, &what, pos)?;
        let a = match &vals[1] {
            Value::Number(x) => *x,
            other => return Err(defensive_type_error(who, other, &pos)),
        };
        let b = match &vals[2] {
            Value::Number(x) => *x,
            other => return Err(defensive_type_error(who, other, &pos)),
        };
        self.check_vertex(&g, a, &what, pos)?;
        self.check_vertex(&g, b, &what, pos)?;
        Ok((g, a, b))
    }

    fn check_vertex(&self, g: &GraphVal, v: i64, what: &str, pos: Pos) -> IResult<()> {
        if g.vertices.contains(&v) {
            Ok(())
        } else {
            Err(Diagnostic::error(
                pos.0,
                pos.1,
                format!("Vertex {} is not in the graph.", v),
            )
            .with_hint(format!("Vertices of {}: {}", what, int_set_to_string(&g.vertices))))
        }
    }

    // ----- sieve & pascal views ---------------------------------------------

    /// `VISUALIZE S AS SIEVE` — the prime/composite grid of a `SIEVE(n)`
    /// result, direct or through a variable.
    fn exec_sieve(&mut self, expr: &Expr) -> IResult<StmtOutcome> {
        let limit = self.sieve_limit_of(expr)?;
        let (primes, smallest) = algo::sieve_grid(limit as u64);
        let prime_count = primes.len();
        let viz = VizIr::Sieve(SieveIr {
            expression: pretty(expr),
            limit,
            primes: primes.iter().map(|&p| p as i64).collect(),
            smallest_factor: smallest
                .into_iter()
                .map(|(k, v)| (k as i64, v as i64))
                .collect(),
            prime_count,
            note: None,
        });
        let summary =
            format!("Sieve grid up to {}: {} primes found", limit, prime_count);
        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    /// The limit `n` behind a `SIEVE(n)` call, direct or via a variable.
    fn sieve_limit_of(&mut self, expr: &Expr) -> IResult<i64> {
        match expr {
            Expr::Call { func: Builtin::Sieve, args, .. } => {
                let v = self.eval(&args[0], None)?;
                match v {
                    Value::Number(n) => Ok(n),
                    other => Err(defensive_type_error("SIEVE", &other, &expr.pos())),
                }
            }
            Expr::Var { name, .. } if self.sieve_limits.contains_key(name) => {
                Ok(self.sieve_limits[name])
            }
            _ => Err(provenance_error("SIEVE", expr, "S = SIEVE(30), then VISUALIZE S AS SIEVE")),
        }
    }

    /// `VISUALIZE P AS PASCAL` — the first `n` rows of Pascal's triangle, or
    /// the triangle around a highlighted `C(n, k)` cell when the value came
    /// from a `COMB(n, k)` call.
    fn exec_pascal(&mut self, expr: &Expr) -> IResult<StmtOutcome> {
        let (rows, highlight) = self.pascal_data_of(expr)?;
        let row_count = rows.len();
        let note = highlight.map(|(n, k)| {
            let value = rows
                .get(n as usize)
                .and_then(|r| r.get(k as usize))
                .copied()
                .unwrap_or_else(|| algo::comb_checked(n, k).unwrap_or(0));
            let cell = format!("Highlighted cell: C({}, {}) = {}", n, k, value);
            if (n as usize) >= rows.len() {
                format!(
                    "{} (row {} lies beyond the {} shown rows; the view shows at most {}).",
                    cell,
                    n,
                    rows.len(),
                    algo::PASCAL_MAX_ROWS + 1
                )
            } else {
                cell
            }
        });
        let viz = VizIr::Pascal(PascalIr {
            expression: pretty(expr),
            rows,
            row_count,
            highlight,
            note,
        });
        let summary = format!("Pascal's triangle: {} rows", row_count);
        Ok(StmtOutcome {
            outputs: Vec::new(),
            visualizations: vec![viz],
            warnings: Vec::new(),
            summary,
            intermediates: std::mem::take(&mut self.intermediates),
        })
    }

    /// Rows plus optional highlighted cell for `VISUALIZE ... AS PASCAL`.
    fn pascal_data_of(&mut self, expr: &Expr) -> IResult<(Vec<Vec<i64>>, Option<(i64, i64)>)> {
        match expr {
            Expr::Call { func: Builtin::Pascal, args, .. } => {
                let v = self.eval(&args[0], None)?;
                let n = match v {
                    Value::Number(n) => n,
                    other => {
                        return Err(defensive_type_error("PASCAL", &other, &expr.pos()))
                    }
                };
                Ok((algo::pascal_rows(n as usize), None))
            }
            Expr::Call { func: Builtin::Comb, args, .. } => {
                let (n, k) = self
                    .eval_two_numbers(args)
                    .ok_or_else(|| provenance_error("PASCAL", expr, "C = COMB(5, 2), then VISUALIZE C AS PASCAL"))?;
                // C(n, k) lives in row n (0-indexed), so n + 1 rows are needed
                // for the highlighted cell to exist; one extra row is allowed
                // beyond the PASCAL(n) function cap.
                let shown = ((n + 1).clamp(1, algo::PASCAL_MAX_ROWS + 1)) as usize;
                Ok((algo::pascal_rows(shown), Some((n, k))))
            }
            Expr::Var { name, .. } => {
                if let Some(n) = self.pascal_rows.get(name).copied() {
                    Ok((algo::pascal_rows(n as usize), None))
                } else if let Some((n, k)) = self.comb_cells.get(name).copied() {
                    let shown = ((n + 1).clamp(1, algo::PASCAL_MAX_ROWS + 1)) as usize;
                    Ok((algo::pascal_rows(shown), Some((n, k))))
                } else {
                    Err(provenance_error(
                        "PASCAL",
                        expr,
                        "P = PASCAL(5) or C = COMB(5, 2), then VISUALIZE P AS PASCAL",
                    ))
                }
            }
            _ => Err(provenance_error(
                "PASCAL",
                expr,
                "P = PASCAL(5) or C = COMB(5, 2), then VISUALIZE P AS PASCAL",
            )),
        }
    }

    fn auto_graph_view(&mut self, call: &Expr) -> IResult<Option<VizIr>> {
        let Expr::Call { func, args, pos } = call else { return Ok(None) };
        let pos = *pos;
        let gval = self.eval(&args[0], None)?;
        let g = match gval {
            Value::Graph(g) => g,
            _ => return Ok(None),
        };
        let what = describe_operand(&args[0]);
        let expression = pretty(call);
        match func {
            Builtin::Bfs | Builtin::Dfs => {
                let start = match self.eval(&args[1], None)? {
                    Value::Number(n) => n,
                    _ => return Ok(None),
                };
                self.check_vertex(&g, start, &what, pos)?;
                let order =
                    if matches!(func, Builtin::Bfs) { algo::bfs(&g, start) } else { algo::dfs(&g, start) };
                let seq: Vec<String> = order.iter().map(|v| v.to_string()).collect();
                let note =
                    format!("{} from {} visits: {}", func.name(), start, seq.join(" → "));
                Ok(Some(graph_view_ir(
                    &g,
                    expression,
                    None,
                    Some(order),
                    None,
                    None,
                    Some(note),
                    false,
                )))
            }
            Builtin::ShortestPath => {
                let from = match self.eval(&args[1], None)? {
                    Value::Number(n) => n,
                    _ => return Ok(None),
                };
                let to = match self.eval(&args[2], None)? {
                    Value::Number(n) => n,
                    _ => return Ok(None),
                };
                self.check_vertex(&g, from, &what, pos)?;
                self.check_vertex(&g, to, &what, pos)?;
                let (path, weighted) =
                    algo::shortest_path(&g, from, to).map_err(|e| algo_error(e, &pos, &what))?;
                let (note, highlight) = if path.is_empty() {
                    (
                        Some(format!("Vertex {} is not reachable from {}.", to, from)),
                        None,
                    )
                } else {
                    let seq: Vec<String> = path.iter().map(|v| v.to_string()).collect();
                    let detail = if weighted {
                        format!(" ({} edges, weight {})", path.len() - 1, path_weight(&g, &path))
                    } else {
                        format!(" ({} edges)", path.len() - 1)
                    };
                    (
                        Some(format!(
                            "Shortest path from {} to {}: {}{}",
                            from,
                            to,
                            seq.join(" → "),
                            detail
                        )),
                        Some(path.clone()),
                    )
                };
                Ok(Some(graph_view_ir(
                    &g,
                    expression,
                    highlight,
                    None,
                    None,
                    None,
                    note,
                    false,
                )))
            }
            _ => Ok(None),
        }
    }

    // ----- expression evaluation --------------------------------------------

    fn eval(
        &mut self,
        e: &Expr,
        overrides: Option<&HashMap<String, bool>>,
    ) -> IResult<Value> {
        match e {
            Expr::Num { v, .. } => Ok(Value::Number(*v)),
            Expr::Bool { v, .. } => Ok(Value::Bool(*v)),
            Expr::SetLit { items, .. } => {
                let s: BTreeSet<SetElem> = items
                    .iter()
                    .map(item_to_elem)
                    .collect();
                Ok(Value::Set(s))
            }
            Expr::Var { name, pos } => {
                if let Some(ov) = overrides {
                    if let Some(b) = ov.get(name) {
                        return Ok(Value::Bool(*b));
                    }
                }
                self.env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| Diagnostic::error(pos.0, pos.1, format!("Undefined variable '{}'.", name)))
            }
            Expr::Call { func, args, pos } => {
                self.eval_call(*func, args, e, overrides, pos)
            }
            Expr::GraphLit { directed, vertices, edges, .. } => {
                let vs: BTreeSet<i64> = vertices.iter().copied().collect();
                let list = edges
                    .iter()
                    .map(|e| Edge { from: e.from, to: e.to, weight: e.weight })
                    .collect();
                Ok(Value::Graph(GraphVal::build(*directed, vs, list)))
            }
            Expr::Unary { op, arg, pos } => {
                let v = self.eval(arg, overrides)?;
                match op {
                    UnOp::Not => match v {
                        Value::Bool(b) => {
                            self.record_unary(e, &v, "NOT");
                            Ok(Value::Bool(!b))
                        }
                        other => Err(defensive_type_error("NOT", &other, pos)),
                    },
                    UnOp::Complement => match (&v, &self.universe) {
                        (Value::Set(s), Some(u)) => {
                            let comp: BTreeSet<SetElem> = u.difference(s).cloned().collect();
                            let detail = format!("UNIVERSE \\ {} = {}", set_to_string(u), set_to_string(s));
                            self.record(Intermediate {
                                expr: pretty(e),
                                result: set_to_string(&comp),
                                detail: Some(detail),
                            });
                            Ok(Value::Set(comp))
                        }
                        (Value::Set(_), None) => Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            "COMPLEMENT requires a universal set.",
                        )
                        .with_hint("Declare one first: UNIVERSE = {1, 2, 3, 4, 5, 6}")),
                        (other, _) => Err(defensive_type_error("COMPLEMENT", other, pos)),
                    },
                    UnOp::Neg => match v {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        other => Err(defensive_type_error("-", &other, pos)),
                    },
                }
            }
            Expr::Binary { op, lhs, rhs, pos } => {
                let lv = self.eval(lhs, overrides)?;
                let rv = self.eval(rhs, overrides)?;
                if op.is_logic() {
                    let (lb, rb) = match (&lv, &rv) {
                        (Value::Bool(a), Value::Bool(b)) => (*a, *b),
                        _ => return Err(defensive_type_error(op.symbol(), &if !matches!(lv, Value::Bool(_)) { lv } else { rv }, pos)),
                    };
                    let result = match op {
                        BinOp::And => lb && rb,
                        BinOp::Or => lb || rb,
                        BinOp::Implies => !lb || rb,
                        BinOp::Iff => lb == rb,
                        BinOp::Xor => lb != rb,
                        BinOp::Nand => !(lb && rb),
                        BinOp::Nor => !(lb || rb),
                        _ => unreachable!(),
                    };
                    self.record(Intermediate {
                        expr: pretty(e),
                        result: if result { "TRUE" } else { "FALSE" }.to_string(),
                        detail: Some(format!(
                            "{} {} {}",
                            if lb { "TRUE" } else { "FALSE" },
                            op.symbol(),
                            if rb { "TRUE" } else { "FALSE" }
                        )),
                    });
                    Ok(Value::Bool(result))
                } else if op.is_set_op() {
                    let (ls, rs) = match (&lv, &rv) {
                        (Value::Set(a), Value::Set(b)) => (a, b),
                        _ => {
                            let bad = if !matches!(lv, Value::Set(_)) { &lv } else { &rv };
                            return Err(defensive_type_error(op.symbol(), bad, pos));
                        }
                    };
                    let result: BTreeSet<SetElem> = match op {
                        BinOp::Union => ls | rs,
                        BinOp::Intersect => ls & rs,
                        BinOp::Difference => ls - rs,
                        BinOp::SymDiff => ls ^ rs,
                        BinOp::Cross => {
                            if ls.len().saturating_mul(rs.len()) > MAX_CROSS_PRODUCT {
                                return Err(Diagnostic::error(
                                    pos.0,
                                    pos.1,
                                    format!(
                                        "Cartesian product would contain {} pairs; the limit is {}.",
                                        ls.len() * rs.len(),
                                        MAX_CROSS_PRODUCT
                                    ),
                                ));
                            }
                            let mut acc = BTreeSet::new();
                            for a in ls {
                                for b in rs {
                                    acc.insert(SetElem::Tuple(vec![a.clone(), b.clone()]));
                                }
                            }
                            acc
                        }
                        _ => unreachable!(),
                    };
                    let result_display = set_to_string(&result);
                    self.record(Intermediate {
                        expr: pretty(e),
                        result: result_display.clone(),
                        detail: Some(format!(
                            "{} {} {}",
                            set_to_string(ls),
                            op.symbol(),
                            set_to_string(rs)
                        )),
                    });
                    Ok(Value::Set(result))
                } else {
                    // Subset / Equals
                    let (ls, rs) = match (&lv, &rv) {
                        (Value::Set(a), Value::Set(b)) => (a, b),
                        _ => {
                            let bad = if !matches!(lv, Value::Set(_)) { &lv } else { &rv };
                            return Err(defensive_type_error(op.symbol(), bad, pos));
                        }
                    };
                    let result = match op {
                        BinOp::Subset => ls.is_subset(rs),
                        BinOp::Equals => ls == rs,
                        _ => unreachable!(),
                    };
                    self.record(Intermediate {
                        expr: pretty(e),
                        result: if result { "TRUE" } else { "FALSE" }.to_string(),
                        detail: Some(format!(
                            "{} {} {}",
                            set_to_string(ls),
                            op.symbol(),
                            set_to_string(rs)
                        )),
                    });
                    Ok(Value::Bool(result))
                }
            }
        }
    }

    fn record(&mut self, item: Intermediate) {
        if self.intermediates.len() < MAX_INTERMEDIATES {
            self.intermediates.push(item);
        }
    }

    fn record_unary(&mut self, node: &Expr, operand: &Value, op: &str) {
        let detail = format!("{} {}", op, operand.display());
        self.record(Intermediate {
            expr: pretty(node),
            result: operand.display(),
            detail: Some(detail),
        });
    }
}

/// A set-literal item as a runtime element (nested sets are sorted and unique).
fn item_to_elem(i: &SetItem) -> SetElem {
    match i {
        SetItem::Num { v, .. } => SetElem::Num(*v),
        SetItem::Tuple { items, .. } => {
            SetElem::Tuple(items.iter().map(|v| SetElem::Num(*v)).collect())
        }
        SetItem::Set { items, .. } => {
            let s: BTreeSet<SetElem> = items.iter().map(item_to_elem).collect();
            SetElem::Set(s.into_iter().collect())
        }
    }
}

fn defensive_type_error(op: &str, got: &Value, pos: &Pos) -> Diagnostic {
    Diagnostic::error(
        pos.0,
        pos.1,
        format!(
            "Type error: '{}' cannot be applied to a value of type {}.",
            op,
            got.type_name()
        ),
    )
}

/// Which relation visualization to build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelationViz {
    Digraph,
    Hasse,
    Matrix,
}

impl RelationViz {
    fn name(self) -> &'static str {
        match self {
            RelationViz::Digraph => "DIGRAPH",
            RelationViz::Hasse => "HASSE",
            RelationViz::Matrix => "MATRIX",
        }
    }
}

fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

/// Extract a sorted, deduplicated pair list from a set value, requiring every
/// element to be an ordered pair of numbers (a relation).
fn relation_pairs(
    s: &BTreeSet<SetElem>,
    who: &str,
    what: &str,
    pos: Pos,
) -> IResult<Vec<(i64, i64)>> {
    let mut out = Vec::with_capacity(s.len());
    for e in s {
        match e {
            SetElem::Tuple(items) if items.len() == 2 => match (&items[0], &items[1]) {
                (SetElem::Num(a), SetElem::Num(b)) => out.push((*a, *b)),
                _ => return Err(relation_error(who, what, pos)),
            },
            _ => return Err(relation_error(who, what, pos)),
        }
    }
    Ok(out)
}

fn relation_error(who: &str, what: &str, pos: Pos) -> Diagnostic {
    Diagnostic::error(
        pos.0,
        pos.1,
        format!(
            "'{}' requires a relation (a set of ordered pairs), but '{}' contains non-pair elements.",
            who, what
        ),
    )
    .with_hint("Build a relation like R = {(1, 2), (2, 3)} or with A CROSS B.")
}

/// Turn a pair set back into a Set value of 2-tuples.
fn set_from_pairs(pairs: BTreeSet<(i64, i64)>) -> Value {
    Value::Set(
        pairs
            .into_iter()
            .map(|(a, b)| SetElem::Tuple(vec![SetElem::Num(a), SetElem::Num(b)]))
            .collect(),
    )
}

/// Convert a non-negative `i64` argument for the `u64`-based number theory
/// algorithms; negatives are rejected with a domain-specific message.
fn i64_to_u64(n: i64, pos: &Pos) -> IResult<u64> {
    if n < 0 {
        Err(Diagnostic::error(
            pos.0,
            pos.1,
            format!("IS_PRIME requires a non-negative integer, but got {}.", n),
        )
        .with_hint("Primality is defined for integers 0, 1, 2, …"))
    } else {
        Ok(n as u64)
    }
}

/// Require a strictly positive integer (1, 2, 3, …) for functions whose
/// mathematical domain starts there (FACTORS / DIVISORS / TOTIENT).
fn positive_int(n: i64, who: &str, _what: &str, pos: Pos) -> IResult<u64> {
    if n < 1 {
        Err(Diagnostic::error(
            pos.0,
            pos.1,
            format!("{} requires a positive integer, but got {}.", who, n),
        )
        .with_hint(format!(
            "{} is defined for the integers 1, 2, 3, …",
            who
        )))
    } else {
        Ok(n as u64)
    }
}

/// Generic out-of-range message for the number-theory builtins.
fn range_error(who: &str, got: i64, requirement: &str, pos: &Pos) -> Diagnostic {
    Diagnostic::error(
        pos.0,
        pos.1,
        format!("{} requires {} input, but got {}.", who, requirement, got),
    )
}

/// `k > n` in PERM / COMB: there is no way to choose k items from n.
fn k_exceeds_n_error(who: &str, n: i64, k: i64, pos: &Pos) -> Diagnostic {
    Diagnostic::error(
        pos.0,
        pos.1,
        format!("{} requires k ≤ n, but here k = {} and n = {}.", who, k, n),
    )
    .with_hint("You cannot choose more items than the set contains.")
}

/// A PERM / COMB whose value would not fit in a 64-bit integer.
fn overflow_error(who: &str, n: i64, k: i64, pos: &Pos) -> Diagnostic {
    Diagnostic::error(
        pos.0,
        pos.1,
        format!("{}({}, {}) exceeds the 64-bit integer range.", who, n, k),
    )
    .with_hint("Try smaller inputs: the result must fit in 64 bits.")
}

/// A VISUALIZE ... AS SIEVE / PASCAL whose expression is not the result of
/// the corresponding call (checked at runtime, where provenance is known).
fn provenance_error(kind: &str, expr: &Expr, recipe: &str) -> Diagnostic {
    let pos = expr.pos();
    Diagnostic::error(
        pos.0,
        pos.1,
        format!(
            "'{}' requires the result of a {} call, but '{}' is not one.",
            kind,
            kind,
            pretty(expr)
        ),
    )
    .with_hint(format!("Try {}.", recipe))
}

/// TOPO_SORT / IS_DAG on an undirected graph.
fn needs_digraph(who: &str, pos: &Pos) -> Diagnostic {
    Diagnostic::error(pos.0, pos.1, format!("{} needs a directed graph.", who))
        .with_hint("Write it as DIGRAPH {vertices} {edges}: an arrow only goes one way.")
}

/// Build a `Value::List` of numbers.
fn list_value(items: impl IntoIterator<Item = i64>) -> Value {
    Value::List(items.into_iter().map(SetElem::Num).collect())
}

/// Build the `Value::List` of row-tuples that represents Pascal's triangle:
/// `[(1), (1, 1), (1, 2, 1), …]`.
fn pascal_value(rows: usize) -> Value {
    Value::List(
        algo::pascal_rows(rows)
            .into_iter()
            .map(|row| SetElem::Tuple(row.into_iter().map(SetElem::Num).collect()))
            .collect(),
    )
}

fn algo_error(e: algo::AlgoError, pos: &Pos, what: &str) -> Diagnostic {
    match e {
        algo::AlgoError::NegativeWeight { from, to, weight } => Diagnostic::error(
            pos.0,
            pos.1,
            format!(
                "Edge weights must be non-negative, but edge ({}, {}) has weight {}.",
                from, to, weight
            ),
        )
        .with_hint("Shortest paths and spanning trees need non-negative weights."),
        algo::AlgoError::Disconnected => Diagnostic::error(
            pos.0,
            pos.1,
            format!("MST requires a connected graph, but '{}' is not connected.", what),
        )
        .with_hint("Add edges to connect all vertices."),
    }
}

/// Build the graph_view IR for a graph value.
#[allow(clippy::too_many_arguments)]
fn graph_view_ir(
    g: &GraphVal,
    expression: String,
    highlight_path: Option<Vec<i64>>,
    highlight_visited: Option<Vec<i64>>,
    colors: Option<BTreeMap<i64, i64>>,
    total_weight: Option<i64>,
    note: Option<String>,
    tree_edges: bool,
) -> VizIr {
    let edges = g
        .edges
        .iter()
        .map(|e| GraphEdgeView {
            from: e.from,
            to: e.to,
            weight: e.weight,
            highlight: if tree_edges { Some("tree") } else { None },
        })
        .collect();
    VizIr::GraphView(GraphViewIr {
        expression,
        directed: g.directed,
        vertices: g.vertices.iter().copied().collect(),
        edges,
        highlight_path,
        highlight_visited,
        colors,
        total_weight,
        note,
    })
}

/// Total weight of a path (unweighted edges count as 1).
fn path_weight(g: &GraphVal, path: &[i64]) -> i64 {
    let mut w: BTreeMap<(i64, i64), i64> = BTreeMap::new();
    for e in &g.edges {
        w.insert((e.from, e.to), e.weight.unwrap_or(1));
        if !g.directed {
            w.insert((e.to, e.from), e.weight.unwrap_or(1));
        }
    }
    path.windows(2).map(|p| w.get(&(p[0], p[1])).copied().unwrap_or(1)).sum()
}

fn op_symbol(op: &BinOp) -> &'static str {
    op.symbol()
}

fn op_label(op: &BinOp) -> &'static str {
    match op {
        BinOp::Union => "Union",
        BinOp::Intersect => "Intersection",
        BinOp::Difference => "Difference",
        BinOp::SymDiff => "Symmetric difference",
        BinOp::Cross => "Cartesian product",
        BinOp::And => "Conjunction",
        BinOp::Or => "Disjunction",
        BinOp::Implies => "Implication",
        BinOp::Iff => "Biconditional",
        BinOp::Xor => "Exclusive or",
        BinOp::Nand => "Not-and",
        BinOp::Nor => "Not-or",
        BinOp::Subset => "Subset",
        BinOp::Equals => "Equality",
    }
}

/// Ensure every element reachable in the expression is a plain number, since
/// Venn region elements are placed on the diagram.
fn check_venn_numeric(expr: &Expr, interp: &Interpreter) -> IResult<()> {
    let mut bad: Option<(String, Pos)> = None;
    expr.walk(&mut |sub| {
        if bad.is_some() {
            return;
        }
        if let Expr::Var { name, pos } = sub {
            if let Some(Value::Set(s)) = interp.env.get(name) {
                if s.iter().any(|e| e.is_tuple()) {
                    bad = Some((name.clone(), *pos));
                }
            }
        }
    });
    match bad {
        Some((name, pos)) => Err(Diagnostic::error(
            pos.0,
            pos.1,
            format!(
                "Type error: VENN diagrams visualize sets of numbers, but '{}' contains ordered pairs (produced by CROSS).",
                name
            ),
        )
        .with_hint("Display Cartesian products with DISPLAY instead.")),
        None => Ok(()),
    }
}

fn truncate_regions(regions: VennRegions) -> VennRegions {
    fn trunc(v: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
        if v.len() > MAX_VENN_ELEMENTS_PER_REGION {
            v.into_iter().take(MAX_VENN_ELEMENTS_PER_REGION).collect()
        } else {
            v
        }
    }
    VennRegions {
        left_only: trunc(regions.left_only),
        intersection: trunc(regions.intersection),
        right_only: trunc(regions.right_only),
        outside: trunc(regions.outside),
    }
}

fn regions_truncated(r: &VennRegions) -> bool {
    [r.left_only.len(), r.intersection.len(), r.right_only.len(), r.outside.len()]
        .iter()
        .any(|&n| n == MAX_VENN_ELEMENTS_PER_REGION)
}

/// Run a full program and produce the result envelope.
pub fn run_program(
    parse_diags: Vec<Diagnostic>,
    program: &crate::ast::Program,
) -> Envelope {
    let mut diags: Vec<DiagJson> = parse_diags.iter().cloned().map(DiagJson::from).collect();
    let mut outputs = Vec::new();
    let mut visualizations = Vec::new();
    let mut trace = Vec::new();

    let mut types: SymTable = HashMap::new();
    let mut defs: crate::semantic::DefTable = HashMap::new();
    let mut origins: crate::semantic::OriginTable = HashMap::new();
    let mut interp = Interpreter::new();

    for (idx, stmt) in program.stmts.iter().enumerate() {
        let step = idx + 1;
        let statement = stmt_text(stmt);
        let kind = stmt_kind(stmt);

        let checked = match crate::semantic::check_statement(stmt, &types, &defs, &origins) {
            Ok(c) => c,
            Err(d) => {
                diags.push(DiagJson::from(d));
                trace.push(TraceStep {
                    step,
                    statement,
                    kind,
                    status: "error",
                    summary: "Statement rejected by the semantic analyzer.".to_string(),
                    intermediates: Vec::new(),
                });
                continue;
            }
        };

        match interp.exec(&checked) {
            Ok(outcome) => {
                outputs.extend(outcome.outputs);
                visualizations.extend(outcome.visualizations);
                for w in outcome.warnings {
                    diags.push(DiagJson::from(w));
                }
                if let CheckedStmt::Assign { name, dtype, expr } = &checked {
                    types.insert(name.clone(), *dtype);
                    if *dtype == DType::Bool {
                        defs.insert(name.clone(), expr.clone());
                    }
                    // Track COLOR / MST provenance for VISUALIZE ... AS GRAPH,
                    // and COMB provenance for VISUALIZE ... AS PASCAL.
                    let origin = match expr {
                        Expr::Call { func: Builtin::Color, .. } => {
                            crate::semantic::ValueOrigin::Color
                        }
                        Expr::Call { func: Builtin::Mst, .. } => {
                            crate::semantic::ValueOrigin::Mst
                        }
                        Expr::Call { func: Builtin::Comb, .. } => {
                            crate::semantic::ValueOrigin::Comb
                        }
                        Expr::Var { name: src, .. } => origins
                            .get(src)
                            .copied()
                            .unwrap_or(crate::semantic::ValueOrigin::Plain),
                        _ => crate::semantic::ValueOrigin::Plain,
                    };
                    origins.insert(name.clone(), origin);
                }
                trace.push(TraceStep {
                    step,
                    statement,
                    kind,
                    status: "ok",
                    summary: outcome.summary,
                    intermediates: outcome.intermediates,
                });
            }
            Err(d) => {
                diags.push(DiagJson::from(d));
                trace.push(TraceStep {
                    step,
                    statement,
                    kind,
                    status: "error",
                    summary: "Statement failed during evaluation.".to_string(),
                    intermediates: std::mem::take(&mut interp.intermediates),
                });
            }
        }
    }

    let ok = !diags.iter().any(|d| matches!(d.severity, crate::token::Severity::Error));
    Envelope {
        contract: CONTRACT_VERSION,
        ok,
        outputs,
        visualizations,
        trace,
        diagnostics: diags,
    }
}

pub fn stmt_text(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Assign { name, expr, .. } => format!("{} = {}", name, pretty(expr)),
        Stmt::Display { exprs, .. } => {
            let parts: Vec<String> = exprs.iter().map(pretty).collect();
            format!("DISPLAY {}", parts.join(", "))
        }
        Stmt::Visualize { expr, kind, .. } => {
            format!("VISUALIZE {} AS {}", pretty(expr), kind.name())
        }
        Stmt::CheckEquivalent { left, right, .. } => {
            format!("CHECK EQUIVALENT {}, {}", pretty(left), pretty(right))
        }
        Stmt::CheckProperties { expr, .. } => format!("CHECK PROPERTIES {}", pretty(expr)),
    }
}

pub fn stmt_kind(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::Assign { .. } => "assignment",
        Stmt::Display { .. } => "display",
        Stmt::Visualize { .. } => "visualize",
        Stmt::CheckEquivalent { .. } => "check_equivalent",
        Stmt::CheckProperties { .. } => "check_properties",
    }
}

#[allow(dead_code)]
fn _unused(_: &DType, _: &Pos, _: &VizKind) {}

