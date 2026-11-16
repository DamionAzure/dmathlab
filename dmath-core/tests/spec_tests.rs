//! Spec §8 — Testing and Validation for DMath Core.
//!
//! Covers: propositional logic (§8.1), set theory (§8.2), the DSL itself
//! (§8.3) and visualization contracts (§8.4). Tests run against the public
//! `run_source` entry point and assert on the serialized JSON envelope, so
//! the wire contract is validated too.

use dmath_core::run_source;
use serde_json::Value;

fn run(src: &str) -> Value {
    let env = run_source(src);
    serde_json::to_value(&env).expect("envelope must serialize")
}

fn errors(json: &Value) -> Vec<&str> {
    json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| d["message"].as_str().unwrap())
        .collect()
}

fn warnings(json: &Value) -> Vec<&str> {
    json["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "warning")
        .map(|d| d["message"].as_str().unwrap())
        .collect()
}

fn has_error_containing(json: &Value, needle: &str) -> bool {
    errors(json).iter().any(|m| m.contains(needle))
}

fn output_display(json: &Value, name: &str) -> String {
    json["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["name"] == name || (!name.is_empty() && o["expression"] == name))
        .map(|o| o["display"].as_str().unwrap().to_string())
        .unwrap_or_else(|| panic!("no output named {}", name))
}

// ===========================================================================
// §8.1 Propositional logic
// ===========================================================================

#[test]
fn pl_and_or_not_implies_iff_truth_values() {
    // AND
    assert_eq!(output_display(&run("P = TRUE\nQ = TRUE\nR = P AND Q\nDISPLAY R"), "R"), "TRUE");
    assert_eq!(output_display(&run("P = TRUE\nQ = FALSE\nR = P AND Q\nDISPLAY R"), "R"), "FALSE");
    // OR
    assert_eq!(output_display(&run("P = FALSE\nQ = TRUE\nR = P OR Q\nDISPLAY R"), "R"), "TRUE");
    assert_eq!(output_display(&run("P = FALSE\nQ = FALSE\nR = P OR Q\nDISPLAY R"), "R"), "FALSE");
    // NOT
    assert_eq!(output_display(&run("P = TRUE\nR = NOT P\nDISPLAY R"), "R"), "FALSE");
    assert_eq!(output_display(&run("P = FALSE\nR = NOT P\nDISPLAY R"), "R"), "TRUE");
    // IMPLIES (T->T=T, T->F=F, F->T=T, F->F=T)
    assert_eq!(output_display(&run("P = TRUE\nQ = TRUE\nR = P IMPLIES Q\nDISPLAY R"), "R"), "TRUE");
    assert_eq!(output_display(&run("P = TRUE\nQ = FALSE\nR = P IMPLIES Q\nDISPLAY R"), "R"), "FALSE");
    assert_eq!(output_display(&run("P = FALSE\nQ = TRUE\nR = P IMPLIES Q\nDISPLAY R"), "R"), "TRUE");
    assert_eq!(output_display(&run("P = FALSE\nQ = FALSE\nR = P IMPLIES Q\nDISPLAY R"), "R"), "TRUE");
    // IFF
    assert_eq!(output_display(&run("P = TRUE\nQ = TRUE\nR = P IFF Q\nDISPLAY R"), "R"), "TRUE");
    assert_eq!(output_display(&run("P = TRUE\nQ = FALSE\nR = P IFF Q\nDISPLAY R"), "R"), "FALSE");
}

#[test]
fn pl_truth_table_row_counts_and_values() {
    let json = run("P = TRUE\nQ = TRUE\nR = TRUE\nVISUALIZE (P AND Q) OR R AS TRUTH_TABLE");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["type"], "truth_table");
    assert_eq!(viz["row_count"], 8);
    assert_eq!(viz["variables"], serde_json::json!(["P", "Q", "R"]));
    let rows = viz["rows"].as_array().unwrap();
    // Enumeration order: first variable is the most significant bit.
    assert_eq!(rows[0], serde_json::json!({"P": true, "Q": true, "R": true, "result": true}));
    assert_eq!(rows[1], serde_json::json!({"P": true, "Q": true, "R": false, "result": true}));
    assert_eq!(rows[2]["result"], serde_json::json!(true));  // (T AND F) OR T
    // Row where P=T, Q=F, R=F: (T AND F) OR F = FALSE
    assert_eq!(rows[3]["result"], serde_json::json!(false));
}

#[test]
fn pl_truth_table_ignores_runtime_assignment() {
    // Spec §2.1: even with P/Q assigned, the table enumerates all combos.
    let json = run("P = TRUE\nQ = FALSE\nRESULT = P AND Q\nVISUALIZE RESULT AS TRUTH_TABLE");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["row_count"], 4);
    assert_eq!(viz["expression"], "P AND Q");
    assert_eq!(viz["classification"], "contingency");
}

#[test]
fn pl_tautology_classification() {
    let json = run("P = TRUE\nVISUALIZE (P OR NOT P) AS TRUTH_TABLE");
    assert_eq!(json["visualizations"][0]["classification"], "tautology");
    assert_eq!(json["visualizations"][0]["row_count"], 2);
}

#[test]
fn pl_contradiction_classification() {
    let json = run("P = TRUE\nVISUALIZE (P AND NOT P) AS TRUTH_TABLE");
    assert_eq!(json["visualizations"][0]["classification"], "contradiction");
}

#[test]
fn pl_contingency_classification() {
    let json = run("P = TRUE\nQ = FALSE\nVISUALIZE (P AND Q) AS TRUTH_TABLE");
    assert_eq!(json["visualizations"][0]["classification"], "contingency");
}

#[test]
fn pl_iff_chain_is_tautology() {
    let json = run(
        "P = TRUE\nQ = TRUE\nR = (P IFF Q) IFF ((P IMPLIES Q) AND (Q IMPLIES P))\nVISUALIZE R AS TRUTH_TABLE",
    );
    assert_eq!(json["visualizations"][0]["classification"], "tautology");
}

#[test]
fn pl_equivalence_of_material_implication() {
    // Spec §3.3: P IMPLIES Q  ≡  (NOT P) OR Q
    let json = run("P = TRUE\nQ = FALSE\nCHECK EQUIVALENT P IMPLIES Q, NOT P OR Q");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["type"], "equivalence_check");
    assert_eq!(viz["equivalent"], true);
    assert_eq!(viz["rows"].as_array().unwrap().len(), 4);
}

#[test]
fn pl_non_equivalence_converse() {
    // P IMPLIES Q  is NOT equivalent to  Q IMPLIES P
    let json = run("P = TRUE\nQ = TRUE\nCHECK EQUIVALENT P IMPLIES Q, Q IMPLIES P");
    assert_eq!(json["visualizations"][0]["equivalent"], false);
}

#[test]
fn pl_de_morgan_equivalences() {
    let a = run("P = TRUE\nQ = TRUE\nCHECK EQUIVALENT NOT (P AND Q), (NOT P) OR (NOT Q)");
    assert_eq!(a["visualizations"][0]["equivalent"], true);
    let b = run("P = TRUE\nQ = TRUE\nCHECK EQUIVALENT NOT (P OR Q), (NOT P) AND (NOT Q)");
    assert_eq!(b["visualizations"][0]["equivalent"], true);
}

#[test]
fn pl_closed_expression_single_row() {
    let json = run("VISUALIZE (TRUE AND FALSE) OR TRUE AS TRUTH_TABLE");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["row_count"], 1);
    assert_eq!(viz["rows"][0]["result"], serde_json::json!(true));
    assert_eq!(viz["classification"], "tautology");
}

// ===========================================================================
// §8.2 Set theory
// ===========================================================================

#[test]
fn st_union_intersection_difference() {
    let json = run(
        "A = {1, 2, 3, 4}\nB = {3, 4, 5, 6}\nC = A UNION B\nD = A INTERSECT B\nE = A DIFFERENCE B\nDISPLAY C, D, E",
    );
    assert_eq!(output_display(&json, "C"), "{1, 2, 3, 4, 5, 6}");
    assert_eq!(output_display(&json, "D"), "{3, 4}");
    assert_eq!(output_display(&json, "E"), "{1, 2}");
}

#[test]
fn st_complement_against_universal_set() {
    let json = run(
        "UNIVERSE = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10}\nA = {2, 4, 6, 8}\nC = COMPLEMENT A\nDISPLAY C",
    );
    assert_eq!(output_display(&json, "C"), "{1, 3, 5, 7, 9, 10}");
}

#[test]
fn st_complement_without_universe_errors() {
    let json = run("A = {1, 2}\nC = COMPLEMENT A\nDISPLAY C");
    assert!(has_error_containing(&json, "universal set"));
}

#[test]
fn st_subset_relationships() {
    let json = run("A = {1, 2, 3}\nB = {1, 2, 3, 4}\nS1 = A SUBSET B\nS2 = B SUBSET A\nS3 = A SUBSET A\nDISPLAY S1, S2, S3");
    assert_eq!(output_display(&json, "S1"), "TRUE");
    assert_eq!(output_display(&json, "S2"), "FALSE");
    assert_eq!(output_display(&json, "S3"), "TRUE");
}

#[test]
fn st_set_equality() {
    let json = run("A = {1, 2, 3}\nB = {3, 2, 1}\nQ1 = A EQUALS B\nQ2 = A EQUALS {1, 2}\nDISPLAY Q1, Q2");
    assert_eq!(output_display(&json, "Q1"), "TRUE"); // order is irrelevant
    assert_eq!(output_display(&json, "Q2"), "FALSE");
}

#[test]
fn st_cartesian_product() {
    let json = run("A = {1, 2}\nB = {3, 4}\nP = A CROSS B\nDISPLAY P");
    assert_eq!(output_display(&json, "P"), "{(1, 3), (1, 4), (2, 3), (2, 4)}");
}

#[test]
fn st_duplicate_elimination() {
    let json = run("A = {1, 2, 2, 3, 3, 3}\nDISPLAY A");
    assert_eq!(output_display(&json, "A"), "{1, 2, 3}");
    // A warning points out the duplicates.
    assert!(warnings(&json).iter().any(|m| m.contains("Duplicate element 2")));
}

#[test]
fn st_union_dedupes_shared_elements() {
    let json = run("A = {1, 2}\nB = {2, 3}\nC = A UNION B\nDISPLAY C");
    assert_eq!(output_display(&json, "C"), "{1, 2, 3}");
}

#[test]
fn st_empty_set_operations() {
    let json = run("A = {}\nB = {1, 2}\nC = A UNION B\nD = A INTERSECT B\nDISPLAY C, D");
    assert_eq!(output_display(&json, "C"), "{1, 2}");
    assert_eq!(output_display(&json, "D"), "{}");
}

#[test]
fn st_set_ops_inside_logic() {
    let json = run(
        "A = {1, 2}\nB = {1, 2, 3}\nR = (A SUBSET B) AND NOT (B SUBSET A)\nDISPLAY R",
    );
    // TRUE AND NOT FALSE = TRUE
    assert_eq!(output_display(&json, "R"), "TRUE");
}

// ===========================================================================
// §8.3 DSL behavior
// ===========================================================================

#[test]
fn dsl_valid_assignments_and_nested_expressions() {
    let json = run(
        "A = {1, 2, 3}\nB = {3, 4, 5}\nRESULT = (A UNION B) INTERSECT {3, 4, 5}\nDISPLAY RESULT",
    );
    assert_eq!(output_display(&json, "RESULT"), "{3, 4, 5}");
}

#[test]
fn dsl_operator_precedence_logic() {
    // AND binds tighter than OR: FALSE OR (TRUE AND FALSE) = FALSE
    let json = run("R = FALSE OR TRUE AND FALSE\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "FALSE");
    // NOT binds tighter than AND: (NOT TRUE) AND TRUE = FALSE
    let json = run("R = NOT TRUE AND TRUE\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "FALSE");
    // IMPLIES binds looser than OR: (TRUE OR FALSE) IMPLIES TRUE = TRUE
    let json = run("R = TRUE OR FALSE IMPLIES TRUE\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "TRUE");
    // Right associativity: TRUE IMPLIES (FALSE IMPLIES FALSE) = TRUE
    let json = run("R = TRUE IMPLIES FALSE IMPLIES FALSE\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "TRUE");
}

#[test]
fn dsl_operator_precedence_sets() {
    // INTERSECT binds tighter than UNION: {1,2} UNION ({2,3} INTERSECT {3,4}) = {1,2,3}
    let json = run("R = {1, 2} UNION {2, 3} INTERSECT {3, 4}\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "{1, 2, 3}");
    // Parentheses override: ({1, 2} UNION {2, 3}) INTERSECT {3, 4} = {3}
    let json = run("R = ({1, 2} UNION {2, 3}) INTERSECT {3, 4}\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "{3}");
    // DIFFERENCE is left-associative: ({5,4,3} - {4}) - {3} = {5}
    let json = run("R = {5, 4, 3} DIFFERENCE {4} DIFFERENCE {3}\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "{5}");
}

#[test]
fn dsl_parenthesized_logic() {
    let json = run("R = (TRUE OR FALSE) AND (FALSE OR TRUE)\nDISPLAY R");
    assert_eq!(output_display(&json, "R"), "TRUE");
}

#[test]
fn dsl_undefined_variable_is_an_error() {
    let json = run("R = P AND Q\nDISPLAY R");
    assert!(has_error_containing(&json, "Undefined variable 'P'"));
    // The statement is skipped, so there is no output and no result value.
    assert!(json["outputs"].as_array().unwrap().is_empty());
}

#[test]
fn dsl_type_error_set_and_boolean() {
    // Spec §3.4: A AND P must produce a type error, not a silent result.
    let json = run("A = {1, 2, 3}\nP = TRUE\nRESULT = A AND P\nDISPLAY RESULT");
    assert!(has_error_containing(&json, "Type error"));
    assert!(has_error_containing(&json, "'AND'"));
}

#[test]
fn dsl_type_error_logic_on_sets() {
    let json = run("A = {1}\nB = {2}\nR = A OR B\nDISPLAY R");
    assert!(has_error_containing(&json, "Type error"));
}

#[test]
fn dsl_invalid_syntax_missing_brace() {
    let json = run("A = {1, 2\nDISPLAY A");
    assert!(has_error_containing(&json, "'}'"));
}

#[test]
fn dsl_invalid_syntax_stray_operator() {
    let json = run("R = P AND\nDISPLAY R");
    assert!(!errors(&json).is_empty());
}

#[test]
fn dsl_error_recovery_continues_next_statements() {
    let json = run(
        "BROKEN = P AND\nA = {1, 2}\nDISPLAY A",
    );
    // The broken line produces an error...
    assert!(!errors(&json).is_empty());
    // ...but the following statements still execute.
    assert_eq!(output_display(&json, "A"), "{1, 2}");
    assert_eq!(json["ok"], false);
}

#[test]
fn dsl_repeated_execution_fresh_environment() {
    let src = "A = {1, 2}\nB = A UNION {3}\nDISPLAY B";
    let first = run(src);
    let second = run(src);
    assert_eq!(first, second); // deterministic, no leaked state between runs
    assert_eq!(output_display(&second, "B"), "{1, 2, 3}");
}

#[test]
fn dsl_variables_can_be_reassigned() {
    let json = run("A = {1}\nA = A UNION {2}\nDISPLAY A");
    assert_eq!(output_display(&json, "A"), "{1, 2}");
}

#[test]
fn dsl_case_insensitive_keywords() {
    let json = run("A = {1, 2}\nB = {2, 3}\nC = A union B intersect {2}\nDisplay C");
    assert_eq!(output_display(&json, "C"), "{1, 2}");
}

#[test]
fn dsl_display_inline_expression() {
    let json = run("DISPLAY {1, 2} UNION {3}\nDISPLAY TRUE AND FALSE");
    let outs = json["outputs"].as_array().unwrap();
    assert_eq!(outs[0]["expression"], "{1, 2} UNION {3}");
    assert_eq!(outs[0]["display"], "{1, 2, 3}");
    assert_eq!(outs[1]["display"], "FALSE");
    assert_eq!(outs[0]["name"], serde_json::Value::Null); // inline, unnamed
}

#[test]
fn dsl_truth_table_var_limit() {
    let mut src = String::new();
    for i in 1..=13 {
        src.push_str(&format!("P{} = TRUE\n", i));
    }
    src.push_str("A = P1 AND P2 AND P3 AND P4 AND P5 AND P6 AND P7 AND P8 AND P9 AND P10 AND P11 AND P12 AND P13\nVISUALIZE A AS TRUTH_TABLE");
    let json = run(&src);
    assert!(has_error_containing(&json, "limit"));
}

// ===========================================================================
// §8.4 Visualization contracts
// ===========================================================================

#[test]
fn viz_truth_table_rows_match_expression() {
    let json = run("P = TRUE\nQ = TRUE\nVISUALIZE (P IMPLIES Q) AND (Q IMPLIES P) AS TRUTH_TABLE");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["type"], "truth_table");
    let rows = viz["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    // P IFF Q is TRUE only when both match.
    assert_eq!(rows[0]["result"], serde_json::json!(true));  // T T
    assert_eq!(rows[1]["result"], serde_json::json!(false)); // T F
    assert_eq!(rows[2]["result"], serde_json::json!(false)); // F T
    assert_eq!(rows[3]["result"], serde_json::json!(true));  // F F
}

#[test]
fn viz_venn_represents_union_operation() {
    let json = run("A = {1, 2, 3, 4}\nB = {3, 4, 5, 6}\nC = A UNION B\nVISUALIZE C AS VENN");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["type"], "venn");
    assert_eq!(viz["operation"], "UNION");
    assert_eq!(viz["operands"]["A"], serde_json::json!([1, 2, 3, 4]));
    assert_eq!(viz["operands"]["B"], serde_json::json!([3, 4, 5, 6]));
    assert_eq!(viz["result"], serde_json::json!([1, 2, 3, 4, 5, 6]));
    assert_eq!(viz["regions"]["leftOnly"], serde_json::json!([1, 2]));
    assert_eq!(viz["regions"]["intersection"], serde_json::json!([3, 4]));
    assert_eq!(viz["regions"]["rightOnly"], serde_json::json!([5, 6]));
}

#[test]
fn viz_venn_outside_region_uses_universe() {
    let json = run(
        "UNIVERSE = {1, 2, 3, 4, 5, 6}\nA = {1, 2}\nB = {2, 3}\nC = A UNION B\nVISUALIZE C AS VENN",
    );
    let viz = &json["visualizations"][0];
    assert_eq!(viz["universe"], serde_json::json!([1, 2, 3, 4, 5, 6]));
    assert_eq!(viz["regions"]["outside"], serde_json::json!([4, 5, 6]));
}

#[test]
fn viz_venn_complement_shades_outside() {
    let json = run(
        "UNIVERSE = {1, 2, 3, 4, 5}\nA = {1, 2}\nC = COMPLEMENT A\nVISUALIZE C AS VENN",
    );
    let viz = &json["visualizations"][0];
    assert_eq!(viz["operation"], "complement");
    assert_eq!(viz["regions"]["leftOnly"], serde_json::json!([1, 2]));
    assert_eq!(viz["regions"]["outside"], serde_json::json!([3, 4, 5]));
}

#[test]
fn viz_malformed_truth_table_request() {
    let json = run("A = {1, 2}\nVISUALIZE A AS TRUTH_TABLE");
    assert!(has_error_containing(&json, "TRUTH_TABLE"));
}

#[test]
fn viz_malformed_venn_request() {
    let json = run("P = TRUE\nVISUALIZE P AS VENN");
    assert!(has_error_containing(&json, "VENN"));
}

#[test]
fn viz_venn_rejects_tuple_sets() {
    let json = run("A = {1, 2}\nB = {3, 4}\nP = A CROSS B\nVISUALIZE P AS VENN");
    assert!(has_error_containing(&json, "ordered pairs"));
}

#[test]
fn viz_displayed_values_match_interpreter() {
    // The displayed value of an output must equal the trace summary.
    let json = run("A = {1, 2}\nB = {2, 3}\nC = A INTERSECT B\nDISPLAY C");
    let displayed = output_display(&json, "C");
    let summary = json["trace"][2]["summary"].as_str().unwrap();
    assert_eq!(summary, format!("C = {}", displayed));
}

#[test]
fn viz_execution_trace_records_intermediates() {
    // Spec ST-09: intermediate results for composed operations.
    let json = run("A = {1, 2, 3, 4}\nB = {3, 4, 5, 6}\nR = (A UNION B) INTERSECT {3, 4, 5}\nDISPLAY R");
    let steps = json["trace"].as_array().unwrap();
    let assign = &steps[2];
    assert_eq!(assign["kind"], "assignment");
    let inter = assign["intermediates"].as_array().unwrap();
    assert!(inter.iter().any(|i| i["expr"] == "(A UNION B) INTERSECT {3, 4, 5}"
        && i["result"] == "{3, 4, 5}"));
    assert!(inter.iter().any(|i| i["expr"] == "A UNION B"
        && i["result"] == "{1, 2, 3, 4, 5, 6}"));
}

#[test]
fn viz_logic_intermediates_show_substitution() {
    let json = run("P = TRUE\nQ = FALSE\nR = P IMPLIES Q\nDISPLAY R");
    let inter = &json["trace"][2]["intermediates"];
    let arr = inter.as_array().unwrap();
    assert!(arr.iter().any(|i| i["expr"] == "P IMPLIES Q"
        && i["result"] == "FALSE"
        && i["detail"] == "TRUE IMPLIES FALSE"));
}

#[test]
fn contract_envelope_shape() {
    let json = run("P = TRUE\nDISPLAY P");
    assert_eq!(json["contract"], "dmath.viz.v1");
    assert!(json["ok"].is_boolean());
    assert!(json["outputs"].is_array());
    assert!(json["visualizations"].is_array());
    assert!(json["trace"].is_array());
    assert!(json["diagnostics"].is_array());
}

#[test]
fn contract_truth_table_json_matches_spec_example() {
    // Spec §4.4 example shape.
    let json = run("P = TRUE\nQ = FALSE\nRESULT = P AND Q\nVISUALIZE RESULT AS TRUTH_TABLE");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["type"], "truth_table");
    assert_eq!(viz["variables"], serde_json::json!(["P", "Q"]));
    assert_eq!(viz["expression"], "P AND Q");
    assert_eq!(viz["classification"], "contingency");
    let rows = viz["rows"].as_array().unwrap();
    assert_eq!(rows[0], serde_json::json!({"P": true, "Q": true, "result": true}));
    assert_eq!(rows[1], serde_json::json!({"P": true, "Q": false, "result": false}));
    assert_eq!(rows[2], serde_json::json!({"P": false, "Q": true, "result": false}));
    assert_eq!(rows[3], serde_json::json!({"P": false, "Q": false, "result": false}));
}

#[test]
fn contract_set_result_json_matches_spec_example() {
    // Spec §4.4: set_result fields ride along the venn IR.
    let json = run("A = {1, 2, 3, 4}\nB = {3, 4, 5, 6}\nC = A UNION B\nVISUALIZE C AS VENN");
    let viz = &json["visualizations"][0];
    assert_eq!(viz["operation"], "union".to_uppercase());
    assert_eq!(viz["operands"]["A"], serde_json::json!([1, 2, 3, 4]));
    assert_eq!(viz["operands"]["B"], serde_json::json!([3, 4, 5, 6]));
    assert_eq!(viz["result"], serde_json::json!([1, 2, 3, 4, 5, 6]));
}
