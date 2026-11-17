//! Step-by-step solutions for graph algorithms (`VISUALIZE P AS STEPS` where
//! `P = BFS(G, 1)`, `SHORTEST_PATH(G, 1, 5)`, `MST(G)`, …).
//!
//! Same contract as `explain.rs`: each section that reaches its own answer
//! records it, and `finish` compares it against what the real engine returned
//! (`algo::*`), so a narration can never silently drift from the truth.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::algo;
use crate::explain::{finish, Sec, Val};
use crate::ir::StepsIr;
use crate::token::Builtin;
use crate::value::{Edge, GraphVal};

pub fn supports(func: Builtin) -> bool {
    matches!(
        func,
        Builtin::Bfs
            | Builtin::Dfs
            | Builtin::ShortestPath
            | Builtin::Mst
            | Builtin::Color
            | Builtin::IsBipartite
            | Builtin::IsConnected
    )
}

/// How many extra (non-graph) arguments each supported function takes.
pub fn extra_args(func: Builtin) -> usize {
    match func {
        Builtin::Bfs | Builtin::Dfs => 1,
        Builtin::ShortestPath => 2,
        _ => 0,
    }
}

// ───────────────────────── formatting ─────────────────────────

fn list(v: &[i64]) -> String {
    if v.is_empty() {
        "∅".to_string()
    } else {
        format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", "))
    }
}

fn set<'a>(v: impl IntoIterator<Item = &'a i64>) -> String {
    let items: Vec<String> = v.into_iter().map(|x| x.to_string()).collect();
    if items.is_empty() {
        "∅".to_string()
    } else {
        format!("{{{}}}", items.join(", "))
    }
}

fn edge_text(g: &GraphVal, e: &Edge) -> String {
    let (a, b) = if g.directed { (e.from, e.to) } else { (e.from.min(e.to), e.from.max(e.to)) };
    match e.weight {
        Some(w) => format!("({a}, {b}, {w})"),
        None => format!("({a}, {b})"),
    }
}

fn pair(g: &GraphVal, a: i64, b: i64) -> String {
    if g.directed { format!("{a}→{b}") } else { format!("{a}–{b}") }
}

fn path_text(p: &[i64]) -> String {
    p.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" → ")
}

fn dash_if_empty(s: String) -> String {
    if s.is_empty() { "—".to_string() } else { s }
}

// ───────────────────────── entry point ─────────────────────────

pub fn explain_graph(func: Builtin, g: &GraphVal, args: &[i64], expression: String) -> Result<StepsIr, String> {
    if args.len() != extra_args(func) {
        return Err(format!("{} takes {} vertex argument(s) after the graph.", func.name(), extra_args(func)));
    }
    for &v in args {
        if !g.vertices.contains(&v) {
            return Err(format!("Vertex {} is not in the graph (vertices are {}).", v, set(&g.vertices)));
        }
    }
    let mut secs = vec![graph_note(g)];
    let (title, engine, result_text) = match func {
        Builtin::Bfs => bfs_secs(g, args[0], &mut secs),
        Builtin::Dfs => dfs_secs(g, args[0], &mut secs),
        Builtin::ShortestPath => path_secs(g, args[0], args[1], &mut secs)?,
        Builtin::Mst => mst_secs(g, &mut secs)?,
        Builtin::Color => color_secs(g, &mut secs),
        Builtin::IsBipartite => bipartite_secs(g, &mut secs),
        Builtin::IsConnected => connected_secs(g, &mut secs),
        other => return Err(format!("{} has no step-by-step view.", other.name())),
    };
    let mut ir = finish(func.name(), expression, title, engine, secs);
    if let Some(t) = result_text {
        ir.result = t;
    }
    Ok(ir)
}

fn graph_note(g: &GraphVal) -> Sec {
    let kind = if g.directed { "directed graph" } else { "undirected graph" };
    let mut s = Sec::new("note", "The graph").intro(format!(
        "{} vertices, {} edges, {}{}.",
        g.vertices.len(),
        g.edges.len(),
        kind,
        if g.edges.iter().any(|e| e.weight.is_some()) { ", weighted" } else { "" }
    ));
    s.line(format!("Vertices: {}", set(&g.vertices)));
    if g.edges.is_empty() {
        s.line("Edges: none");
    } else {
        s.line(format!("Edges: {}", g.edges.iter().map(|e| edge_text(g, e)).collect::<Vec<_>>().join(", ")));
    }
    let adj = algo::adjacency(g, false);
    let label = if g.directed { "Out-neighbors" } else { "Neighbors" };
    s.head(format!("{label} (always tried in ascending order):"));
    for (v, ns) in &adj {
        s.line(format!("{v} → {}", set(ns)));
    }
    s
}

// ───────────────────────── BFS ─────────────────────────

fn bfs_secs(g: &GraphVal, start: i64, out: &mut Vec<Sec>) -> (String, Val, Option<String>) {
    let adj = algo::adjacency(g, false);
    let engine = algo::bfs(g, start);

    let mut sec = Sec::new("traversal", "Breadth-first search").intro(
        "Use a queue (first in, first out). Take the front vertex, add every unvisited neighbor to the back in ascending order, and mark it visited immediately.",
    );
    let mut visited: BTreeSet<i64> = BTreeSet::new();
    let mut order = Vec::new();
    let mut dist: BTreeMap<i64, i64> = BTreeMap::new();
    let mut queue: VecDeque<i64> = VecDeque::new();
    visited.insert(start);
    order.push(start);
    dist.insert(start, 0);
    queue.push_back(start);
    sec.head(format!("Start: mark {start} visited and put it in the queue."));
    let mut rows = Vec::new();
    let mut step = 0;
    while let Some(v) = queue.pop_front() {
        step += 1;
        let mut found = Vec::new();
        for &w in &adj[&v] {
            if visited.insert(w) {
                order.push(w);
                dist.insert(w, dist[&v] + 1);
                queue.push_back(w);
                found.push(w);
            }
        }
        rows.push(vec![
            step.to_string(),
            v.to_string(),
            dash_if_empty(found.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            list(&queue.iter().copied().collect::<Vec<_>>()),
        ]);
        if found.is_empty() {
            sec.line(format!("Take {v} from the queue. Every neighbor is already visited, nothing new."));
        } else {
            sec.line(format!("Take {v} from the queue. New neighbors: {} → add to the back.", list(&found)));
        }
    }
    sec.table(&["Step", "Take from queue", "Newly discovered", "Queue afterwards"], rows);
    sec.good("The queue is empty, so every vertex reachable from the start has been visited.");
    let unreached: Vec<i64> = g.vertices.iter().copied().filter(|v| !visited.contains(v)).collect();
    if !unreached.is_empty() {
        sec.line(format!("Not reachable from {start}: {}", set(&unreached)));
    }
    sec.conclude(format!("Visit order: {}", list(&order)), Val::List(order.clone()));
    out.push(sec);

    let mut lv = Sec::new("note", "Levels (distance from the start)")
        .intro("BFS finds vertices in rings: first the start, then everything 1 edge away, then 2 edges away, …");
    let mut by_level: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for (&v, &d) in &dist {
        by_level.entry(d).or_default().push(v);
    }
    let rows = by_level
        .iter()
        .map(|(d, vs)| {
            // keep discovery order inside a level
            let mut vs = vs.clone();
            vs.sort_by_key(|v| order.iter().position(|x| x == v));
            vec![d.to_string(), set(&vs)]
        })
        .collect();
    lv.table(&["Edges from start", "Vertices"], rows);
    out.push(lv);
    (
        format!("BFS from {start}"),
        Val::List(engine.clone()),
        Some(list(&engine)),
    )
}

// ───────────────────────── DFS ─────────────────────────

fn dfs_secs(g: &GraphVal, start: i64, out: &mut Vec<Sec>) -> (String, Val, Option<String>) {
    let adj = algo::adjacency(g, false);
    let engine = algo::dfs(g, start);

    let mut sec = Sec::new("traversal", "Depth-first search").intro(
        "Go as deep as possible: from the current vertex step to its smallest unvisited neighbor; when there is none left, backtrack to the previous vertex.",
    );
    let mut visited: BTreeSet<i64> = BTreeSet::new();
    let mut order = vec![start];
    visited.insert(start);
    let mut stack: Vec<(i64, Vec<i64>)> = vec![(start, adj[&start].iter().rev().copied().collect())];
    let mut rows = vec![vec!["1".to_string(), format!("visit {start}"), start.to_string(), list(&order)]];
    sec.head(format!("Start at {start}."));
    let mut step = 1;
    while !stack.is_empty() {
        let next = {
            let frame = stack.last_mut().unwrap();
            loop {
                match frame.1.pop() {
                    Some(c) if !visited.contains(&c) => break Some(c),
                    Some(_) => continue,
                    None => break None,
                }
            }
        };
        step += 1;
        match next {
            Some(c) => {
                let from = stack.last().unwrap().0;
                visited.insert(c);
                order.push(c);
                stack.push((c, adj[&c].iter().rev().copied().collect()));
                sec.line(format!("From {from}, the smallest unvisited neighbor is {c} → go deeper."));
                rows.push(vec![
                    step.to_string(),
                    format!("go {from} → {c}"),
                    path_text(&stack.iter().map(|f| f.0).collect::<Vec<_>>()),
                    list(&order),
                ]);
            }
            None => {
                let (done, _) = stack.pop().unwrap();
                match stack.last() {
                    Some(&(back, _)) => {
                        sec.line(format!("{done} has no unvisited neighbors left → backtrack to {back}."));
                        rows.push(vec![
                            step.to_string(),
                            format!("backtrack from {done}"),
                            path_text(&stack.iter().map(|f| f.0).collect::<Vec<_>>()),
                            list(&order),
                        ]);
                    }
                    None => {
                        sec.line(format!("{done} has no unvisited neighbors left and nothing is below it on the stack."));
                        rows.push(vec![step.to_string(), format!("finish at {done}"), "—".to_string(), list(&order)]);
                    }
                }
            }
        }
    }
    sec.table(&["Step", "Action", "Current path (stack)", "Visited so far"], rows);
    sec.good("The stack is empty: the search is complete.");
    sec.conclude(format!("Visit order (preorder): {}", list(&order)), Val::List(order));
    out.push(sec);
    (format!("DFS from {start}"), Val::List(engine.clone()), Some(list(&engine)))
}

// ───────────────────────── shortest path ─────────────────────────

fn path_secs(g: &GraphVal, from: i64, to: i64, out: &mut Vec<Sec>) -> Result<(String, Val, Option<String>), String> {
    if let Some((a, b, w)) = algo::has_negative_weight(g) {
        return Err(format!(
            "Dijkstra's algorithm needs non-negative weights, but edge ({a}, {b}) has weight {w}."
        ));
    }
    let (engine, weighted) = algo::shortest_path(g, from, to).map_err(|_| "Shortest path failed.".to_string())?;
    let adj = algo::adjacency(g, false);
    let mut pred: BTreeMap<i64, i64> = BTreeMap::new();
    let mut total: Option<i64> = None;

    if !weighted {
        let mut sec = Sec::new("traversal", "Breadth-first search (every edge counts 1)").intro(
            "With no weights, the shortest path is the one with the fewest edges, which is exactly what BFS discovers first. Remember who discovered each vertex.",
        );
        let mut visited: BTreeSet<i64> = BTreeSet::new();
        let mut q: VecDeque<i64> = VecDeque::new();
        visited.insert(from);
        q.push_back(from);
        sec.head(format!("Start at {from}."));
        let mut rows = Vec::new();
        let mut step = 0;
        'outer: while let Some(v) = q.pop_front() {
            step += 1;
            let mut found = Vec::new();
            for &w in &adj[&v] {
                if visited.insert(w) {
                    pred.insert(w, v);
                    q.push_back(w);
                    found.push(format!("{w} (via {v})"));
                }
            }
            rows.push(vec![step.to_string(), v.to_string(), dash_if_empty(found.join(", "))]);
            if visited.contains(&to) {
                sec.line(format!("Take {v}. New: {}. The target {to} has been discovered, so we can stop.", found.join(", ")));
                break 'outer;
            }
            if found.is_empty() {
                sec.line(format!("Take {v}. Nothing new."));
            } else {
                sec.line(format!("Take {v}. New: {}.", found.join(", ")));
            }
        }
        sec.table(&["Step", "Take from queue", "Newly discovered (via)"], rows);
        if visited.contains(&to) {
            sec.good(format!("{to} reached."));
        } else {
            sec.bad(format!("The queue ran out before reaching {to}: there is no path."));
        }
        out.push(sec);
    } else {
        let wm: BTreeMap<(i64, i64), i64> = {
            let mut m = BTreeMap::new();
            for e in &g.edges {
                let key = if g.directed { (e.from, e.to) } else { algo::ukey(e.from, e.to) };
                m.entry(key).or_insert(e.weight.unwrap_or(1));
            }
            m
        };
        let mut sec = Sec::new("traversal", "Dijkstra's algorithm").intro(
            "Keep the best known distance to every vertex. Repeatedly settle the unsettled vertex with the smallest distance, then try to improve its neighbors (relax): a new distance counts only if it is strictly smaller.",
        );
        let mut dist: BTreeMap<i64, i64> = BTreeMap::new();
        dist.insert(from, 0);
        let mut pq: BTreeSet<(i64, i64)> = BTreeSet::new();
        pq.insert((0, from));
        let mut settled: BTreeSet<i64> = BTreeSet::new();
        sec.head(format!("Start: distance to {from} is 0, everything else is ∞."));
        let mut rows = Vec::new();
        let mut step = 0;
        while let Some(&(d, v)) = pq.iter().next() {
            pq.remove(&(d, v));
            if !settled.insert(v) {
                continue;
            }
            step += 1;
            if v == to {
                sec.good(format!("Settle {v} at distance {d}. That is the target, so we can stop."));
                rows.push(vec![step.to_string(), format!("{v}  (dist {d})"), "target reached".to_string(), dist_text(&dist, &settled)]);
                break;
            }
            let mut notes = Vec::new();
            for &w in &adj[&v] {
                if settled.contains(&w) {
                    continue;
                }
                let ew = wm.get(&(v, w)).copied().unwrap_or(1);
                let Some(nd) = d.checked_add(ew) else { continue };
                let old = dist.get(&w).copied();
                if nd < old.unwrap_or(i64::MAX) {
                    dist.insert(w, nd);
                    pred.insert(w, v);
                    pq.insert((nd, w));
                    notes.push(format!("{w}: {d}+{ew}={nd} < {} ✓", old.map(|o| o.to_string()).unwrap_or("∞".into())));
                } else {
                    notes.push(format!("{w}: {d}+{ew}={nd} ≥ {} ✗", old.unwrap()));
                }
            }
            if notes.is_empty() {
                sec.line(format!("Settle {v} at distance {d}. No unsettled neighbors to relax."));
            } else {
                sec.line(format!("Settle {v} at distance {d}. Relax: {}.", notes.join("; ")));
            }
            rows.push(vec![step.to_string(), format!("{v}  (dist {d})"), dash_if_empty(notes.join("; ")), dist_text(&dist, &settled)]);
        }
        sec.table(&["Step", "Settle", "Relaxations (✓ improved, ✗ not better)", "Distances (* = settled)"], rows);
        if let Some(&d) = dist.get(&to) {
            total = Some(d);
        } else {
            sec.bad(format!("{to} never got a finite distance: there is no path."));
        }
        out.push(sec);
    }

    // Rebuild the path from the "who discovered me" links.
    let mut rebuild = Sec::new("note", "Rebuild the path").intro("Follow the 'via' links backwards from the target to the start, then reverse.");
    let mut path = Vec::new();
    let reachable = from == to || pred.contains_key(&to);
    if reachable {
        let mut cur = to;
        path.push(cur);
        rebuild.line(format!("{cur}"));
        while cur != from {
            let p = pred[&cur];
            rebuild.line(format!("{cur} was reached via {p}"));
            path.push(p);
            cur = p;
        }
        path.reverse();
        rebuild.good(format!("Path: {}", path_text(&path)));
        if weighted {
            let sum: i64 = path
                .windows(2)
                .map(|w| {
                    let key = if g.directed { (w[0], w[1]) } else { algo::ukey(w[0], w[1]) };
                    g.edges
                        .iter()
                        .find(|e| (if g.directed { (e.from, e.to) } else { algo::ukey(e.from, e.to) }) == key)
                        .map(|e| e.weight.unwrap_or(1))
                        .unwrap_or(1)
                })
                .sum();
            rebuild.line(format!("Total weight along the path: {sum}"));
            if Some(sum) != total {
                rebuild.bad("This does not match the settled distance — please report this.");
            }
        } else {
            rebuild.line(format!("Length: {} edge(s)", path.len() - 1));
        }
        rebuild.conclude(format!("Shortest path: {}", path_text(&path)), Val::List(path.clone()));
    } else {
        rebuild.bad(format!("There is no path from {from} to {to}."));
        rebuild.conclude("No path (empty result)", Val::List(Vec::new()));
    }
    out.push(rebuild);

    let text = if engine.is_empty() {
        "no path".to_string()
    } else if weighted {
        format!("{}  (total weight {})", path_text(&engine), total.unwrap_or(0))
    } else {
        format!("{}  ({} edge(s))", path_text(&engine), engine.len() - 1)
    };
    Ok((format!("Shortest path {from} → {to}"), Val::List(engine), Some(text)))
}

fn dist_text(dist: &BTreeMap<i64, i64>, settled: &BTreeSet<i64>) -> String {
    dist.iter()
        .map(|(v, d)| format!("{v}={d}{}", if settled.contains(v) { "*" } else { "" }))
        .collect::<Vec<_>>()
        .join(", ")
}

// ───────────────────────── minimum spanning tree ─────────────────────────

fn mst_secs(g: &GraphVal, out: &mut Vec<Sec>) -> Result<(String, Val, Option<String>), String> {
    if let Some((a, b, w)) = algo::has_negative_weight(g) {
        return Err(format!("MST needs non-negative weights, but edge ({a}, {b}) has weight {w}."));
    }
    let tree = algo::kruskal(g).map_err(|_| {
        "The graph is not connected, so it has no spanning tree. (Every vertex must be reachable from every other.)".to_string()
    })?;
    let engine_total: i64 = tree.edges.iter().map(|e| e.weight.unwrap_or(1)).sum();
    let n = g.vertices.len();

    // distinct undirected edges, cheapest first (ties by endpoints), loops dropped later
    let mut cands: Vec<(i64, i64, i64)> = g.edges.iter().map(|e| (e.weight.unwrap_or(1), e.from, e.to)).collect();
    cands.sort();
    let mut seen: BTreeSet<(i64, i64)> = BTreeSet::new();
    let mut sorted: Vec<(i64, i64, i64)> = Vec::new();
    for (w, a, b) in cands {
        if seen.insert(algo::ukey(a, b)) {
            let (a, b) = algo::ukey(a, b);
            sorted.push((w, a, b));
        }
    }

    // ── Kruskal ──
    let mut k = Sec::new("rounds", "Kruskal's algorithm").intro(
        "Sort the edges from cheapest to most expensive. Walk through them and keep an edge only if it joins two different groups; skip it if its ends are already connected (it would close a cycle).",
    );
    let mut group: BTreeMap<i64, BTreeSet<i64>> = g.vertices.iter().map(|&v| (v, BTreeSet::from([v]))).collect();
    let mut root: BTreeMap<i64, i64> = g.vertices.iter().map(|&v| (v, v)).collect();
    let mut total = 0;
    let mut chosen = 0usize;
    let mut rows = Vec::new();
    k.head(format!("Sorted edges: {}", sorted.iter().map(|(w, a, b)| format!("{a}–{b} ({w})")).collect::<Vec<_>>().join(", ")));
    for (i, &(w, a, b)) in sorted.iter().enumerate() {
        if n > 0 && chosen == n - 1 {
            k.good(format!(
                "{chosen} edge(s) chosen: that connects all {n} vertices, so the remaining {} edge(s) are skipped.",
                sorted.len() - i
            ));
            rows.push(vec!["…".into(), "…".into(), "—".into(), "tree is complete".into()]);
            break;
        }
        let (ra, rb) = (root[&a], root[&b]);
        if ra == rb {
            let why = if a == b { "a loop".to_string() } else { format!("{a} and {b} are already connected") };
            k.tone("bad", format!("{}–{} (weight {w}): skip, {why}.", a, b));
            rows.push(vec![(i + 1).to_string(), format!("{a}–{b}"), w.to_string(), format!("skip: {why}")]);
        } else {
            let moved = group.remove(&rb).unwrap();
            for &v in &moved {
                root.insert(v, ra);
            }
            group.get_mut(&ra).unwrap().extend(moved);
            total += w;
            chosen += 1;
            k.good(format!("{a}–{b} (weight {w}): add, it joins two groups. Total so far {total}."));
            rows.push(vec![(i + 1).to_string(), format!("{a}–{b}"), w.to_string(), format!("add → total {total}")]);
        }
    }
    k.table(&["#", "Edge", "Weight", "Decision"], rows);
    k.conclude(format!("Total weight {total} using {chosen} edge(s)"), Val::Num(total));
    out.push(k);

    // ── Prim (independent cross-check of the total) ──
    let mut p = Sec::new("rounds", "Prim's algorithm (cross-check)").intro(
        "A different strategy: grow one tree from the smallest vertex, always adding the cheapest edge that reaches a new vertex. The edge set may differ when weights tie, but the total weight must be the same.",
    );
    let mut best: BTreeMap<(i64, i64), i64> = BTreeMap::new();
    for &(w, a, b) in &sorted {
        if a != b {
            best.entry((a, b)).or_insert(w);
        }
    }
    let mut ptotal = 0;
    let mut prows = Vec::new();
    if let Some(&s) = g.vertices.iter().next() {
        let mut inside: BTreeSet<i64> = BTreeSet::from([s]);
        p.head(format!("Start the tree at {s}."));
        let mut step = 0;
        while inside.len() < n {
            let mut pick: Option<(i64, i64, i64)> = None;
            for (&(a, b), &w) in &best {
                let cross = inside.contains(&a) != inside.contains(&b);
                if cross && pick.map_or(true, |(pw, pa, pb)| (w, a, b) < (pw, pa, pb)) {
                    pick = Some((w, a, b));
                }
            }
            let Some((w, a, b)) = pick else { break };
            let new = if inside.contains(&a) { b } else { a };
            inside.insert(new);
            ptotal += w;
            step += 1;
            p.line(format!("Cheapest edge leaving the tree: {a}–{b} (weight {w}) → bring in {new}. Total {ptotal}."));
            prows.push(vec![step.to_string(), format!("{a}–{b}"), w.to_string(), set(&inside), ptotal.to_string()]);
        }
        p.table(&["Step", "Edge added", "Weight", "Tree vertices", "Total"], prows);
    }
    p.conclude(format!("Total weight {ptotal}"), Val::Num(ptotal));
    out.push(p);

    let chosen_edges = tree.edges.iter().map(|e| edge_text(g, e)).collect::<Vec<_>>().join(", ");
    let text = format!("edges {{{}}} — total weight {}", chosen_edges, engine_total);
    Ok(("Minimum spanning tree".to_string(), Val::Num(engine_total), Some(text)))
}

// ───────────────────────── greedy coloring ─────────────────────────

fn color_secs(g: &GraphVal, out: &mut Vec<Sec>) -> (String, Val, Option<String>) {
    let engine = algo::greedy_color(g);
    let adj = algo::adjacency(g, true);
    let mut sec = Sec::new("rounds", "Greedy coloring").intro(
        "Go through the vertices in ascending order. Give each one the smallest color number (0, 1, 2, …) that none of its already-colored neighbors has.",
    );
    let mut colors: BTreeMap<i64, i64> = BTreeMap::new();
    let mut rows = Vec::new();
    for &v in &g.vertices {
        let mut blocked: BTreeSet<i64> = BTreeSet::new();
        let mut seen_n = Vec::new();
        for &w in &adj[&v] {
            if w != v {
                if let Some(&c) = colors.get(&w) {
                    blocked.insert(c);
                    seen_n.push(format!("{w}:{c}"));
                }
            }
        }
        let mut c = 0;
        while blocked.contains(&c) {
            c += 1;
        }
        colors.insert(v, c);
        rows.push(vec![v.to_string(), dash_if_empty(seen_n.join(", ")), set(&blocked), c.to_string()]);
        sec.line(format!(
            "Vertex {v}: colored neighbors {} → blocked colors {} → smallest free color is {c}.",
            if seen_n.is_empty() { "none".to_string() } else { seen_n.join(", ") },
            set(&blocked)
        ));
    }
    sec.table(&["Vertex", "Colored neighbors (vertex:color)", "Blocked colors", "Gets color"], rows);
    let used = colors.values().copied().collect::<BTreeSet<_>>().len();
    sec.conclude(
        format!("{} color(s): {}", used, colors.iter().map(|(v, c)| format!("{v}→{c}")).collect::<Vec<_>>().join(", ")),
        Val::List(colors.values().copied().collect()),
    );
    out.push(sec);

    let mut chk = Sec::new("verify", "Check: no two neighbors share a color");
    let mut bad = 0;
    for e in &g.edges {
        if e.from != e.to && colors[&e.from] == colors[&e.to] {
            bad += 1;
            chk.bad(format!("Edge {} joins two vertices with color {}.", pair(g, e.from, e.to), colors[&e.from]));
        }
    }
    if bad == 0 {
        chk.good(format!("All {} edge(s) join different colors. This is a proper coloring.", g.edges.iter().filter(|e| e.from != e.to).count()));
    }
    if g.edges.iter().any(|e| e.from == e.to) {
        chk.line("Self-loops are ignored: a vertex can't conflict with itself here.");
    }
    chk.line("Greedy coloring is quick but not always optimal: a different vertex order can sometimes use fewer colors.");
    out.push(chk);

    let text = format!(
        "{} color(s): {}",
        used,
        engine.iter().map(|(v, c)| format!("{v}→{c}")).collect::<Vec<_>>().join(", ")
    );
    ("Greedy coloring".to_string(), Val::List(engine.values().copied().collect()), Some(text))
}

// ───────────────────────── bipartite ─────────────────────────

fn bipartite_secs(g: &GraphVal, out: &mut Vec<Sec>) -> (String, Val, Option<String>) {
    let engine = algo::is_bipartite(g);
    let adj = algo::adjacency(g, true);
    let mut sec = Sec::new("rounds", "Two-coloring with BFS").intro(
        "A graph is bipartite if its vertices split into two teams so that every edge goes between the teams. Try it: color the start vertex A, all its neighbors B, their neighbors A, and so on. A conflict means it is impossible.",
    );
    let name = |c: i64| if c == 0 { "A" } else { "B" };
    let mut result = true;
    let mut witness: Option<(i64, i64)> = None;
    let mut color: BTreeMap<i64, i64> = BTreeMap::new();
    let mut parent: BTreeMap<i64, i64> = BTreeMap::new();
    let mut rows = Vec::new();
    if let Some(e) = g.edges.iter().find(|e| e.from == e.to) {
        sec.bad(format!("Vertex {} has a loop to itself, so it would have to be on both teams. Not bipartite.", e.from));
        result = false;
        witness = Some((e.from, e.from));
    } else {
        let mut step = 0;
        'all: for &s in &g.vertices {
            if color.contains_key(&s) {
                continue;
            }
            color.insert(s, 0);
            sec.head(format!("New component: color {s} with A."));
            let mut q: VecDeque<i64> = VecDeque::from([s]);
            while let Some(u) = q.pop_front() {
                step += 1;
                let cu = color[&u];
                let mut notes = Vec::new();
                for &w in &adj[&u] {
                    match color.get(&w) {
                        None => {
                            color.insert(w, 1 - cu);
                            parent.insert(w, u);
                            q.push_back(w);
                            notes.push(format!("{w}→{}", name(1 - cu)));
                        }
                        Some(&cw) if cw == cu => {
                            notes.push(format!("{w} is also {} ✗", name(cw)));
                            sec.bad(format!("Take {u} ({}). Its neighbor {w} is also {}: two connected vertices on the same team.", name(cu), name(cw)));
                            rows.push(vec![step.to_string(), format!("{u} ({})", name(cu)), notes.join(", "), "conflict".into()]);
                            result = false;
                            witness = Some((u, w));
                            break 'all;
                        }
                        Some(_) => notes.push(format!("{w} ok")),
                    }
                }
                sec.line(format!("Take {u} ({}). Neighbors: {}.", name(cu), if notes.is_empty() { "none".to_string() } else { notes.join(", ") }));
                rows.push(vec![step.to_string(), format!("{u} ({})", name(cu)), dash_if_empty(notes.join(", ")), "".into()]);
            }
        }
    }
    if !rows.is_empty() {
        sec.table(&["Step", "Take", "Neighbors", ""], rows);
    }
    if result {
        let a: Vec<i64> = color.iter().filter(|(_, &c)| c == 0).map(|(&v, _)| v).collect();
        let b: Vec<i64> = color.iter().filter(|(_, &c)| c == 1).map(|(&v, _)| v).collect();
        sec.good(format!("No conflicts. Team A = {}, team B = {}.", set(&a), set(&b)));
    }
    sec.conclude(if result { "Bipartite: TRUE" } else { "Not bipartite: FALSE" }, Val::Bool(result));
    out.push(sec);

    if let Some((u, w)) = witness {
        let mut oc = Sec::new("verify", "Why it fails: an odd cycle").intro("A graph is bipartite exactly when it has no cycle with an odd number of edges.");
        if u == w {
            oc.line(format!("The loop at {u} is a cycle of length 1."));
            oc.good("1 is odd, so the two teams can never work.");
        } else {
            // walk both vertices up the BFS tree until they meet
            let mut a_chain = vec![u];
            while let Some(&p) = parent.get(a_chain.last().unwrap()) {
                a_chain.push(p);
            }
            let mut b_chain = vec![w];
            while let Some(&p) = parent.get(b_chain.last().unwrap()) {
                b_chain.push(p);
            }
            let lca = a_chain.iter().copied().find(|x| b_chain.contains(x)).unwrap();
            let mut cyc: Vec<i64> = a_chain.iter().copied().take_while(|&x| x != lca).collect();
            cyc.push(lca);
            let tail: Vec<i64> = b_chain.iter().copied().take_while(|&x| x != lca).collect();
            cyc.extend(tail.into_iter().rev());
            let len = cyc.len();
            oc.line(format!("Join the tree paths from {u} and {w} back to their common ancestor {lca}, then close with the edge {u}–{w}."));
            oc.line(format!("Cycle: {} → {}", path_text(&cyc), cyc[0]));
            oc.tone(if len % 2 == 1 { "good" } else { "bad" }, format!("It has {len} edges, which is odd, so the two teams can never work."));
        }
        out.push(oc);
    }
    (
        "Is the graph bipartite?".to_string(),
        Val::Bool(engine),
        Some(if engine { "TRUE" } else { "FALSE" }.to_string()),
    )
}

// ───────────────────────── connectivity ─────────────────────────

fn connected_secs(g: &GraphVal, out: &mut Vec<Sec>) -> (String, Val, Option<String>) {
    let engine = algo::is_connected(g);
    let adj = algo::adjacency(g, true);
    let intro = if g.directed {
        "Connected here means weakly connected: ignore edge directions and check that one search reaches every vertex."
    } else {
        "Connected means one search from any vertex reaches every vertex."
    };
    let mut sec = Sec::new("traversal", "Search from the smallest vertex").intro(intro);
    let mut reached = Vec::new();
    if let Some(&s) = g.vertices.iter().next() {
        let mut visited: BTreeSet<i64> = BTreeSet::from([s]);
        let mut q: VecDeque<i64> = VecDeque::from([s]);
        reached.push(s);
        sec.head(format!("Start at {s}."));
        let mut rows = Vec::new();
        let mut step = 0;
        while let Some(v) = q.pop_front() {
            step += 1;
            let mut found = Vec::new();
            for &w in &adj[&v] {
                if visited.insert(w) {
                    reached.push(w);
                    q.push_back(w);
                    found.push(w);
                }
            }
            rows.push(vec![step.to_string(), v.to_string(), dash_if_empty(found.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")), visited.len().to_string()]);
            sec.line(format!("Take {v}. {}", if found.is_empty() { "Nothing new.".to_string() } else { format!("New: {}.", list(&found)) }));
        }
        sec.table(&["Step", "Take", "Newly reached", "Reached so far"], rows);
    } else {
        sec.line("The graph has no vertices; an empty graph counts as connected.");
    }
    let missing: Vec<i64> = g.vertices.iter().copied().filter(|v| !reached.contains(v)).collect();
    let ok = missing.is_empty();
    if ok {
        sec.good(format!("Reached all {} vertices.", g.vertices.len()));
    } else {
        sec.bad(format!("Reached {} of {} vertices. Unreachable: {}.", reached.len(), g.vertices.len(), set(&missing)));
    }
    sec.conclude(if ok { "Connected: TRUE" } else { "Not connected: FALSE" }, Val::Bool(ok));
    out.push(sec);
    (
        "Is the graph connected?".to_string(),
        Val::Bool(engine),
        Some(if engine { "TRUE" } else { "FALSE" }.to_string()),
    )
}
