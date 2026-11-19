//! Modular arithmetic, extra counting functions and extra graph tools.

use dmath_core::run_source;
use serde_json::Value;

fn run(src: &str) -> Value {
    serde_json::to_value(run_source(src)).unwrap()
}

/// The `display` text of the first output.
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

fn error_of(src: &str) -> String {
    let j = run(src);
    j["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["severity"] == "error")
        .unwrap_or_else(|| panic!("expected an error for {src:?}"))["message"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn mod_is_never_negative() {
    assert_eq!(shown("DISPLAY MOD(17, 5)"), "2");
    assert_eq!(shown("DISPLAY MOD(-7, 5)"), "3");
    assert!(error_of("DISPLAY MOD(4, 0)").contains("positive modulus"));
}

#[test]
fn modpow_matches_known_values() {
    assert_eq!(shown("DISPLAY MODPOW(2, 10, 1000)"), "24");
    assert_eq!(shown("DISPLAY MODPOW(3, 0, 7)"), "1");
    assert_eq!(shown("DISPLAY MODPOW(7, 222, 1000000007)"), "286423514");
    assert_eq!(shown("DISPLAY MODPOW(5, 3, 1)"), "0");
    assert!(error_of("DISPLAY MODPOW(2, -1, 7)").contains("non-negative exponent"));
}

#[test]
fn modinv_and_its_failure() {
    assert_eq!(shown("DISPLAY MODINV(3, 7)"), "5");
    assert_eq!(shown("DISPLAY MODINV(-3, 7)"), "2");
    assert!(error_of("DISPLAY MODINV(4, 8)").contains("no inverse"));
}

#[test]
fn egcd_satisfies_bezout() {
    assert_eq!(shown("DISPLAY EGCD(240, 46)"), "[2, -9, 47]");
    assert_eq!(shown("DISPLAY EGCD(0, 5)"), "[5, 0, 1]");
}

#[test]
fn crt_coprime_and_shared_factor() {
    assert_eq!(shown("DISPLAY CRT(2, 3, 3, 5)"), "[8, 15]");
    assert_eq!(shown("DISPLAY CRT(1, 4, 3, 6)"), "[9, 12]");
    assert!(error_of("DISPLAY CRT(1, 4, 2, 6)").contains("contradict"));
}

#[test]
fn counting_sequences() {
    assert_eq!(shown("DISPLAY FIB(10)"), "55");
    assert_eq!(shown("DISPLAY FIB(92)"), "7540113804746346429");
    assert!(error_of("DISPLAY FIB(93)").contains("FIB requires"));
    assert_eq!(shown("DISPLAY CATALAN(5)"), "42");
    assert_eq!(shown("DISPLAY CATALAN(0)"), "1");
    assert_eq!(shown("DISPLAY CATALAN(35)"), "3116285494907301262");
    assert_eq!(shown("DISPLAY DERANGE(4)"), "9");
    assert_eq!(shown("DISPLAY DERANGE(0)"), "1");
    assert_eq!(shown("DISPLAY DERANGE(20)"), "895014631192902121");
    assert_eq!(shown("DISPLAY STIRLING2(5, 2)"), "15");
    assert_eq!(shown("DISPLAY STIRLING2(4, 4)"), "1");
    assert_eq!(shown("DISPLAY STIRLING2(0, 0)"), "1");
    assert_eq!(shown("DISPLAY STIRLING2(3, 0)"), "0");
    assert!(error_of("DISPLAY STIRLING2(3, 5)").contains("k ≤ n"));
}

#[test]
fn topological_order_and_cycles() {
    let dag = "D = DIGRAPH {1,2,3,4,5} {(5,2), (5,1), (2,3), (3,4), (1,4)}\n";
    assert_eq!(shown(&format!("{dag}DISPLAY TOPO_SORT(D)")), "[5, 1, 2, 3, 4]");
    assert_eq!(shown(&format!("{dag}DISPLAY IS_DAG(D)")), "TRUE");
    let cyc = "C = DIGRAPH {1,2,3} {(1,2), (2,3), (3,1)}\n";
    assert_eq!(shown(&format!("{cyc}DISPLAY IS_DAG(C)")), "FALSE");
    assert!(error_of(&format!("{cyc}DISPLAY TOPO_SORT(C)")).contains("cycle"));
    assert!(error_of("G = GRAPH {1,2} {(1,2)}\nDISPLAY TOPO_SORT(G)").contains("directed"));
}

#[test]
fn components_and_euler() {
    let g = "G = GRAPH {1,2,3,4,5,6} {(1,2), (2,3), (4,5)}\n";
    assert_eq!(shown(&format!("{g}DISPLAY COMPONENTS(G)")), "{1 -> 1, 2 -> 1, 3 -> 1, 4 -> 2, 5 -> 2, 6 -> 3}");
    let tri = "T = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}\n";
    assert_eq!(shown(&format!("{tri}DISPLAY IS_EULERIAN(T)")), "TRUE");
    let path = "P = GRAPH {1,2,3} {(1,2), (2,3)}\n";
    assert_eq!(shown(&format!("{path}DISPLAY IS_EULERIAN(P)")), "FALSE");
    let two = "X = GRAPH {1,2,3,4,5,6} {(1,2), (2,3), (1,3), (4,5), (5,6), (4,6)}\n";
    assert_eq!(shown(&format!("{two}DISPLAY IS_EULERIAN(X)")), "FALSE");
    let cyc = "C = DIGRAPH {1,2,3} {(1,2), (2,3), (3,1)}\n";
    assert_eq!(shown(&format!("{cyc}DISPLAY IS_EULERIAN(C)")), "TRUE");
    assert_eq!(shown(&format!("{tri}DISPLAY DEGREE_SEQUENCE(T)")), "[2, 2, 2]");
}

fn steps_verified(src: &str) -> bool {
    let j = run(src);
    let v = &j["visualizations"][0];
    assert_eq!(v["type"], "steps", "no steps view for {src:?}: {:?}", j["diagnostics"]);
    v["verified"].as_bool().unwrap()
}

#[test]
fn worked_solutions_agree_with_the_engine() {
    for src in [
        "A = EGCD(240, 46)\nVISUALIZE A AS STEPS",
        "A = EGCD(-12, 18)\nVISUALIZE A AS STEPS",
        "A = EGCD(0, 5)\nVISUALIZE A AS STEPS",
        "A = MODINV(3, 7)\nVISUALIZE A AS STEPS",
        "A = MODINV(-3, 10)\nVISUALIZE A AS STEPS",
        "A = MODINV(17, 3120)\nVISUALIZE A AS STEPS",
        "A = MODPOW(7, 222, 1000000007)\nVISUALIZE A AS STEPS",
        "A = MODPOW(-3, 5, 7)\nVISUALIZE A AS STEPS",
        "A = MODPOW(2, 0, 9)\nVISUALIZE A AS STEPS",
        "A = MODPOW(5, 3, 1)\nVISUALIZE A AS STEPS",
    ] {
        assert!(steps_verified(src), "methods disagree for {src:?}");
    }
}
