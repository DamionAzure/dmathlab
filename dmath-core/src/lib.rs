//! DMath Core — the computation engine of the DMath Visualizer.
//!
//! Pipeline: Lexer → Parser → Semantic Analyzer → Interpreter → Result
//! Generator. The engine produces mathematical results, execution traces and
//! a Visualization IR (JSON). It has no dependency on any graphical
//! framework, so it is linked natively into the desktop app and the CLI.

pub mod algo;
pub mod ast;
pub mod explain;
pub mod explain_graph;
pub mod interp;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod pretty;
pub mod semantic;
pub mod token;
pub mod value;

pub use ir::{Envelope, CONTRACT_VERSION};

/// Execute a DMath program and produce the structured result envelope.
///
/// This is the single entry point used by the CLI, the desktop app and the
/// test suite. Errors are reported per statement, so a broken line does not
/// prevent the remaining lines from executing.
pub fn run_source(source: &str) -> Envelope {
    let is_empty = source.trim().is_empty();

    let tokens = match lexer::lex(source) {
        Ok(t) => t,
        Err(d) => {
            let mut env = Envelope::empty();
            env.ok = false;
            env.diagnostics.push(d.into());
            return env;
        }
    };

    let (program, parse_diags) = parser::parse(tokens);
    let mut envelope = interp::run_program(parse_diags, &program);

    if is_empty && envelope.diagnostics.is_empty() {
        envelope.diagnostics.push(
            token::Diagnostic::info(1, 1, "No statements to execute: the program is empty.").into(),
        );
    }
    envelope
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source_is_ok() {
        let env = run_source("   \n# nothing\n");
        assert!(env.ok);
    }

    #[test]
    fn spec_example_logic() {
        let env = run_source(
            "P = TRUE\nQ = FALSE\nRESULT = P AND Q\nDISPLAY RESULT\nVISUALIZE RESULT AS TRUTH_TABLE\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        let out = &json["outputs"][0];
        assert_eq!(out["display"], "FALSE");
        assert_eq!(out["value_type"], "Boolean");
        let viz = &json["visualizations"][0];
        assert_eq!(viz["type"], "truth_table");
        assert_eq!(viz["classification"], "contingency");
        assert_eq!(viz["variables"], serde_json::json!(["P", "Q"]));
        assert_eq!(viz["rows"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn spec_example_sets() {
        let env = run_source(
            "A = {1, 2, 3, 4}\nB = {3, 4, 5, 6}\nC = A UNION B\nD = A INTERSECT B\nE = A DIFFERENCE B\nDISPLAY C\nDISPLAY D\nDISPLAY E\nVISUALIZE C AS VENN\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["outputs"][0]["display"], "{1, 2, 3, 4, 5, 6}");
        assert_eq!(json["outputs"][1]["display"], "{3, 4}");
        assert_eq!(json["outputs"][2]["display"], "{1, 2}");
        let viz = &json["visualizations"][0];
        assert_eq!(viz["type"], "venn");
        assert_eq!(viz["operation"], "UNION");
        assert_eq!(viz["regions"]["intersection"], serde_json::json!([3, 4]));
        assert_eq!(viz["regions"]["leftOnly"], serde_json::json!([1, 2]));
        assert_eq!(viz["regions"]["rightOnly"], serde_json::json!([5, 6]));
    }

    // ----- v1.2: number theory ---------------------------------------------

    #[test]
    fn number_theory_basics() {
        let env = run_source(
            "P1 = IS_PRIME(97)\nP2 = IS_PRIME(1)\nF = FACTORS(360)\nD = DIVISORS(12)\nG = GCD(48, 18)\nL = LCM(4, 6)\nT = TOTIENT(36)\nDISPLAY P1\nDISPLAY P2\nDISPLAY F\nDISPLAY D\nDISPLAY G\nDISPLAY L\nDISPLAY T\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        let outs = &json["outputs"];
        assert_eq!(outs[0]["display"], "TRUE");
        assert_eq!(outs[0]["value_type"], "Boolean");
        assert_eq!(outs[1]["display"], "FALSE");
        assert_eq!(outs[2]["display"], "[2, 2, 2, 3, 3, 5]");
        assert_eq!(outs[3]["display"], "[1, 2, 3, 4, 6, 12]");
        assert_eq!(outs[4]["display"], "6");
        assert_eq!(outs[5]["display"], "12");
        assert_eq!(outs[6]["display"], "12");
        assert_eq!(outs[6]["value_type"], "Number");
    }

    #[test]
    fn sieve_function_and_view() {
        let env = run_source(
            "S = SIEVE(30)\nDISPLAY S\nVISUALIZE S AS SIEVE\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(
            json["outputs"][0]["display"],
            "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29]"
        );
        let viz = &json["visualizations"][0];
        assert_eq!(viz["type"], "sieve_grid");
        assert_eq!(viz["limit"], 30);
        assert_eq!(viz["prime_count"], 10);
        assert_eq!(viz["smallest_factor"]["4"], 2);
        assert_eq!(viz["smallest_factor"]["9"], 3);
        assert_eq!(viz["smallest_factor"]["25"], 5);
        assert_eq!(viz["smallest_factor"]["23"], serde_json::Value::Null);
    }

    #[test]
    fn number_theory_domain_errors_recover() {
        // FACT(21) overflows 64 bits, a negative arg is rejected, and the
        // program keeps running afterwards (statement-level recovery).
        let env = run_source(
            "A = FACT(21)\nB = TOTIENT(0)\nC = COMB(3, 5)\nD = FACT(5)\nDISPLAY D\n",
        );
        assert!(!env.ok);
        assert_eq!(env.diagnostics.len(), 3, "three errors expected");
        let json = serde_json::to_value(&env).unwrap();
        assert!(json["outputs"][0]["display"].as_str().unwrap().contains("120"));
        assert_eq!(json["trace"][0]["status"], "error");
        assert_eq!(json["trace"][3]["status"], "ok");
    }

    // ----- v1.3: combinatorics ----------------------------------------------

    #[test]
    fn combinatorics_basics() {
        let env = run_source(
            "A = FACT(5)\nB = PERM(10, 2)\nC = COMB(52, 5)\nDISPLAY A\nDISPLAY B\nDISPLAY C\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        let outs = &json["outputs"];
        assert_eq!(outs[0]["display"], "120");
        assert_eq!(outs[1]["display"], "90");
        assert_eq!(outs[2]["display"], "2598960");
    }

    #[test]
    fn pascal_function_and_view() {
        let env = run_source(
            "P = PASCAL(5)\nDISPLAY P\nVISUALIZE P AS PASCAL\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["outputs"][0]["display"], "[(1), (1, 1), (1, 2, 1), (1, 3, 3, 1), (1, 4, 6, 4, 1)]");
        let viz = &json["visualizations"][0];
        assert_eq!(viz["type"], "pascal");
        assert_eq!(viz["row_count"], 5);
        assert_eq!(
            viz["rows"],
            serde_json::json!([[1], [1, 1], [1, 2, 1], [1, 3, 3, 1], [1, 4, 6, 4, 1]])
        );
        assert!(viz["highlight"].is_null());
    }

    #[test]
    fn comb_highlights_pascal_cell() {
        // A COMB result carries its (n, k) provenance into the view. C(n, k)
        // lives in row n (0-indexed), so n + 1 rows are shown.
        let env = run_source(
            "C = COMB(5, 2)\nDISPLAY C\nVISUALIZE C AS PASCAL\n",
        );
        assert!(env.ok, "diagnostics: {:?}", env.diagnostics);
        let json = serde_json::to_value(&env).unwrap();
        let viz = &json["visualizations"][0];
        assert_eq!(viz["type"], "pascal");
        assert_eq!(viz["highlight"], serde_json::json!([5, 2]));
        let rows = viz["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[5][2], 10); // the highlighted C(5, 2) cell exists
    }

    #[test]
    fn pascal_provenance_is_enforced() {
        // A plain list may not be visualized as a Pascal triangle.
        let env = run_source("L = DIVISORS(12)\nVISUALIZE L AS PASCAL\n");
        assert!(!env.ok);
        assert!(env.diagnostics.iter().any(|d| d.message.contains("PASCAL")));
    }
}
