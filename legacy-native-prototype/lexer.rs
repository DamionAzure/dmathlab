//! Lexer: source text -> tokens.
//!
//! Hand-written (no external crates). Every token carries a line/column so
//! later stages can report errors like "line 3, col 5: ...".
//! Statements end at a `Newline` token; `#` starts a comment.

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // literals
    Number(i64),
    Ident(String),
    True,
    False,

    // statement keywords
    Let,
    Relation,
    Function,
    On,
    Check,
    Closure,
    Truthtable,
    Classify,
    Equiv,

    // set operators (words or unicode)
    Union,
    Intersect,
    Minus,
    Symdiff,
    Cross,
    In,
    Subset,

    // logic operators (symbol, word, or unicode forms all map here)
    And,    // &   and  ∧
    Or,     // |   or   ∨
    Not,    // !   not  ¬
    Xor,    // ^   xor  ⊕
    Arrow,  // ->  implies  →   (also used for function mappings)
    BiArrow, // <-> iff     ↔

    // comparison
    EqEq,
    NotEq,

    // punctuation
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Assign,
    DotDot,

    // structure
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub message: String,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.bump();
        }
    }

    fn read_digits(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek(0) {
            if c.is_ascii_digit() {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        s
    }
}

fn keyword(word: &str) -> Option<TokenKind> {
    use TokenKind::*;
    Some(match word {
        "true" => True,
        "false" => False,
        "let" => Let,
        "relation" => Relation,
        "function" => Function,
        "on" => On,
        "check" => Check,
        "closure" => Closure,
        "truthtable" => Truthtable,
        "classify" => Classify,
        "equiv" => Equiv,
        "union" => Union,
        "intersect" => Intersect,
        "minus" => Minus,
        "symdiff" => Symdiff,
        "cross" => Cross,
        "in" => In,
        "subset" => Subset,
        "and" => And,
        "or" => Or,
        "not" => Not,
        "xor" => Xor,
        "implies" => Arrow,
        "iff" => BiArrow,
        _ => return None,
    })
}

/// Turns source text into tokens. Collects *all* lexical errors instead of
/// stopping at the first one. The returned token list always ends with `Eof`.
pub fn lex(src: &str) -> Result<Vec<Token>, Vec<LexError>> {
    use TokenKind::*;

    let mut lx = Lexer {
        chars: src.chars().collect(),
        pos: 0,
        line: 1,
        col: 1,
    };
    let mut tokens: Vec<Token> = Vec::new();
    let mut errors: Vec<LexError> = Vec::new();

    while let Some(c) = lx.peek(0) {
        let (line, col) = (lx.line, lx.col);
        let mut emit = |kind: TokenKind, tokens: &mut Vec<Token>| {
            tokens.push(Token { kind, line, col });
        };
        let mut err = |msg: String, errors: &mut Vec<LexError>| {
            errors.push(LexError { line, col, message: msg });
        };

        match c {
            ' ' | '\t' | '\r' => {
                lx.bump();
            }
            '\n' => {
                lx.bump();
                emit(Newline, &mut tokens);
            }
            '#' => {
                while let Some(ch) = lx.peek(0) {
                    if ch == '\n' {
                        break;
                    }
                    lx.bump();
                }
            }

            '0'..='9' => {
                let digits = lx.read_digits();
                match digits.parse::<i64>() {
                    Ok(n) => emit(Number(n), &mut tokens),
                    Err(_) => err(format!("number `{}` is too large", digits), &mut errors),
                }
            }

            '-' => match lx.peek(1) {
                Some('>') => {
                    lx.bump_n(2);
                    emit(Arrow, &mut tokens);
                }
                // The DSL has no binary minus (set difference is `minus`),
                // so `-5` is always a negative number literal.
                Some(d) if d.is_ascii_digit() => {
                    lx.bump();
                    let digits = lx.read_digits();
                    match format!("-{}", digits).parse::<i64>() {
                        Ok(n) => emit(Number(n), &mut tokens),
                        Err(_) => err(format!("number `-{}` is too large", digits), &mut errors),
                    }
                }
                _ => {
                    lx.bump();
                    err("unexpected `-` (use `minus` for set difference, `->` for implication)".into(), &mut errors);
                }
            },

            'a'..='z' | 'A'..='Z' | '_' => {
                let mut word = String::new();
                while let Some(ch) = lx.peek(0) {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        word.push(ch);
                        lx.bump();
                    } else {
                        break;
                    }
                }
                match keyword(&word) {
                    Some(k) => emit(k, &mut tokens),
                    None => emit(Ident(word), &mut tokens),
                }
            }

            '<' => {
                if lx.peek(1) == Some('-') && lx.peek(2) == Some('>') {
                    lx.bump_n(3);
                    emit(BiArrow, &mut tokens);
                } else {
                    lx.bump();
                    err("unexpected `<` (did you mean `<->`?)".into(), &mut errors);
                }
            }
            '.' => {
                if lx.peek(1) == Some('.') {
                    lx.bump_n(2);
                    emit(DotDot, &mut tokens);
                } else {
                    lx.bump();
                    err("unexpected `.` (did you mean `..` for a range?)".into(), &mut errors);
                }
            }
            '=' => {
                if lx.peek(1) == Some('=') {
                    lx.bump_n(2);
                    emit(EqEq, &mut tokens);
                } else {
                    lx.bump();
                    emit(Assign, &mut tokens);
                }
            }
            '!' => {
                if lx.peek(1) == Some('=') {
                    lx.bump_n(2);
                    emit(NotEq, &mut tokens);
                } else {
                    lx.bump();
                    emit(Not, &mut tokens);
                }
            }
            '&' => {
                if lx.peek(1) == Some('&') {
                    lx.bump_n(2);
                    err("unknown operator `&&`; use `&` (or the word `and`)".into(), &mut errors);
                } else {
                    lx.bump();
                    emit(And, &mut tokens);
                }
            }
            '|' => {
                if lx.peek(1) == Some('|') {
                    lx.bump_n(2);
                    err("unknown operator `||`; use `|` (or the word `or`)".into(), &mut errors);
                } else {
                    lx.bump();
                    emit(Or, &mut tokens);
                }
            }

            // single-character tokens (ASCII + unicode math symbols)
            _ => {
                let kind = match c {
                    '^' | '⊕' => Some(Xor),
                    '∧' => Some(And),
                    '∨' => Some(Or),
                    '¬' => Some(Not),
                    '→' => Some(Arrow),
                    '↔' => Some(BiArrow),
                    '∪' => Some(Union),
                    '∩' => Some(Intersect),
                    '∖' => Some(Minus),
                    '×' => Some(Cross),
                    '∈' => Some(In),
                    '⊆' => Some(Subset),
                    '≠' => Some(NotEq),
                    '(' => Some(LParen),
                    ')' => Some(RParen),
                    '{' => Some(LBrace),
                    '}' => Some(RBrace),
                    '[' => Some(LBracket),
                    ']' => Some(RBracket),
                    ',' => Some(Comma),
                    ':' => Some(Colon),
                    _ => None,
                };
                lx.bump();
                match kind {
                    Some(k) => emit(k, &mut tokens),
                    None => err(format!("unknown character `{}`", c), &mut errors),
                }
            }
        }
    }

    tokens.push(Token { kind: Eof, line: lx.line, col: lx.col });

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn let_statement() {
        assert_eq!(
            kinds("let A = {1, 2}"),
            vec![Let, Ident("A".into()), Assign, LBrace, Number(1), Comma, Number(2), RBrace, Eof]
        );
    }

    #[test]
    fn logic_symbols_and_words_match() {
        assert_eq!(kinds("p & q"), kinds("p and q"));
        assert_eq!(kinds("p -> q"), kinds("p implies q"));
        assert_eq!(kinds("p <-> q"), kinds("p iff q"));
        assert_eq!(kinds("!p"), kinds("not p"));
        assert_eq!(kinds("p ∧ q → ¬r"), kinds("p & q -> !r"));
    }

    #[test]
    fn set_words_and_unicode_match() {
        assert_eq!(kinds("A union B"), kinds("A ∪ B"));
        assert_eq!(kinds("A intersect B"), kinds("A ∩ B"));
        assert_eq!(kinds("A cross B"), kinds("A × B"));
    }

    #[test]
    fn negative_numbers_and_arrow() {
        assert_eq!(kinds("-5"), vec![Number(-5), Eof]);
        assert_eq!(kinds("1 -> a"), vec![Number(1), Arrow, Ident("a".into()), Eof]);
    }

    #[test]
    fn range_and_comparison() {
        assert_eq!(kinds("{1..5}"), vec![LBrace, Number(1), DotDot, Number(5), RBrace, Eof]);
        assert_eq!(kinds("a == b != c"), vec![Ident("a".into()), EqEq, Ident("b".into()), NotEq, Ident("c".into()), Eof]);
    }

    #[test]
    fn comments_and_newlines() {
        assert_eq!(kinds("p # note\nq"), vec![Ident("p".into()), Newline, Ident("q".into()), Eof]);
    }

    #[test]
    fn tracks_line_and_column() {
        let t = lex("let A = 1\nlet B = 2").unwrap();
        let b = t.iter().find(|t| t.kind == Ident("B".into())).unwrap();
        assert_eq!((b.line, b.col), (2, 5));
    }

    #[test]
    fn reports_bad_operators() {
        let e = lex("p && q").unwrap_err();
        assert_eq!(e.len(), 1);
        assert!(e[0].message.contains("&&"));
        assert_eq!((e[0].line, e[0].col), (1, 3));
    }

    #[test]
    fn collects_multiple_errors() {
        let e = lex("a @ b\nc $ d").unwrap_err();
        assert_eq!(e.len(), 2);
        assert_eq!(e[1].line, 2);
    }

    #[test]
    fn number_overflow() {
        assert!(lex("99999999999999999999").is_err());
    }
}
