//! Runs the program and returns structured output blocks for the frontend.
//! STUB: lexes the source and shows the tokens (or lexical errors).
//! Next: lexer -> parser (AST) -> interpreter.

use crate::lexer::{lex, TokenKind};
use serde::Serialize;

/// Sent to the frontend as JSON. Add variants later, e.g. TruthTable, Venn, Graph.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Block {
    Text { text: String },
    Error { line: usize, message: String },
}

pub fn run(source: &str) -> Vec<Block> {
    match lex(source) {
        Err(errors) => errors
            .into_iter()
            .map(|e| Block::Error {
                line: e.line,
                message: format!("col {}: {}", e.col, e.message),
            })
            .collect(),
        Ok(tokens) => {
            let mut out = Vec::new();
            let mut current: Vec<String> = Vec::new();
            let mut line_no = 1;
            for t in tokens {
                match t.kind {
                    TokenKind::Newline | TokenKind::Eof => {
                        if !current.is_empty() {
                            out.push(Block::Text {
                                text: format!("[line {}] {}", line_no, current.join(" ")),
                            });
                            current.clear();
                        }
                        line_no = t.line + 1;
                    }
                    k => current.push(format!("{:?}", k)),
                }
            }
            out
        }
    }
}
