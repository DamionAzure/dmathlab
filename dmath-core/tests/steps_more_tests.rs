//! Worked solutions (AS STEPS) for set operators, POWERSET, inclusion-exclusion, TOPO_SORT and closures.

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

fn steps(src: &str) -> Value {
    let j = run(src);
    let errs: Vec<&Value> =
        j["diagnostics"].as_array().unwrap().iter().filter(|d| d["severity"] == "error").collect();
    assert!(errs.is_empty(), "unexpected errors for {src:?}: {errs:?}");
    let v = j["visualizations"][0].clone();
    assert_eq!(v["type"], "steps", "{src:?}");
    assert_eq!(v["verified"], true, "sections disagree for {src:?}: {v}");
    v
}

fn methods(v: &Value) -> usize {
    v["sections"].as_array().unwrap().iter().filter(|s| s["agrees"] == true).count()
}

#[test]
fn set_operator_steps() {
    for (op, want) in [
        ("UNION", "{1, 2, 3, 4, 5}"),
        ("INTERSECT", "{3, 4}"),
        ("DIFFERENCE", "{1, 2}"),
        ("SYMDIFF", "{1, 2, 5}"),
    ] {
        let v = steps(&format!("A = {{1, 2, 3, 4}}\nB = {{3, 4, 5}}\nC = A {op} B\nVISUALIZE C AS STEPS"));
        assert_eq!(v["result"], want, "{op}");
        assert!(methods(&v) >= 2);
    }
    // Direct expression, no variable.
    let v = steps("A = {1, 2}\nB = {2, 3}\nVISUALIZE A SYMDIFF B AS STEPS");
    assert_eq!(v["result"], "{1, 3}");
    // Empty operands still work.
    steps("A = {}\nB = {1}\nC = A UNION B\nVISUALIZE C AS STEPS");
}

#[test]
fn powerset_steps() {
    let v = steps("P = POWERSET({1, 2, 3})\nVISUALIZE P AS STEPS");
    assert_eq!(v["result"], "{{}, {1}, {2}, {3}, {1, 2}, {1, 3}, {2, 3}, {1, 2, 3}}");
    assert_eq!(methods(&v), 2);
    steps("VISUALIZE POWERSET({}) AS STEPS");
}

#[test]
fn inclusion_exclusion_steps() {
    let v = steps("A = {1,2,3,4}\nB = {3,4,5}\nC = {4,5,6}\nN = SIZE(A UNION B UNION C)\nVISUALIZE N AS STEPS");
    assert_eq!(v["result"], "6");
    assert_eq!(methods(&v), 2);
    let two = steps("A = {1,2}\nB = {2,3}\nVISUALIZE SIZE(A UNION B) AS STEPS");
    assert_eq!(two["result"], "3");
    let (msg, _) = error_of("A={1}\nB={2}\nC={3}\nD={4}\nE={5}\nF={6}\nVISUALIZE SIZE(A UNION B UNION C UNION D UNION E UNION F) AS STEPS");
    assert!(msg.contains("2 to 5"), "{msg}");
}

#[test]
fn topo_sort_steps() {
    let v = steps("G = DIGRAPH {1,2,3,4,5} {(1,2),(1,3),(2,4),(3,4),(4,5)}\nT = TOPO_SORT(G)\nVISUALIZE T AS STEPS");
    assert_eq!(v["result"], "[1, 2, 3, 4, 5]");
    assert_eq!(methods(&v), 2);
    // Several valid orders: smallest available first.
    let w = steps("G = DIGRAPH {3,1,2} {(3,1)}\nVISUALIZE TOPO_SORT(G) AS STEPS");
    assert_eq!(w["result"], "[2, 3, 1]");
    // A cycle is a clean error.
    assert!(error_of("G = DIGRAPH {1,2} {(1,2),(2,1)}\nT = TOPO_SORT(G)\nVISUALIZE T AS STEPS").0.contains("cycle"));
}

#[test]
fn closure_steps() {
    let r = "R = {(1,2),(2,3),(3,4)}\n";
    let t = steps(&format!("{r}C = TRANSITIVE_CLOSURE(R)\nVISUALIZE C AS STEPS"));
    assert_eq!(t["result"], "{(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)}");
    assert_eq!(methods(&t), 2);
    let f = steps(&format!("{r}C = REFLEXIVE_CLOSURE(R)\nVISUALIZE C AS STEPS"));
    assert_eq!(f["result"], "{(1, 1), (1, 2), (2, 2), (2, 3), (3, 3), (3, 4), (4, 4)}");
    let s = steps(&format!("{r}VISUALIZE SYMMETRIC_CLOSURE(R) AS STEPS"));
    assert_eq!(s["result"], "{(1, 2), (2, 1), (2, 3), (3, 2), (3, 4), (4, 3)}");
    // A cycle makes every pair reachable, including self-pairs.
    let c = steps("R = {(1,2),(2,3),(3,1)}\nVISUALIZE TRANSITIVE_CLOSURE(R) AS STEPS");
    assert_eq!(c["result"].as_str().unwrap().matches('(').count(), 9);
}

#[test]
fn steps_follow_the_value_at_assignment_time() {
    // Reassigning A afterwards must not change what C's worked solution shows.
    let v = steps("A = {1, 2}\nB = {2, 3}\nC = A UNION B\nA = {9}\nVISUALIZE C AS STEPS");
    assert_eq!(v["result"], "{1, 2, 3}");
}

#[test]
fn unsupported_steps_target_gets_a_helpful_error() {
    let (msg, _) = error_of("X = 5 + 3\nVISUALIZE X AS STEPS");
    assert!(msg.contains("'STEPS' needs"), "{msg}");
}
