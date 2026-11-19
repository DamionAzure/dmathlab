//! Token definitions for the DMath DSL.
//!
//! Keywords are matched case-insensitively so that `and`, `And` and `AND`
//! all denote the logical conjunction operator.

use serde::Serialize;

/// A (line, column) position inside the source text. Both are 1-based.
pub type Pos = (usize, usize);

/// Built-in functions of the relations & graphs domain. Each is a keyword
/// token followed by a parenthesized argument list (`BFS(G, 1)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Builtin {
    // Relations
    Inverse,
    Compose,
    TransitiveClosure,
    ReflexiveClosure,
    SymmetricClosure,
    IsReflexive,
    IsSymmetric,
    IsAntisymmetric,
    IsTransitive,
    IsEquivalence,
    IsPartialOrder,
    // Graph theory
    Vertices,
    Edges,
    Degree,
    Neighbors,
    Adjacent,
    Bfs,
    Dfs,
    ShortestPath,
    IsConnected,
    IsTree,
    IsBipartite,
    IsComplete,
    Color,
    Mst,
    // Number theory
    IsPrime,
    Factors,
    Divisors,
    Gcd,
    Lcm,
    Totient,
    Sieve,
    // Combinatorics
    Fact,
    Perm,
    Comb,
    Pascal,
    // Modular arithmetic, more counting, more graphs
    Mod,
    ModPow,
    ModInv,
    Egcd,
    Crt,
    Fib,
    Catalan,
    Derange,
    Stirling2,
    TopoSort,
    Size,
    Powerset,
    Contains,
    IsDisjoint,
    IsDag,
    Components,
    IsEulerian,
    DegreeSequence,
}

impl Builtin {
    pub fn name(self) -> &'static str {
        match self {
            Builtin::Inverse => "INVERSE",
            Builtin::Compose => "COMPOSE",
            Builtin::TransitiveClosure => "TRANSITIVE_CLOSURE",
            Builtin::ReflexiveClosure => "REFLEXIVE_CLOSURE",
            Builtin::SymmetricClosure => "SYMMETRIC_CLOSURE",
            Builtin::IsReflexive => "IS_REFLEXIVE",
            Builtin::IsSymmetric => "IS_SYMMETRIC",
            Builtin::IsAntisymmetric => "IS_ANTISYMMETRIC",
            Builtin::IsTransitive => "IS_TRANSITIVE",
            Builtin::IsEquivalence => "IS_EQUIVALENCE",
            Builtin::IsPartialOrder => "IS_PARTIAL_ORDER",
            Builtin::Vertices => "VERTICES",
            Builtin::Edges => "EDGES",
            Builtin::Degree => "DEGREE",
            Builtin::Neighbors => "NEIGHBORS",
            Builtin::Adjacent => "ADJACENT",
            Builtin::Bfs => "BFS",
            Builtin::Dfs => "DFS",
            Builtin::ShortestPath => "SHORTEST_PATH",
            Builtin::IsConnected => "IS_CONNECTED",
            Builtin::IsTree => "IS_TREE",
            Builtin::IsBipartite => "IS_BIPARTITE",
            Builtin::IsComplete => "IS_COMPLETE",
            Builtin::Color => "COLOR",
            Builtin::Mst => "MST",
            Builtin::IsPrime => "IS_PRIME",
            Builtin::Factors => "FACTORS",
            Builtin::Divisors => "DIVISORS",
            Builtin::Gcd => "GCD",
            Builtin::Lcm => "LCM",
            Builtin::Totient => "TOTIENT",
            Builtin::Sieve => "SIEVE",
            Builtin::Fact => "FACT",
            Builtin::Perm => "PERM",
            Builtin::Comb => "COMB",
            Builtin::Pascal => "PASCAL",
            Builtin::Mod => "MOD",
            Builtin::ModPow => "MODPOW",
            Builtin::ModInv => "MODINV",
            Builtin::Egcd => "EGCD",
            Builtin::Crt => "CRT",
            Builtin::Fib => "FIB",
            Builtin::Catalan => "CATALAN",
            Builtin::Derange => "DERANGE",
            Builtin::Stirling2 => "STIRLING2",
            Builtin::TopoSort => "TOPO_SORT",
            Builtin::Size => "SIZE",
            Builtin::Powerset => "POWERSET",
            Builtin::Contains => "CONTAINS",
            Builtin::IsDisjoint => "IS_DISJOINT",
            Builtin::IsDag => "IS_DAG",
            Builtin::Components => "COMPONENTS",
            Builtin::IsEulerian => "IS_EULERIAN",
            Builtin::DegreeSequence => "DEGREE_SEQUENCE",
        }
    }

    pub fn lookup(text: &str) -> Option<Builtin> {
        match text.to_ascii_uppercase().as_str() {
            "INVERSE" => Some(Builtin::Inverse),
            "COMPOSE" => Some(Builtin::Compose),
            "TRANSITIVE_CLOSURE" => Some(Builtin::TransitiveClosure),
            "REFLEXIVE_CLOSURE" => Some(Builtin::ReflexiveClosure),
            "SYMMETRIC_CLOSURE" => Some(Builtin::SymmetricClosure),
            "IS_REFLEXIVE" => Some(Builtin::IsReflexive),
            "IS_SYMMETRIC" => Some(Builtin::IsSymmetric),
            "IS_ANTISYMMETRIC" => Some(Builtin::IsAntisymmetric),
            "IS_TRANSITIVE" => Some(Builtin::IsTransitive),
            "IS_EQUIVALENCE" => Some(Builtin::IsEquivalence),
            "IS_PARTIAL_ORDER" => Some(Builtin::IsPartialOrder),
            "VERTICES" => Some(Builtin::Vertices),
            "EDGES" => Some(Builtin::Edges),
            "DEGREE" => Some(Builtin::Degree),
            "NEIGHBORS" => Some(Builtin::Neighbors),
            "ADJACENT" => Some(Builtin::Adjacent),
            "BFS" => Some(Builtin::Bfs),
            "DFS" => Some(Builtin::Dfs),
            "SHORTEST_PATH" => Some(Builtin::ShortestPath),
            "IS_CONNECTED" => Some(Builtin::IsConnected),
            "IS_TREE" => Some(Builtin::IsTree),
            "IS_BIPARTITE" => Some(Builtin::IsBipartite),
            "IS_COMPLETE" => Some(Builtin::IsComplete),
            "COLOR" => Some(Builtin::Color),
            "MST" => Some(Builtin::Mst),
            "IS_PRIME" => Some(Builtin::IsPrime),
            "FACTORS" => Some(Builtin::Factors),
            "DIVISORS" => Some(Builtin::Divisors),
            "GCD" => Some(Builtin::Gcd),
            "LCM" => Some(Builtin::Lcm),
            "TOTIENT" => Some(Builtin::Totient),
            "SIEVE" => Some(Builtin::Sieve),
            "FACT" => Some(Builtin::Fact),
            "PERM" => Some(Builtin::Perm),
            "COMB" => Some(Builtin::Comb),
            "PASCAL" => Some(Builtin::Pascal),
            "MOD" => Some(Builtin::Mod),
            "MODPOW" => Some(Builtin::ModPow),
            "MODINV" => Some(Builtin::ModInv),
            "EGCD" => Some(Builtin::Egcd),
            "CRT" => Some(Builtin::Crt),
            "FIB" => Some(Builtin::Fib),
            "CATALAN" => Some(Builtin::Catalan),
            "DERANGE" => Some(Builtin::Derange),
            "STIRLING2" => Some(Builtin::Stirling2),
            "TOPO_SORT" => Some(Builtin::TopoSort),
            "SIZE" => Some(Builtin::Size),
            "POWERSET" => Some(Builtin::Powerset),
            "CONTAINS" => Some(Builtin::Contains),
            "IS_DISJOINT" => Some(Builtin::IsDisjoint),
            "IS_DAG" => Some(Builtin::IsDag),
            "COMPONENTS" => Some(Builtin::Components),
            "IS_EULERIAN" => Some(Builtin::IsEulerian),
            "DEGREE_SEQUENCE" => Some(Builtin::DegreeSequence),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokKind {
    Ident,
    Number,
    True,
    False,
    // Logical operators
    And,
    Or,
    Not,
    Implies,
    Iff,
    Xor,
    Nand,
    Nor,
    // Set operators
    Union,
    Intersect,
    Difference,
    SymDiff,
    Complement,
    Cross,
    // Set comparisons
    Subset,
    Equals,
    // Statements
    Display,
    Visualize,
    As,
    Check,
    Equivalent,
    TruthTable,
    Venn,
    // Relations & graphs: statements and visualization kinds
    Properties,
    Digraph,
    Hasse,
    Matrix,
    Steps,
    Graph,
    // Built-in function calls
    Func(Builtin),
    // Punctuation
    Assign,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Comma,
    Minus,
    // Layout
    Newline,
    Semicolon,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokKind,
    pub text: String,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn pos(&self) -> Pos {
        (self.line, self.col)
    }
}

/// Human readable description of a token kind, used in error messages.
pub fn describe(kind: TokKind) -> String {
    match kind {
        TokKind::Ident => "an identifier",
        TokKind::Number => "a number",
        TokKind::True => "'TRUE'",
        TokKind::False => "'FALSE'",
        TokKind::And => "'AND'",
        TokKind::Or => "'OR'",
        TokKind::Not => "'NOT'",
        TokKind::Implies => "'IMPLIES'",
        TokKind::Iff => "'IFF'",
        TokKind::Xor => "'XOR'",
        TokKind::Nand => "'NAND'",
        TokKind::Nor => "'NOR'",
        TokKind::Union => "'UNION'",
        TokKind::Intersect => "'INTERSECT'",
        TokKind::Difference => "'DIFFERENCE'",
        TokKind::SymDiff => "'SYMDIFF'",
        TokKind::Complement => "'COMPLEMENT'",
        TokKind::Cross => "'CROSS'",
        TokKind::Subset => "'SUBSET'",
        TokKind::Equals => "'EQUALS'",
        TokKind::Display => "'DISPLAY'",
        TokKind::Visualize => "'VISUALIZE'",
        TokKind::As => "'AS'",
        TokKind::Check => "'CHECK'",
        TokKind::Equivalent => "'EQUIVALENT'",
        TokKind::TruthTable => "'TRUTH_TABLE'",
        TokKind::Venn => "'VENN'",
        TokKind::Properties => "'PROPERTIES'",
        TokKind::Digraph => "'DIGRAPH'",
        TokKind::Hasse => "'HASSE'",
        TokKind::Matrix => "'MATRIX'",
        TokKind::Steps => "'STEPS'",
        TokKind::Graph => "'GRAPH'",
        TokKind::Func(b) => return format!("'{}'", b.name()),
        TokKind::Assign => "'='",
        TokKind::LBrace => "'{'",
        TokKind::RBrace => "'}'",
        TokKind::LParen => "'('",
        TokKind::RParen => "')'",
        TokKind::Comma => "','",
        TokKind::Minus => "'-'",
        TokKind::Newline => "end of line",
        TokKind::Semicolon => "';'",
        TokKind::Eof => "end of input",
    }
    .to_string()
}

/// Map an identifier-shaped piece of text to a keyword token kind, if any.
pub fn keyword(text: &str) -> Option<TokKind> {
    match text.to_ascii_uppercase().as_str() {
        "TRUE" => Some(TokKind::True),
        "FALSE" => Some(TokKind::False),
        "AND" => Some(TokKind::And),
        "OR" => Some(TokKind::Or),
        "NOT" => Some(TokKind::Not),
        "IMPLIES" => Some(TokKind::Implies),
        "IFF" => Some(TokKind::Iff),
        "XOR" => Some(TokKind::Xor),
        "NAND" => Some(TokKind::Nand),
        "NOR" => Some(TokKind::Nor),
        "UNION" => Some(TokKind::Union),
        "INTERSECT" => Some(TokKind::Intersect),
        "DIFFERENCE" => Some(TokKind::Difference),
        "SYMDIFF" => Some(TokKind::SymDiff),
        "COMPLEMENT" => Some(TokKind::Complement),
        "CROSS" => Some(TokKind::Cross),
        "SUBSET" => Some(TokKind::Subset),
        "EQUALS" => Some(TokKind::Equals),
        "DISPLAY" => Some(TokKind::Display),
        "VISUALIZE" => Some(TokKind::Visualize),
        "AS" => Some(TokKind::As),
        "CHECK" => Some(TokKind::Check),
        "EQUIVALENT" => Some(TokKind::Equivalent),
        "TRUTH_TABLE" => Some(TokKind::TruthTable),
        "VENN" => Some(TokKind::Venn),
        "PROPERTIES" => Some(TokKind::Properties),
        "DIGRAPH" => Some(TokKind::Digraph),
        "HASSE" => Some(TokKind::Hasse),
        "MATRIX" => Some(TokKind::Matrix),
        "STEPS" => Some(TokKind::Steps),
        "GRAPH" => Some(TokKind::Graph),
        _ => Builtin::lookup(text).map(TokKind::Func),
    }
}

/// The keywords understood by the lexer (used by the editor highlighter).
pub const KEYWORDS: &[&str] = &[
    "TRUE", "FALSE", "AND", "OR", "NOT", "IMPLIES", "IFF", "XOR", "NAND", "NOR", "UNION",
    "INTERSECT", "SYMDIFF", "DIFFERENCE", "COMPLEMENT", "CROSS", "SUBSET", "EQUALS", "DISPLAY", "VISUALIZE",
    "AS", "CHECK", "EQUIVALENT", "TRUTH_TABLE", "VENN", "UNIVERSE",
    // Relations & graphs
    "PROPERTIES", "DIGRAPH", "HASSE", "MATRIX", "GRAPH", "STEPS",
    "INVERSE", "COMPOSE", "TRANSITIVE_CLOSURE", "REFLEXIVE_CLOSURE", "SYMMETRIC_CLOSURE",
    "IS_REFLEXIVE", "IS_SYMMETRIC", "IS_ANTISYMMETRIC", "IS_TRANSITIVE",
    "IS_EQUIVALENCE", "IS_PARTIAL_ORDER",
    "VERTICES", "EDGES", "DEGREE", "NEIGHBORS", "ADJACENT",
    "BFS", "DFS", "SHORTEST_PATH", "IS_CONNECTED", "IS_TREE", "IS_BIPARTITE",
    "IS_COMPLETE", "COLOR", "MST",
    // Number theory & combinatorics
    "IS_PRIME", "FACTORS", "DIVISORS", "GCD", "LCM", "TOTIENT", "SIEVE",
    "FACT", "PERM", "COMB", "PASCAL",
    "MOD", "MODPOW", "MODINV", "EGCD", "CRT", "FIB", "CATALAN", "DERANGE", "STIRLING2",
    "TOPO_SORT", "IS_DAG", "COMPONENTS", "IS_EULERIAN", "DEGREE_SEQUENCE",
    // Set utilities
    "SIZE", "POWERSET", "CONTAINS", "IS_DISJOINT",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// A diagnostic (error, warning or info) attached to a source position.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(line: usize, col: usize, message: impl Into<String>) -> Self {
        Diagnostic { severity: Severity::Error, message: message.into(), line, col, hint: None }
    }

    pub fn warning(line: usize, col: usize, message: impl Into<String>) -> Self {
        Diagnostic { severity: Severity::Warning, message: message.into(), line, col, hint: None }
    }

    pub fn info(line: usize, col: usize, message: impl Into<String>) -> Self {
        Diagnostic { severity: Severity::Info, message: message.into(), line, col, hint: None }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}
