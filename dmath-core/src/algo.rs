//! Pure discrete-mathematics algorithms for the relations & graphs domain.
//!
//! Everything here is deterministic and side-effect free so the interpreter
//! only has to map results onto values / diagnostics:
//!
//! - Relations: properties, closures (incl. Warshall), composition, classes.
//! - Graphs: adjacency, degree, BFS/DFS, shortest paths (BFS or Dijkstra),
//!   connectivity/tree/bipartite/complete checks, greedy coloring, Kruskal
//!   MST and Hasse-diagram layout (cover reduction + level layering).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::value::{Edge, GraphOrigin, GraphVal, SetElem};

/// Errors raised by algorithms that can fail on well-typed input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgoError {
    /// A negative edge weight reached Dijkstra / Kruskal.
    NegativeWeight { from: i64, to: i64, weight: i64 },
    /// The graph is not connected, so no spanning tree exists.
    Disconnected,
}

/// Largest set POWERSET accepts: 2^10 = 1024 subsets keeps the output readable.
pub const MAX_POWERSET_ELEMS: usize = 10;

/// The power set of `s`: every subset, from the empty set up to `s` itself.
/// The caller has already checked `s.len() <= MAX_POWERSET_ELEMS`.
pub fn powerset(s: &BTreeSet<SetElem>) -> BTreeSet<SetElem> {
    let items: Vec<&SetElem> = s.iter().collect();
    let n = items.len();
    (0u32..(1u32 << n))
        .map(|mask| {
            SetElem::Set(
                (0..n).filter(|i| mask & (1 << i) != 0).map(|i| items[i].clone()).collect(),
            )
        })
        .collect()
}

// ===========================================================================
// Relations (sets of ordered pairs, represented as `Vec<(i64, i64)>`)
// ===========================================================================

/// The field of a relation: every number appearing in any coordinate of any
/// pair, sorted ascending, without duplicates.
pub fn field(pairs: &[(i64, i64)]) -> Vec<i64> {
    let mut s = BTreeSet::new();
    for &(a, b) in pairs {
        s.insert(a);
        s.insert(b);
    }
    s.into_iter().collect()
}

pub fn is_reflexive(pairs: &[(i64, i64)]) -> bool {
    first_reflexive_violation(pairs).is_none()
}

/// Smallest field element `x` with `(x, x)` missing (None = reflexive).
pub fn first_reflexive_violation(pairs: &[(i64, i64)]) -> Option<i64> {
    let set: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    field(pairs).into_iter().find(|&x| !set.contains(&(x, x)))
}

pub fn is_symmetric(pairs: &[(i64, i64)]) -> bool {
    let set: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    pairs.iter().all(|&(a, b)| a == b || set.contains(&(b, a)))
}

/// First pair `(a, b)` (in sorted order, `a != b`) whose inverse is present.
pub fn first_antisymmetric_violation(pairs: &[(i64, i64)]) -> Option<(i64, i64)> {
    let set: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    for &(a, b) in set.iter() {
        if a != b && set.contains(&(b, a)) {
            return Some((a, b));
        }
    }
    None
}

pub fn is_antisymmetric(pairs: &[(i64, i64)]) -> bool {
    first_antisymmetric_violation(pairs).is_none()
}

/// First `(a, b, c)` with `(a, b)` and `(b, c)` present but `(a, c)` missing.
/// Iterates in sorted order so the counterexample is deterministic.
pub fn first_transitive_violation(pairs: &[(i64, i64)]) -> Option<(i64, i64, i64)> {
    let set: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    for &(a, b) in set.iter() {
        for &(c, d) in set.iter() {
            if b == c && !set.contains(&(a, d)) {
                return Some((a, b, d));
            }
        }
    }
    None
}

pub fn is_transitive(pairs: &[(i64, i64)]) -> bool {
    first_transitive_violation(pairs).is_none()
}

pub fn is_equivalence(pairs: &[(i64, i64)]) -> bool {
    is_reflexive(pairs) && is_symmetric(pairs) && is_transitive(pairs)
}

pub fn is_partial_order(pairs: &[(i64, i64)]) -> bool {
    is_reflexive(pairs) && is_antisymmetric(pairs) && is_transitive(pairs)
}

/// `INVERSE(R) = {(b, a) : (a, b) in R}`.
pub fn inverse(pairs: &[(i64, i64)]) -> BTreeSet<(i64, i64)> {
    pairs.iter().map(|&(a, b)| (b, a)).collect()
}

/// `COMPOSE(R, S) = {(x, z) : exists y, (x, y) in R and (y, z) in S}`
/// ("apply R first").
pub fn compose(r: &[(i64, i64)], s: &[(i64, i64)]) -> BTreeSet<(i64, i64)> {
    let mut out = BTreeSet::new();
    for &(a, b) in r {
        for &(c, d) in s {
            if b == c {
                out.insert((a, d));
            }
        }
    }
    out
}

/// Adds `(x, x)` for every element of the field.
pub fn reflexive_closure(pairs: &[(i64, i64)]) -> BTreeSet<(i64, i64)> {
    let mut out: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    for x in field(pairs) {
        out.insert((x, x));
    }
    out
}

/// Adds every missing inverse pair.
pub fn symmetric_closure(pairs: &[(i64, i64)]) -> BTreeSet<(i64, i64)> {
    let mut out: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    for &(a, b) in pairs {
        out.insert((b, a));
    }
    out
}

/// Transitive closure via the Warshall algorithm over the relation's field.
pub fn transitive_closure(pairs: &[(i64, i64)]) -> BTreeSet<(i64, i64)> {
    let f = field(pairs);
    let n = f.len();
    let idx: BTreeMap<i64, usize> = f.iter().cloned().enumerate().map(|(i, v)| (v, i)).collect();
    let mut m = vec![false; n * n];
    for &(a, b) in pairs {
        m[idx[&a] * n + idx[&b]] = true;
    }
    for k in 0..n {
        for i in 0..n {
            if m[i * n + k] {
                for j in 0..n {
                    if m[k * n + j] {
                        m[i * n + j] = true;
                    }
                }
            }
        }
    }
    let mut out = BTreeSet::new();
    for i in 0..n {
        for j in 0..n {
            if m[i * n + j] {
                out.insert((f[i], f[j]));
            }
        }
    }
    out
}

/// Partition of the field into equivalence classes (union-find). Every class
/// is sorted ascending; classes are ordered by their smallest element.
pub fn equivalence_classes(pairs: &[(i64, i64)]) -> Vec<Vec<i64>> {
    let f = field(pairs);
    let mut parent: BTreeMap<i64, i64> = f.iter().map(|&x| (x, x)).collect();
    for &(a, b) in pairs {
        let ra = uf_find(&mut parent, a);
        let rb = uf_find(&mut parent, b);
        if ra != rb {
            parent.insert(ra, rb);
        }
    }
    let mut groups: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    for &x in &f {
        groups.entry(uf_find(&mut parent, x)).or_default().insert(x);
    }
    groups.into_values().map(|s| s.into_iter().collect()).collect()
}

fn uf_find(parent: &mut BTreeMap<i64, i64>, mut x: i64) -> i64 {
    let root = {
        let mut cur = x;
        loop {
            let p = *parent.get(&cur).unwrap();
            if p == cur {
                break cur;
            }
            cur = p;
        }
    };
    while x != root {
        let next = *parent.get(&x).unwrap();
        parent.insert(x, root);
        x = next;
    }
    root
}

// ===========================================================================
// Hasse diagrams
// ===========================================================================

/// Layout data for a Hasse diagram of a (reflexive, antisymmetric,
/// transitive) relation: `(levels, cover_edges)`.
///
/// - `levels`: vertices grouped by longest-path layer (minimal elements at
///   level 0), each level sorted ascending, levels ordered bottom-first.
/// - `cover_edges`: the cover relation (strict order minus transitive
///   shortcuts), sorted by `(from, to)`.
pub fn hasse_layout(pairs: &[(i64, i64)]) -> (Vec<Vec<i64>>, Vec<(i64, i64)>) {
    let tc = transitive_closure(pairs);
    // Strict reachability: transitive closure minus the diagonal.
    let strict: BTreeSet<(i64, i64)> = tc.into_iter().filter(|&(a, b)| a != b).collect();
    // Cover edges: (a, b) in strict with no c strictly between a and b.
    let mut covers: Vec<(i64, i64)> = strict
        .iter()
        .copied()
        .filter(|&(a, b)| !strict.iter().any(|&(x, c)| x == a && strict.contains(&(c, b))))
        .collect();
    covers.sort();

    // Longest-path layering over the cover DAG (Kahn topological order).
    let verts = field(pairs);
    let mut level: BTreeMap<i64, usize> = verts.iter().map(|&v| (v, 0)).collect();
    let mut indeg: BTreeMap<i64, usize> = verts.iter().map(|&v| (v, 0)).collect();
    let mut outs: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for &(a, b) in &covers {
        outs.entry(a).or_default().push(b);
        *indeg.get_mut(&b).unwrap() += 1;
    }
    let mut queue: Vec<i64> = verts.iter().copied().filter(|&v| indeg[&v] == 0).collect();
    let mut qi = 0usize;
    while qi < queue.len() {
        let v = queue[qi];
        qi += 1;
        if let Some(ws) = outs.get(&v) {
            for &w in ws {
                let cand = level[&v] + 1;
                if cand > level[&w] {
                    level.insert(w, cand);
                }
                let d = indeg.get_mut(&w).unwrap();
                *d -= 1;
                if *d == 0 {
                    queue.push(w);
                }
            }
        }
    }
    let mut by_level: BTreeMap<usize, Vec<i64>> = BTreeMap::new();
    for (&v, &l) in &level {
        by_level.entry(l).or_default().push(v);
    }
    for (_, vs) in by_level.iter_mut() {
        vs.sort();
    }
    (by_level.into_values().collect(), covers)
}

// ===========================================================================
// Graph theory
// ===========================================================================

/// Normalized undirected edge key: `(min, max)`.
pub fn ukey(a: i64, b: i64) -> (i64, i64) {
    (a.min(b), a.max(b))
}

/// Adjacency lists (every list sorted ascending). `underlying = true` treats
/// a directed graph as undirected (both directions); self-loops appear as a
/// vertex's own neighbor.
pub fn adjacency(g: &GraphVal, underlying: bool) -> BTreeMap<i64, BTreeSet<i64>> {
    let mut adj: BTreeMap<i64, BTreeSet<i64>> =
        g.vertices.iter().map(|&v| (v, BTreeSet::new())).collect();
    let both = !g.directed || underlying;
    for e in &g.edges {
        adj.entry(e.from).or_default().insert(e.to);
        if both {
            adj.entry(e.to).or_default().insert(e.from);
        }
    }
    adj
}

/// Number of edge-ends at `v`: self-loops count 2; for digraphs the in-degree
/// plus the out-degree (a loop still counts 2).
pub fn degree(g: &GraphVal, v: i64) -> i64 {
    let mut d = 0i64;
    for e in &g.edges {
        if e.from == e.to {
            if e.from == v {
                d += 2;
            }
        } else if !g.directed {
            if e.from == v || e.to == v {
                d += 1;
            }
        } else {
            if e.from == v {
                d += 1;
            }
            if e.to == v {
                d += 1;
            }
        }
    }
    d
}

/// Sorted neighbor list of `v` (successors for digraphs).
pub fn neighbors(g: &GraphVal, v: i64) -> Vec<i64> {
    adjacency(g, false).get(&v).map(|s| s.iter().copied().collect()).unwrap_or_default()
}

/// `ADJACENT(G, a, b)`: direction-sensitive for digraphs.
pub fn adjacent(g: &GraphVal, a: i64, b: i64) -> bool {
    let key = if g.directed { (a, b) } else { ukey(a, b) };
    g.edges.iter().any(|e| (e.from, e.to) == key)
}

/// BFS over adjacency lists: queue, mark-on-enqueue, ascending neighbors.
/// Returns the visit order starting at `start` (empty if `start` is unknown).
pub fn bfs_adj(adj: &BTreeMap<i64, BTreeSet<i64>>, start: i64) -> Vec<i64> {
    if !adj.contains_key(&start) {
        return Vec::new();
    }
    let mut visited: BTreeSet<i64> = BTreeSet::new();
    let mut order = Vec::new();
    let mut queue: VecDeque<i64> = VecDeque::new();
    visited.insert(start);
    order.push(start);
    queue.push_back(start);
    while let Some(v) = queue.pop_front() {
        if let Some(ns) = adj.get(&v) {
            for &w in ns {
                if visited.insert(w) {
                    order.push(w);
                    queue.push_back(w);
                }
            }
        }
    }
    order
}

pub fn bfs(g: &GraphVal, start: i64) -> Vec<i64> {
    bfs_adj(&adjacency(g, false), start)
}

/// DFS preorder with ascending neighbors. Implemented with an explicit stack
/// that exactly emulates the recursive algorithm (safe for deep graphs).
pub fn dfs(g: &GraphVal, start: i64) -> Vec<i64> {
    let adj = adjacency(g, false);
    if !adj.contains_key(&start) {
        return Vec::new();
    }
    let mut visited: BTreeSet<i64> = BTreeSet::new();
    let mut order = Vec::new();
    // Stack frames: (vertex, remaining children in reverse-ascending order).
    let mut stack: Vec<(i64, Vec<i64>)> = Vec::new();
    visited.insert(start);
    order.push(start);
    stack.push((start, adj[&start].iter().rev().copied().collect()));
    while !stack.is_empty() {
        let next = {
            let frame = stack.last_mut().unwrap();
            loop {
                match frame.1.pop() {
                    Some(c) => {
                        if !visited.contains(&c) {
                            break Some(c);
                        }
                    }
                    None => break None,
                }
            }
        };
        match next {
            Some(c) => {
                visited.insert(c);
                order.push(c);
                let kids: Vec<i64> = adj[&c].iter().rev().copied().collect();
                stack.push((c, kids));
            }
            None => {
                stack.pop();
            }
        }
    }
    order
}

/// Edge -> weight map (default weight 1), keyed for the graph's direction.
fn weight_map(g: &GraphVal) -> BTreeMap<(i64, i64), i64> {
    let mut m = BTreeMap::new();
    for e in &g.edges {
        let key = if g.directed { (e.from, e.to) } else { ukey(e.from, e.to) };
        m.entry(key).or_insert(e.weight.unwrap_or(1));
    }
    m
}

pub fn has_negative_weight(g: &GraphVal) -> Option<(i64, i64, i64)> {
    for e in &g.edges {
        if let Some(w) = e.weight {
            if w < 0 {
                return Some((e.from, e.to, w));
            }
        }
    }
    None
}

/// Shortest path `from -> to`. Uses Dijkstra when the graph has at least one
/// explicit weight, plain BFS otherwise. Returns `(path, used_weights)`; an
/// unreachable target yields an empty path. BFS uses first-discoverer
/// predecessors; Dijkstra pops the smallest `(dist, vertex)` and relaxes in
/// ascending neighbor order with strict `<`.
pub fn shortest_path(g: &GraphVal, from: i64, to: i64) -> Result<(Vec<i64>, bool), AlgoError> {
    if let Some((a, b, w)) = has_negative_weight(g) {
        return Err(AlgoError::NegativeWeight { from: a, to: b, weight: w });
    }
    let adj = adjacency(g, false);
    if !adj.contains_key(&from) {
        return Ok((Vec::new(), false));
    }
    let weighted = g.edges.iter().any(|e| e.weight.is_some());
    let mut pred: BTreeMap<i64, i64> = BTreeMap::new();
    if !weighted {
        let mut visited: BTreeSet<i64> = BTreeSet::new();
        let mut q: VecDeque<i64> = VecDeque::new();
        visited.insert(from);
        q.push_back(from);
        while let Some(v) = q.pop_front() {
            if let Some(ns) = adj.get(&v) {
                for &w in ns {
                    if visited.insert(w) {
                        pred.insert(w, v);
                        q.push_back(w);
                    }
                }
            }
        }
        if !visited.contains(&to) {
            return Ok((Vec::new(), false));
        }
    } else {
        let wm = weight_map(g);
        let mut dist: BTreeMap<i64, i64> = BTreeMap::new();
        dist.insert(from, 0);
        let mut pq: BTreeSet<(i64, i64)> = BTreeSet::new(); // (dist, vertex)
        pq.insert((0, from));
        let mut settled: BTreeSet<i64> = BTreeSet::new();
        while let Some(&(d, v)) = pq.iter().next() {
            pq.remove(&(d, v));
            if !settled.insert(v) {
                continue;
            }
            if v == to {
                break;
            }
            if let Some(ns) = adj.get(&v) {
                for &w in ns {
                    if settled.contains(&w) {
                        continue;
                    }
                    let ew = wm.get(&(v, w)).copied().unwrap_or(1);
                    let Some(nd) = d.checked_add(ew) else { continue };
                    if nd < *dist.get(&w).unwrap_or(&i64::MAX) {
                        dist.insert(w, nd);
                        pred.insert(w, v);
                        pq.insert((nd, w));
                    }
                }
            }
        }
        if !dist.contains_key(&to) {
            return Ok((Vec::new(), true));
        }
    }
    let mut path = vec![to];
    while *path.last().unwrap() != from {
        let p = pred[&*path.last().unwrap()];
        path.push(p);
    }
    path.reverse();
    Ok((path, weighted))
}

/// BFS from the smallest vertex (weak connectivity for digraphs).
pub fn unreachable_count(g: &GraphVal) -> usize {
    if g.vertices.is_empty() {
        return 0;
    }
    let start = *g.vertices.iter().next().unwrap();
    let reached = bfs_adj(&adjacency(g, true), start);
    g.vertices.len() - reached.len()
}

pub fn is_connected(g: &GraphVal) -> bool {
    unreachable_count(g) == 0
}

/// Distinct undirected edge keys (loops included as `(a, a)`).
fn undirected_edge_keys(g: &GraphVal) -> BTreeSet<(i64, i64)> {
    g.edges.iter().map(|e| ukey(e.from, e.to)).collect()
}

/// Connected (underlying) and edges == n-1. Empty graphs are not trees.
pub fn is_tree(g: &GraphVal) -> bool {
    if g.vertices.is_empty() {
        return false;
    }
    if !is_connected(g) {
        return false;
    }
    undirected_edge_keys(g).len() == g.vertices.len() - 1
}

/// BFS 2-coloring over the underlying undirected graph; self-loops make the
/// graph non-bipartite; isolated vertices are fine.
pub fn is_bipartite(g: &GraphVal) -> bool {
    if g.edges.iter().any(|e| e.from == e.to) {
        return false;
    }
    let adj = adjacency(g, true);
    let mut color: BTreeMap<i64, i64> = BTreeMap::new();
    for &v in &g.vertices {
        if color.contains_key(&v) {
            continue;
        }
        color.insert(v, 0);
        let mut q: VecDeque<i64> = VecDeque::new();
        q.push_back(v);
        while let Some(u) = q.pop_front() {
            if let Some(ns) = adj.get(&u) {
                for &w in ns {
                    match color.get(&w) {
                        None => {
                            color.insert(w, 1 - color[&u]);
                            q.push_back(w);
                        }
                        Some(&c) => {
                            if c == color[&u] {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    true
}

/// Underlying-undirected edge count == n(n-1)/2, with loops disqualifying.
pub fn is_complete(g: &GraphVal) -> bool {
    if g.edges.iter().any(|e| e.from == e.to) {
        return false;
    }
    let n = g.vertices.len();
    undirected_edge_keys(g).len() == n * (n - 1) / 2
}

/// Greedy coloring: vertices ascending, each gets the smallest color index
/// not used by its already-colored (underlying-undirected) neighbors.
pub fn greedy_color(g: &GraphVal) -> BTreeMap<i64, i64> {
    let adj = adjacency(g, true);
    let mut colors: BTreeMap<i64, i64> = BTreeMap::new();
    for &v in &g.vertices {
        let mut used: BTreeSet<i64> = BTreeSet::new();
        if let Some(ns) = adj.get(&v) {
            for &w in ns {
                if w != v {
                    if let Some(&c) = colors.get(&w) {
                        used.insert(c);
                    }
                }
            }
        }
        let mut c = 0i64;
        while used.contains(&c) {
            c += 1;
        }
        colors.insert(v, c);
    }
    colors
}

/// Kruskal MST over the underlying undirected graph. Edges are sorted by
/// `(weight.unwrap_or(1), from, to)`; unweighted edges count as weight 1.
/// The result is an undirected graph carrying `GraphOrigin::Mst`.
pub fn kruskal(g: &GraphVal) -> Result<GraphVal, AlgoError> {
    if let Some((a, b, w)) = has_negative_weight(g) {
        return Err(AlgoError::NegativeWeight { from: a, to: b, weight: w });
    }
    let mut cands: Vec<(i64, i64, i64, Option<i64>)> =
        g.edges.iter().map(|e| (e.weight.unwrap_or(1), e.from, e.to, e.weight)).collect();
    cands.sort();
    // For digraphs, (a, b) and (b, a) are the same undirected edge: keep the
    // first candidate in sorted order (i.e. the cheaper one).
    let mut seen: BTreeSet<(i64, i64)> = BTreeSet::new();
    let mut picked: Vec<(i64, i64, Option<i64>)> = Vec::new();
    for (_w, a, b, orig) in cands {
        let key = ukey(a, b);
        if seen.insert(key) {
            picked.push((key.0, key.1, orig));
        }
    }
    let mut parent: BTreeMap<i64, i64> = g.vertices.iter().map(|&v| (v, v)).collect();
    let mut components = g.vertices.len();
    let mut chosen: Vec<Edge> = Vec::new();
    for (a, b, w) in picked {
        let ra = uf_find(&mut parent, a);
        let rb = uf_find(&mut parent, b);
        if ra != rb {
            parent.insert(ra, rb);
            components -= 1;
            chosen.push(Edge { from: a, to: b, weight: w });
        }
    }
    if components > 1 {
        return Err(AlgoError::Disconnected);
    }
    let total = chosen.iter().map(|e| e.weight.unwrap_or(1)).sum();
    Ok(GraphVal {
        directed: false,
        vertices: g.vertices.clone(),
        edges: chosen,
        origin: Some(GraphOrigin::Mst { total_weight: total }),
    })
}

// ===========================================================================
// Number theory (v1.2)
// ===========================================================================

/// `FACT` upper bound: 20! = 2 432 902 008 176 640 000 is the largest
/// factorial that fits in `i64` (21! already overflows).
pub const FACT_MAX_N: i64 = 20;

/// `PASCAL` row cap for display sanity: row 25's widest value is
/// C(25, 12) = 5 200 300, well inside `i64`.
pub const PASCAL_MAX_ROWS: i64 = 25;

/// `SIEVE` upper bound: 100 000 keeps both the prime list and the grid view
/// comfortably small for a teaching tool (9 592 primes below it).
pub const SIEVE_MAX_N: i64 = 100_000;

/// Deterministic Miller–Rabin primality test for `u64`.
///
/// The witness set {2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37} is proven to
/// be a correct primality certificate for every n < 3.3 × 10^24, i.e. for the
/// whole `u64` range — no probabilistic error is possible here.
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for &p in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n == p {
            return true;
        }
        if n % p == 0 {
            return false;
        }
    }
    // Write n - 1 = d · 2^s with d odd.
    let mut d = n - 1;
    let mut s = 0u32;
    while d % 2 == 0 {
        d /= 2;
        s += 1;
    }
    'witnesses: for &a in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = pow_mod_u64(a, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..s - 1 {
            x = mul_mod_u64(x, x, n);
            if x == n - 1 {
                continue 'witnesses;
            }
        }
        return false; // composite witness: n is definitely composite
    }
    true
}

fn mul_mod_u64(a: u64, b: u64, m: u64) -> u64 {
    (((a as u128) * (b as u128)) % (m as u128)) as u64
}

fn pow_mod_u64(mut base: u64, mut exp: u64, m: u64) -> u64 {
    if m == 1 {
        return 0;
    }
    let mut result: u64 = 1;
    base %= m;
    while exp > 0 {
        if exp % 2 == 1 {
            result = mul_mod_u64(result, base, m);
        }
        base = mul_mod_u64(base, base, m);
        exp /= 2;
    }
    result
}

/// Pollard's rho with Floyd cycle detection: one non-trivial factor of an
/// odd composite `n` that has no prime factor below 41 (the trial-division
/// stage of `prime_factors` guarantees that). Finding the smallest prime
/// factor `p ≤ √n ≤ 3·10^9` takes O(√p) ≈ 55 000 steps in the worst case,
/// each a single mulmod + gcd — microseconds to low milliseconds. The
/// polynomial constant `c` increments on every failed cycle so the search
/// always terminates with a factor.
fn pollard_rho(n: u64) -> u64 {
    debug_assert!(n > 1 && n % 2 == 1);
    if n % 3 == 0 {
        return 3;
    }
    let mut c: u64 = 1;
    loop {
        let f = |x: u64| (mul_mod_u64(x, x, n) + c) % n;
        let (mut x, mut y, mut d) = (2u64, 2u64, 1u64);
        loop {
            x = f(x);
            y = f(f(y));
            if x == y {
                break; // cycle closed without meeting a factor: change c
            }
            d = gcd_u64(x.abs_diff(y), n);
            if d != 1 {
                break;
            }
        }
        if d > 1 && d < n {
            return d;
        }
        c += 1;
    }
}

/// Split `n` into two factors using Pollard rho, recursing on both halves.
fn factor_large(n: u64, out: &mut Vec<u64>) {
    if n == 1 {
        return;
    }
    if is_prime(n) {
        out.push(n);
        return;
    }
    let d = pollard_rho(n);
    factor_large(d, out);
    factor_large(n / d, out);
}

/// Full prime factorization with multiplicity, ascending:
/// `prime_factors(360)` = `[2, 2, 2, 3, 3, 5]`.
///
/// Small primes are stripped by trial division; any remaining cofactor
/// (which can be up to ~9.2 × 10^18) is split by Miller–Rabin + Pollard–Brent,
/// so even hard semiprimes factor quickly.
pub fn prime_factors(n: u64) -> Vec<u64> {
    if n == 0 {
        return Vec::new(); // 0 has no prime factorization
    }
    let mut out: Vec<u64> = Vec::new();
    let mut m = n;
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        while m % p == 0 {
            out.push(p);
            m /= p;
        }
    }
    if m > 1 {
        // Trial division by odd candidates up to 10^6 catches everything
        // below 10^12 without touching the rho machinery.
        let mut d: u64 = 41;
        while d * d <= m && d < 1_000_000 {
            while m % d == 0 {
                out.push(d);
                m /= d;
            }
            d += 2;
        }
        if m > 1 {
            factor_large(m, &mut out);
        }
    }
    out.sort_unstable();
    out
}

/// All positive divisors of `n` in ascending order, derived from the prime
/// factorization (so `divisors(36)` = `[1, 2, 3, 4, 6, 9, 12, 18, 36]`).
pub fn divisors(n: i64) -> Vec<i64> {
    if n <= 0 {
        return Vec::new();
    }
    let n = n as u64;
    let factors = prime_factors(n);
    let mut divs: Vec<u64> = vec![1];
    // Group equal primes into prime powers and multiply each divisor by
    // every power of the current prime.
    let mut i = 0;
    while i < factors.len() {
        let p = factors[i];
        let mut j = i;
        while j < factors.len() && factors[j] == p {
            j += 1;
        }
        let exponent = j - i;
        let mut powers = vec![1u64];
        let mut cur = 1u64;
        for _ in 0..exponent {
            cur *= p;
            powers.push(cur);
        }
        let mut next = Vec::with_capacity(divs.len() * powers.len());
        for &base in &divs {
            for &pw in &powers {
                next.push(base * pw);
            }
        }
        divs = next;
        i = j;
    }
    divs.sort_unstable();
    divs.into_iter().map(|d| d as i64).collect()
}

/// Greatest common divisor via the Euclidean algorithm; `gcd(0, 0) = 0`.
pub fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.unsigned_abs(), b.unsigned_abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a as i64
}

pub fn gcd_u64(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

/// Least common multiple, `None` when the result would overflow `i64`.
pub fn lcm_checked(a: i64, b: i64) -> Option<i64> {
    if a == 0 || b == 0 {
        return Some(0);
    }
    let g = gcd(a, b) as u128;
    let prod = (a.unsigned_abs() as u128) * (b.unsigned_abs() as u128) / g;
    if prod > i64::MAX as u128 {
        None
    } else {
        Some(prod as i64)
    }
}

/// Euler's totient via the product formula
/// φ(n) = n · Π (1 − 1/p) over the distinct prime factors.
/// `totient(1) = 1` by convention.
pub fn totient(n: u64) -> u64 {
    if n == 0 {
        return 0; // φ(0) is undefined; refuse rather than hang
    }
    let factors = prime_factors(n);
    let mut phi = n;
    let mut i = 0;
    while i < factors.len() {
        let p = factors[i];
        phi = phi / p * (p - 1);
        while i < factors.len() && factors[i] == p {
            i += 1;
        }
    }
    phi
}

/// Sieve of Eratosthenes: all primes ≤ `limit`, ascending.
pub fn sieve(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }
    let n = limit as usize;
    let mut composite = vec![false; n + 1];
    let mut primes = Vec::new();
    let mut p: usize = 2;
    while p * p <= n {
        if !composite[p] {
            let mut m = p * p;
            while m <= n {
                composite[m] = true;
                m += p;
            }
        }
        p += 1;
    }
    for v in 2..=n {
        if !composite[v] {
            primes.push(v as u64);
        }
    }
    primes
}

/// Sieve variant for the visualization: returns the primes plus, for every
/// composite ≤ `limit`, its **smallest prime factor** (the sieve "crosses
/// out" each composite through one of its prime divisors).
pub fn sieve_grid(limit: u64) -> (Vec<u64>, std::collections::BTreeMap<u64, u64>) {
    let primes = sieve(limit);
    let mut smallest: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();
    if limit >= 4 {
        let n = limit as usize;
        let mut spf: Vec<u64> = vec![0; n + 1]; // 0 = prime or < 2
        let mut p: usize = 2;
        while p <= n {
            if spf[p] == 0 {
                let mut m = p * p;
                while m <= n {
                    if spf[m] == 0 {
                        spf[m] = p as u64;
                    }
                    m += p;
                }
            }
            p += 1;
        }
        for v in 4..=n {
            if spf[v] != 0 {
                smallest.insert(v as u64, spf[v]);
            }
        }
    }
    (primes, smallest)
}

// ===========================================================================
// Combinatorics (v1.3)
// ===========================================================================

/// `n!`, `None` for `n > 20` (21! overflows `i64`).
pub fn factorial_checked(n: i64) -> Option<i64> {
    if n < 0 || n > FACT_MAX_N {
        return None;
    }
    let mut acc: i64 = 1;
    for k in 2..=n {
        acc = acc.checked_mul(k)?;
    }
    Some(acc)
}

/// `P(n, k) = n · (n−1) · … · (n−k+1)`, computed iteratively in 128-bit
/// arithmetic so intermediate products never wrap silently.
/// `None` when the value does not fit in `i64`.
pub fn perm_checked(n: i64, k: i64) -> Option<i64> {
    if n < 0 || k < 0 || k > n {
        return None;
    }
    let mut acc: i128 = 1;
    for i in 0..k {
        acc *= (n - i) as i128;
        if acc > i64::MAX as i128 {
            return None;
        }
    }
    Some(acc as i64)
}

/// `C(n, k)`, computed iteratively by `r = r · (n−i) / (i+1)` — every
/// intermediate value is itself a binomial coefficient, so divisions are
/// exact and no factorial ever materializes. `None` when `k > n` or the
/// result overflows `i64` (C(67, 33) is the first such case).
pub fn comb_checked(n: i64, k: i64) -> Option<i64> {
    if n < 0 || k < 0 || k > n {
        return None;
    }
    let k = k.min(n - k); // symmetry keeps the loop short
    let mut r: i128 = 1;
    for i in 0..k {
        r = r.checked_mul((n - i) as i128)?;
        r /= (i + 1) as i128;
        if r > i64::MAX as i128 {
            return None;
        }
    }
    Some(r as i64)
}

/// The first `rows` rows of Pascal's triangle: row `i` holds
/// `C(i, 0) … C(i, i)`. Every value up to row 25 fits in `i64`.
pub fn pascal_rows(rows: usize) -> Vec<Vec<i64>> {
    let mut out: Vec<Vec<i64>> = Vec::with_capacity(rows);
    for i in 0..rows {
        let i64_i = i as i64;
        let mut row = Vec::with_capacity(i + 1);
        for j in 0..=i {
            // Within the 25-row cap C(i, j) always fits; fall back gracefully.
            row.push(comb_checked(i64_i, j as i64).unwrap_or(i64::MAX));
        }
        out.push(row);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(pairs: &[(i64, i64)]) -> BTreeSet<(i64, i64)> {
        pairs.iter().copied().collect()
    }

    #[test]
    fn warshall_basic_chain() {
        let tc = transitive_closure(&[(1, 2), (2, 3)]);
        assert_eq!(tc, set(&[(1, 2), (2, 3), (1, 3)]));
    }

    #[test]
    fn warshall_is_idempotent() {
        let r = [(1, 2), (2, 3), (3, 1), (1, 4)];
        let once = transitive_closure(&r);
        let twice = transitive_closure(&once.iter().copied().collect::<Vec<_>>());
        assert_eq!(once, twice);
    }

    #[test]
    fn warshall_cycle_closes_diagonal() {
        let tc = transitive_closure(&[(1, 2), (2, 1)]);
        assert_eq!(tc, set(&[(1, 1), (1, 2), (2, 1), (2, 2)]));
    }

    #[test]
    fn warshall_empty_relation() {
        assert!(transitive_closure(&[]).is_empty());
    }

    #[test]
    fn relation_properties_and_closures() {
        let r = [(1, 1), (1, 2), (2, 2), (2, 3)];
        assert!(!is_reflexive(&r));
        assert_eq!(first_reflexive_violation(&r), Some(3));
        assert!(!is_symmetric(&r));
        assert!(is_antisymmetric(&r));
        assert!(!is_transitive(&r));
        let rc = reflexive_closure(&r);
        assert!(is_reflexive(&rc.iter().copied().collect::<Vec<_>>()));
        let sc = symmetric_closure(&r);
        assert!(is_symmetric(&sc.iter().copied().collect::<Vec<_>>()));
    }

    #[test]
    fn antisymmetric_violation_is_first_sorted_pair() {
        assert_eq!(first_antisymmetric_violation(&[(3, 0), (0, 3)]), Some((0, 3)));
    }

    #[test]
    fn compose_applies_left_first() {
        let out = compose(&[(1, 2), (2, 3)], &[(3, 4)]);
        assert_eq!(out, set(&[(2, 4)]));
    }

    #[test]
    fn inverse_swaps_pairs() {
        assert_eq!(inverse(&[(1, 2), (2, 3)]), set(&[(2, 1), (3, 2)]));
    }

    #[test]
    fn equivalence_classes_partition_field() {
        let classes = equivalence_classes(&[(1, 1), (2, 2), (3, 3), (1, 2), (2, 1)]);
        assert_eq!(classes, vec![vec![1, 2], vec![3]]);
    }

    #[test]
    fn hasse_cover_reduction_on_divisibility() {
        // Divisibility on {1, 2, 3, 6}.
        let r = [(1, 1), (1, 2), (1, 3), (1, 6), (2, 2), (2, 6), (3, 3), (3, 6), (6, 6)];
        let (levels, edges) = hasse_layout(&r);
        assert_eq!(levels, vec![vec![1], vec![2, 3], vec![6]]);
        assert_eq!(edges, vec![(1, 2), (1, 3), (2, 6), (3, 6)]);
    }

    #[test]
    fn hasse_single_element() {
        let (levels, edges) = hasse_layout(&[(7, 7)]);
        assert_eq!(levels, vec![vec![7]]);
        assert!(edges.is_empty());
    }

    #[test]
    fn hasse_chain_levels() {
        // 1 < 2 < 3 with reflexivity.
        let r = [(1, 1), (2, 2), (3, 3), (1, 2), (2, 3), (1, 3)];
        let (levels, edges) = hasse_layout(&r);
        assert_eq!(levels, vec![vec![1], vec![2], vec![3]]);
        assert_eq!(edges, vec![(1, 2), (2, 3)]);
    }

    fn graph(directed: bool, vs: &[i64], es: &[(i64, i64)]) -> GraphVal {
        GraphVal::build(
            directed,
            vs.iter().copied().collect(),
            es.iter().map(|&(a, b)| Edge { from: a, to: b, weight: None }).collect(),
        )
    }

    #[test]
    fn build_normalizes_and_dedupes_undirected_edges() {
        let g = graph(false, &[1, 2, 3], &[(2, 1), (1, 2), (2, 3)]);
        assert_eq!(g.edges.len(), 2);
        assert_eq!(g.edges[0], Edge { from: 1, to: 2, weight: None });
        assert_eq!(g.edges[1], Edge { from: 2, to: 3, weight: None });
    }

    #[test]
    fn directed_keeps_both_directions() {
        let g = graph(true, &[1, 2], &[(1, 2), (2, 1)]);
        assert_eq!(g.edges.len(), 2);
    }

    #[test]
    fn degree_counts_loop_ends_twice() {
        let g = graph(false, &[1, 2], &[(1, 1), (1, 2)]);
        assert_eq!(degree(&g, 1), 3);
        assert_eq!(degree(&g, 2), 1);
        let d = graph(true, &[1, 2], &[(1, 1), (1, 2)]);
        assert_eq!(degree(&d, 1), 3); // loop (2) + out-edge (1)
        assert_eq!(degree(&d, 2), 1); // in-edge
    }

    #[test]
    fn bfs_and_dfs_orders() {
        // The six-vertex reference graph.
        let g = graph(false, &[1, 2, 3, 4, 5, 6], &[(1, 2), (1, 3), (2, 4), (3, 4), (4, 5), (5, 6)]);
        assert_eq!(bfs(&g, 1), vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(dfs(&g, 1), vec![1, 2, 4, 3, 5, 6]);
    }

    #[test]
    fn shortest_path_unweighted_uses_bfs() {
        let g = graph(false, &[1, 2, 3, 4, 5, 6], &[(1, 2), (1, 3), (2, 4), (3, 4), (4, 5), (5, 6)]);
        let (path, weighted) = shortest_path(&g, 1, 6).unwrap();
        assert_eq!(path, vec![1, 2, 4, 5, 6]);
        assert!(!weighted);
    }

    #[test]
    fn shortest_path_unreachable_is_empty() {
        let g = graph(false, &[1, 2, 3], &[(1, 2)]);
        let (path, _) = shortest_path(&g, 1, 3).unwrap();
        assert!(path.is_empty());
    }

    #[test]
    fn dijkstra_prefers_cheaper_route() {
        let w = GraphVal::build(
            false,
            [1, 2, 3, 4, 5].into_iter().collect(),
            vec![
                Edge { from: 1, to: 2, weight: Some(7) },
                Edge { from: 1, to: 3, weight: Some(3) },
                Edge { from: 2, to: 3, weight: Some(2) },
                Edge { from: 2, to: 4, weight: Some(2) },
                Edge { from: 3, to: 4, weight: Some(8) },
                Edge { from: 4, to: 5, weight: Some(3) },
            ],
        );
        let (path, weighted) = shortest_path(&w, 1, 4).unwrap();
        assert_eq!(path, vec![1, 3, 2, 4]);
        assert!(weighted);
    }

    #[test]
    fn negative_weight_is_rejected() {
        let g = GraphVal::build(
            false,
            [1, 2].into_iter().collect(),
            vec![Edge { from: 1, to: 2, weight: Some(-3) }],
        );
        assert!(matches!(shortest_path(&g, 1, 2), Err(AlgoError::NegativeWeight { weight: -3, .. })));
        assert!(matches!(kruskal(&g), Err(AlgoError::NegativeWeight { weight: -3, .. })));
    }

    #[test]
    fn connectivity_tree_bipartite_complete() {
        let path6 = graph(false, &[1, 2, 3, 4, 5, 6], &[(1, 2), (1, 3), (2, 4), (3, 4), (4, 5), (5, 6)]);
        assert!(is_connected(&path6));
        assert!(!is_tree(&path6)); // 6 edges != 5
        assert!(is_bipartite(&path6));
        assert!(!is_complete(&path6));

        let triangle = graph(false, &[1, 2, 3], &[(1, 2), (2, 3), (1, 3)]);
        assert!(is_connected(&triangle));
        assert!(is_tree(&triangle) == false);
        assert!(!is_bipartite(&triangle));
        assert!(is_complete(&triangle));

        let tree = graph(false, &[1, 2, 3], &[(1, 2), (2, 3)]);
        assert!(is_tree(&tree));

        let with_loop = graph(false, &[1, 2], &[(1, 1), (1, 2)]);
        assert!(!is_complete(&with_loop));
        assert!(!is_bipartite(&with_loop));

        // Digraph: weak connectivity over the underlying structure.
        let digraph = graph(true, &[1, 2, 3], &[(2, 1), (3, 2)]);
        assert!(is_connected(&digraph));
        assert!(is_tree(&digraph)); // underlying: 2 edges, 3 vertices, connected
    }

    #[test]
    fn greedy_color_triangle_and_path() {
        let triangle = graph(false, &[1, 2, 3], &[(1, 2), (2, 3), (1, 3)]);
        let c = greedy_color(&triangle);
        let distinct: BTreeSet<i64> = c.values().copied().collect();
        assert_eq!(distinct.len(), 3);
        let path = graph(false, &[1, 2, 3], &[(1, 2), (2, 3)]);
        let c = greedy_color(&path);
        let distinct: BTreeSet<i64> = c.values().copied().collect();
        assert_eq!(distinct.len(), 2);
    }

    #[test]
    fn kruskal_picks_minimum_spanning_tree() {
        let w = GraphVal::build(
            false,
            [1, 2, 3, 4, 5].into_iter().collect(),
            vec![
                Edge { from: 1, to: 2, weight: Some(7) },
                Edge { from: 1, to: 3, weight: Some(3) },
                Edge { from: 2, to: 3, weight: Some(2) },
                Edge { from: 2, to: 4, weight: Some(2) },
                Edge { from: 3, to: 4, weight: Some(8) },
                Edge { from: 4, to: 5, weight: Some(3) },
            ],
        );
        let mst = kruskal(&w).unwrap();
        assert_eq!(mst.edges.len(), 4);
        assert_eq!(mst.origin, Some(GraphOrigin::Mst { total_weight: 10 }));
        assert!(!mst.directed);
        assert!(is_connected(&mst));
    }

    #[test]
    fn kruskal_unweighted_edges_count_as_one() {
        let g = graph(false, &[1, 2, 3], &[(1, 2), (2, 3), (1, 3)]);
        let mst = kruskal(&g).unwrap();
        assert_eq!(mst.edges.len(), 2);
        assert_eq!(mst.origin, Some(GraphOrigin::Mst { total_weight: 2 }));
    }

    #[test]
    fn kruskal_disconnected_fails() {
        let g = graph(false, &[1, 2, 3], &[(1, 2)]);
        assert_eq!(kruskal(&g), Err(AlgoError::Disconnected));
    }

    #[test]
    fn kruskal_directed_uses_underlying_edges() {
        let g = graph(true, &[1, 2, 3], &[(2, 1), (3, 2), (1, 3)]);
        let mst = kruskal(&g).unwrap();
        assert_eq!(mst.edges.len(), 2);
        assert_eq!(mst.origin, Some(GraphOrigin::Mst { total_weight: 2 }));
    }

    // ----- number theory ---------------------------------------------------

    #[test]
    fn miller_rabin_edges_and_classics() {
        for n in [0u64, 1, 4, 6, 9, 15, 25, 100] {
            assert!(!is_prime(n), "{n} must not be prime");
        }
        for n in [2u64, 3, 5, 7, 11, 13, 97, 7919] {
            assert!(is_prime(n), "{n} must be prime");
        }
        // Carmichael numbers fool Fermat tests but not Miller–Rabin.
        for n in [561u64, 1105, 1729, 41041] {
            assert!(!is_prime(n), "{n} is a Carmichael number");
        }
        // The largest 63-bit prime and a Mersenne prime.
        assert!(is_prime(9_223_372_036_854_775_783));
        assert!(!is_prime(9_223_372_036_854_775_782));
    }

    #[test]
    fn prime_factors_with_multiplicity() {
        assert!(prime_factors(1).is_empty());
        assert_eq!(prime_factors(2), vec![2]);
        assert_eq!(prime_factors(360), vec![2, 2, 2, 3, 3, 5]);
        assert_eq!(prime_factors(5 * 5 * 13), vec![5, 5, 13]);
        // A large prime stays intact.
        assert_eq!(prime_factors(1_000_003), vec![1_000_003]);
    }

    #[test]
    fn prime_factors_hard_semiprime() {
        // 4 611 685 795 089 090 127 = 2 147 483 629 × 2 147 483 563 (two
        // 31-bit primes, both above the trial-division bound) — the rho path.
        let a: u64 = 2_147_483_629;
        let b: u64 = 2_147_483_563;
        let n = a * b;
        assert_eq!(prime_factors(n), vec![a.min(b), a.max(b)]);
    }

    #[test]
    fn divisors_match_definition() {
        assert_eq!(divisors(1), vec![1]);
        assert_eq!(divisors(36), vec![1, 2, 3, 4, 6, 9, 12, 18, 36]);
        assert_eq!(divisors(7), vec![1, 7]); // primes have exactly two
        assert!(divisors(0).is_empty());
        let d = divisors(960);
        assert_eq!(d.len(), 28); // 960 = 2^6·3·5 -> (6+1)(1+1)(1+1) = 28
        assert_eq!(d[0], 1);
        assert_eq!(d[27], 960);
    }

    #[test]
    fn gcd_lcm_basics() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(18, 48), 6);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(-48, 18), 6);
        assert_eq!(gcd(0, 0), 0);
        assert_eq!(lcm_checked(4, 6), Some(12));
        assert_eq!(lcm_checked(0, 5), Some(0));
        assert_eq!(lcm_checked(21, 6), Some(42));
        // 9_223_372_036_854_775_807 × 2 overflows i64.
        assert_eq!(lcm_checked(9_223_372_036_854_775_807, 2), None);
    }

    #[test]
    fn totient_matches_known_values() {
        assert_eq!(totient(1), 1);
        assert_eq!(totient(10), 4); // 1, 3, 7, 9
        assert_eq!(totient(36), 12);
        assert_eq!(totient(97), 96); // prime: φ(p) = p − 1
        assert_eq!(totient(2 * 3 * 5 * 7), 48);
    }

    #[test]
    fn sieve_counts_primes() {
        assert!(sieve(1).is_empty());
        assert_eq!(sieve(10), vec![2, 3, 5, 7]);
        assert_eq!(sieve(100).len(), 25);
        assert_eq!(sieve(100_000).len(), 9_592);
        let (primes, smallest) = sieve_grid(30);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert_eq!(smallest[&4], 2);
        assert_eq!(smallest[&9], 3);
        assert_eq!(smallest[&25], 5);
        assert!(!smallest.contains_key(&23));
    }

    // ----- combinatorics ---------------------------------------------------

    #[test]
    fn factorial_boundaries() {
        assert_eq!(factorial_checked(0), Some(1));
        assert_eq!(factorial_checked(5), Some(120));
        assert_eq!(factorial_checked(20), Some(2_432_902_008_176_640_000));
        assert_eq!(factorial_checked(21), None); // 21! overflows i64
        assert_eq!(factorial_checked(-1), None);
    }

    #[test]
    fn perm_values() {
        assert_eq!(perm_checked(5, 0), Some(1));
        assert_eq!(perm_checked(5, 5), Some(120));
        assert_eq!(perm_checked(10, 2), Some(90));
        assert_eq!(perm_checked(26, 3), Some(15_600)); // license plates
        assert_eq!(perm_checked(3, 5), None); // k > n
        // P(99, 12) ≈ 9.3 × 10^22 overflows i64.
        assert_eq!(perm_checked(99, 12), None);
    }

    #[test]
    fn comb_values() {
        assert_eq!(comb_checked(5, 0), Some(1));
        assert_eq!(comb_checked(5, 5), Some(1));
        assert_eq!(comb_checked(52, 5), Some(2_598_960)); // poker hands
        assert_eq!(comb_checked(40, 20), Some(137_846_528_820));
        assert_eq!(comb_checked(3, 5), None);
        assert_eq!(comb_checked(67, 33), None); // first binomial beyond i64
        assert_eq!(comb_checked(66, 33), Some(7_219_428_434_016_265_740));
    }

    #[test]
    fn pascal_rows_shape_and_values() {
        let rows = pascal_rows(6);
        assert_eq!(rows[0], vec![1]);
        assert_eq!(rows[1], vec![1, 1]);
        assert_eq!(rows[4], vec![1, 4, 6, 4, 1]);
        assert_eq!(rows[5], vec![1, 5, 10, 10, 5, 1]);
        // Row sums equal 2^row.
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row.iter().sum::<i64>(), 1 << i);
        }
    }
}

// ===========================================================================
// Modular arithmetic
// ===========================================================================

/// Largest exponent-free values stay in i64; products are widened to i128.
fn to_i64(v: i128) -> Option<i64> {
    i64::try_from(v).ok()
}

/// `a mod m` in `0..m` (m > 0).
pub fn modulo(a: i64, m: i64) -> i64 {
    a.rem_euclid(m)
}

/// `base^exp mod m` by square-and-multiply (m > 0, exp ≥ 0).
pub fn modpow(base: i64, exp: u64, m: i64) -> i64 {
    let m = m as i128;
    let mut result: i128 = 1 % m;
    let mut b = (base as i128).rem_euclid(m);
    let mut e = exp;
    while e > 0 {
        if e & 1 == 1 {
            result = result * b % m;
        }
        b = b * b % m;
        e >>= 1;
    }
    result as i64
}

/// Extended Euclid: `(g, x, y)` with `a·x + b·y = g = gcd(a, b) ≥ 0`.
pub fn egcd(a: i64, b: i64) -> (i64, i64, i64) {
    let (mut r0, mut r1) = (a as i128, b as i128);
    let (mut x0, mut x1) = (1i128, 0i128);
    let (mut y0, mut y1) = (0i128, 1i128);
    while r1 != 0 {
        let q = r0 / r1;
        (r0, r1) = (r1, r0 - q * r1);
        (x0, x1) = (x1, x0 - q * x1);
        (y0, y1) = (y1, y0 - q * y1);
    }
    if r0 < 0 {
        (r0, x0, y0) = (-r0, -x0, -y0);
    }
    (r0 as i64, x0 as i64, y0 as i64)
}

/// Multiplicative inverse of `a` modulo `m` (m > 1), if gcd(a, m) = 1.
pub fn modinv(a: i64, m: i64) -> Option<i64> {
    let (g, x, _) = egcd(modulo(a, m), m);
    if g != 1 {
        None
    } else {
        Some(modulo(x, m))
    }
}

/// Chinese remainder theorem for `x ≡ a1 (mod m1)`, `x ≡ a2 (mod m2)`.
/// Returns `(x, lcm)` with `0 ≤ x < lcm`. Works when the moduli share a
/// factor, as long as the two congruences agree on it.
pub fn crt(a1: i64, m1: i64, a2: i64, m2: i64) -> Result<(i64, i64), String> {
    let (g, p, _) = egcd(m1, m2);
    let diff = a2 as i128 - a1 as i128;
    if diff % g as i128 != 0 {
        return Err(format!(
            "x ≡ {} (mod {}) and x ≡ {} (mod {}) contradict each other: {} does not divide {} − {}.",
            modulo(a1, m1), m1, modulo(a2, m2), m2, g, a2, a1
        ));
    }
    let lcm = to_i64((m1 as i128 / g as i128) * m2 as i128)
        .ok_or_else(|| "The combined modulus does not fit in a 64-bit number.".to_string())?;
    let step = (m2 / g) as i128;
    let t = (diff / g as i128 * p as i128).rem_euclid(step);
    let x = (a1 as i128 + m1 as i128 * t).rem_euclid(lcm as i128);
    Ok((x as i64, lcm))
}

// ===========================================================================
// More counting
// ===========================================================================

pub const FIB_MAX_N: i64 = 92;
pub const CATALAN_MAX_N: i64 = 35;
pub const DERANGE_MAX_N: i64 = 20;
pub const STIRLING_MAX_N: i64 = 25;

/// Fibonacci with F(0) = 0, F(1) = 1.
pub fn fib(n: i64) -> i64 {
    let (mut a, mut b) = (0i128, 1i128);
    for _ in 0..n {
        (a, b) = (b, a + b);
    }
    a as i64
}

/// Catalan number C(n) = C(2n, n) / (n + 1), or `None` on overflow.
pub fn catalan_checked(n: i64) -> Option<i64> {
    let mut c: i128 = 1;
    for k in 0..n as i128 {
        c = c * 2 * (2 * k + 1) / (k + 2);
    }
    to_i64(c)
}

/// Derangements D(n): permutations with no fixed point. D(n) = (n−1)(D(n−1) + D(n−2)).
pub fn derange_checked(n: i64) -> Option<i64> {
    if n == 0 {
        return Some(1);
    }
    let (mut a, mut b) = (1i128, 0i128); // D(0), D(1)
    for k in 2..=n as i128 {
        (a, b) = (b, (k - 1) * (a + b));
    }
    to_i64(b)
}

/// Stirling numbers of the second kind S(n, k): ways to split n items into
/// k non-empty groups. S(n, k) = k·S(n−1, k) + S(n−1, k−1).
pub fn stirling2_checked(n: i64, k: i64) -> Option<i64> {
    let (n, k) = (n as usize, k as usize);
    let mut row = vec![0i128; k + 1];
    row[0] = 1; // S(0, 0)
    for i in 1..=n {
        for j in (0..=k).rev() {
            let prev = if j > 0 { row[j - 1] } else { 0 };
            row[j] = if j == 0 { 0 } else { j as i128 * row[j] + prev };
        }
        let _ = i;
    }
    to_i64(row[k])
}

// ===========================================================================
// More graph tools
// ===========================================================================

/// Topological order of a digraph (Kahn's algorithm, always taking the
/// smallest available vertex so the result is repeatable). `Err` holds the
/// vertices that sit on, or behind, a cycle.
pub fn topo_sort(g: &GraphVal) -> Result<Vec<i64>, Vec<i64>> {
    let mut indeg: BTreeMap<i64, usize> = g.vertices.iter().map(|&v| (v, 0)).collect();
    let adj = adjacency(g, false);
    for e in &g.edges {
        *indeg.entry(e.to).or_insert(0) += 1;
    }
    let mut ready: BTreeSet<i64> = indeg.iter().filter(|(_, &d)| d == 0).map(|(&v, _)| v).collect();
    let mut order = Vec::new();
    while let Some(&v) = ready.iter().next() {
        ready.remove(&v);
        order.push(v);
        if let Some(next) = adj.get(&v) {
            for &w in next {
                let d = indeg.get_mut(&w).unwrap();
                *d -= 1;
                if *d == 0 {
                    ready.insert(w);
                }
            }
        }
    }
    if order.len() == g.vertices.len() {
        Ok(order)
    } else {
        let done: BTreeSet<i64> = order.into_iter().collect();
        Err(g.vertices.iter().copied().filter(|v| !done.contains(v)).collect())
    }
}

/// Connected components (directions ignored): vertex → component number,
/// numbered 1, 2, … by each component's smallest vertex.
pub fn components(g: &GraphVal) -> BTreeMap<i64, i64> {
    let adj = adjacency(g, true);
    let mut label: BTreeMap<i64, i64> = BTreeMap::new();
    let mut next = 0i64;
    for &v in &g.vertices {
        if label.contains_key(&v) {
            continue;
        }
        next += 1;
        for u in bfs_adj(&adj, v) {
            label.insert(u, next);
        }
    }
    label
}

/// Has an Euler circuit: every edge can be walked exactly once, ending where
/// it began. Needs all edges in one component and balanced degrees.
pub fn is_eulerian(g: &GraphVal) -> bool {
    if g.edges.is_empty() {
        return true;
    }
    let comp = components(g);
    let first = comp[&g.edges[0].from];
    if g.edges.iter().any(|e| comp[&e.from] != first) {
        return false;
    }
    if g.directed {
        let mut balance: BTreeMap<i64, i64> = BTreeMap::new();
        for e in &g.edges {
            *balance.entry(e.from).or_insert(0) += 1;
            *balance.entry(e.to).or_insert(0) -= 1;
        }
        balance.values().all(|&b| b == 0)
    } else {
        g.vertices.iter().all(|&v| degree(g, v) % 2 == 0)
    }
}

/// Vertex degrees, largest first.
pub fn degree_sequence(g: &GraphVal) -> Vec<i64> {
    let mut d: Vec<i64> = g.vertices.iter().map(|&v| degree(g, v)).collect();
    d.sort_unstable_by(|a, b| b.cmp(a));
    d
}

// ===========================================================================
// Graph structure: complement, diameter, bridges, cut vertices, Hamiltonicity,
// chromatic number
// ===========================================================================

/// Largest vertex count IS_HAMILTONIAN will search (bitmask DP: 2^n · n).
pub const MAX_HAMILTONIAN_VERTICES: usize = 20;
/// Work budget for the exact chromatic-number search.
const CHROMATIC_BUDGET: u64 = 5_000_000;
/// Largest graph BRIDGES / CUT_VERTICES examine (vertices + edges).
pub const MAX_STRUCTURE_SIZE: usize = 3_000;

/// The complement graph: same vertices, an edge exactly where `g` has none
/// (no self-loops, weights dropped; directed graphs complement the arcs).
pub fn graph_complement(g: &GraphVal) -> GraphVal {
    let present: BTreeSet<(i64, i64)> = g
        .edges
        .iter()
        .map(|e| if g.directed { (e.from, e.to) } else { ukey(e.from, e.to) })
        .collect();
    let mut edges = Vec::new();
    for &a in &g.vertices {
        for &b in &g.vertices {
            if a == b || (!g.directed && a > b) {
                continue;
            }
            if !present.contains(&(a, b)) {
                edges.push(Edge { from: a, to: b, weight: None });
            }
        }
    }
    GraphVal { directed: g.directed, vertices: g.vertices.clone(), edges, origin: Some(GraphOrigin::Literal) }
}

/// Longest shortest path, counted in edges. `Err` says why it is undefined.
pub fn diameter(g: &GraphVal) -> Result<i64, &'static str> {
    if g.vertices.is_empty() {
        return Err("the graph has no vertices");
    }
    let adj = adjacency(g, false);
    let mut best = 0i64;
    for &s in &g.vertices {
        let mut dist: BTreeMap<i64, i64> = BTreeMap::new();
        dist.insert(s, 0);
        let mut q = VecDeque::from([s]);
        while let Some(v) = q.pop_front() {
            let d = dist[&v];
            for &w in &adj[&v] {
                if !dist.contains_key(&w) {
                    dist.insert(w, d + 1);
                    q.push_back(w);
                }
            }
        }
        if dist.len() != g.vertices.len() {
            return Err(if g.directed {
                "some vertex cannot reach another (the digraph is not strongly connected)"
            } else {
                "the graph is not connected"
            });
        }
        best = best.max(dist.values().copied().max().unwrap_or(0));
    }
    Ok(best)
}

fn components_without(g: &GraphVal, skip_vertex: Option<i64>, skip_edge: Option<(i64, i64)>) -> usize {
    let mut adj: BTreeMap<i64, BTreeSet<i64>> = BTreeMap::new();
    for &v in &g.vertices {
        if Some(v) != skip_vertex {
            adj.insert(v, BTreeSet::new());
        }
    }
    for e in &g.edges {
        if Some(e.from) == skip_vertex || Some(e.to) == skip_vertex || e.from == e.to {
            continue;
        }
        if skip_edge == Some(ukey(e.from, e.to)) {
            continue;
        }
        adj.entry(e.from).or_default().insert(e.to);
        adj.entry(e.to).or_default().insert(e.from);
    }
    let mut seen: BTreeSet<i64> = BTreeSet::new();
    let mut count = 0;
    for &v in adj.keys() {
        if seen.contains(&v) {
            continue;
        }
        count += 1;
        for u in bfs_adj(&adj, v) {
            seen.insert(u);
        }
    }
    count
}

/// Edges whose removal splits a component (undirected graphs), as `(a, b)`
/// with `a < b`, in ascending order.
pub fn bridges(g: &GraphVal) -> Vec<(i64, i64)> {
    let base = components_without(g, None, None);
    let mut out: Vec<(i64, i64)> = g
        .edges
        .iter()
        .filter(|e| e.from != e.to)
        .map(|e| ukey(e.from, e.to))
        .filter(|&k| components_without(g, None, Some(k)) > base)
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// Vertices whose removal splits a component (undirected graphs).
pub fn cut_vertices(g: &GraphVal) -> Vec<i64> {
    let base = components_without(g, None, None);
    g.vertices
        .iter()
        .copied()
        .filter(|&v| {
            // Removing an isolated vertex lowers the count; a cut vertex raises it
            // relative to the count with that vertex's own component kept whole.
            let others = components_without(g, Some(v), None);
            let isolated = adjacency(g, true)[&v].iter().all(|&w| w == v);
            let expected = if isolated { base - 1 } else { base };
            others > expected
        })
        .collect()
}

/// Does the graph contain a cycle through every vertex exactly once?
/// The caller has checked `vertices.len() <= MAX_HAMILTONIAN_VERTICES`.
pub fn is_hamiltonian(g: &GraphVal) -> bool {
    let n = g.vertices.len();
    if n < if g.directed { 2 } else { 3 } {
        return false;
    }
    let idx: BTreeMap<i64, usize> = g.vertices.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let adj = adjacency(g, false);
    let mut out_mask = vec![0u32; n];
    for (&v, ns) in &adj {
        for &w in ns {
            if w != v {
                out_mask[idx[&v]] |= 1 << idx[&w];
            }
        }
    }
    // dp[mask] = set of end vertices of a path that starts at vertex 0 and visits `mask`.
    let full = (1usize << n) - 1;
    let mut dp = vec![0u32; 1 << n];
    dp[1] = 1;
    for mask in 1..=full {
        if mask & 1 == 0 || dp[mask] == 0 {
            continue;
        }
        for last in 0..n {
            if dp[mask] & (1 << last) == 0 {
                continue;
            }
            let mut nexts = out_mask[last] & !(mask as u32);
            while nexts != 0 {
                let w = nexts.trailing_zeros() as usize;
                nexts &= nexts - 1;
                dp[mask | (1 << w)] |= 1 << w;
            }
        }
    }
    (0..n).any(|last| dp[full] & (1 << last) != 0 && out_mask[last] & 1 != 0)
}

/// Smallest k such that the graph (directions ignored) is k-colorable;
/// `None` when the search budget runs out.
pub fn chromatic_number(g: &GraphVal) -> Option<i64> {
    if g.vertices.is_empty() {
        return Some(0);
    }
    let adj = adjacency(g, true);
    let mut order: Vec<i64> = g.vertices.iter().copied().collect();
    order.sort_by_key(|v| std::cmp::Reverse(adj[v].len()));
    let mut budget = CHROMATIC_BUDGET;
    for k in 1..=order.len() as i64 {
        let mut colors: BTreeMap<i64, i64> = BTreeMap::new();
        match try_color(&order, 0, k, &adj, &mut colors, &mut budget) {
            Some(true) => return Some(k),
            Some(false) => continue,
            None => return None,
        }
    }
    Some(order.len() as i64)
}

fn try_color(
    order: &[i64],
    i: usize,
    k: i64,
    adj: &BTreeMap<i64, BTreeSet<i64>>,
    colors: &mut BTreeMap<i64, i64>,
    budget: &mut u64,
) -> Option<bool> {
    if i == order.len() {
        return Some(true);
    }
    let v = order[i];
    let used: BTreeSet<i64> = adj[&v].iter().filter_map(|w| colors.get(w).copied()).collect();
    // Symmetry breaking: never open more than one new color at a time.
    let max_used = colors.values().copied().max().map(|m| m + 1).unwrap_or(0);
    for c in 0..k.min(max_used + 1) {
        if used.contains(&c) {
            continue;
        }
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        colors.insert(v, c);
        match try_color(order, i + 1, k, adj, colors, budget) {
            Some(true) => return Some(true),
            None => return None,
            Some(false) => {}
        }
        colors.remove(&v);
    }
    Some(false)
}

// ===========================================================================
// Number theory and counting: sigma, order, bases, partitions, Bell numbers
// ===========================================================================

/// Sum of the positive divisors of `n` (σ(n)); `None` on overflow.
pub fn sigma(n: i64) -> Option<i64> {
    let mut total: i128 = 0;
    for d in divisors(n) {
        total += d as i128;
    }
    to_i64(total)
}

/// Multiplicative order of `a` modulo `m` (m ≥ 2, gcd(a, m) = 1): the least
/// k ≥ 1 with a^k ≡ 1 (mod m). Found by shrinking φ(m) one prime at a time.
pub fn mult_order(a: i64, m: i64) -> i64 {
    let a = modulo(a, m);
    let mut k = totient(m as u64);
    for p in group_primes(&prime_factors(k)) {
        while k % p == 0 && modpow(a, k / p, m) == 1 {
            k /= p;
        }
    }
    k as i64
}

fn group_primes(f: &[u64]) -> Vec<u64> {
    let mut v = f.to_vec();
    v.dedup();
    v
}

/// Digits of `n` (≥ 0) in `base` (2..=36), most significant first.
pub fn to_base(n: i64, base: i64) -> Vec<i64> {
    if n == 0 {
        return vec![0];
    }
    let mut digits = Vec::new();
    let mut x = n;
    while x > 0 {
        digits.push(x % base);
        x /= base;
    }
    digits.reverse();
    digits
}

/// Value of the digits (most significant first) in `base`; `None` on overflow.
pub fn from_base(digits: &[i64], base: i64) -> Option<i64> {
    let mut acc: i128 = 0;
    for &d in digits {
        acc = acc * base as i128 + d as i128;
        if acc > i64::MAX as i128 {
            return None;
        }
    }
    to_i64(acc)
}

/// Number of integer partitions p(n); `None` on overflow.
pub fn partitions_checked(n: usize) -> Option<i64> {
    let mut p = vec![0i128; n + 1];
    p[0] = 1;
    for part in 1..=n {
        for s in part..=n {
            p[s] += p[s - part];
            if p[s] > i64::MAX as i128 {
                return None;
            }
        }
    }
    to_i64(p[n])
}

/// Bell number B(n) (set partitions) by the Bell triangle; `None` on overflow.
pub fn bell_checked(n: usize) -> Option<i64> {
    if n > 40 {
        return None; // the triangle's rows would overflow i128
    }
    let mut row: Vec<i128> = vec![1];
    for _ in 0..n {
        let mut next: Vec<i128> = vec![*row.last().unwrap()];
        for &x in &row {
            let v = next.last().unwrap() + x;
            next.push(v);
        }
        row = next;
    }
    to_i64(row[0])
}
