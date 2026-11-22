//! Minimal-parentheses pretty printing of expressions.
//!
//! Produces the canonical textual form used in outputs, truth tables and
//! execution traces, e.g. `(A UNION B) INTERSECT {3, 4, 5}`.

use crate::ast::{BinOp, Expr, GraphEdgeLit, SetItem, UnOp};

const ATOM_PREC: u8 = 15;
const UNARY_PREC: u8 = 14; // NOT / COMPLEMENT bind tighter than any binary operator
const NEG_PREC: u8 = 12; // unary minus: tighter than * and /, looser than ^

pub fn pretty(e: &Expr) -> String {
    go(e, 0)
}

fn go(e: &Expr, min_prec: u8) -> String {
    match e {
        Expr::Num { v, .. } => {
            // A negative literal as the base of `^` must keep its parentheses.
            if *v < 0 && min_prec > NEG_PREC && min_prec < ATOM_PREC {
                format!("({})", v)
            } else {
                v.to_string()
            }
        }
        Expr::Bool { v, .. } => if *v { "TRUE" } else { "FALSE" }.to_string(),
        Expr::Var { name, .. } => name.clone(),
        Expr::SetLit { items, .. } => {
            if items.is_empty() {
                "{}".to_string()
            } else {
                let inner: Vec<String> = items.iter().map(set_item_to_string).collect();
                format!("{{{}}}", inner.join(", "))
            }
        }
        Expr::GraphLit { directed, vertices, edges, .. } => {
            let vs: Vec<String> = vertices.iter().map(|v| v.to_string()).collect();
            let es: Vec<String> = edges.iter().map(edge_to_string).collect();
            format!(
                "{} {{{}}} {{{}}}",
                if *directed { "DIGRAPH" } else { "GRAPH" },
                vs.join(", "),
                es.join(", ")
            )
        }
        Expr::Call { func, args, .. } => {
            let inner: Vec<String> = args.iter().map(|a| go(a, ATOM_PREC)).collect();
            format!("{}({})", func.name(), inner.join(", "))
        }
        Expr::Unary { op, arg, .. } => {
            let own = if *op == UnOp::Neg { NEG_PREC } else { UNARY_PREC };
            let inner = go(arg, own);
            let s = match op {
                UnOp::Neg => format!("-{}", inner),
                _ => format!("{} {}", op.symbol(), inner),
            };
            // A unary operator binds tighter than anything around it, except
            // when it appears as an operand of a set operator that expects an
            // atom-ish operand (e.g. `{1} CROSS (COMPLEMENT A)` reads better
            // parenthesized); the precedence numbers handle this uniformly.
            if own < min_prec {
                format!("({})", s)
            } else {
                s
            }
        }
        Expr::Binary { op, lhs, rhs, .. } => {
            let p = op.precedence();
            let (lmin, rmin) = if op.is_comparison() {
                (p + 1, p + 1)
            } else if op.right_assoc() {
                (p + 1, p)
            } else {
                (p, p + 1)
            };
            let l = go(lhs, lmin);
            let r = go(rhs, rmin);
            let s = format!("{} {} {}", l, op.symbol(), r);
            if p < min_prec {
                format!("({})", s)
            } else {
                s
            }
        }
    }
}

/// Convenience: precedence of an expression's root node (atoms are maximal).
pub fn root_precedence(e: &Expr) -> u8 {
    match e {
        Expr::Binary { op, .. } => op.precedence(),
        Expr::Unary { .. } => UNARY_PREC,
        _ => ATOM_PREC,
    }
}

/// Canonical text of one set-literal item: `3` or `(1, 2)` (any arity).
fn set_item_to_string(item: &SetItem) -> String {
    match item {
        SetItem::Num { v, .. } => v.to_string(),
        SetItem::Tuple { items, .. } => {
            let inner: Vec<String> = items.iter().map(|i| i.to_string()).collect();
            format!("({})", inner.join(", "))
        }
        SetItem::Set { items, .. } => {
            let inner: Vec<String> = items.iter().map(set_item_to_string).collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

/// Canonical text of one graph-literal edge: `(1, 2)` or `(1, 2, 7)`.
fn edge_to_string(e: &GraphEdgeLit) -> String {
    match e.weight {
        Some(w) => format!("({}, {}, {})", e.from, e.to, w),
        None => format!("({}, {})", e.from, e.to),
    }
}

/// Truncate a long expression string for compact display contexts.
pub fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{}…", cut.trim_end())
    }
}

#[allow(dead_code)]
fn _atom_prec_assert() {
    let _ = ATOM_PREC;
    let _ = BinOp::And;
}
