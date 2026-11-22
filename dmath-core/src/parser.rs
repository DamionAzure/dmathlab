//! Parser: converts the token stream into a `Program` (list of statements).
//!
//! The parser performs error recovery at statement granularity: when a
//! statement cannot be parsed, a diagnostic is recorded, the parser skips to
//! the end of the line and continues with the next statement. This gives the
//! user useful errors for every broken line in one run.

use crate::ast::{BinOp, Expr, GraphEdgeLit, Program, SetItem, Stmt, UnOp, VizKind};
use crate::token::{describe, Builtin, Diagnostic, Pos, TokKind, Token};

const MAX_EXPR_DEPTH: usize = 64;
/// Largest number of elements a range literal such as `{1..10000}` may expand to.
const MAX_RANGE_LEN: usize = 10_000;

pub fn parse(tokens: Vec<Token>) -> (Program, Vec<Diagnostic>) {
    let mut p = Parser { toks: tokens, i: 0, depth: 0, diags: Vec::new() };
    let program = p.parse_program();
    (program, p.diags)
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
    depth: usize,
    diags: Vec<Diagnostic>,
}

type PResult<T> = Result<T, Diagnostic>;

impl Parser {
    fn peek(&self) -> &Token {
        &self.toks[self.i.min(self.toks.len() - 1)]
    }

    fn peek2(&self) -> Option<&Token> {
        self.toks.get(self.i + 1)
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.i.min(self.toks.len() - 1)].clone();
        if self.i < self.toks.len() - 1 {
            self.i += 1;
        }
        t
    }

    fn at(&self, kind: TokKind) -> bool {
        self.peek().kind == kind
    }

    fn error_here(&self, message: impl Into<String>) -> Diagnostic {
        let t = self.peek();
        Diagnostic::error(t.line, t.col, message)
    }

    fn expect(&mut self, kind: TokKind, what: &str) -> PResult<Token> {
        if self.at(kind) {
            Ok(self.advance())
        } else {
            let t = self.peek();
            Err(Diagnostic::error(
                t.line,
                t.col,
                format!("Expected {}, found {}.", what, describe(t.kind)),
            ))
        }
    }

    /// Skip tokens until a statement boundary has been consumed.
    fn synchronize(&mut self) {
        while !self.at(TokKind::Eof)
            && !self.at(TokKind::Newline)
            && !self.at(TokKind::Semicolon)
        {
            self.advance();
        }
        if self.at(TokKind::Newline) || self.at(TokKind::Semicolon) {
            self.advance();
        }
    }

    fn parse_program(&mut self) -> Program {
        let mut stmts = Vec::new();
        loop {
            while self.at(TokKind::Newline) || self.at(TokKind::Semicolon) {
                self.advance();
            }
            if self.at(TokKind::Eof) {
                break;
            }
            match self.parse_statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(d) => {
                    self.diags.push(d);
                    self.synchronize();
                    continue;
                }
            }
            // A statement must be followed by a newline, ';' or EOF.
            if self.at(TokKind::Newline) || self.at(TokKind::Semicolon) {
                self.advance();
            } else if !self.at(TokKind::Eof) {
                let d = self
                    .error_here(format!(
                        "Expected end of statement, found {}.",
                        describe(self.peek().kind)
                    ))
                    .with_hint("Write one statement per line.");
                self.diags.push(d);
                self.synchronize();
            }
        }
        Program { stmts }
    }

    fn parse_statement(&mut self) -> PResult<Stmt> {
        let t = self.peek().clone();
        match t.kind {
            TokKind::Display => {
                self.advance();
                let mut exprs = vec![self.parse_expr()?];
                while self.at(TokKind::Comma) {
                    self.advance();
                    exprs.push(self.parse_expr()?);
                }
                Ok(Stmt::Display { exprs, pos: t.pos() })
            }
            TokKind::Visualize => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokKind::As, "'AS'")?;
                let kind = match self.peek().kind {
                    TokKind::TruthTable => VizKind::TruthTable,
                    TokKind::Venn => VizKind::Venn,
                    TokKind::Digraph => VizKind::Digraph,
                    TokKind::Hasse => VizKind::Hasse,
                    TokKind::Matrix => VizKind::Matrix,
                    TokKind::Graph => VizKind::Graph,
                    TokKind::Steps => VizKind::Steps,
                    // SIEVE and PASCAL double as builtin function names, so
                    // after `AS` they arrive wrapped in TokKind::Func — the
                    // same positional disambiguation GRAPH/DIGRAPH use.
                    TokKind::Func(Builtin::Sieve) => VizKind::Sieve,
                    TokKind::Func(Builtin::Pascal) => VizKind::Pascal,
                    other => {
                        return Err(Diagnostic::error(
                            self.peek().line,
                            self.peek().col,
                            format!(
                                "Expected 'TRUTH_TABLE', 'VENN', 'DIGRAPH', 'HASSE', 'MATRIX', 'GRAPH', 'SIEVE', 'PASCAL' or 'STEPS' after 'AS', found {}.",
                                describe(other)
                            ),
                        ))
                    }
                };
                self.advance();
                Ok(Stmt::Visualize { expr, kind, pos: t.pos() })
            }
            TokKind::Check => {
                self.advance();
                if self.at(TokKind::Properties) {
                    self.advance();
                    let expr = self.parse_expr()?;
                    return Ok(Stmt::CheckProperties { expr, pos: t.pos() });
                }
                self.expect(TokKind::Equivalent, "'EQUIVALENT' after 'CHECK'")?;
                let left = self.parse_expr()?;
                self.expect(TokKind::Comma, "',' between the two expressions")?;
                let right = self.parse_expr()?;
                Ok(Stmt::CheckEquivalent { left, right, pos: t.pos() })
            }
            TokKind::Ident if self.peek2().map(|t| t.kind) == Some(TokKind::Assign) => {
                self.advance(); // identifier
                self.advance(); // '='
                let expr = self.parse_expr()?;
                Ok(Stmt::Assign { name: t.text.clone(), expr, pos: t.pos() })
            }
            TokKind::Ident => Err(Diagnostic::error(
                t.line,
                t.col,
                format!("Expected '=' after the variable name '{}'.", t.text),
            )
            .with_hint(format!(
                "Assign a value with '{} = <expression>', or show one with 'DISPLAY {}'.",
                t.text, t.text
            ))),
            TokKind::True
            | TokKind::False
            | TokKind::Number
            | TokKind::LBrace
            | TokKind::LParen
            | TokKind::Not
            | TokKind::Complement
            | TokKind::Minus
            | TokKind::Graph
            | TokKind::Digraph
            | TokKind::Func(_) => Err(Diagnostic::error(
                t.line,
                t.col,
                "A statement cannot start with an expression.",
            )
            .with_hint(
                "Start a statement with NAME = ..., DISPLAY ..., VISUALIZE ... AS ..., or CHECK EQUIVALENT ..., ...",
            )),
            other => Err(Diagnostic::error(
                t.line,
                t.col,
                format!("Unexpected {} at the start of a statement.", describe(other)),
            )),
        }
    }

    fn parse_expr(&mut self) -> PResult<Expr> {
        self.depth += 1;
        if self.depth > MAX_EXPR_DEPTH {
            return Err(Diagnostic::error(
                self.peek().line,
                self.peek().col,
                format!("Expression is nested too deeply (limit is {} levels).", MAX_EXPR_DEPTH),
            ));
        }
        let r = self.parse_iff();
        self.depth -= 1;
        r
    }

    fn parse_iff(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_implies()?;
        while self.at(TokKind::Iff) {
            let op_tok = self.advance();
            let rhs = self.parse_implies()?;
            lhs = Expr::Binary {
                op: BinOp::Iff,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_implies(&mut self) -> PResult<Expr> {
        let lhs = self.parse_or()?;
        if self.at(TokKind::Implies) {
            let op_tok = self.advance();
            let rhs = self.parse_implies()?; // right associative: A -> B -> C = A -> (B -> C)
            return Ok(Expr::Binary {
                op: BinOp::Implies,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            });
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_xor()?;
        while self.at(TokKind::Or) || self.at(TokKind::Nor) {
            let op_tok = self.advance();
            let op = if op_tok.kind == TokKind::Or { BinOp::Or } else { BinOp::Nor };
            let rhs = self.parse_xor()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_xor(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_and()?;
        while self.at(TokKind::Xor) {
            let op_tok = self.advance();
            let rhs = self.parse_and()?;
            lhs = Expr::Binary {
                op: BinOp::Xor,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_unary()?;
        while self.at(TokKind::And) || self.at(TokKind::Nand) {
            let op_tok = self.advance();
            let op = if op_tok.kind == TokKind::And { BinOp::And } else { BinOp::Nand };
            let rhs = self.parse_unary()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        if self.at(TokKind::Not) {
            let t = self.advance();
            let arg = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::Not, arg: Box::new(arg), pos: t.pos() });
        }
        self.parse_compare()
    }

    fn parse_compare(&mut self) -> PResult<Expr> {
        let lhs = self.parse_set_union()?;
        let cmp = match self.peek().kind {
            TokKind::Subset => Some(BinOp::Subset),
            TokKind::PSubset => Some(BinOp::PSubset),
            TokKind::Equals => Some(BinOp::Equals),
            TokKind::In => Some(BinOp::In),
            TokKind::Lt => Some(BinOp::Lt),
            TokKind::Gt => Some(BinOp::Gt),
            TokKind::Le => Some(BinOp::Le),
            TokKind::Ge => Some(BinOp::Ge),
            TokKind::EqEq => Some(BinOp::Eq),
            TokKind::Ne => Some(BinOp::Ne),
            _ => None,
        };
        if let Some(op) = cmp {
            let op_tok = self.advance();
            let rhs = self.parse_set_union()?;
            return Ok(Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            });
        }
        Ok(lhs)
    }

    fn parse_set_union(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_set_intersect()?;
        while self.at(TokKind::Union) || self.at(TokKind::Difference) || self.at(TokKind::SymDiff) {
            let op_tok = self.advance();
            let op = match op_tok.kind {
                TokKind::Union => BinOp::Union,
                TokKind::SymDiff => BinOp::SymDiff,
                _ => BinOp::Difference,
            };
            let rhs = self.parse_set_intersect()?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_set_intersect(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_set_cross()?;
        while self.at(TokKind::Intersect) {
            let op_tok = self.advance();
            let rhs = self.parse_set_cross()?;
            lhs = Expr::Binary {
                op: BinOp::Intersect,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_set_cross(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_add()?;
        while self.at(TokKind::Cross) {
            let op_tok = self.advance();
            let rhs = self.parse_add()?;
            lhs = Expr::Binary {
                op: BinOp::Cross,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            };
        }
        Ok(lhs)
    }

    fn parse_add(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_mul()?;
        while self.at(TokKind::Plus) || self.at(TokKind::Minus) {
            let op_tok = self.advance();
            let op = if op_tok.kind == TokKind::Plus { BinOp::Add } else { BinOp::Sub };
            let rhs = self.parse_mul()?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), pos: op_tok.pos() };
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> PResult<Expr> {
        let mut lhs = self.parse_neg()?;
        loop {
            let op = match self.peek().kind {
                TokKind::Star => BinOp::Mul,
                TokKind::Slash => BinOp::Div,
                TokKind::Percent => BinOp::Rem,
                _ => break,
            };
            let op_tok = self.advance();
            let rhs = self.parse_neg()?;
            lhs = Expr::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs), pos: op_tok.pos() };
        }
        Ok(lhs)
    }

    /// Unary minus: looser than `^` (so `-2 ^ 2` is `-(2 ^ 2)`), tighter than `*`.
    fn parse_neg(&mut self) -> PResult<Expr> {
        if self.at(TokKind::Minus) {
            let t = self.advance();
            self.depth += 1;
            if self.depth > MAX_EXPR_DEPTH {
                return Err(Diagnostic::error(
                    t.line,
                    t.col,
                    format!("Expression is nested too deeply (limit is {} levels).", MAX_EXPR_DEPTH),
                ));
            }
            let arg = self.parse_neg();
            self.depth -= 1;
            let arg = arg?;
            // Fold a plain literal so `-3` stays the number -3.
            if let Expr::Num { v, .. } = &arg {
                if let Some(n) = v.checked_neg() {
                    return Ok(Expr::Num { v: n, pos: t.pos() });
                }
            }
            return Ok(Expr::Unary { op: UnOp::Neg, arg: Box::new(arg), pos: t.pos() });
        }
        self.parse_pow()
    }

    fn parse_pow(&mut self) -> PResult<Expr> {
        let base = self.parse_complement()?;
        if self.at(TokKind::Caret) {
            let op_tok = self.advance();
            let rhs = self.parse_neg()?; // right associative
            return Ok(Expr::Binary {
                op: BinOp::Pow,
                lhs: Box::new(base),
                rhs: Box::new(rhs),
                pos: op_tok.pos(),
            });
        }
        Ok(base)
    }

    fn parse_complement(&mut self) -> PResult<Expr> {
        if self.at(TokKind::Complement) {
            let t = self.advance();
            let arg = self.parse_complement()?;
            return Ok(Expr::Unary { op: UnOp::Complement, arg: Box::new(arg), pos: t.pos() });
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> PResult<Expr> {
        let t = self.peek().clone();
        match t.kind {
            TokKind::Number => {
                self.advance();
                let v: i64 = t.text.parse().map_err(|_| {
                    Diagnostic::error(t.line, t.col, format!("Invalid number '{}'.", t.text))
                })?;
                Ok(Expr::Num { v, pos: t.pos() })
            }
            TokKind::True => {
                self.advance();
                Ok(Expr::Bool { v: true, pos: t.pos() })
            }
            TokKind::False => {
                self.advance();
                Ok(Expr::Bool { v: false, pos: t.pos() })
            }
            TokKind::Ident => {
                self.advance();
                Ok(Expr::Var { name: t.text.clone(), pos: t.pos() })
            }
            TokKind::Graph | TokKind::Digraph => {
                self.advance();
                let directed = t.kind == TokKind::Digraph;
                self.parse_graph_literal(directed, t.pos())
            }
            TokKind::Func(builtin) => {
                self.advance();
                self.expect(TokKind::LParen, "'(' after the function name")?;
                let mut args = Vec::new();
                if !self.at(TokKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if self.at(TokKind::Comma) {
                            self.advance();
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokKind::RParen, "')' to close the argument list")?;
                Ok(Expr::Call { func: builtin, args, pos: t.pos() })
            }
            TokKind::LBrace => {
                self.advance();
                let mut items: Vec<SetItem> = Vec::new();
                let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
                self.skip_newlines();
                if self.at(TokKind::RBrace) {
                    self.advance();
                    return Ok(Expr::SetLit { items, pos: t.pos() });
                }
                loop {
                    for item in self.parse_set_items()? {
                        let key = set_item_key(&item);
                        if seen.insert(key.clone()) {
                            items.push(item);
                        } else {
                            let (wline, wcol) = item.pos();
                            self.diags.push(Diagnostic::warning(
                                wline,
                                wcol,
                                format!("Duplicate element {} in set literal is ignored.", key),
                            ));
                        }
                    }
                    if self.at(TokKind::Comma) {
                        self.advance();
                        self.skip_newlines();
                        continue;
                    }
                    break;
                }
                self.expect(TokKind::RBrace, "'}' to close the set literal")?;
                Ok(Expr::SetLit { items, pos: t.pos() })
            }
            TokKind::LParen => {
                self.advance();
                let e = self.parse_expr()?;
                self.expect(TokKind::RParen, "')' to close the parenthesized expression")?;
                Ok(e)
            }
            other => Err(Diagnostic::error(
                t.line,
                t.col,
                format!("Expected a value, found {}.", describe(other)),
            )
            .with_hint(
                "Values are numbers, TRUE/FALSE, variables, set literals like {1, 2}, graph literals like GRAPH {1, 2} {(1, 2)}, function calls like BFS(G, 1) and parenthesized expressions.",
            )),
        }
    }

    /// Parse an optionally negative integer literal (used by set items,
    /// tuples, graph vertices and edges).
    fn parse_signed_number(&mut self, context: &str) -> PResult<(i64, Pos)> {
        let negative = if self.at(TokKind::Minus) {
            self.advance();
            true
        } else {
            false
        };
        let n = self.peek().clone();
        if n.kind != TokKind::Number {
            return Err(Diagnostic::error(
                n.line,
                n.col,
                format!("Expected a number in {}, found {}.", context, describe(n.kind)),
            ));
        }
        self.advance();
        let v: i64 = n.text.parse().map_err(|_| {
            Diagnostic::error(n.line, n.col, format!("Invalid number '{}'.", n.text))
        })?;
        let v = if negative {
            v.checked_neg().ok_or_else(|| {
                Diagnostic::error(n.line, n.col, "Number literal is too large.")
            })?
        } else {
            v
        };
        Ok((v, n.pos()))
    }

    /// One slot of a set literal: an element, or a range `a..b` that expands
    /// to every integer from `a` to `b`.
    fn parse_set_items(&mut self) -> PResult<Vec<SetItem>> {
        if self.at(TokKind::LParen) || self.at(TokKind::LBrace) {
            return Ok(vec![self.parse_set_item()?]);
        }
        let (a, pos) = self.parse_signed_number("the set literal")?;
        if !self.at(TokKind::DotDot) {
            return Ok(vec![SetItem::Num { v: a, pos }]);
        }
        let dd = self.advance();
        let (b, _) = self.parse_signed_number("the range")?;
        if b < a {
            return Err(Diagnostic::error(
                dd.line,
                dd.col,
                format!("Range {}..{} is empty: the start is larger than the end.", a, b),
            )
            .with_hint("Write the smaller number first, e.g. {1..5}."));
        }
        let count = (b as i128) - (a as i128) + 1;
        if count > MAX_RANGE_LEN as i128 {
            return Err(Diagnostic::error(
                dd.line,
                dd.col,
                format!("Range {}..{} would contain {} elements; the limit is {}.", a, b, count, MAX_RANGE_LEN),
            ));
        }
        Ok((a..=b).map(|v| SetItem::Num { v, pos }).collect())
    }

    /// One element of a set literal: a number, a tuple `(1, 2)` or a nested set `{1, 2}`.
    fn parse_set_item(&mut self) -> PResult<SetItem> {
        if self.at(TokKind::LParen) {
            return self.parse_tuple_item();
        }
        if self.at(TokKind::LBrace) {
            let lb = self.advance();
            self.depth += 1;
            if self.depth > MAX_EXPR_DEPTH {
                return Err(Diagnostic::error(
                    lb.line,
                    lb.col,
                    format!("Sets are nested too deeply (limit is {} levels).", MAX_EXPR_DEPTH),
                ));
            }
            let mut items: Vec<SetItem> = Vec::new();
            self.skip_newlines();
            if self.at(TokKind::RBrace) {
                self.advance();
                self.depth -= 1;
                return Ok(SetItem::Set { items, pos: lb.pos() });
            }
            loop {
                items.extend(self.parse_set_items()?);
                if self.at(TokKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                    continue;
                }
                break;
            }
            self.expect(TokKind::RBrace, "'}' to close the nested set")?;
            self.depth -= 1;
            return Ok(SetItem::Set { items, pos: lb.pos() });
        }
        let (v, pos) = self.parse_signed_number("the set literal")?;
        Ok(SetItem::Num { v, pos })
    }

    /// Parse one tuple item inside a set literal: `( a, b {, c } )` with two
    /// or more numeric components.
    fn parse_tuple_item(&mut self) -> PResult<SetItem> {
        let lp = self.expect(TokKind::LParen, "'(' to start a tuple")?;
        let mut comps: Vec<i64> = Vec::new();
        loop {
            let (v, _) = self.parse_signed_number("the tuple")?;
            comps.push(v);
            if self.at(TokKind::Comma) {
                self.advance();
                self.skip_newlines();
                continue;
            }
            break;
        }
        self.expect(TokKind::RParen, "')' to close the tuple")?;
        if comps.len() < 2 {
            return Err(Diagnostic::error(
                lp.line,
                lp.col,
                "Expected a tuple with at least two components, e.g. (1, 2).",
            )
            .with_hint("Ordered pairs look like (1, 2); a single number needs no parentheses."));
        }
        Ok(SetItem::Tuple { items: comps, pos: lp.pos() })
    }

    /// Consume newline tokens (line breaks are allowed inside braced
    /// literals: sets, tuples and graph vertex/edge sets).
    fn skip_newlines(&mut self) {
        while self.at(TokKind::Newline) {
            self.advance();
        }
    }

    /// Parse a graph literal: `GRAPH {vertices} {edges}` or
    /// `DIGRAPH {vertices} {edges}` where an edge is
    /// `( from , to [ , weight ] )`.
    fn parse_graph_literal(&mut self, directed: bool, pos: Pos) -> PResult<Expr> {
        self.expect(TokKind::LBrace, "'{' with the vertex set")?;
        self.skip_newlines();
        let mut vertices = Vec::new();
        let mut seen_v: std::collections::HashSet<i64> = std::collections::HashSet::new();
        if !self.at(TokKind::RBrace) {
            loop {
                let (v, _) = self.parse_signed_number("the vertex set")?;
                if seen_v.insert(v) {
                    vertices.push(v);
                }
                if self.at(TokKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                    continue;
                }
                break;
            }
        }
        self.expect(TokKind::RBrace, "'}' to close the vertex set")?;
        self.expect(TokKind::LBrace, "'{' with the edge set")?;
        self.skip_newlines();
        let mut edges: Vec<GraphEdgeLit> = Vec::new();
        if !self.at(TokKind::RBrace) {
            loop {
                let ep = self.expect(TokKind::LParen, "'(' to start an edge")?;
                let (from, _) = self.parse_signed_number("the edge")?;
                self.expect(TokKind::Comma, "',' between the edge endpoints")?;
                let (to, _) = self.parse_signed_number("the edge")?;
                let mut weight = None;
                if self.at(TokKind::Comma) {
                    self.advance();
                    let (w, _) = self.parse_signed_number("the edge weight")?;
                    weight = Some(w);
                    if self.at(TokKind::Comma) {
                        return Err(Diagnostic::error(
                            self.peek().line,
                            self.peek().col,
                            "An edge has at most three components: (from, to, weight).",
                        )
                        .with_hint("Example: (1, 2, 7) for an edge from 1 to 2 with weight 7."));
                    }
                }
                self.expect(TokKind::RParen, "')' to close the edge")?;
                edges.push(GraphEdgeLit { from, to, weight, pos: ep.pos() });
                if self.at(TokKind::Comma) {
                    self.advance();
                    self.skip_newlines();
                    continue;
                }
                break;
            }
        }
        self.expect(TokKind::RBrace, "'}' to close the edge set")?;
        Ok(Expr::GraphLit { directed, vertices, edges, pos })
    }
}

/// Deduplication key of a set item: its canonical display text.
fn set_item_key(item: &SetItem) -> String {
    match item {
        SetItem::Num { v, .. } => v.to_string(),
        SetItem::Tuple { items, .. } => {
            let inner: Vec<String> = items.iter().map(|i| i.to_string()).collect();
            format!("({})", inner.join(", "))
        }
        SetItem::Set { items, .. } => {
            // Order inside a nested set does not matter: {1, 2} is {2, 1}.
            let mut inner: Vec<String> = items.iter().map(set_item_key).collect();
            inner.sort();
            inner.dedup();
            format!("{{{}}}", inner.join(", "))
        }
    }
}
