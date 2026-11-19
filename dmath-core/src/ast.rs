//! Abstract syntax tree for the DMath DSL.

use crate::token::{Builtin, Pos};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Complement,
    Neg,
}

impl UnOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            UnOp::Not => "NOT",
            UnOp::Complement => "COMPLEMENT",
            UnOp::Neg => "-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Propositional logic
    And,
    Or,
    Implies,
    Iff,
    Xor,
    Nand,
    Nor,
    // Set algebra
    Union,
    Intersect,
    Difference,
    SymDiff,
    Cross,
    // Set comparisons (yield Boolean)
    Subset,
    Equals,
}

impl BinOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            BinOp::And => "AND",
            BinOp::Or => "OR",
            BinOp::Implies => "IMPLIES",
            BinOp::Iff => "IFF",
            BinOp::Xor => "XOR",
            BinOp::Nand => "NAND",
            BinOp::Nor => "NOR",
            BinOp::Union => "UNION",
            BinOp::Intersect => "INTERSECT",
            BinOp::Difference => "DIFFERENCE",
            BinOp::SymDiff => "SYMDIFF",
            BinOp::Cross => "CROSS",
            BinOp::Subset => "SUBSET",
            BinOp::Equals => "EQUALS",
        }
    }

    /// Binding power, from loosest (1) to tightest (9).
    pub fn precedence(&self) -> u8 {
        match self {
            BinOp::Subset | BinOp::Equals => 1,
            BinOp::Iff => 2,
            BinOp::Implies => 3,
            BinOp::Or | BinOp::Nor => 4,
            BinOp::Xor => 5,
            BinOp::And | BinOp::Nand => 6,
            BinOp::Union | BinOp::Difference | BinOp::SymDiff => 7,
            BinOp::Intersect => 8,
            BinOp::Cross => 9,
        }
    }

    pub fn is_logic(&self) -> bool {
        matches!(
            self,
            BinOp::And | BinOp::Or | BinOp::Implies | BinOp::Iff | BinOp::Xor | BinOp::Nand | BinOp::Nor
        )
    }

    pub fn is_set_op(&self) -> bool {
        matches!(
            self,
            BinOp::Union | BinOp::Intersect | BinOp::Difference | BinOp::SymDiff | BinOp::Cross
        )
    }

    pub fn is_comparison(&self) -> bool {
        matches!(self, BinOp::Subset | BinOp::Equals)
    }

    pub fn right_assoc(&self) -> bool {
        matches!(self, BinOp::Implies)
    }
}

/// One item inside a set literal: a bare number or an ordered tuple like
/// `(1, 2)` with two or more numeric components.
#[derive(Debug, Clone)]
pub enum SetItem {
    Num { v: i64, pos: Pos },
    Tuple { items: Vec<i64>, pos: Pos },
    /// A nested set literal such as `{1, 2}` inside `{{1, 2}, {3}}`.
    Set { items: Vec<SetItem>, pos: Pos },
}

impl SetItem {
    pub fn pos(&self) -> Pos {
        match self {
            SetItem::Num { pos, .. } | SetItem::Tuple { pos, .. } | SetItem::Set { pos, .. } => *pos,
        }
    }
}

/// One edge of a graph literal: `(from, to)` or `(from, to, weight)`.
#[derive(Debug, Clone)]
pub struct GraphEdgeLit {
    pub from: i64,
    pub to: i64,
    pub weight: Option<i64>,
    pub pos: Pos,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Num { v: i64, pos: Pos },
    Bool { v: bool, pos: Pos },
    Var { name: String, pos: Pos },
    SetLit { items: Vec<SetItem>, pos: Pos },
    /// `GRAPH {vertices} {edges}` (undirected) or `DIGRAPH ...` (directed).
    GraphLit { directed: bool, vertices: Vec<i64>, edges: Vec<GraphEdgeLit>, pos: Pos },
    /// `FUNC(arg, ...)` for one of the built-in functions.
    Call { func: Builtin, args: Vec<Expr>, pos: Pos },
    Unary { op: UnOp, arg: Box<Expr>, pos: Pos },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, pos: Pos },
}

impl Expr {
    pub fn pos(&self) -> Pos {
        match self {
            Expr::Num { pos, .. }
            | Expr::Bool { pos, .. }
            | Expr::Var { pos, .. }
            | Expr::SetLit { pos, .. }
            | Expr::GraphLit { pos, .. }
            | Expr::Call { pos, .. }
            | Expr::Unary { pos, .. }
            | Expr::Binary { pos, .. } => *pos,
        }
    }

    pub fn walk<'a>(&'a self, f: &mut impl FnMut(&'a Expr)) {
        f(self);
        match self {
            Expr::Unary { arg, .. } => arg.walk(f),
            Expr::Binary { lhs, rhs, .. } => {
                lhs.walk(f);
                rhs.walk(f);
            }
            Expr::Call { args, .. } => {
                for a in args {
                    a.walk(f);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VizKind {
    TruthTable,
    Venn,
    Digraph,
    Hasse,
    Matrix,
    Graph,
    /// `VISUALIZE S AS SIEVE` — prime/composite grid from `SIEVE(n)`.
    Sieve,
    /// `VISUALIZE P AS PASCAL` — Pascal's triangle from `PASCAL(n)`
    /// (or a highlighted cell from `COMB(n, k)`).
    Pascal,
    Steps,
}

impl VizKind {
    pub fn name(&self) -> &'static str {
        match self {
            VizKind::TruthTable => "TRUTH_TABLE",
            VizKind::Venn => "VENN",
            VizKind::Digraph => "DIGRAPH",
            VizKind::Hasse => "HASSE",
            VizKind::Matrix => "MATRIX",
            VizKind::Graph => "GRAPH",
            VizKind::Sieve => "SIEVE",
            VizKind::Pascal => "PASCAL",
            VizKind::Steps => "STEPS",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Assign { name: String, expr: Expr, pos: Pos },
    Display { exprs: Vec<Expr>, pos: Pos },
    Visualize { expr: Expr, kind: VizKind, pos: Pos },
    CheckEquivalent { left: Expr, right: Expr, pos: Pos },
    CheckProperties { expr: Expr, pos: Pos },
}

impl Stmt {
    pub fn pos(&self) -> Pos {
        match self {
            Stmt::Assign { pos, .. }
            | Stmt::Display { pos, .. }
            | Stmt::Visualize { pos, .. }
            | Stmt::CheckEquivalent { pos, .. }
            | Stmt::CheckProperties { pos, .. } => *pos,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
