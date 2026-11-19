//! XOR, NAND, NOR, SYMDIFF and the set utilities SIZE, CONTAINS, IS_DISJOINT.

use dmath_core::run_source;
use serde_json::{json, Value};

fn run(src: &str) -> Value {
    serde_json::to_value(run_source(src)).unwrap()
}

fn shown(src: &str) -> String {
    let j = run(src);
    let errs: Vec<&str> = j["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["severity"] == "error")
        .map(|d| d["message"].as_str().unwrap())
        .collect();
    assert!(errs.is_empty(), "unexpected errors for {src:?}: {errs:?}");
    j["outputs"][0]["display"].as_str().unwrap().to_string()
}

fn error_of(src: &str) -> (String, Option<String>) {
    let j = run(src);
    let d = j["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["severity"] == "error")
        .unwrap_or_else(|| panic!("expected an error for {src:?}"))
        .clone();
    (d["message"].as_str().unwrap().to_string(), d["hint"].as_str().map(String::from))
}

#[test]
fn xor_nand_nor_truth_values() {
    for (p, q) in [(true, true), (true, false), (false, true), (false, false)] {
        let b = |v: bool| if v { "TRUE" } else { "FALSE" };
        assert_eq!(shown(&format!("DISPLAY {} XOR {}", b(p), b(q))), b(p != q));
        assert_eq!(shown(&format!("DISPLAY {} NAND {}", b(p), b(q))), b(!(p && q)));
        assert_eq!(shown(&format!("DISPLAY {} NOR {}", b(p), b(q))), b(!(p || q)));
    }
}

#[test]
fn precedence_and_below_xor_below_or() {
    // AND binds tighter than XOR, XOR tighter than OR.
    // TRUE OR (FALSE XOR (TRUE AND TRUE)) = TRUE
    assert_eq!(shown("DISPLAY TRUE OR FALSE XOR TRUE AND TRUE"), "TRUE");
    // FALSE XOR TRUE AND TRUE = FALSE XOR TRUE = TRUE
    assert_eq!(shown("DISPLAY FALSE XOR TRUE AND TRUE"), "TRUE");
    // (TRUE XOR TRUE) OR TRUE = TRUE, but TRUE XOR (TRUE OR TRUE) = FALSE
    assert_eq!(shown("DISPLAY (TRUE XOR TRUE) OR FALSE"), "FALSE");
    assert_eq!(shown("DISPLAY TRUE XOR (TRUE OR TRUE)"), "FALSE");
}

#[test]
fn nand_and_nor_are_left_associative_not_associative() {
    // (T NAND T) NAND F = F NAND F = T ; T NAND (T NAND F) = T NAND T = F
    assert_eq!(shown("DISPLAY TRUE NAND TRUE NAND FALSE"), "TRUE");
    assert_eq!(shown("DISPLAY TRUE NAND (TRUE NAND FALSE)"), "FALSE");
}

#[test]
fn trace_keeps_parentheses_that_matter() {
    let j = run("P = TRUE\nQ = FALSE\nR = TRUE\nA = P XOR (Q XOR R)\nB = P NAND (Q NAND R)\nC = (P XOR Q) XOR R\nD = P OR Q NOR R");
    let stmts: Vec<&str> =
        j["trace"].as_array().unwrap().iter().map(|t| t["statement"].as_str().unwrap()).collect();
    assert_eq!(stmts[3], "A = P XOR (Q XOR R)");
    assert_eq!(stmts[4], "B = P NAND (Q NAND R)");
    assert_eq!(stmts[5], "C = P XOR Q XOR R");
    assert_eq!(stmts[6], "D = P OR Q NOR R");
}

#[test]
fn xor_identities_are_proved_by_equivalence_check() {
    let eq = |a: &str, b: &str| {
        let j = run(&format!("P = TRUE\nQ = TRUE\nCHECK EQUIVALENT {a}, {b}"));
        j["visualizations"][0]["equivalent"] == json!(true)
    };
    assert!(eq("P XOR Q", "(P OR Q) AND NOT (P AND Q)"));
    assert!(eq("P XOR Q", "NOT (P IFF Q)"));
    assert!(eq("P NAND Q", "NOT (P AND Q)"));
    assert!(eq("P NOR Q", "NOT (P OR Q)"));
    assert!(eq("P NAND Q", "NOT P OR NOT Q")); // De Morgan
    assert!(eq("P NOR Q", "NOT P AND NOT Q")); // De Morgan
    assert!(!eq("P XOR Q", "P OR Q"));
}

#[test]
fn xor_truth_table_has_expected_rows() {
    let j = run("P = TRUE\nQ = TRUE\nVISUALIZE P XOR Q AS TRUTH_TABLE");
    let viz = &j["visualizations"][0];
    assert_eq!(viz["type"], "truth_table");
    let results: Vec<bool> =
        viz["rows"].as_array().unwrap().iter().map(|r| r["result"].as_bool().unwrap()).collect();
    assert_eq!(results, vec![false, true, true, false]);
}

#[test]
fn symdiff_matches_union_minus_intersection() {
    assert_eq!(shown("DISPLAY {1,2,3,4} SYMDIFF {3,4,5}"), "{1, 2, 5}");
    assert_eq!(
        shown("A = {1,2,3,4}\nB = {3,4,5}\nDISPLAY (A UNION B) DIFFERENCE (A INTERSECT B)"),
        "{1, 2, 5}"
    );
    assert_eq!(shown("DISPLAY {1,2} SYMDIFF {1,2}"), "{}");
    // Same precedence as UNION / DIFFERENCE, left to right.
    assert_eq!(shown("DISPLAY {1,2} SYMDIFF {2,3} UNION {9}"), "{1, 3, 9}");
}

#[test]
fn symdiff_venn_regions() {
    let j = run("A = {1,2,3,4}\nB = {3,4,5}\nC = A SYMDIFF B\nVISUALIZE C AS VENN");
    let v = &j["visualizations"][0];
    assert_eq!(v["type"], "venn");
    assert_eq!(v["operation"], "SYMDIFF");
    assert_eq!(v["operation_label"], "Symmetric difference");
    assert_eq!(v["regions"]["leftOnly"], json!([1, 2]));
    assert_eq!(v["regions"]["intersection"], json!([3, 4]));
    assert_eq!(v["regions"]["rightOnly"], json!([5]));
}

#[test]
fn size_contains_and_disjoint() {
    assert_eq!(shown("DISPLAY SIZE({4, 4, 7, 9})"), "3");
    assert_eq!(shown("DISPLAY SIZE({})"), "0");
    assert_eq!(shown("DISPLAY SIZE({(1,2),(2,3)})"), "2");
    assert_eq!(shown("DISPLAY CONTAINS({1,2,3}, 2)"), "TRUE");
    assert_eq!(shown("DISPLAY CONTAINS({1,2,3}, 5)"), "FALSE");
    assert_eq!(shown("DISPLAY CONTAINS({(1,2)}, 1)"), "FALSE");
    assert_eq!(shown("DISPLAY IS_DISJOINT({1,2}, {3,4})"), "TRUE");
    assert_eq!(shown("DISPLAY IS_DISJOINT({1,2}, {2,3})"), "FALSE");
    // Inclusion-exclusion: |A U B| = |A| + |B| - |A n B| (5 = 4 + 3 - 2)
    let src = "A = {1,2,3,4}\nB = {3,4,5}\n";
    assert_eq!(shown(&format!("{src}DISPLAY SIZE(A UNION B)")), "5");
    assert_eq!(shown(&format!("{src}DISPLAY SIZE(A INTERSECT B)")), "2");
    assert_eq!(shown(&format!("{src}DISPLAY SIZE(A SYMDIFF B)")), "3");
}

#[test]
fn type_errors_have_helpful_hints() {
    let (msg, hint) = error_of("A = {1,2}\nx = A XOR TRUE");
    assert!(msg.contains("'XOR' requires Boolean operands"), "{msg}");
    assert!(hint.unwrap().contains("SYMDIFF"));
    let (msg, _) = error_of("x = TRUE SYMDIFF {1}");
    assert!(msg.contains("'SYMDIFF' requires set operands"), "{msg}");
    let (msg, hint) = error_of("x = SIZE(5)");
    assert!(msg.contains("'SIZE' requires a set"), "{msg}");
    assert!(hint.unwrap().contains("SIZE(set)"));
    let (msg, _) = error_of("x = CONTAINS({1,2})");
    assert!(msg.contains("expects 2 arguments"), "{msg}");
}

#[test]
fn keywords_are_case_insensitive() {
    assert_eq!(shown("DISPLAY true xor false"), "TRUE");
    assert_eq!(shown("DISPLAY {1} symdiff {2}"), "{1, 2}");
}

#[test]
fn powerset_lists_every_subset_smallest_first() {
    assert_eq!(
        shown("DISPLAY POWERSET({1, 2, 3})"),
        "{{}, {1}, {2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}"
    );
    assert_eq!(shown("DISPLAY POWERSET({})"), "{{}}");
    assert_eq!(shown("DISPLAY POWERSET({7})"), "{{}, {7}}");
}

#[test]
fn powerset_has_two_to_the_n_subsets() {
    for n in 0..=10u32 {
        let elems: Vec<String> = (1..=n).map(|i| i.to_string()).collect();
        let src = format!("DISPLAY SIZE(POWERSET({{{}}}))", elems.join(", "));
        assert_eq!(shown(&src), (1u64 << n).to_string(), "n = {n}");
    }
}

#[test]
fn powerset_is_a_normal_value_you_can_store_and_query() {
    let src = "A = {1, 2, 3}\nP = POWERSET(A)\n";
    assert_eq!(shown(&format!("{src}DISPLAY CONTAINS(P, {{1, 3}})")), "TRUE");
    assert_eq!(shown(&format!("{src}DISPLAY CONTAINS(P, {{3, 1}})")), "TRUE"); // order is irrelevant
    assert_eq!(shown(&format!("{src}DISPLAY CONTAINS(P, {{4}})")), "FALSE");
    assert_eq!(shown(&format!("{src}DISPLAY CONTAINS(P, {{}})")), "TRUE");
    assert_eq!(shown(&format!("{src}DISPLAY CONTAINS(P, 1)")), "FALSE");
    // Every subset of A is in P(A): the empty set and A itself included.
    assert_eq!(shown(&format!("{src}DISPLAY CONTAINS(P, A)")), "TRUE");
}

#[test]
fn nested_set_literals_ignore_order_and_duplicates() {
    assert_eq!(shown("DISPLAY {{2, 1}, {1, 2}}"), "{{1, 2}}");
    assert_eq!(
        shown("DISPLAY POWERSET({1, 2}) EQUALS {{}, {1}, {2}, {1, 2}}"),
        "TRUE"
    );
    assert_eq!(shown("DISPLAY POWERSET({1, 2}) EQUALS {{}, {1}, {2}}"), "FALSE");
    // Set operations work on a power set like on any set.
    assert_eq!(
        shown("DISPLAY SIZE(POWERSET({1, 2, 3}) INTERSECT POWERSET({2, 3, 4}))"),
        "4"
    );
}

#[test]
fn powerset_of_powerset_and_subset_relation() {
    assert_eq!(shown("DISPLAY SIZE(POWERSET(POWERSET({1, 2})))"), "16");
    assert_eq!(shown("DISPLAY POWERSET({1}) SUBSET POWERSET({1, 2})"), "TRUE");
}

#[test]
fn powerset_guards_and_mixing_errors() {
    let (msg, hint) = error_of("x = POWERSET({1,2,3,4,5,6,7,8,9,10,11})");
    assert!(msg.contains("at most 10 elements"), "{msg}");
    assert!(hint.unwrap().contains("2048"));
    let (msg, _) = error_of("x = {{1}, 2}");
    assert!(msg.contains("cannot mix nested sets"), "{msg}");
    let (msg, _) = error_of("x = POWERSET(5)");
    assert!(msg.contains("'POWERSET' requires a set"), "{msg}");
    let (msg, _) = error_of("P = POWERSET({1,2})\nx = TRANSITIVE_CLOSURE(P)");
    assert!(msg.contains("non-pair"), "{msg}");
}

#[test]
fn trace_prints_nested_literals_as_typed() {
    let j = run("P = {{1, 2}, {3}}\nQ = POWERSET({1})");
    let stmts: Vec<&str> =
        j["trace"].as_array().unwrap().iter().map(|t| t["statement"].as_str().unwrap()).collect();
    assert_eq!(stmts[0], "P = {{1, 2}, {3}}");
    assert_eq!(stmts[1], "Q = POWERSET({1})");
    // JSON: nested sets travel as text so they are never confused with tuples.
    assert_eq!(j["outputs"].as_array().map(|a| a.len()), Some(0));
}
