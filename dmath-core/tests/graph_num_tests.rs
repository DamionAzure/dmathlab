//! Graph structure tools, bases, partitions and other counting functions.

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

const PATH4: &str = "P = GRAPH {1,2,3,4} {(1,2),(2,3),(3,4)}\n";
const C4: &str = "C = GRAPH {1,2,3,4} {(1,2),(2,3),(3,4),(4,1)}\n";
const K4: &str = "K = GRAPH {1,2,3,4} {(1,2),(1,3),(1,4),(2,3),(2,4),(3,4)}\n";

#[test]
fn complement() {
    assert_eq!(shown(&format!("{PATH4}DISPLAY EDGES(GRAPH_COMPLEMENT(P))")), "{(1, 3), (1, 4), (2, 4)}");
    assert_eq!(shown(&format!("{K4}DISPLAY SIZE(EDGES(GRAPH_COMPLEMENT(K)))")), "0");
    assert_eq!(shown("D = DIGRAPH {1,2} {(1,2)}\nDISPLAY EDGES(GRAPH_COMPLEMENT(D))"), "{(2, 1)}");
}

#[test]
fn diameter() {
    assert_eq!(shown(&format!("{PATH4}DISPLAY DIAMETER(P)")), "3");
    assert_eq!(shown(&format!("{C4}DISPLAY DIAMETER(C)")), "2");
    assert_eq!(shown(&format!("{K4}DISPLAY DIAMETER(K)")), "1");
    assert_eq!(shown("G = GRAPH {7} {}\nDISPLAY DIAMETER(G)"), "0");
    assert!(error_of("G = GRAPH {1,2,3} {(1,2)}\nDISPLAY DIAMETER(G)").0.contains("undefined"));
}

#[test]
fn bridges_and_cut_vertices() {
    assert_eq!(shown(&format!("{PATH4}DISPLAY BRIDGES(P)")), "{(1, 2), (2, 3), (3, 4)}");
    assert_eq!(shown(&format!("{PATH4}DISPLAY CUT_VERTICES(P)")), "{2, 3}");
    assert_eq!(shown(&format!("{C4}DISPLAY BRIDGES(C)")), "{}");
    assert_eq!(shown(&format!("{C4}DISPLAY CUT_VERTICES(C)")), "{}");
    // Two triangles joined by one edge, plus an isolated vertex.
    let g = "G = GRAPH {1,2,3,4,5,6,9} {(1,2),(2,3),(1,3),(3,4),(4,5),(5,6),(4,6)}\n";
    assert_eq!(shown(&format!("{g}DISPLAY BRIDGES(G)")), "{(3, 4)}");
    assert_eq!(shown(&format!("{g}DISPLAY CUT_VERTICES(G)")), "{3, 4}");
    assert!(error_of("D = DIGRAPH {1,2} {(1,2)}\nDISPLAY BRIDGES(D)").0.contains("undirected"));
}

#[test]
fn hamiltonian() {
    assert_eq!(shown(&format!("{C4}DISPLAY IS_HAMILTONIAN(C)")), "TRUE");
    assert_eq!(shown(&format!("{K4}DISPLAY IS_HAMILTONIAN(K)")), "TRUE");
    assert_eq!(shown(&format!("{PATH4}DISPLAY IS_HAMILTONIAN(P)")), "FALSE");
    // Star graph: no Hamiltonian cycle.
    assert_eq!(shown("S = GRAPH {1,2,3,4} {(1,2),(1,3),(1,4)}\nDISPLAY IS_HAMILTONIAN(S)"), "FALSE");
    assert_eq!(shown("D = DIGRAPH {1,2,3} {(1,2),(2,3),(3,1)}\nDISPLAY IS_HAMILTONIAN(D)"), "TRUE");
    assert_eq!(shown("D = DIGRAPH {1,2,3} {(1,2),(2,3)}\nDISPLAY IS_HAMILTONIAN(D)"), "FALSE");
    // Petersen graph is famously not Hamiltonian.
    let pet = "G = GRAPH {1,2,3,4,5,6,7,8,9,10} {(1,2),(2,3),(3,4),(4,5),(5,1),(1,6),(2,7),(3,8),(4,9),(5,10),(6,8),(8,10),(10,7),(7,9),(9,6)}\n";
    assert_eq!(shown(&format!("{pet}DISPLAY IS_HAMILTONIAN(G)")), "FALSE");
    let big = (1..=21).map(|i| i.to_string()).collect::<Vec<_>>().join(",");
    assert!(error_of(&format!("G = GRAPH {{{big}}} {{}}\nDISPLAY IS_HAMILTONIAN(G)")).0.contains("at most 20"));
}

#[test]
fn chromatic() {
    assert_eq!(shown(&format!("{PATH4}DISPLAY CHROMATIC_NUMBER(P)")), "2");
    assert_eq!(shown(&format!("{C4}DISPLAY CHROMATIC_NUMBER(C)")), "2");
    assert_eq!(shown(&format!("{K4}DISPLAY CHROMATIC_NUMBER(K)")), "4");
    assert_eq!(shown("O = GRAPH {1,2,3,4,5} {(1,2),(2,3),(3,4),(4,5),(5,1)}\nDISPLAY CHROMATIC_NUMBER(O)"), "3");
    assert_eq!(shown("E = GRAPH {1,2,3} {}\nDISPLAY CHROMATIC_NUMBER(E)"), "1");
    assert_eq!(shown("E = GRAPH {} {}\nDISPLAY CHROMATIC_NUMBER(E)"), "0");
    let pet = "G = GRAPH {1,2,3,4,5,6,7,8,9,10} {(1,2),(2,3),(3,4),(4,5),(5,1),(1,6),(2,7),(3,8),(4,9),(5,10),(6,8),(8,10),(10,7),(7,9),(9,6)}\n";
    assert_eq!(shown(&format!("{pet}DISPLAY CHROMATIC_NUMBER(G)")), "3");
}

#[test]
fn adjacency_matrix_view() {
    let j = run(&format!("{PATH4}VISUALIZE P AS MATRIX"));
    assert!(j["diagnostics"].as_array().unwrap().iter().all(|d| d["severity"] != "error"));
    let viz = &j["visualizations"][0];
    assert_eq!(viz["type"], "relation_matrix");
    assert_eq!(viz["matrix"], serde_json::json!([[0,1,0,0],[1,0,1,0],[0,1,0,1],[0,0,1,0]]));
}

#[test]
fn sigma_order_bases() {
    assert_eq!(shown("DISPLAY SIGMA(12)"), "28");
    assert_eq!(shown("DISPLAY SIGMA(1)"), "1");
    assert_eq!(shown("DISPLAY SIGMA(28)"), "56"); // perfect number
    assert_eq!(shown("DISPLAY ORDER_MOD(2, 7)"), "3");
    assert_eq!(shown("DISPLAY ORDER_MOD(3, 7)"), "6");
    assert_eq!(shown("DISPLAY ORDER_MOD(10, 1000000007)"), "1000000006");
    assert!(error_of("DISPLAY ORDER_MOD(2, 4)").0.contains("undefined"));
    assert_eq!(shown("DISPLAY TO_BASE(10, 2)"), "[1, 0, 1, 0]");
    assert_eq!(shown("DISPLAY TO_BASE(255, 16)"), "[15, 15]");
    assert_eq!(shown("DISPLAY TO_BASE(0, 2)"), "[0]");
    assert_eq!(shown("DISPLAY FROM_BASE(1010, 2)"), "10");
    assert_eq!(shown("DISPLAY FROM_BASE(TO_BASE(255, 16), 16)"), "255");
    assert_eq!(shown("DISPLAY FROM_BASE(TO_BASE(123456789, 36), 36)"), "123456789");
    assert!(error_of("DISPLAY FROM_BASE(102, 2)").0.contains("not valid"));
    assert!(error_of("DISPLAY TO_BASE(5, 1)").0.contains("base"));
}

#[test]
fn partitions_and_bell() {
    let p: Vec<&str> = vec!["1", "1", "2", "3", "5", "7", "11", "15", "22", "30", "42"];
    for (n, want) in p.iter().enumerate() {
        assert_eq!(shown(&format!("DISPLAY PARTITIONS({n})")), *want);
    }
    assert_eq!(shown("DISPLAY PARTITIONS(100)"), "190569292");
    let b = ["1", "1", "2", "5", "15", "52", "203", "877", "4140"];
    for (n, want) in b.iter().enumerate() {
        assert_eq!(shown(&format!("DISPLAY BELL({n})")), *want);
    }
    assert_eq!(shown("DISPLAY BELL(25)"), "4638590332229999353");
    assert!(error_of("DISPLAY BELL(26)").0.contains("64-bit"));
    assert!(error_of("DISPLAY PARTITIONS(406)").0.contains("64-bit"));
    // Bell(n) = sum of Stirling numbers of the second kind.
    assert_eq!(shown("DISPLAY BELL(5) == STIRLING2(5,1) + STIRLING2(5,2) + STIRLING2(5,3) + STIRLING2(5,4) + STIRLING2(5,5)"), "TRUE");
}
