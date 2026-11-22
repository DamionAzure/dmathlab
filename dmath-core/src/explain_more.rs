//! Worked solutions (`VISUALIZE X AS STEPS`) for set operators, POWERSET,
//! inclusion–exclusion (`SIZE` of a union), TOPO_SORT and the relation
//! closures.
//!
//! Same contract as `explain.rs`: each section that reaches its own answer
//! records it, and `finish` checks it against what the real engine computed.

use std::collections::{BTreeMap, BTreeSet};

use crate::algo;
use crate::ast::BinOp;
use crate::explain::{finish, Sec, Val};
use crate::ir::StepsIr;
use crate::token::Builtin;
use crate::value::{set_to_string, GraphVal, SetElem};

type Elems = BTreeSet<SetElem>;

/// What a `STEPS` request on one of these calls needs, captured when the
/// statement (or the assignment behind a variable) ran.
#[derive(Debug, Clone)]
pub enum MoreStep {
    /// `A UNION B`, `A INTERSECT B`, `A DIFFERENCE B`, `A SYMDIFF B`.
    SetOp { op: BinOp, a: Elems, b: Elems, a_text: String, b_text: String },
    Power { a: Elems },
    /// `SIZE(A UNION B UNION …)`: the operands with their source text.
    InclExcl { sets: Vec<(String, Elems)> },
    Topo { g: GraphVal },
    Closure { func: Builtin, pairs: Vec<(i64, i64)> },
}

pub const MAX_INCL_EXCL_SETS: usize = 5;
const TABLE_CAP: usize = 40;

pub fn explain_more(step: &MoreStep, expression: String) -> Result<StepsIr, String> {
    match step {
        MoreStep::SetOp { op, a, b, a_text, b_text } => Ok(set_op(*op, a, b, a_text, b_text, expression)),
        MoreStep::Power { a } => Ok(power(a, expression)),
        MoreStep::InclExcl { sets } => incl_excl(sets, expression),
        MoreStep::Topo { g } => topo(g, expression),
        MoreStep::Closure { func, pairs } => closure(*func, pairs, expression),
    }
}

fn txt(s: &Elems) -> String {
    set_to_string(s)
}

fn label(s: &str) -> String {
    crate::pretty::ellipsize(s, 14)
}

fn yn(b: bool) -> String {
    if b { "✓" } else { "·" }.to_string()
}

// ───────────────────────── set operators ─────────────────────────

fn set_op(op: BinOp, a: &Elems, b: &Elems, at: &str, bt: &str, expression: String) -> StepsIr {
    let rule = |in_a: bool, in_b: bool| match op {
        BinOp::Union => in_a || in_b,
        BinOp::Intersect => in_a && in_b,
        BinOp::Difference => in_a && !in_b,
        _ => in_a != in_b, // SymDiff
    };
    let (rule_text, name) = match op {
        BinOp::Union => ("in the result when it is in A or in B (or both)", "Union"),
        BinOp::Intersect => ("in the result when it is in both A and B", "Intersection"),
        BinOp::Difference => ("in the result when it is in A but not in B", "Difference"),
        _ => ("in the result when it is in exactly one of A and B", "Symmetric difference"),
    };
    let engine_set: Elems = match op {
        BinOp::Union => a | b,
        BinOp::Intersect => a & b,
        BinOp::Difference => a - b,
        _ => a ^ b,
    };
    let engine = Val::Text(txt(&engine_set));

    let mut secs = Vec::new();
    let mut note = Sec::new("note", "The two sets").intro(format!(
        "{} {} {} — each element is {}.",
        label(at),
        op.symbol(),
        label(bt),
        rule_text
    ));
    note.line(format!("A = {}   (|A| = {})", txt(a), a.len()));
    note.line(format!("B = {}   (|B| = {})", txt(b), b.len()));
    secs.push(note);

    // Method 1: decide every element on its own.
    let all: Elems = a | b;
    let mut rows = Vec::new();
    let mut built = Elems::new();
    for x in &all {
        let (ia, ib) = (a.contains(x), b.contains(x));
        let keep = rule(ia, ib);
        if keep {
            built.insert(x.clone());
        }
        if rows.len() < TABLE_CAP {
            rows.push(vec![x.to_string(), yn(ia), yn(ib), yn(keep)]);
        }
    }
    let mut m1 = Sec::new("listing", "Method 1 — decide each element");
    m1.table(&["element", "in A", "in B", "in result"], rows);
    if all.len() > TABLE_CAP {
        m1.line(format!("(showing the first {} of {} elements)", TABLE_CAP, all.len()));
    }
    m1.conclude(format!("Result = {}", txt(&built)), Val::Text(txt(&built)));
    secs.push(m1);

    // Method 2: an identity built from other operations.
    let diff = |x: &Elems, y: &Elems| -> Elems { x.iter().filter(|e| !y.contains(*e)).cloned().collect() };
    let meet: Elems = a.iter().filter(|e| b.contains(*e)).cloned().collect();
    let (title, lines, result): (&str, Vec<String>, Elems) = match op {
        BinOp::Union => {
            let extra = diff(b, a);
            (
                "Method 2 — A ∪ (B ∖ A)",
                vec![format!("B ∖ A = {}  (what B adds)", txt(&extra)), format!("A ∪ (B ∖ A) = {}", txt(&(a.iter().cloned().chain(extra.iter().cloned()).collect::<Elems>())))],
                a.iter().cloned().chain(extra.into_iter()).collect(),
            )
        }
        BinOp::Intersect => {
            let out = diff(a, &diff(a, b));
            ("Method 2 — A ∖ (A ∖ B)", vec![format!("A ∖ B = {}", txt(&diff(a, b))), format!("A ∖ (A ∖ B) = {}", txt(&out))], out)
        }
        BinOp::Difference => {
            let out = diff(a, &meet);
            ("Method 2 — A ∖ (A ∩ B)", vec![format!("A ∩ B = {}", txt(&meet)), format!("A ∖ (A ∩ B) = {}", txt(&out))], out)
        }
        _ => {
            let (l, r) = (diff(a, b), diff(b, a));
            let out: Elems = l.iter().cloned().chain(r.iter().cloned()).collect();
            ("Method 2 — (A ∖ B) ∪ (B ∖ A)", vec![format!("A ∖ B = {}", txt(&l)), format!("B ∖ A = {}", txt(&r)), format!("their union = {}", txt(&out))], out)
        }
    };
    let mut m2 = Sec::new("formula", title);
    for l in lines {
        m2.line(l);
    }
    m2.conclude(format!("Result = {}", txt(&result)), Val::Text(txt(&result)));
    secs.push(m2);

    // Size check.
    let (n, m, k) = (a.len() as i64, b.len() as i64, meet.len() as i64);
    let (formula, expect) = match op {
        BinOp::Union => (format!("|A| + |B| − |A ∩ B| = {} + {} − {}", n, m, k), n + m - k),
        BinOp::Intersect => (format!("|A ∩ B| = {}", k), k),
        BinOp::Difference => (format!("|A| − |A ∩ B| = {} − {}", n, k), n - k),
        _ => (format!("|A| + |B| − 2·|A ∩ B| = {} + {} − 2·{}", n, m, k), n + m - 2 * k),
    };
    let mut size = Sec::new("verify", "Size check");
    if expect == engine_set.len() as i64 {
        size.good(format!("{} = {} = |result| ✓", formula, expect));
    } else {
        size.bad(format!("{} = {} but |result| = {}", formula, expect, engine_set.len()));
    }
    secs.push(size);

    finish(op.symbol(), expression, name.to_string(), engine, secs)
}

// ───────────────────────── power set ─────────────────────────

fn power(a: &Elems, expression: String) -> StepsIr {
    let engine_set = algo::powerset(a);
    let engine = Val::Text(txt(&engine_set));
    let items: Vec<&SetElem> = a.iter().collect();
    let n = items.len();

    let mut secs = Vec::new();
    let mut note = Sec::new("note", "The set").intro("A set with n elements has 2ⁿ subsets: each element is either in a subset or out.".to_string());
    note.line(format!("A = {}   (n = {})", txt(a), n));
    note.line(format!("2^{} = {} subsets expected", n, 1u64 << n));
    secs.push(note);

    // Method 1: doubling.
    let mut subsets: Vec<Vec<SetElem>> = vec![vec![]];
    let mut rows = vec![vec!["start".to_string(), "{}".to_string(), "1".to_string()]];
    for x in &items {
        let added: Vec<Vec<SetElem>> = subsets
            .iter()
            .map(|s| {
                let mut t = s.clone();
                t.push((*x).clone());
                t
            })
            .collect();
        let preview: Vec<String> = added.iter().take(4).map(|s| txt(&s.iter().cloned().collect())).collect();
        subsets.extend(added.clone());
        rows.push(vec![
            format!("add {}", x),
            format!("{}{}", preview.join(", "), if added.len() > 4 { ", …" } else { "" }),
            subsets.len().to_string(),
        ]);
    }
    let built: Elems = subsets.iter().map(|s| SetElem::Set(s.clone())).collect();
    let mut m1 = Sec::new("rounds", "Method 1 — doubling").intro("Start with {}. For each element, copy every subset so far and add the element to the copies.".to_string());
    m1.table(&["step", "new subsets (first few)", "total"], rows);
    m1.conclude(format!("{} subsets", built.len()), Val::Text(txt(&built)));
    secs.push(m1);

    // Method 2: binary counting.
    let mut built2 = Elems::new();
    let mut rows2 = Vec::new();
    for mask in 0u32..(1u32 << n) {
        let sub: Vec<SetElem> = (0..n).filter(|i| mask >> i & 1 == 1).map(|i| items[i].clone()).collect();
        if rows2.len() < 16 {
            let bits: String = (0..n).rev().map(|i| if mask >> i & 1 == 1 { '1' } else { '0' }).collect();
            rows2.push(vec![bits, txt(&sub.iter().cloned().collect())]);
        }
        built2.insert(SetElem::Set(sub));
    }
    let mut m2 = Sec::new("listing", "Method 2 — binary counting").intro("Count from 0 to 2ⁿ − 1; bit i says whether the i-th element is in the subset.".to_string());
    m2.table(&["bits", "subset"], rows2);
    if n > 4 {
        m2.line(format!("(showing the first 16 of {} rows)", 1u64 << n));
    }
    m2.conclude(format!("{} subsets", built2.len()), Val::Text(txt(&built2)));
    secs.push(m2);

    let mut chk = Sec::new("verify", "Count check");
    if engine_set.len() == 1usize << n {
        chk.good(format!("|P(A)| = {} = 2^{} ✓", engine_set.len(), n));
    } else {
        chk.bad(format!("|P(A)| = {} but 2^{} = {}", engine_set.len(), n, 1u64 << n));
    }
    secs.push(chk);

    finish("POWERSET", expression, "Power set".to_string(), engine, secs)
}

// ───────────────────────── inclusion–exclusion ─────────────────────────

fn incl_excl(sets: &[(String, Elems)], expression: String) -> Result<StepsIr, String> {
    let k = sets.len();
    if !(2..=MAX_INCL_EXCL_SETS).contains(&k) {
        return Err(format!(
            "Inclusion–exclusion walks through 2 to {} sets, but this union has {}.",
            MAX_INCL_EXCL_SETS, k
        ));
    }
    let union: Elems = sets.iter().flat_map(|(_, s)| s.iter().cloned()).collect();
    let engine = Val::Num(union.len() as i64);

    let mut secs = Vec::new();
    let mut note = Sec::new("note", "The sets").intro("Adding the sizes counts shared elements more than once, so each overlap is subtracted and each triple overlap added back, and so on.".to_string());
    for (name, s) in sets {
        note.line(format!("{} = {}   (|{}| = {})", label(name), txt(s), label(name), s.len()));
    }
    secs.push(note);

    let mut total: i64 = 0;
    let mut rows = Vec::new();
    let mut formula = Sec::new("formula", "Method 1 — inclusion–exclusion");
    for j in 1..=k {
        let sign = if j % 2 == 1 { 1 } else { -1 };
        let mut subtotal: i64 = 0;
        let mut parts = Vec::new();
        for mask in 1u32..(1u32 << k) {
            if mask.count_ones() as usize != j {
                continue;
            }
            let idx: Vec<usize> = (0..k).filter(|i| mask >> i & 1 == 1).collect();
            let mut inter: Elems = sets[idx[0]].1.clone();
            for &i in &idx[1..] {
                inter = inter.iter().filter(|e| sets[i].1.contains(*e)).cloned().collect();
            }
            let names: Vec<String> = idx.iter().map(|&i| label(&sets[i].0)).collect();
            parts.push(inter.len() as i64);
            rows.push(vec![
                format!("|{}|", names.join(" ∩ ")),
                j.to_string(),
                inter.len().to_string(),
                if sign > 0 { "+".to_string() } else { "−".to_string() },
            ]);
            subtotal += inter.len() as i64;
        }
        total += sign * subtotal;
        formula.line(format!(
            "{}-way overlaps: {} = {}  → {}{}",
            j,
            parts.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" + "),
            subtotal,
            if sign > 0 { "add " } else { "subtract " },
            subtotal
        ));
    }
    formula.table(&["term", "sets", "size", "sign"], rows);
    formula.conclude(format!("Total = {}", total), Val::Num(total));
    secs.push(formula);

    let mut direct = Sec::new("listing", "Method 2 — count the union directly");
    direct.line(format!("Union = {}", txt(&union)));
    direct.conclude(format!("|union| = {}", union.len()), Val::Num(union.len() as i64));
    secs.push(direct);

    Ok(finish("SIZE", expression, "Inclusion–exclusion".to_string(), engine, secs))
}

// ───────────────────────── topological sort ─────────────────────────

fn topo(g: &GraphVal, expression: String) -> Result<StepsIr, String> {
    if !g.directed {
        return Err("TOPO_SORT needs a directed graph.".to_string());
    }
    let order = algo::topo_sort(g).map_err(|stuck| {
        format!("The graph has a cycle (vertices on or behind it: {:?}), so there is no order to narrate.", stuck)
    })?;
    let engine = Val::List(order.clone());

    let mut secs = Vec::new();
    let mut note = Sec::new("note", "The digraph").intro("A topological order lists every vertex before the vertices it points to.".to_string());
    note.line(format!("Vertices: {}", join(g.vertices.iter().copied())));
    let edges: Vec<String> = g.edges.iter().map(|e| format!("{}→{}", e.from, e.to)).collect();
    note.line(format!("Arrows: {}", if edges.is_empty() { "none".to_string() } else { edges.join(", ") }));
    secs.push(note);

    // Kahn's algorithm, smallest available vertex first.
    let mut indeg: BTreeMap<i64, usize> = g.vertices.iter().map(|&v| (v, 0)).collect();
    let mut out: BTreeMap<i64, BTreeSet<i64>> = g.vertices.iter().map(|&v| (v, BTreeSet::new())).collect();
    for e in &g.edges {
        if out.get_mut(&e.from).unwrap().insert(e.to) {
            *indeg.get_mut(&e.to).unwrap() += 1;
        }
    }
    let mut kahn = Sec::new("rounds", "Method 1 — Kahn's algorithm").intro(format!(
        "In-degrees: {}. Repeatedly take the smallest vertex with in-degree 0, then remove its arrows.",
        indeg.iter().map(|(v, d)| format!("{}:{}", v, d)).collect::<Vec<_>>().join(", ")
    ));
    let mut ready: BTreeSet<i64> = indeg.iter().filter(|(_, &d)| d == 0).map(|(&v, _)| v).collect();
    let mut rows = Vec::new();
    let mut result = Vec::new();
    while let Some(&v) = ready.iter().next() {
        let available = join(ready.iter().copied());
        ready.remove(&v);
        result.push(v);
        let mut lowered = Vec::new();
        for &w in &out[&v] {
            let d = indeg.get_mut(&w).unwrap();
            *d -= 1;
            lowered.push(format!("{} → {}", w, d));
            if *d == 0 {
                ready.insert(w);
            }
        }
        rows.push(vec![
            (result.len()).to_string(),
            available,
            v.to_string(),
            if lowered.is_empty() { "—".to_string() } else { lowered.join(", ") },
        ]);
    }
    kahn.table(&["step", "available (in-degree 0)", "pick", "new in-degrees"], rows);
    kahn.conclude(format!("Order: {}", join(result.iter().copied())), Val::List(result.clone()));
    secs.push(kahn);

    // Method 2: verify every arrow goes forward.
    let pos: BTreeMap<i64, usize> = result.iter().enumerate().map(|(i, &v)| (v, i)).collect();
    let mut chk = Sec::new("verify", "Method 2 — every arrow points forward");
    let mut ok = true;
    for e in &g.edges {
        let (pa, pb) = (pos[&e.from], pos[&e.to]);
        if pa < pb {
            chk.good(format!("{}→{}: {} comes at position {}, before {} at {} ✓", e.from, e.to, e.from, pa + 1, e.to, pb + 1));
        } else {
            ok = false;
            chk.bad(format!("{}→{}: {} comes at position {}, not before {} at {}", e.from, e.to, e.from, pa + 1, e.to, pb + 1));
        }
    }
    if g.edges.is_empty() {
        chk.line("No arrows, so any order works; the smallest-first order is used.");
    }
    chk.conclude(
        if ok { "The order respects every arrow".to_string() } else { "Some arrow points backwards".to_string() },
        if ok { Val::List(result) } else { Val::List(vec![]) },
    );
    secs.push(chk);

    Ok(finish("TOPO_SORT", expression, "Topological sort".to_string(), engine, secs))
}

fn join(it: impl IntoIterator<Item = i64>) -> String {
    let v: Vec<String> = it.into_iter().map(|x| x.to_string()).collect();
    if v.is_empty() { "∅".to_string() } else { v.join(", ") }
}

// ───────────────────────── relation closures ─────────────────────────

fn pairs_text(p: &BTreeSet<(i64, i64)>) -> String {
    let s: Elems = p.iter().map(|&(a, b)| SetElem::Tuple(vec![SetElem::Num(a), SetElem::Num(b)])).collect();
    txt(&s)
}

fn pair_list(p: &[(i64, i64)]) -> String {
    if p.is_empty() {
        return "none".to_string();
    }
    p.iter().map(|(a, b)| format!("({}, {})", a, b)).collect::<Vec<_>>().join(", ")
}

fn closure(func: Builtin, pairs: &[(i64, i64)], expression: String) -> Result<StepsIr, String> {
    let r: BTreeSet<(i64, i64)> = pairs.iter().copied().collect();
    let field = algo::field(pairs);
    let (name, title, engine_set) = match func {
        Builtin::TransitiveClosure => ("TRANSITIVE_CLOSURE", "Transitive closure", algo::transitive_closure(pairs)),
        Builtin::ReflexiveClosure => ("REFLEXIVE_CLOSURE", "Reflexive closure", algo::reflexive_closure(pairs)),
        Builtin::SymmetricClosure => ("SYMMETRIC_CLOSURE", "Symmetric closure", algo::symmetric_closure(pairs)),
        other => return Err(format!("{} has no step-by-step view.", other.name())),
    };
    let engine = Val::Text(pairs_text(&engine_set));
    let goal = match func {
        Builtin::TransitiveClosure => "the smallest transitive relation containing R: whenever (a, b) and (b, c) hold, (a, c) must too",
        Builtin::ReflexiveClosure => "the smallest reflexive relation containing R: every element must be related to itself",
        _ => "the smallest symmetric relation containing R: whenever (a, b) holds, (b, a) must too",
    };
    let mut secs = Vec::new();
    let mut note = Sec::new("note", "The relation").intro(format!("The closure is {}.", goal));
    note.line(format!("R = {}", pairs_text(&r)));
    note.line(format!("Elements: {}", join(field.iter().copied())));
    secs.push(note);

    match func {
        Builtin::TransitiveClosure => {
            let mut cur = r.clone();
            let mut rows = Vec::new();
            for &k in &field {
                let mut added = Vec::new();
                for &i in &field {
                    if !cur.contains(&(i, k)) {
                        continue;
                    }
                    for &j in &field {
                        if cur.contains(&(k, j)) && cur.insert((i, j)) {
                            added.push((i, j));
                        }
                    }
                }
                added.sort_unstable();
                rows.push(vec![k.to_string(), pair_list(&added), cur.len().to_string()]);
            }
            let mut m1 = Sec::new("rounds", "Method 1 — Warshall's algorithm")
                .intro("For each element k in turn, allow paths that pass through k: if (i, k) and (k, j) are in, add (i, j).".to_string());
            m1.table(&["k", "pairs added", "total pairs"], rows);
            m1.conclude(format!("{} pairs", cur.len()), Val::Text(pairs_text(&cur)));
            secs.push(m1);

            let mut m2 = Sec::new("formula", "Method 2 — keep composing")
                .intro("R ∪ R² ∪ R³ … (R² = R∘R means two steps), stopping when nothing new appears.".to_string());
            let mut acc = r.clone();
            let mut power: Vec<(i64, i64)> = pairs.to_vec();
            let mut round = 1;
            loop {
                round += 1;
                let next = algo::compose(&power, pairs);
                let new: Vec<(i64, i64)> = next.iter().copied().filter(|p| !acc.contains(p)).collect();
                m2.line(format!("R^{} adds {}", round, pair_list(&new)));
                if new.is_empty() {
                    break;
                }
                acc.extend(new);
                power = next.into_iter().collect();
                if round > field.len() + 1 {
                    break;
                }
            }
            m2.conclude(format!("{} pairs", acc.len()), Val::Text(pairs_text(&acc)));
            secs.push(m2);
        }
        Builtin::ReflexiveClosure => {
            let missing: Vec<(i64, i64)> = field.iter().map(|&x| (x, x)).filter(|p| !r.contains(p)).collect();
            let rows: Vec<Vec<String>> = field
                .iter()
                .map(|&x| {
                    let has = r.contains(&(x, x));
                    vec![x.to_string(), format!("({}, {})", x, x), if has { "already in R".to_string() } else { "add".to_string() }]
                })
                .collect();
            let mut m1 = Sec::new("listing", "Method 1 — check each element");
            m1.table(&["element", "needs", "status"], rows);
            let mut built = r.clone();
            built.extend(missing.iter().copied());
            m1.conclude(format!("Adds {}", pair_list(&missing)), Val::Text(pairs_text(&built)));
            secs.push(m1);

            let ident: BTreeSet<(i64, i64)> = field.iter().map(|&x| (x, x)).collect();
            let union: BTreeSet<(i64, i64)> = r.union(&ident).copied().collect();
            let mut m2 = Sec::new("formula", "Method 2 — R ∪ identity");
            m2.line(format!("Identity on the elements = {}", pairs_text(&ident)));
            m2.conclude(format!("{} pairs", union.len()), Val::Text(pairs_text(&union)));
            secs.push(m2);
        }
        _ => {
            let missing: Vec<(i64, i64)> = r.iter().filter(|&&(a, b)| !r.contains(&(b, a))).map(|&(a, b)| (b, a)).collect();
            let rows: Vec<Vec<String>> = r
                .iter()
                .map(|&(a, b)| {
                    vec![format!("({}, {})", a, b), format!("({}, {})", b, a), if r.contains(&(b, a)) { "already in R".to_string() } else { "add".to_string() }]
                })
                .take(TABLE_CAP)
                .collect();
            let mut m1 = Sec::new("listing", "Method 1 — mirror every pair");
            m1.table(&["pair", "its mirror", "status"], rows);
            let mut built = r.clone();
            built.extend(missing.iter().copied());
            m1.conclude(format!("Adds {}", pair_list(&{ let mut m = missing.clone(); m.sort_unstable(); m })), Val::Text(pairs_text(&built)));
            secs.push(m1);

            let inv = algo::inverse(pairs);
            let union: BTreeSet<(i64, i64)> = r.union(&inv).copied().collect();
            let mut m2 = Sec::new("formula", "Method 2 — R ∪ R⁻¹");
            m2.line(format!("R⁻¹ = {}", pairs_text(&inv)));
            m2.conclude(format!("{} pairs", union.len()), Val::Text(pairs_text(&union)));
            secs.push(m2);
        }
    }

    Ok(finish(name, expression, title.to_string(), engine, secs))
}
