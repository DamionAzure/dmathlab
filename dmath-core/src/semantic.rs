//! Semantic analysis: type checking and static validation of statements.
//!
//! Statements are checked in program order against the symbol table of the
//! variables declared so far — mirroring the sequential semantics of the DSL.
//! A successfully checked statement is turned into a `CheckedStmt`, which
//! carries the facts the interpreter needs (inferred types, free propositional
//! variables for truth tables, and so on).

use std::collections::{HashMap, HashSet};

use crate::ast::{BinOp, Expr, SetItem, Stmt, UnOp, VizKind};
use crate::token::{Builtin, Diagnostic};
use crate::value::DType;

/// Maximum number of distinct propositional variables in a truth table
/// (2^12 = 4096 rows). Keeps NFR-06 guarantees trivially satisfiable.
pub const MAX_TRUTH_TABLE_VARS: usize = 12;

/// Where a variable's value came from, used to allow `VISUALIZE C AS GRAPH`
/// when `C = COLOR(G)` holds a coloring instead of a graph, and
/// `VISUALIZE C AS PASCAL` when `C = COMB(n, k)` holds a plain number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueOrigin {
    Plain,
    Mst,
    Color,
    Comb,
}

pub type SymTable = HashMap<String, DType>;
pub type OriginTable = HashMap<String, ValueOrigin>;

pub enum CheckedStmt {
    Assign { name: String, dtype: DType, expr: Expr },
    Display { items: Vec<(Expr, DType)> },
    Visualize { expr: Expr, dtype: DType, kind: VizKind, free_bool_vars: Vec<String> },
    CheckEquivalent { left: Expr, right: Expr, vars: Vec<String> },
    CheckProperties { expr: Expr },
}

/// Definitions of Boolean variables (name -> defining expression). Used to
/// expand compound propositions for truth-table analysis: this is how the
/// `Expression` value type of the spec is realized — Boolean variables carry
/// their symbolic expression so tables can enumerate their free variables.
pub type DefTable = HashMap<String, Expr>;

pub fn check_statement(
    stmt: &Stmt,
    types: &SymTable,
    defs: &DefTable,
    origins: &OriginTable,
) -> Result<CheckedStmt, Diagnostic> {
    match stmt {
        Stmt::Assign { name, expr, .. } => {
            let dtype = infer(expr, types)?;
            Ok(CheckedStmt::Assign { name: name.clone(), dtype, expr: expr.clone() })
        }
        Stmt::Display { exprs, .. } => {
            let mut items = Vec::new();
            for e in exprs {
                let t = infer(e, types)?;
                items.push((e.clone(), t));
            }
            Ok(CheckedStmt::Display { items })
        }
        Stmt::Visualize { expr, kind, pos } => {
            let dtype = infer(expr, types)?;
            match kind {
                VizKind::TruthTable => {
                    if dtype != DType::Bool {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "Type error: TRUTH_TABLE requires a Boolean (propositional) expression, but this expression has type {}.",
                                dtype.name()
                            ),
                        )
                        .with_hint("Write propositional formulas with AND, OR, NOT, XOR, NAND, NOR, IMPLIES, IFF."));
                    }
                    // Expand compound Boolean variables into their symbolic
                    // expressions so the table enumerates the real free
                    // variables (spec §2.1: P and Q are columns even when
                    // they were assigned concrete values).
                    let expanded = expand_bool_expr(expr, types, defs, &mut HashSet::new());
                    let mut vars = Vec::new();
                    collect_bool_vars(&expanded, types, &mut vars);
                    if vars.len() > MAX_TRUTH_TABLE_VARS {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "Truth table would need 2^{} = {} rows for {} variables; the limit is {} variables ({} rows).",
                                vars.len(),
                                1usize << vars.len(),
                                vars.len(),
                                MAX_TRUTH_TABLE_VARS,
                                1usize << MAX_TRUTH_TABLE_VARS
                            ),
                        ));
                    }
                    Ok(CheckedStmt::Visualize {
                        expr: expanded,
                        dtype,
                        kind: *kind,
                        free_bool_vars: vars,
                    })
                }
                VizKind::Venn => {
                    if dtype != DType::Set {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "Type error: VENN requires a set expression, but this expression has type {}.",
                                dtype.name()
                            ),
                        )
                        .with_hint("Venn diagrams visualize set relationships."));
                    }
                    Ok(CheckedStmt::Visualize {
                        expr: expr.clone(),
                        dtype,
                        kind: *kind,
                        free_bool_vars: Vec::new(),
                    })
                }
                VizKind::Digraph | VizKind::Hasse | VizKind::Matrix => {
                    // MATRIX also draws a graph's adjacency matrix.
                    if *kind == VizKind::Matrix && dtype == DType::Graph {
                        return Ok(CheckedStmt::Visualize {
                            expr: expr.clone(),
                            dtype,
                            kind: *kind,
                            free_bool_vars: Vec::new(),
                        });
                    }
                    if dtype != DType::Set {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "'{}' requires a relation (a set of ordered pairs), but '{}' has type {}.",
                                kind.name(),
                                describe_operand(expr),
                                dtype.name()
                            ),
                        )
                        .with_hint(
                            "Build a relation like R = {(1, 2), (2, 3)} or with A CROSS B.",
                        ));
                    }
                    Ok(CheckedStmt::Visualize {
                        expr: expr.clone(),
                        dtype,
                        kind: *kind,
                        free_bool_vars: Vec::new(),
                    })
                }
                VizKind::Graph => {
                    // A COLOR result (a Mapping with a known source graph) is
                    // also acceptable: the view shows the graph with colors.
                    let is_coloring =
                        expr_origin(expr, origins) == ValueOrigin::Color && dtype == DType::Mapping;
                    if dtype != DType::Graph && !is_coloring {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "'GRAPH' requires a graph value, but '{}' has type {}.",
                                describe_operand(expr),
                                dtype.name()
                            ),
                        )
                        .with_hint("Build one with G = GRAPH { ... } { ... }."));
                    }
                    Ok(CheckedStmt::Visualize {
                        expr: expr.clone(),
                        dtype,
                        kind: *kind,
                        free_bool_vars: Vec::new(),
                    })
                }
                VizKind::Sieve => {
                    if dtype != DType::List {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "'SIEVE' requires the result of a SIEVE call, but '{}' has type {}.",
                                describe_operand(expr),
                                dtype.name()
                            ),
                        )
                        .with_hint("Try S = SIEVE(30) first, then VISUALIZE S AS SIEVE."));
                    }
                    // The SIEVE provenance itself is verified at runtime.
                    Ok(CheckedStmt::Visualize {
                        expr: expr.clone(),
                        dtype,
                        kind: *kind,
                        free_bool_vars: Vec::new(),
                    })
                }
                VizKind::Steps => {
                    // Which call produced the value is verified at runtime
                    // (same provenance approach as SIEVE / PASCAL).
                    Ok(CheckedStmt::Visualize {
                        expr: expr.clone(),
                        dtype,
                        kind: *kind,
                        free_bool_vars: Vec::new(),
                    })
                }
                VizKind::Pascal => {
                    // A COMB(n, k) result (a plain Number with known
                    // provenance) is also acceptable: the view highlights
                    // the C(n, k) cell inside the triangle.
                    let is_comb =
                        expr_origin(expr, origins) == ValueOrigin::Comb && dtype == DType::Number;
                    if dtype != DType::List && !is_comb {
                        return Err(Diagnostic::error(
                            pos.0,
                            pos.1,
                            format!(
                                "'PASCAL' requires the result of a PASCAL or COMB call, but '{}' has type {}.",
                                describe_operand(expr),
                                dtype.name()
                            ),
                        )
                        .with_hint(
                            "Try P = PASCAL(5) or C = COMB(5, 2), then VISUALIZE P AS PASCAL.",
                        ));
                    }
                    Ok(CheckedStmt::Visualize {
                        expr: expr.clone(),
                        dtype,
                        kind: *kind,
                        free_bool_vars: Vec::new(),
                    })
                }
            }
        }
        Stmt::CheckEquivalent { left, right, pos } => {
            let lt = infer(left, types)?;
            let rt = infer(right, types)?;
            if lt != DType::Bool {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "Type error: EQUIVALENT compares propositional expressions, but the left expression has type {}.",
                        lt.name()
                    ),
                )
                .with_hint("For sets, use 'A EQUALS B' or 'A SUBSET B' instead."));
            }
            if rt != DType::Bool {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "Type error: EQUIVALENT compares propositional expressions, but the right expression has type {}.",
                        rt.name()
                    ),
                )
                .with_hint("For sets, use 'A EQUALS B' or 'A SUBSET B' instead."));
            }
            let mut vars = Vec::new();
            let lexpanded = expand_bool_expr(left, types, defs, &mut HashSet::new());
            let rexpanded = expand_bool_expr(right, types, defs, &mut HashSet::new());
            collect_bool_vars(&lexpanded, types, &mut vars);
            collect_bool_vars(&rexpanded, types, &mut vars);
            if vars.len() > MAX_TRUTH_TABLE_VARS {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "Equivalence check would need 2^{} rows; the limit is {} variables.",
                        vars.len(),
                        MAX_TRUTH_TABLE_VARS
                    ),
                ));
            }
            Ok(CheckedStmt::CheckEquivalent {
                left: lexpanded,
                right: rexpanded,
                vars,
            })
        }
        Stmt::CheckProperties { expr, pos } => {
            let dtype = infer(expr, types)?;
            if !matches!(dtype, DType::Set | DType::Graph) {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "'CHECK PROPERTIES' expects a relation or a graph, but '{}' has type {}.",
                        describe_operand(expr),
                        dtype.name()
                    ),
                )
                .with_hint(
                    "Use a relation like R = {(1, 2), (2, 3)} or a graph like G = GRAPH { ... } { ... }.",
                ));
            }
            Ok(CheckedStmt::CheckProperties { expr: expr.clone() })
        }
    }
}

/// Resolve the origin of an expression's value (for COLOR/MST provenance).
fn expr_origin(e: &Expr, origins: &OriginTable) -> ValueOrigin {
    match e {
        Expr::Call { func: Builtin::Color, .. } => ValueOrigin::Color,
        Expr::Call { func: Builtin::Mst, .. } => ValueOrigin::Mst,
        Expr::Call { func: Builtin::Comb, .. } => ValueOrigin::Comb,
        Expr::Var { name, .. } => origins.get(name).copied().unwrap_or(ValueOrigin::Plain),
        _ => ValueOrigin::Plain,
    }
}

// ===========================================================================
// Builtin signature table
// ===========================================================================

/// The static parameter kinds of the built-in functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Param {
    /// A set (at runtime required to be a relation: only ordered pairs).
    Relation,
    /// Any set (numbers or pairs), unlike `Relation` which needs ordered pairs.
    SetAny,
    /// A number or a set, used where either can be an element of a set.
    Element,
    /// A list of digits (from TO_BASE) or a number whose decimal digits are read.
    Digits,
    Graph,
    Number,
}

impl Param {
    pub fn name(self) -> &'static str {
        match self {
            Param::Relation => "relation",
            Param::SetAny => "set",
            Param::Element => "element",
            Param::Digits => "digits",
            Param::Graph => "Graph",
            Param::Number => "vertex",
        }
    }
}

/// Static signature: parameter kinds and result type.
pub fn builtin_signature(f: Builtin) -> (&'static [Param], DType) {
    const R: &[Param] = &[Param::Relation];
    const RR: &[Param] = &[Param::Relation, Param::Relation];
    const S: &[Param] = &[Param::SetAny];
    const SS: &[Param] = &[Param::SetAny, Param::SetAny];
    const SE: &[Param] = &[Param::SetAny, Param::Element];
    const RS: &[Param] = &[Param::Relation, Param::SetAny];
    const RN: &[Param] = &[Param::Relation, Param::Number];
    const DN: &[Param] = &[Param::Digits, Param::Number];
    const G: &[Param] = &[Param::Graph];
    const GN: &[Param] = &[Param::Graph, Param::Number];
    const GNN: &[Param] = &[Param::Graph, Param::Number, Param::Number];
    const N: &[Param] = &[Param::Number];
    const NN: &[Param] = &[Param::Number, Param::Number];
    const NNN: &[Param] = &[Param::Number, Param::Number, Param::Number];
    const NNNN: &[Param] = &[Param::Number, Param::Number, Param::Number, Param::Number];
    match f {
        Builtin::Inverse => (R, DType::Set),
        Builtin::Compose => (RR, DType::Set),
        Builtin::TransitiveClosure => (R, DType::Set),
        Builtin::ReflexiveClosure => (R, DType::Set),
        Builtin::SymmetricClosure => (R, DType::Set),
        Builtin::IsReflexive => (R, DType::Bool),
        Builtin::IsSymmetric => (R, DType::Bool),
        Builtin::IsAntisymmetric => (R, DType::Bool),
        Builtin::IsTransitive => (R, DType::Bool),
        Builtin::IsEquivalence => (R, DType::Bool),
        Builtin::IsPartialOrder => (R, DType::Bool),
        Builtin::Vertices => (G, DType::Set),
        Builtin::Edges => (G, DType::Set),
        Builtin::Degree => (GN, DType::Number),
        Builtin::Neighbors => (GN, DType::List),
        Builtin::Adjacent => (GNN, DType::Bool),
        Builtin::Bfs => (GN, DType::List),
        Builtin::Dfs => (GN, DType::List),
        Builtin::ShortestPath => (GNN, DType::List),
        Builtin::IsConnected => (G, DType::Bool),
        Builtin::IsTree => (G, DType::Bool),
        Builtin::IsBipartite => (G, DType::Bool),
        Builtin::IsComplete => (G, DType::Bool),
        Builtin::Color => (G, DType::Mapping),
        Builtin::Mst => (G, DType::Graph),
        Builtin::IsPrime => (N, DType::Bool),
        Builtin::Factors => (N, DType::List),
        Builtin::Divisors => (N, DType::List),
        Builtin::Gcd => (NN, DType::Number),
        Builtin::Lcm => (NN, DType::Number),
        Builtin::Totient => (N, DType::Number),
        Builtin::Sieve => (N, DType::List),
        Builtin::Fact => (N, DType::Number),
        Builtin::Perm => (NN, DType::Number),
        Builtin::Comb => (NN, DType::Number),
        Builtin::Pascal => (N, DType::List),
        Builtin::Mod => (NN, DType::Number),
        Builtin::ModPow => (NNN, DType::Number),
        Builtin::ModInv => (NN, DType::Number),
        Builtin::Egcd => (NN, DType::List),
        Builtin::Crt => (NNNN, DType::List),
        Builtin::Fib => (N, DType::Number),
        Builtin::Catalan => (N, DType::Number),
        Builtin::Derange => (N, DType::Number),
        Builtin::Stirling2 => (NN, DType::Number),
        Builtin::TopoSort => (G, DType::List),
        Builtin::Size => (S, DType::Number),
        Builtin::Contains => (SE, DType::Bool),
        Builtin::Powerset => (S, DType::Set),
        Builtin::IsDisjoint => (SS, DType::Bool),
        Builtin::IsDag => (G, DType::Bool),
        Builtin::Components => (G, DType::Mapping),
        Builtin::IsEulerian => (G, DType::Bool),
        Builtin::DegreeSequence => (G, DType::List),
        Builtin::Min | Builtin::Max | Builtin::Sum => (S, DType::Number),
        Builtin::IsEmpty => (S, DType::Bool),
        Builtin::Domain | Builtin::Range => (R, DType::Set),
        Builtin::Image => (RS, DType::Set),
        Builtin::IsFunction | Builtin::IsSurjective | Builtin::IsBijective => (RS, DType::Bool),
        Builtin::IsInjective => (R, DType::Bool),
        Builtin::Classes => (R, DType::Set),
        Builtin::ClassOf => (RN, DType::Set),
        Builtin::GraphComplement => (G, DType::Graph),
        Builtin::Diameter | Builtin::ChromaticNumber => (G, DType::Number),
        Builtin::Bridges | Builtin::CutVertices => (G, DType::Set),
        Builtin::IsHamiltonian => (G, DType::Bool),
        Builtin::Sigma | Builtin::Partitions | Builtin::Bell => (N, DType::Number),
        Builtin::OrderMod => (NN, DType::Number),
        Builtin::ToBase => (NN, DType::List),
        Builtin::FromBase => (DN, DType::Number),
    }
}

/// Human-readable usage hint for a builtin, e.g. `BFS(Graph, start vertex)`.
pub fn builtin_usage(f: Builtin) -> String {
    let (params, _) = builtin_signature(f);
    let names: Vec<String> = match f {
        Builtin::Bfs | Builtin::Dfs => vec!["Graph".into(), "start vertex".into()],
        Builtin::ShortestPath => {
            vec!["Graph".into(), "from vertex".into(), "to vertex".into()]
        }
        Builtin::Degree => vec!["Graph".into(), "vertex".into()],
        Builtin::Neighbors => vec!["Graph".into(), "vertex".into()],
        Builtin::Adjacent => vec!["Graph".into(), "vertex".into(), "vertex".into()],
        Builtin::IsPrime | Builtin::Totient => vec!["non-negative integer".into()],
        Builtin::Factors | Builtin::Divisors => vec!["positive integer".into()],
        Builtin::Sieve => vec!["non-negative integer".into()],
        Builtin::Fact => vec!["n (0 to 20)".into()],
        Builtin::Perm | Builtin::Comb => vec!["n".into(), "k".into()],
        Builtin::Pascal => vec!["rows (1 to 25)".into()],
        Builtin::Mod => vec!["a".into(), "m".into()],
        Builtin::ModPow => vec!["base".into(), "exponent".into(), "m".into()],
        Builtin::ModInv => vec!["a".into(), "m".into()],
        Builtin::Egcd => vec!["a".into(), "b".into()],
        Builtin::Contains => vec!["set".into(), "element".into()],
        Builtin::Image => vec!["relation".into(), "input set".into()],
        Builtin::IsFunction => vec!["relation".into(), "domain set".into()],
        Builtin::IsSurjective | Builtin::IsBijective => vec!["relation".into(), "codomain set".into()],
        Builtin::ClassOf => vec!["equivalence relation".into(), "element".into()],
        Builtin::Sigma | Builtin::Partitions | Builtin::Bell => vec!["non-negative integer n".into()],
        Builtin::OrderMod => vec!["a".into(), "m".into()],
        Builtin::ToBase => vec!["n".into(), "base (2 to 36)".into()],
        Builtin::FromBase => vec!["digits".into(), "base (2 to 36)".into()],
        Builtin::Crt => vec!["a1".into(), "m1".into(), "a2".into(), "m2".into()],
        Builtin::Stirling2 => vec!["n".into(), "k".into()],
        _ => params.iter().map(|p| p.name().to_string()).collect(),
    };
    format!("{}({})", f.name(), names.join(", "))
}

// ===========================================================================
// Type inference
// ===========================================================================

/// Infer the static type of an expression.
pub fn infer(e: &Expr, types: &SymTable) -> Result<DType, Diagnostic> {
    match e {
        Expr::Num { .. } => Ok(DType::Number),
        Expr::Bool { .. } => Ok(DType::Bool),
        Expr::SetLit { items, .. } => {
            // A set literal must not mix bare numbers and ordered pairs.
            if let Some(first) = items.first() {
                let kind0 = set_item_kind(first);
                for item in items {
                    if set_item_kind(item) != kind0 {
                        return Err(Diagnostic::error(
                            item.pos().0,
                            item.pos().1,
                            if kind0 == 2 || set_item_kind(item) == 2 {
                                "Set literal cannot mix nested sets with numbers or ordered pairs."
                            } else {
                                "Set literal cannot mix numbers and ordered pairs."
                            },
                        )
                        .with_hint(
                            "Use either all numbers, e.g. {1, 2, 3}, or all pairs, e.g. {(1, 2), (2, 3)}.",
                        ));
                    }
                }
            }
            Ok(DType::Set)
        }
        Expr::GraphLit { vertices, edges, .. } => {
            let vs: std::collections::BTreeSet<i64> = vertices.iter().copied().collect();
            for e in edges {
                let missing = if !vs.contains(&e.from) {
                    Some(e.from)
                } else if !vs.contains(&e.to) {
                    Some(e.to)
                } else {
                    None
                };
                if let Some(m) = missing {
                    return Err(Diagnostic::error(
                        e.pos.0,
                        e.pos.1,
                        format!(
                            "Edge ({}, {}) references vertex {}, which is not in the vertex set.",
                            e.from, e.to, m
                        ),
                    )
                    .with_hint(format!("Add {} to the vertex set or remove the edge.", m)));
                }
                if let Some(w) = e.weight {
                    if w < 0 {
                        return Err(Diagnostic::error(
                            e.pos.0,
                            e.pos.1,
                            format!(
                                "Edge weights must be non-negative, but edge ({}, {}) has weight {}.",
                                e.from, e.to, w
                            ),
                        )
                        .with_hint("Use a weight of 0 or greater."));
                    }
                }
            }
            Ok(DType::Graph)
        }
        Expr::Call { func, args, pos } => {
            let (params, ret) = builtin_signature(*func);
            if args.len() != params.len() {
                let plural = if params.len() == 1 { "" } else { "s" };
                let given = if args.len() == 1 { "was" } else { "were" };
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "'{}' expects {} argument{}, but {} {} given.",
                        func.name(),
                        params.len(),
                        plural,
                        args.len(),
                        given
                    ),
                )
                .with_hint(format!("Usage: {}.", builtin_usage(*func))));
            }
            for (i, arg) in args.iter().enumerate() {
                let t = infer(arg, types)?;
                let want = params[i];
                let ok = match want {
                    Param::Relation | Param::SetAny => t == DType::Set,
                    Param::Element => t == DType::Set || t == DType::Number,
                    Param::Digits => t == DType::List || t == DType::Number,
                    Param::Graph => t == DType::Graph,
                    Param::Number => t == DType::Number,
                };
                if !ok {
                    let (message, hint) = match want {
                        Param::Relation => (
                            format!(
                                "'{}' requires a relation (a set of ordered pairs), but '{}' has type {}.",
                                func.name(),
                                describe_operand(arg),
                                t.name()
                            ),
                            "Build a relation like R = {(1, 2), (2, 3)} or with A CROSS B.".to_string(),
                        ),
                        Param::Digits => (
                            format!(
                                "'{}' needs a list of digits (from TO_BASE) or a number, but '{}' has type {}.",
                                func.name(),
                                describe_operand(arg),
                                t.name()
                            ),
                            format!("Usage: {}.", builtin_usage(*func)),
                        ),
                        Param::Element => (
                            format!(
                                "'{}' needs a number or a set as the element, but '{}' has type {}.",
                                func.name(),
                                describe_operand(arg),
                                t.name()
                            ),
                            format!("Usage: {}.", builtin_usage(*func)),
                        ),
                        Param::SetAny => (
                            format!(
                                "'{}' requires a set, but '{}' has type {}.",
                                func.name(),
                                describe_operand(arg),
                                t.name()
                            ),
                            format!("Usage: {}. Build a set like A = {{1, 2, 3}}.", builtin_usage(*func)),
                        ),
                        Param::Graph => (
                            format!(
                                "'{}' requires a Graph, but '{}' has type {}.",
                                func.name(),
                                describe_operand(arg),
                                t.name()
                            ),
                            "Create a graph with G = GRAPH { ... } { ... }.".to_string(),
                        ),
                        Param::Number => (
                            format!(
                                "'{}' requires a Number, but '{}' has type {}.",
                                func.name(),
                                describe_operand(arg),
                                t.name()
                            ),
                            format!("Usage: {}.", builtin_usage(*func)),
                        ),
                    };
                    return Err(Diagnostic::error(pos.0, pos.1, message).with_hint(hint));
                }
            }
            Ok(ret)
        }
        Expr::Var { name, pos } => match types.get(name) {
            Some(t) => Ok(*t),
            None => Err(Diagnostic::error(
                pos.0,
                pos.1,
                format!("Undefined variable '{}'.", name),
            )
            .with_hint(format!(
                "Declare it first, e.g. {} = <expression>.",
                name
            ))),
        },
        Expr::Unary { op, arg, pos } => {
            let t = infer(arg, types)?;
            match op {
                UnOp::Not => {
                    if t == DType::Bool {
                        Ok(DType::Bool)
                    } else {
                        Err(type_error_unary("NOT", "Boolean", arg, t, pos.0, pos.1))
                    }
                }
                UnOp::Complement => {
                    if t == DType::Set {
                        ensure_universe_declared(types, pos.0, pos.1)?;
                        Ok(DType::Set)
                    } else {
                        Err(type_error_unary("COMPLEMENT", "Set", arg, t, pos.0, pos.1))
                    }
                }
                UnOp::Neg => {
                    if t == DType::Number {
                        Ok(DType::Number)
                    } else {
                        Err(type_error_unary("-", "Number", arg, t, pos.0, pos.1))
                    }
                }
            }
        }
        Expr::Binary { op, lhs, rhs, pos } => {
            let lt = infer(lhs, types)?;
            if mismatch(*op, lt, false) {
                return Err(type_error_binary(*op, lhs, lt, pos.0, pos.1));
            }
            let rt = infer(rhs, types)?;
            if mismatch(*op, rt, true) {
                return Err(type_error_binary(*op, rhs, rt, pos.0, pos.1));
            }
            if matches!(op, BinOp::Eq | BinOp::Ne) && lt != rt {
                return Err(Diagnostic::error(
                    pos.0,
                    pos.1,
                    format!(
                        "Type error: '{}' compares two values of the same type, but got {} and {}.",
                        op.symbol(),
                        lt.name(),
                        rt.name()
                    ),
                )
                .with_hint("Compare two numbers or two sets; use IN to test membership."));
            }
            Ok(result_type(*op))
        }
    }
}

fn set_item_kind(item: &SetItem) -> u8 {
    match item {
        SetItem::Num { .. } => 0,
        SetItem::Tuple { .. } => 1,
        SetItem::Set { .. } => 2,
    }
}

fn mismatch(op: BinOp, t: DType, rhs_side: bool) -> bool {
    if op.is_logic() {
        t != DType::Bool
    } else if op.is_set_op() {
        t != DType::Set
    } else if op.is_arith() || op.is_ordering() {
        t != DType::Number
    } else if matches!(op, BinOp::Eq | BinOp::Ne) {
        t != DType::Number && t != DType::Set
    } else if op == BinOp::In && !rhs_side {
        t != DType::Number && t != DType::Set
    } else {
        // Subset / PSubset / Equals / IN (right side)
        t != DType::Set
    }
}

fn result_type(op: BinOp) -> DType {
    if op.is_arith() {
        DType::Number
    } else if op.is_comparison() {
        DType::Bool
    } else if op.is_set_op() {
        DType::Set
    } else {
        DType::Bool
    }
}

/// Human-readable description of an operand in error messages: the variable
/// name when possible, otherwise a compact rendering of the expression.
pub fn describe_operand(e: &Expr) -> String {
    let s = crate::pretty::pretty(e);
    crate::pretty::ellipsize(&s, 28)
}

fn type_error_binary(
    op: BinOp,
    operand: &Expr,
    got: DType,
    line: usize,
    col: usize,
) -> Diagnostic {
    let what = describe_operand(operand);
    let mut d = Diagnostic::error(
        line,
        col,
        format!(
            "Type error: '{}' requires {} operands, but '{}' has type {}.",
            op.symbol(),
            if op.is_arith() || op.is_ordering() {
                "Number"
            } else if matches!(op, BinOp::Eq | BinOp::Ne) {
                "Number or set"
            } else if op.is_set_op() || op.is_comparison() {
                "set"
            } else {
                "Boolean"
            },
            what,
            got.name()
        ),
    );
    if op == BinOp::Xor && got == DType::Set {
        d = d.with_hint("XOR works on Booleans. For sets use SYMDIFF (elements in exactly one of the two sets).");
    } else if op.is_logic() && got == DType::Set {
        d = d.with_hint(format!(
            "Sets do not support '{}'. Did you mean a set operation like UNION or INTERSECT?",
            op.symbol()
        ));
    }
    if op.is_comparison() && !op.is_ordering() && !matches!(op, BinOp::Eq | BinOp::Ne) && got == DType::Bool {
        d = d.with_hint(format!(
            "For propositions use IFF (or CHECK EQUIVALENT) instead of '{}'.",
            op.symbol()
        ));
    }
    d
}

fn type_error_unary(
    op: &str,
    expected: &str,
    operand: &Expr,
    got: DType,
    line: usize,
    col: usize,
) -> Diagnostic {
    let what = describe_operand(operand);
    Diagnostic::error(
        line,
        col,
        format!(
            "Type error: '{}' requires a {} operand, but '{}' has type {}.",
            op, expected, what, got.name()
        ),
    )
}

fn ensure_universe_declared(types: &SymTable, line: usize, col: usize) -> Result<(), Diagnostic> {
    match types.get("UNIVERSE") {
        Some(DType::Set) => Ok(()),
        Some(other) => Err(Diagnostic::error(
            line,
            col,
            format!(
                "COMPLEMENT requires UNIVERSE to be a set, but it has type {}.",
                other.name()
            ),
        )
        .with_hint("Reassign it, e.g. UNIVERSE = {1, 2, 3, 4, 5, 6}.")),
        None => Err(Diagnostic::error(
            line,
            col,
            "COMPLEMENT requires a universal set.",
        )
        .with_hint("Declare one first: UNIVERSE = {1, 2, 3, 4, 5, 6}")),
    }
}

/// Replace compound Boolean variables by their defining expressions, so
/// truth-table analysis sees the underlying propositional variables.
/// Variables assigned a literal (P = TRUE) or a formula without any
/// propositional variable inside stay as columns themselves.
fn expand_bool_expr(
    e: &Expr,
    types: &SymTable,
    defs: &DefTable,
    visiting: &mut HashSet<String>,
) -> Expr {
    match e {
        Expr::Var { name, .. } if types.get(name) == Some(&DType::Bool) => {
            if let Some(def) = defs.get(name) {
                if def_has_bool_var(def, types) && !visiting.contains(name) {
                    visiting.insert(name.clone());
                    let expanded = expand_bool_expr(def, types, defs, visiting);
                    visiting.remove(name);
                    return expanded;
                }
            }
            e.clone()
        }
        Expr::Unary { op, arg, pos } => Expr::Unary {
            op: *op,
            arg: Box::new(expand_bool_expr(arg, types, defs, visiting)),
            pos: *pos,
        },
        Expr::Binary { op, lhs, rhs, pos } => Expr::Binary {
            op: *op,
            lhs: Box::new(expand_bool_expr(lhs, types, defs, visiting)),
            rhs: Box::new(expand_bool_expr(rhs, types, defs, visiting)),
            pos: *pos,
        },
        other => other.clone(),
    }
}

fn def_has_bool_var(e: &Expr, types: &SymTable) -> bool {
    let mut found = false;
    e.walk(&mut |sub| {
        if let Expr::Var { name, .. } = sub {
            if types.get(name) == Some(&DType::Bool) {
                found = true;
            }
        }
    });
    found
}

/// Collect the Boolean-typed variables appearing in an expression, in order
/// of first appearance. These are exactly the columns of a truth table.
fn collect_bool_vars(e: &Expr, types: &SymTable, out: &mut Vec<String>) {
    e.walk(&mut |sub| {
        if let Expr::Var { name, .. } = sub {
            if types.get(name) == Some(&DType::Bool) && !out.contains(name) {
                out.push(name.clone());
            }
        }
    });
}
