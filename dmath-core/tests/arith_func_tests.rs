//! Arithmetic, comparisons, ranges, set statistics, functions and equivalence classes.

use dmath_core::run_source;
use serde_json::Value;

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
fn arithmetic_and_precedence() {
    assert_eq!(shown("DISPLAY 2 + 3 * 4"), "14");
    assert_eq!(shown("DISPLAY (2 + 3) * 4"), "20");
    assert_eq!(shown("DISPLAY 2 ^ 3 ^ 2"), "512"); // right associative
    assert_eq!(shown("DISPLAY -2 ^ 2"), "-4");
    assert_eq!(shown("DISPLAY (-2) ^ 2"), "4");
    assert_eq!(shown("DISPLAY 10 - 3 - 2"), "5");
    assert_eq!(shown("DISPLAY 5 * -3"), "-15");
    assert_eq!(shown("DISPLAY 0 ^ 0"), "1");
}

#[test]
fn floor_division_and_remainder() {
    assert_eq!(shown("DISPLAY 7 / 2"), "3");
    assert_eq!(shown("DISPLAY -7 / 2"), "-4");
    assert_eq!(shown("DISPLAY 7 % 3"), "1");
    assert_eq!(shown("DISPLAY -7 % 3"), "2");
    assert_eq!(shown("DISPLAY 7 % -3"), "-2");
    assert_eq!(shown("x = 17\nDISPLAY x / 5 * 5 + x % 5"), "17");
}

#[test]
fn arithmetic_errors() {
    assert!(error_of("DISPLAY 1 / 0").0.contains("Division by zero"));
    assert!(error_of("DISPLAY 1 % 0").0.contains("Division by zero"));
    assert!(error_of("DISPLAY 2 ^ -1").0.contains("Negative exponent"));
    assert!(error_of("DISPLAY 2 ^ 64").0.contains("overflow"));
    assert!(error_of("DISPLAY 9223372036854775807 + 1").0.contains("overflow"));
    assert!(error_of("DISPLAY {1} + 2").0.contains("Type error"));
}

#[test]
fn numeric_comparisons() {
    assert_eq!(shown("DISPLAY 3 < 5"), "TRUE");
    assert_eq!(shown("DISPLAY 5 <= 5"), "TRUE");
    assert_eq!(shown("DISPLAY 5 > 5"), "FALSE");
    assert_eq!(shown("DISPLAY 5 >= 6"), "FALSE");
    assert_eq!(shown("DISPLAY 2 + 2 == 4"), "TRUE");
    assert_eq!(shown("DISPLAY 2 + 2 != 4"), "FALSE");
    assert_eq!(shown("DISPLAY {1, 2} == {2, 1}"), "TRUE");
    assert_eq!(shown("DISPLAY {1, 2} != {1}"), "TRUE");
    assert_eq!(shown("x = 4\nDISPLAY x > 1 AND x < 10"), "TRUE");
    assert!(error_of("DISPLAY 1 == {1}").0.contains("same type"));
    assert!(error_of("DISPLAY {1} < {2}").0.contains("Number"));
}

#[test]
fn comparison_inside_truth_table() {
    let j = run("x = 3\nP = TRUE\nVISUALIZE (x < 5) AND P AS TRUTH_TABLE");
    assert!(j["diagnostics"].as_array().unwrap().iter().all(|d| d["severity"] != "error"));
    assert_eq!(j["visualizations"][0]["rows"].as_array().unwrap().len(), 2);
}

#[test]
fn ranges() {
    assert_eq!(shown("DISPLAY {1..5}"), "{1, 2, 3, 4, 5}");
    assert_eq!(shown("DISPLAY {1, 3..5, 9}"), "{1, 3, 4, 5, 9}");
    assert_eq!(shown("DISPLAY {-2..1}"), "{-2, -1, 0, 1}");
    assert_eq!(shown("DISPLAY SIZE({1..1000})"), "1000");
    assert_eq!(shown("DISPLAY SUM({1..100})"), "5050");
    assert!(error_of("DISPLAY {5..1}").0.contains("empty"));
    assert!(error_of("DISPLAY {1..20000}").0.contains("limit"));
}

#[test]
fn membership_and_proper_subset() {
    assert_eq!(shown("DISPLAY 3 IN {1..5}"), "TRUE");
    assert_eq!(shown("DISPLAY 9 IN {1..5}"), "FALSE");
    assert_eq!(shown("A = {1, 2}\nDISPLAY A IN POWERSET({1, 2, 3})"), "TRUE");
    assert_eq!(shown("DISPLAY {1} PSUBSET {1, 2}"), "TRUE");
    assert_eq!(shown("DISPLAY {1, 2} PSUBSET {1, 2}"), "FALSE");
    assert_eq!(shown("DISPLAY {1, 2} SUBSET {1, 2}"), "TRUE");
    assert!(error_of("DISPLAY TRUE IN {1}").0.contains("Type error"));
}

#[test]
fn min_max_sum_is_empty() {
    assert_eq!(shown("DISPLAY MIN({4, 9, 2})"), "2");
    assert_eq!(shown("DISPLAY MAX({4, 9, 2})"), "9");
    assert_eq!(shown("DISPLAY SUM({4, 9, 2})"), "15");
    assert_eq!(shown("DISPLAY SUM({})"), "0");
    assert_eq!(shown("DISPLAY IS_EMPTY({})"), "TRUE");
    assert_eq!(shown("DISPLAY IS_EMPTY({1})"), "FALSE");
    assert!(error_of("DISPLAY MIN({})").0.contains("empty"));
    assert!(error_of("DISPLAY MAX({(1, 2)})").0.contains("plain numbers"));
}

#[test]
fn domain_range_image() {
    let src = "R = {(1, 4), (2, 5), (3, 5)}\n";
    assert_eq!(shown(&format!("{src}DISPLAY DOMAIN(R)")), "{1, 2, 3}");
    assert_eq!(shown(&format!("{src}DISPLAY RANGE(R)")), "{4, 5}");
    assert_eq!(shown(&format!("{src}DISPLAY IMAGE(R, {{2, 3}})")), "{5}");
    assert_eq!(shown(&format!("{src}DISPLAY IMAGE(R, {{9}})")), "{}");
}

#[test]
fn function_kinds() {
    let f = "F = {(1, 4), (2, 5), (3, 6)}\n"; // bijection {1,2,3} -> {4,5,6}
    assert_eq!(shown(&format!("{f}DISPLAY IS_FUNCTION(F, {{1, 2, 3}})")), "TRUE");
    assert_eq!(shown(&format!("{f}DISPLAY IS_FUNCTION(F, {{1, 2, 3, 4}})")), "FALSE");
    assert_eq!(shown(&format!("{f}DISPLAY IS_INJECTIVE(F)")), "TRUE");
    assert_eq!(shown(&format!("{f}DISPLAY IS_SURJECTIVE(F, {{4, 5, 6}})")), "TRUE");
    assert_eq!(shown(&format!("{f}DISPLAY IS_SURJECTIVE(F, {{4, 5, 6, 7}})")), "FALSE");
    assert_eq!(shown(&format!("{f}DISPLAY IS_BIJECTIVE(F, {{4, 5, 6}})")), "TRUE");

    let g = "G = {(1, 4), (2, 4)}\n"; // not injective
    assert_eq!(shown(&format!("{g}DISPLAY IS_FUNCTION(G, {{1, 2}})")), "TRUE");
    assert_eq!(shown(&format!("{g}DISPLAY IS_INJECTIVE(G)")), "FALSE");
    assert_eq!(shown(&format!("{g}DISPLAY IS_BIJECTIVE(G, {{4}})")), "FALSE");

    let h = "H = {(1, 4), (1, 5)}\n"; // one input, two outputs
    assert_eq!(shown(&format!("{h}DISPLAY IS_FUNCTION(H, {{1}})")), "FALSE");
    assert_eq!(shown(&format!("{h}DISPLAY IS_INJECTIVE(H)")), "FALSE");
}

#[test]
fn equivalence_classes() {
    let r = "R = {(1, 1), (2, 2), (3, 3), (1, 2), (2, 1)}\n";
    assert_eq!(shown(&format!("{r}DISPLAY CLASSES(R)")), "{{3}, {1, 2}}");
    assert_eq!(shown(&format!("{r}DISPLAY CLASS_OF(R, 2)")), "{1, 2}");
    assert_eq!(shown(&format!("{r}DISPLAY SIZE(CLASSES(R))")), "2");
    assert!(error_of("R = {(1, 2)}\nDISPLAY CLASSES(R)").0.contains("equivalence"));
    assert!(error_of(&format!("{r}DISPLAY CLASS_OF(R, 9)")).0.contains("not an element"));
}

#[test]
fn pretty_printing_round_trips() {
    let j = run("x = 2\nDISPLAY (x + 1) * 3 ^ 2, -(x + 1), (-x) ^ 2, 2 ^ (x - 1)");
    let outs: Vec<String> = j["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["display"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(outs, ["27", "-3", "4", "2"]);
}
