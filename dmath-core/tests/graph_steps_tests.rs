//! Graph-algorithm worked solutions must agree with the engine on thousands
//! of random graphs (directed/undirected, weighted/unweighted, loops,
//! duplicate edges, disconnected pieces).

use std::collections::BTreeSet;

use dmath_core::explain_graph::{explain_graph, extra_args};
use dmath_core::run_source;
use dmath_core::token::Builtin;
use dmath_core::value::{Edge, GraphVal};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn random_graph(r: &mut Rng) -> GraphVal {
    let n = 1 + r.below(8) as i64;
    let directed = r.below(2) == 0;
    let weighted = r.below(2) == 0;
    let m = r.below(14);
    let verts: BTreeSet<i64> = (1..=n).collect();
    let mut edges = Vec::new();
    for _ in 0..m {
        let a = 1 + r.below(n as u64) as i64;
        let b = 1 + r.below(n as u64) as i64;
        // mostly avoid loops so MST/bipartite true-cases occur often
        if a == b && r.below(6) != 0 {
            continue;
        }
        let w = if weighted { Some(r.below(9) as i64) } else { None };
        edges.push(Edge { from: a, to: b, weight: w });
    }
    GraphVal::build(directed, verts, edges)
}

#[test]
fn every_graph_solution_agrees_with_the_engine() {
    let mut r = Rng(0x9E3779B97F4A7C15);
    let mut seen = std::collections::HashMap::<&str, usize>::new();
    for _ in 0..4000 {
        let g = random_graph(&mut r);
        let vs: Vec<i64> = g.vertices.iter().copied().collect();
        for func in [Builtin::Bfs, Builtin::Dfs, Builtin::ShortestPath, Builtin::Mst, Builtin::Color, Builtin::IsBipartite, Builtin::IsConnected] {
            let combos: Vec<Vec<i64>> = match extra_args(func) {
                0 => vec![vec![]],
                1 => vs.iter().map(|&a| vec![a]).collect(),
                _ => vs.iter().flat_map(|&a| vs.iter().map(move |&b| vec![a, b])).collect(),
            };
            for args in combos {
                match explain_graph(func, &g, &args, "X".into()) {
                    Ok(ir) => {
                        assert!(ir.verified, "{} {:?} on {:?}: disagreement {:#?}", func.name(), args, g, ir.sections.iter().map(|s| (&s.title, s.agrees, &s.outcome)).collect::<Vec<_>>());
                        *seen.entry(func.name()).or_default() += 1;
                        if func == Builtin::IsBipartite && ir.result == "FALSE" {
                            // an odd-cycle explanation must follow
                            let oc = ir.sections.iter().find(|s| s.title.starts_with("Why it fails")).expect("odd cycle section");
                            assert!(oc.steps.iter().any(|s| s.tone == "good"), "odd cycle not odd: {:?} on {:?}", oc.steps.iter().map(|s| &s.text).collect::<Vec<_>>(), g);
                        }
                    }
                    Err(e) => {
                        assert!(func == Builtin::Mst, "{} {:?} failed unexpectedly: {e}", func.name(), args);
                        assert!(e.contains("not connected") || e.contains("non-negative"), "{e}");
                    }
                }
            }
        }
    }
    for f in ["BFS", "DFS", "SHORTEST_PATH", "MST", "COLOR", "IS_BIPARTITE", "IS_CONNECTED"] {
        assert!(seen.get(f).copied().unwrap_or(0) > 200, "{f} was barely exercised: {seen:?}");
    }
}

#[test]
fn errors_and_language_surface() {
    let g = GraphVal::build(false, (1..=3).collect(), vec![Edge { from: 1, to: 2, weight: None }]);
    assert!(explain_graph(Builtin::Bfs, &g, &[9], "BFS(G, 9)".into()).unwrap_err().contains("not in the graph"));
    // disconnected MST
    let e = explain_graph(Builtin::Mst, &g, &[], "MST(G)".into()).unwrap_err();
    assert!(e.contains("not connected"));

    let out = run_source("G = GRAPH {1,2,3} {(1,2), (2,3)}\nP = SHORTEST_PATH(G, 1, 3)\nVISUALIZE P AS STEPS\nQ = P\nVISUALIZE Q AS STEPS\nVISUALIZE MST(G) AS STEPS\n");
    assert!(out.ok, "{:?}", out.diagnostics);
    assert_eq!(out.visualizations.len(), 3);
    // reassigning the graph must not change an old solution's snapshot
    let out = run_source("G = GRAPH {1,2} {(1,2)}\nP = BFS(G, 1)\nG = GRAPH {1,2,3} {(1,3)}\nVISUALIZE P AS STEPS\n");
    assert!(out.ok);
    let json = serde_json::to_string(&out.visualizations).unwrap();
    assert!(json.contains("\"result\":\"[1, 2]\""), "{json}");
    // non-steppable thing still errors helpfully
    let out = run_source("G = GRAPH {1,2} {(1,2)}\nVISUALIZE VERTICES(G) AS STEPS\n");
    assert!(!out.ok);
}
