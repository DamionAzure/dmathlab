//! Spec addendum — Relations (RL) and Graph Theory (GT) test suite.
//!
//! Runs against the public `run_source` entry point and asserts on the
//! serialized JSON envelope, so the wire contract (`dmath.viz.v1`) is
//! validated together with the mathematics.

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

fn has_error_containing(json: &Value, needle: &str) -> bool {
    errors(json).iter().any(|m| m.contains(needle))
}

fn ok(json: &Value) {
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["severity"] == "error")
            .count(),
        0,
        "expected no errors, got {:?}",
        errors(json)
    );
    assert_eq!(json["ok"], true);
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

fn output_type(json: &Value, name: &str) -> String {
    json["outputs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["name"] == name || (!name.is_empty() && o["expression"] == name))
        .map(|o| o["value_type"].as_str().unwrap().to_string())
        .unwrap_or_else(|| panic!("no output named {}", name))
}

/// Find the first visualization of the given type.
fn viz<'a>(json: &'a Value, typ: &str) -> &'a Value {
    json["visualizations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["type"] == typ)
        .unwrap_or_else(|| panic!("no visualization of type {}", typ))
}

/// Find the last visualization of the given type (later statements win).
fn viz_last<'a>(json: &'a Value, typ: &str) -> &'a Value {
    json["visualizations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|v| v["type"] == typ)
        .next_back()
        .unwrap_or_else(|| panic!("no visualization of type {}", typ))
}

const R_CHAIN: &str = "R = {(1,1), (1,2), (2,2), (2,3), (3,3), (3,4), (4,4)}\n";

// ===========================================================================
// RL — relation properties
// ===========================================================================

#[test]
fn rl_reflexive_symmetric_antisymmetric_transitive_flags() {
    let j = &run(&format!("{}\nCHECK PROPERTIES R", R_CHAIN));
    ok(j);
    let p = viz(j, "relation_properties");
    assert_eq!(p["reflexive"], true);
    assert_eq!(p["symmetric"], false);
    assert_eq!(p["antisymmetric"], true);
    assert_eq!(p["transitive"], false);
    assert_eq!(p["is_equivalence"], false);
    assert_eq!(p["is_partial_order"], false);
    assert_eq!(p["base_set"], serde_json::json!([1, 2, 3, 4]));
    assert_eq!(p["classes"], Value::Null);
    let ce = &p["counterexamples"];
    assert_eq!(
        ce["symmetric"],
        "(1, 2) holds, but (2, 1) is missing."
    );
    assert_eq!(
        ce["transitive"],
        "(1, 2) and (2, 3) hold, but (1, 3) is missing."
    );
    assert!(ce.get("reflexive").is_none());
    assert!(ce.get("antisymmetric").is_none());
}

#[test]
fn rl_each_predicate_builtin() {
    // Reflexive
    assert_eq!(
        output_display(&run("R = {(1,1), (2,2)}\nDISPLAY IS_REFLEXIVE(R)"), "IS_REFLEXIVE(R)"),
        "TRUE"
    );
    assert_eq!(
        output_display(&run("R = {(1,2)}\nDISPLAY IS_REFLEXIVE(R)"), "IS_REFLEXIVE(R)"),
        "FALSE"
    );
    // Symmetric
    assert_eq!(
        output_display(&run("R = {(1,2), (2,1)}\nDISPLAY IS_SYMMETRIC(R)"), "IS_SYMMETRIC(R)"),
        "TRUE"
    );
    // Antisymmetric (diagonal pairs never violate it)
    assert_eq!(
        output_display(&run("R = {(1,2), (2,1)}\nDISPLAY IS_ANTISYMMETRIC(R)"), "IS_ANTISYMMETRIC(R)"),
        "FALSE"
    );
    assert_eq!(
        output_display(&run("R = {(1,2), (2,3)}\nDISPLAY IS_ANTISYMMETRIC(R)"), "IS_ANTISYMMETRIC(R)"),
        "TRUE"
    );
    // Transitive
    assert_eq!(
        output_display(&run("R = {(1,2), (2,3), (1,3)}\nDISPLAY IS_TRANSITIVE(R)"), "IS_TRANSITIVE(R)"),
        "TRUE"
    );
    assert_eq!(
        output_display(&run("R = {(1,2), (2,3)}\nDISPLAY IS_TRANSITIVE(R)"), "IS_TRANSITIVE(R)"),
        "FALSE"
    );
}

#[test]
fn rl_equivalence_classes() {
    let src = "M = {(0,0), (0,3), (3,0), (3,3), (1,1), (1,4), (4,1), (4,4), (2,2), (2,5), (5,2), (5,5)}\n\
               CHECK PROPERTIES M\n\
               DISPLAY IS_EQUIVALENCE(M)";
    let j = &run(src);
    ok(j);
    let p = viz(j, "relation_properties");
    assert_eq!(p["is_equivalence"], true);
    assert_eq!(p["classes"], serde_json::json!([[0, 3], [1, 4], [2, 5]]));
    assert_eq!(output_display(j, "IS_EQUIVALENCE(M)"), "TRUE");
}

#[test]
fn rl_partial_order_detection() {
    let src = "P = {(1,1), (2,2), (3,3), (1,2), (1,3), (2,3)}\nCHECK PROPERTIES P";
    let j = &run(src);
    ok(j);
    let p = viz(j, "relation_properties");
    assert_eq!(p["is_partial_order"], true);
    assert_eq!(p["is_equivalence"], false);
    // A partial order fails 'symmetric' by design; the order properties hold.
    let ce = p["counterexamples"].as_object().unwrap();
    assert!(!ce.contains_key("reflexive"));
    assert!(!ce.contains_key("antisymmetric"));
    assert!(!ce.contains_key("transitive"));
}

#[test]
fn rl_closures() {
    // Transitive closure (Warshall)
    assert_eq!(
        output_display(
            &run("R = {(1,2), (2,3)}\nTC = TRANSITIVE_CLOSURE(R)\nDISPLAY TC"),
            "TC"
        ),
        "{(1, 2), (1, 3), (2, 3)}"
    );
    // Idempotent
    assert_eq!(
        output_display(
            &run("R = {(1,2), (2,3), (1,3)}\nTC = TRANSITIVE_CLOSURE(R)\nDISPLAY TC"),
            "TC"
        ),
        "{(1, 2), (1, 3), (2, 3)}"
    );
    // Reflexive closure adds the diagonal over the field
    assert_eq!(
        output_display(
            &run("R = {(1,2)}\nRC = REFLEXIVE_CLOSURE(R)\nDISPLAY RC"),
            "RC"
        ),
        "{(1, 1), (1, 2), (2, 2)}"
    );
    // Symmetric closure adds missing inverses
    assert_eq!(
        output_display(
            &run("RT = {(1,2), (2,2), (3,3)}\nS = SYMMETRIC_CLOSURE(RT)\nDISPLAY S"),
            "S"
        ),
        "{(1, 2), (2, 1), (2, 2), (3, 3)}"
    );
}

#[test]
fn rl_compose_and_inverse() {
    assert_eq!(
        output_display(
            &run("A = {(1,2), (2,3)}\nB = {(3,4)}\nC = COMPOSE(A, B)\nDISPLAY C"),
            "C"
        ),
        "{(2, 4)}"
    );
    assert_eq!(
        output_display(&run("A = {(1,2), (2,3)}\nI = INVERSE(A)\nDISPLAY I"), "I"),
        "{(2, 1), (3, 2)}"
    );
}

#[test]
fn rl_digraph_ir() {
    let j = &run(&format!("{}\nVISUALIZE R AS DIGRAPH", R_CHAIN));
    ok(j);
    let d = viz(j, "digraph");
    assert_eq!(d["expression"], "R");
    assert_eq!(d["vertices"], serde_json::json!([1, 2, 3, 4]));
    let edges = d["edges"].as_array().unwrap();
    assert_eq!(edges.len(), 7);
    assert!(edges.contains(&serde_json::json!([3, 3])), "self-loop included");
    assert_eq!(edges[0], serde_json::json!([1, 1]), "edges sorted");
}

#[test]
fn rl_matrix_ir() {
    let src = "R = {(1,2), (2,3), (1,3)}\nTC = TRANSITIVE_CLOSURE(R)\nRC = REFLEXIVE_CLOSURE(TC)\nVISUALIZE RC AS MATRIX";
    let j = &run(src);
    ok(j);
    let m = viz(j, "relation_matrix");
    assert_eq!(m["vertices"], serde_json::json!([1, 2, 3]));
    assert_eq!(m["matrix"], serde_json::json!([[1, 1, 1], [0, 1, 1], [0, 0, 1]]));
}

#[test]
fn rl_hasse_divisibility() {
    let src = "R = {(1,1), (1,2), (1,3), (1,6), (2,2), (2,6), (3,3), (3,6), (6,6)}\n\
               VISUALIZE R AS HASSE";
    let j = &run(src);
    ok(j);
    let h = viz(j, "hasse");
    assert_eq!(h["levels"], serde_json::json!([[1], [2, 3], [6]]));
    assert_eq!(h["edges"], serde_json::json!([[1, 2], [1, 3], [2, 6], [3, 6]]));
}

#[test]
fn rl_hasse_requires_partial_order() {
    // Not transitive
    let j = &run("R = {(1,1), (2,2), (1,2), (2,3), (3,3)}\nVISUALIZE R AS HASSE");
    assert!(has_error_containing(j, "not transitive"));
    assert!(j["diagnostics"].as_array().unwrap().iter().any(|d| {
        d["hint"].as_str().map_or(false, |h| h.contains("TRANSITIVE_CLOSURE"))
    }));
    // Not antisymmetric
    let j = &run("R = {(1,1), (2,2), (1,2), (2,1)}\nVISUALIZE R AS HASSE");
    assert!(has_error_containing(j, "not antisymmetric"));
    // Not reflexive
    let j = &run("R = {(1,2), (2,2)}\nVISUALIZE R AS HASSE");
    assert!(has_error_containing(j, "not reflexive"));
    assert!(j["diagnostics"].as_array().unwrap().iter().any(|d| {
        d["hint"].as_str().map_or(false, |h| h.contains("REFLEXIVE_CLOSURE"))
    }));
}

#[test]
fn rl_type_and_literal_errors() {
    // Predicate on a set of numbers
    let j = &run("A = {1, 2}\nX = IS_REFLEXIVE(A)\nDISPLAY X");
    assert!(has_error_containing(j, "requires a relation"));
    // Mixed literal
    let j = &run("R = {(1,2), 3}\nDISPLAY R");
    assert!(has_error_containing(j, "cannot mix"));
    // CHECK PROPERTIES on a Boolean
    let j = &run("P = TRUE\nCHECK PROPERTIES P");
    assert!(has_error_containing(j, "expects a relation or a graph"));
}

#[test]
fn rl_empty_relation_is_ok() {
    let j = &run("VISUALIZE {} AS DIGRAPH\nVISUALIZE {} AS MATRIX");
    ok(j);
    assert_eq!(viz(j, "digraph")["vertices"], serde_json::json!([]));
    assert_eq!(viz(j, "relation_matrix")["matrix"], serde_json::json!([]));
}

// ===========================================================================
// GT — graph theory
// ===========================================================================

const G6: &str = "G = GRAPH {1,2,3,4,5,6} {(1,2), (1,3), (2,4), (3,4), (4,5), (5,6)}\n";

#[test]
fn gt_degree_neighbors_adjacent() {
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY DEGREE(G, 4)", G6)), "DEGREE(G, 4)"),
        "3"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY NEIGHBORS(G, 4)", G6)), "NEIGHBORS(G, 4)"),
        "[2, 3, 5]"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY ADJACENT(G, 2, 4)", G6)), "ADJACENT(G, 2, 4)"),
        "TRUE"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY ADJACENT(G, 2, 6)", G6)), "ADJACENT(G, 2, 6)"),
        "FALSE"
    );
}

#[test]
fn gt_bfs_dfs_orders() {
    assert_eq!(
        output_display(&run(&format!("{}\nORDER = BFS(G, 1)\nDISPLAY ORDER", G6)), "ORDER"),
        "[1, 2, 3, 4, 5, 6]"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nWALK = DFS(G, 1)\nDISPLAY WALK", G6)), "WALK"),
        "[1, 2, 4, 3, 5, 6]"
    );
}

#[test]
fn gt_shortest_path_unweighted() {
    assert_eq!(
        output_display(
            &run(&format!("{}\nP = SHORTEST_PATH(G, 1, 6)\nDISPLAY P", G6)),
            "P"
        ),
        "[1, 2, 4, 5, 6]"
    );
}

#[test]
fn gt_display_also_emits_highlighted_graph_view() {
    let j = &run(&format!("{}\nORDER = BFS(G, 1)\nDISPLAY ORDER", G6));
    ok(j);
    let v = viz_last(j, "graph_view");
    assert_eq!(v["expression"], "BFS(G, 1)");
    assert_eq!(v["highlight_visited"], serde_json::json!([1, 2, 3, 4, 5, 6]));
    assert_eq!(v["note"], "BFS from 1 visits: 1 → 2 → 3 → 4 → 5 → 6");

    let j = &run(&format!("{}\nP = SHORTEST_PATH(G, 1, 6)\nDISPLAY P", G6));
    let v = viz_last(j, "graph_view");
    assert_eq!(v["highlight_path"], serde_json::json!([1, 2, 4, 5, 6]));
    assert_eq!(
        v["note"],
        "Shortest path from 1 to 6: 1 → 2 → 4 → 5 → 6 (4 edges)"
    );
}

#[test]
fn gt_weighted_dijkstra() {
    let w = "W = GRAPH {1,2,3,4,5} {(1,2,7), (1,3,3), (3,2,2), (2,4,2), (3,4,8), (4,5,3)}\n";
    let j = &run(&format!("{}P = SHORTEST_PATH(W, 1, 4)\nDISPLAY P", w));
    ok(j);
    assert_eq!(output_display(j, "P"), "[1, 3, 2, 4]");
    let v = viz_last(j, "graph_view");
    assert_eq!(
        v["note"],
        "Shortest path from 1 to 4: 1 → 3 → 2 → 4 (3 edges, weight 7)"
    );
}

#[test]
fn gt_mst_kruskal() {
    let w = "W = GRAPH {1,2,3,4,5} {(1,2,7), (1,3,3), (3,2,2), (2,4,2), (3,4,8), (4,5,3)}\n";
    let j = &run(&format!("{}T = MST(W)\nDISPLAY T\nVISUALIZE T AS GRAPH", w));
    ok(j);
    let disp = output_display(j, "T");
    assert!(disp.contains("total weight = 10"), "display: {}", disp);
    assert_eq!(output_type(j, "T"), "Graph");
    let v = viz_last(j, "graph_view");
    assert_eq!(v["total_weight"], 10);
    assert_eq!(
        v["note"],
        "Minimum spanning tree: 4 edges, total weight 10."
    );
    for e in v["edges"].as_array().unwrap() {
        assert_eq!(e["highlight"], "tree");
    }
}

#[test]
fn gt_mst_requires_connected() {
    let j = &run("W = GRAPH {1,2,3} {(1,2)}\nT = MST(W)\nDISPLAY T");
    assert!(has_error_containing(j, "not connected"));
}

#[test]
fn gt_coloring() {
    let j = &run(
        "K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}\nC = COLOR(K)\nDISPLAY C\nVISUALIZE C AS GRAPH",
    );
    ok(j);
    assert_eq!(output_display(j, "C"), "{1 -> 0, 2 -> 1, 3 -> 2}");
    assert_eq!(output_type(j, "C"), "Mapping");
    let v = viz_last(j, "graph_view");
    assert_eq!(v["colors"], serde_json::json!({"1": 0, "2": 1, "3": 2}));
    assert_eq!(v["note"], "Greedy coloring uses 3 colors.");

    // A path needs only 2 colors.
    let j = &run("P4 = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4)}\nD = COLOR(P4)\nDISPLAY D");
    ok(j);
    assert_eq!(output_display(j, "D"), "{1 -> 0, 2 -> 1, 3 -> 0, 4 -> 1}");
}

#[test]
fn gt_structure_checks() {
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY IS_CONNECTED(G)", G6)), "IS_CONNECTED(G)"),
        "TRUE"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY IS_TREE(G)", G6)), "IS_TREE(G)"),
        "FALSE"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY IS_BIPARTITE(G)", G6)), "IS_BIPARTITE(G)"),
        "TRUE"
    );
    // A path is a tree.
    assert_eq!(
        output_display(
            &run("P4 = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4)}\nDISPLAY IS_TREE(P4)"),
            "IS_TREE(P4)"
        ),
        "TRUE"
    );
    // Triangle has an odd cycle.
    assert_eq!(
        output_display(
            &run("K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}\nDISPLAY IS_BIPARTITE(K)"),
            "IS_BIPARTITE(K)"
        ),
        "FALSE"
    );
    assert_eq!(
        output_display(
            &run("K = GRAPH {1,2,3} {(1,2), (2,3), (1,3)}\nDISPLAY IS_COMPLETE(K)"),
            "IS_COMPLETE(K)"
        ),
        "TRUE"
    );
    // C4 is bipartite; K4 minus an edge is not complete.
    assert_eq!(
        output_display(
            &run("C4 = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4), (4,1)}\nDISPLAY IS_BIPARTITE(C4)"),
            "IS_BIPARTITE(C4)"
        ),
        "TRUE"
    );
    assert_eq!(
        output_display(
            &run("H = GRAPH {1,2,3,4} {(1,2), (2,3), (3,4), (1,4), (1,3)}\nDISPLAY IS_COMPLETE(H)"),
            "IS_COMPLETE(H)"
        ),
        "FALSE"
    );
}

#[test]
fn gt_graph_properties_ir() {
    let j = &run(&format!("{}\nCHECK PROPERTIES G", G6));
    ok(j);
    let p = viz(j, "graph_properties");
    assert_eq!(p["vertex_count"], 6);
    assert_eq!(p["edge_count"], 6);
    assert_eq!(p["connected"], true);
    assert_eq!(p["tree"], false);
    assert_eq!(p["bipartite"], true);
    assert_eq!(p["complete"], false);
    assert_eq!(p["degrees"]["4"], 3);
    assert_eq!(p["degrees"]["6"], 1);
    assert!(p["notes"].as_array().unwrap().is_empty());
}

#[test]
fn gt_vertices_edges_and_dedupe() {
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY VERTICES(G)", G6)), "VERTICES(G)"),
        "{1, 2, 3, 4, 5, 6}"
    );
    assert_eq!(
        output_display(&run(&format!("{}\nDISPLAY EDGES(G)", G6)), "EDGES(G)"),
        "{(1, 2), (1, 3), (2, 4), (3, 4), (4, 5), (5, 6)}"
    );
    // Undirected edges are normalized and deduplicated.
    assert_eq!(
        output_display(
            &run("H = GRAPH {1,2} {(1,2), (2,1), (1,2)}\nDISPLAY EDGES(H)"),
            "EDGES(H)"
        ),
        "{(1, 2)}"
    );
    // Directed edges keep both directions.
    assert_eq!(
        output_display(
            &run("D = DIGRAPH {1,2} {(1,2), (2,1)}\nDISPLAY EDGES(D)"),
            "EDGES(D)"
        ),
        "{(1, 2), (2, 1)}"
    );
    assert_eq!(
        output_display(
            &run("D = DIGRAPH {1,2} {(1,2), (2,1)}\nDISPLAY ADJACENT(D, 2, 1)"),
            "ADJACENT(D, 2, 1)"
        ),
        "TRUE"
    );
}

#[test]
fn gt_graph_view_ir() {
    let j = &run(&format!("{}\nVISUALIZE G AS GRAPH", G6));
    ok(j);
    let v = viz(j, "graph_view");
    assert_eq!(v["directed"], false);
    assert_eq!(v["vertices"], serde_json::json!([1, 2, 3, 4, 5, 6]));
    let e = v["edges"].as_array().unwrap();
    assert_eq!(e.len(), 6);
    assert_eq!(e[0], serde_json::json!({"from": 1, "to": 2}));
}

#[test]
fn gt_vertex_and_weight_errors() {
    let j = &run(&format!("{}\nDISPLAY DEGREE(G, 9)", G6));
    assert!(has_error_containing(j, "Vertex 9 is not in the graph."));
    let j = &run("H = GRAPH {1,2} {(1,5)}\nDISPLAY H");
    assert!(has_error_containing(j, "references vertex 5"));
    let j = &run("H = GRAPH {1,2} {(1,2,-3)}\nDISPLAY H");
    assert!(has_error_containing(j, "non-negative"));
    // Runtime guard too: SHORTEST_PATH on a negative-weight graph.
    let j = &run("D = DIGRAPH {1,2} {(1,2)}\nH = DIGRAPH {1,2,3} {(1,2,5), (2,3,-1)}\nDISPLAY SHORTEST_PATH(H, 1, 3)");
    assert!(has_error_containing(j, "non-negative"));
}

#[test]
fn gt_unreachable_target_is_empty_not_error() {
    let j = &run("G = GRAPH {1,2,3} {(1,2)}\nP = SHORTEST_PATH(G, 1, 3)\nDISPLAY P");
    ok(j);
    assert_eq!(output_display(j, "P"), "[]");
    let v = viz_last(j, "graph_view");
    assert_eq!(v["note"], "Vertex 3 is not reachable from 1.");
}

#[test]
fn gt_builtin_type_and_arity_errors() {
    let j = &run("R = {(1,2)}\nDISPLAY DEGREE(R, 1)");
    assert!(has_error_containing(j, "'DEGREE' requires a Graph"));
    let j = &run(&format!("{}\nDISPLAY BFS(G)", G6));
    assert!(has_error_containing(j, "expects 2 arguments"));
}

#[test]
fn gt_display_types() {
    let j = &run(&format!(
        "{}\nORDER = BFS(G, 1)\nDISPLAY ORDER\nT = G\nDISPLAY T\nC = COLOR(G)\nDISPLAY C",
        G6
    ));
    ok(j);
    assert_eq!(output_type(j, "ORDER"), "List");
    assert_eq!(output_type(j, "T"), "Graph");
    assert_eq!(output_type(j, "C"), "Mapping");
    let disp = output_display(j, "T");
    assert!(disp.starts_with("Graph(V = "), "display: {}", disp);
}

// ===========================================================================
// Cross-domain integration
// ===========================================================================

#[test]
fn xc_check_properties_trace_kind() {
    let j = &run(&format!("{}\nCHECK PROPERTIES G", G6));
    let kinds: Vec<&str> = j["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["kind"].as_str().unwrap())
        .collect();
    assert!(kinds.contains(&"check_properties"), "kinds: {:?}", kinds);
}

#[test]
fn xc_booleans_compose_with_logic() {
    let j = &run(&format!(
        "{}\nOK = IS_CONNECTED(G) AND IS_BIPARTITE(G)\nDISPLAY OK",
        G6
    ));
    ok(j);
    assert_eq!(output_display(j, "OK"), "TRUE");
}

#[test]
fn xc_full_example_program_runs() {
    // The four shipped example programs must all run cleanly.
    let dir = env!("CARGO_MANIFEST_DIR");
    for f in [
        "relations-basics.dm",
        "relations-equivalence.dm",
        "graphs-basics.dm",
        "graphs-weighted.dm",
    ] {
        let path = format!("{}/../examples/{}", dir, f);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", path, e));
        let j = run(&src);
        ok(&j);
    }
}

#[test]
fn xc_determinism_run_twice() {
    let src = format!(
        "{}R = {{(1,1), (1,2), (2,2)}}\nCHECK PROPERTIES R\nTC = TRANSITIVE_CLOSURE(R)\nDISPLAY TC\nORDER = BFS(G, 1)\nDISPLAY ORDER",
        G6
    );
    let a = run(&src);
    let b = run(&src);
    assert_eq!(a, b, "the envelope must be byte-identical across runs");
}

#[test]
fn xc_regression_logic_and_sets_still_work() {
    let j = &run("A = {1, 2, 3}\nB = {2, 3, 4}\nC = A INTERSECT B\nP = TRUE\nQ = FALSE\nR = P AND (NOT Q)\nDISPLAY C\nDISPLAY R");
    ok(j);
    assert_eq!(output_display(j, "C"), "{2, 3}");
    assert_eq!(output_display(j, "R"), "TRUE");
}
