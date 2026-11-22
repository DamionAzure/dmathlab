//! Lexer: converts DMath source text into a stream of tokens.
//!
//! Newlines are significant (they terminate statements). `#` and `//` start
//! line comments. Identifiers and keywords are ASCII letters, digits and
//! underscores; keywords are case-insensitive.

use crate::token::{keyword, Diagnostic, TokKind, Token};

pub const MAX_SOURCE_CHARS: usize = 100_000;

pub fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let char_count = source.chars().count();
    if char_count > MAX_SOURCE_CHARS {
        return Err(Diagnostic::error(
            1,
            1,
            format!(
                "Source is too large ({} characters); the limit is {} characters.",
                char_count, MAX_SOURCE_CHARS
            ),
        ));
    }

    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut toks: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;

    while i < n {
        let c = chars[i];
        match c {
            '\n' => {
                toks.push(Token { kind: TokKind::Newline, text: "\n".into(), line, col });
                i += 1;
                line += 1;
                col = 1;
            }
            '\r' => {
                i += 1; // tolerate CRLF; the \n handler emits the newline token
            }
            ' ' | '\t' => {
                i += 1;
                col += if c == '\t' { 4 } else { 1 };
            }
            '#' => {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if i + 1 < n && chars[i + 1] == '/' => {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
            }
            '{' => push_simple(&mut toks, TokKind::LBrace, "{", &mut i, line, &mut col),
            '}' => push_simple(&mut toks, TokKind::RBrace, "}", &mut i, line, &mut col),
            '(' => push_simple(&mut toks, TokKind::LParen, "(", &mut i, line, &mut col),
            ')' => push_simple(&mut toks, TokKind::RParen, ")", &mut i, line, &mut col),
            '=' if i + 1 < n && chars[i + 1] == '=' => {
                toks.push(Token { kind: TokKind::EqEq, text: "==".into(), line, col });
                i += 2;
                col += 2;
            }
            '<' if i + 1 < n && chars[i + 1] == '=' => {
                toks.push(Token { kind: TokKind::Le, text: "<=".into(), line, col });
                i += 2;
                col += 2;
            }
            '>' if i + 1 < n && chars[i + 1] == '=' => {
                toks.push(Token { kind: TokKind::Ge, text: ">=".into(), line, col });
                i += 2;
                col += 2;
            }
            '!' if i + 1 < n && chars[i + 1] == '=' => {
                toks.push(Token { kind: TokKind::Ne, text: "!=".into(), line, col });
                i += 2;
                col += 2;
            }
            '.' if i + 1 < n && chars[i + 1] == '.' => {
                toks.push(Token { kind: TokKind::DotDot, text: "..".into(), line, col });
                i += 2;
                col += 2;
            }
            '=' => push_simple(&mut toks, TokKind::Assign, "=", &mut i, line, &mut col),
            '<' => push_simple(&mut toks, TokKind::Lt, "<", &mut i, line, &mut col),
            '>' => push_simple(&mut toks, TokKind::Gt, ">", &mut i, line, &mut col),
            '+' => push_simple(&mut toks, TokKind::Plus, "+", &mut i, line, &mut col),
            '*' => push_simple(&mut toks, TokKind::Star, "*", &mut i, line, &mut col),
            '/' => push_simple(&mut toks, TokKind::Slash, "/", &mut i, line, &mut col),
            '%' => push_simple(&mut toks, TokKind::Percent, "%", &mut i, line, &mut col),
            '^' => push_simple(&mut toks, TokKind::Caret, "^", &mut i, line, &mut col),
            ',' => push_simple(&mut toks, TokKind::Comma, ",", &mut i, line, &mut col),
            ';' => push_simple(&mut toks, TokKind::Semicolon, ";", &mut i, line, &mut col),
            '-' => push_simple(&mut toks, TokKind::Minus, "-", &mut i, line, &mut col),
            c if c.is_ascii_digit() => {
                let start_col = col;
                let mut text = String::new();
                while i < n && chars[i].is_ascii_digit() {
                    text.push(chars[i]);
                    i += 1;
                    col += 1;
                }
                // Overflow-safe i64 parsing of the literal.
                if text.parse::<i64>().is_err() {
                    return Err(Diagnostic::error(
                        line,
                        start_col,
                        format!("Number literal '{}' is too large.", text),
                    )
                    .with_hint("Supported numbers fit in a signed 64-bit integer."));
                }
                toks.push(Token { kind: TokKind::Number, text, line, col: start_col });
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let start_col = col;
                let mut text = String::new();
                while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    text.push(chars[i]);
                    i += 1;
                    col += 1;
                }
                let kind = keyword(&text).unwrap_or(TokKind::Ident);
                toks.push(Token { kind, text, line, col: start_col });
            }
            other => {
                return Err(Diagnostic::error(
                    line,
                    col,
                    format!("Unexpected character '{}'.", other),
                )
                .with_hint(
                    "DMath supports letters, digits, spaces and the symbols = == != < > <= >= + - * / % ^ .. { } ( ) , ; #",
                ));
            }
        }
    }

    toks.push(Token { kind: TokKind::Eof, text: String::new(), line, col });
    Ok(toks)
}

fn push_simple(
    toks: &mut Vec<Token>,
    kind: TokKind,
    text: &str,
    i: &mut usize,
    line: usize,
    col: &mut usize,
) {
    toks.push(Token { kind, text: text.into(), line, col: *col });
    *i += 1;
    *col += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_keywords_case_insensitively() {
        let toks = lex("p and q OR r\n").unwrap();
        let kinds: Vec<TokKind> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokKind::Ident,
                TokKind::And,
                TokKind::Ident,
                TokKind::Or,
                TokKind::Ident,
                TokKind::Newline,
                TokKind::Eof
            ]
        );
    }

    #[test]
    fn tracks_line_and_column() {
        let toks = lex("A = {1,\n  2}").unwrap();
        let brace = toks.iter().find(|t| t.kind == TokKind::RBrace).unwrap();
        assert_eq!((brace.line, brace.col), (2, 4));
    }

    #[test]
    fn comments_are_skipped() {
        let toks = lex("# a comment\nA = 1 // trailing").unwrap();
        assert!(toks.iter().any(|t| t.kind == TokKind::Number && t.text == "1"));
    }

    #[test]
    fn unknown_character_is_an_error() {
        let d = lex("A = @").unwrap_err();
        assert_eq!(d.line, 1);
        assert!(d.message.contains('@'));
    }
}
