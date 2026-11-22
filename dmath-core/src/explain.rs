//! Step-by-step solutions for the number-theory and combinatorics functions
//! (`VISUALIZE G AS STEPS`).
//!
//! Every operation is solved by several *independent* methods (for GCD: a
//! quick prime check, the Euclidean algorithm, prime factorization and
//! listing divisors). Each method that computes its own answer records it,
//! and `finish` compares every one against the engine's result, so the view
//! can show "all methods agree" — and the tests fail loudly if one ever
//! disagrees.

use crate::algo;
use crate::ir::{Step, StepSection, StepTable, StepsIr};
use crate::token::Builtin;

/// Largest number we narrate by hand with trial division (√ = 10^6 tests).
pub const TRIAL_LIMIT: u64 = 1_000_000_000_000;
const LIST_CAP: usize = 36;

/// The value a method arrives at.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Val {
    Num(i64),
    Bool(bool),
    List(Vec<i64>),
    /// A set, already formatted (used by the set / relation walk-throughs).
    Text(String),
}

impl Val {
    /// Same text as `Value::display` in the interpreter.
    pub(crate) fn show(&self) -> String {
        match self {
            Val::Num(n) => n.to_string(),
            Val::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
            Val::Text(t) => t.clone(),
            Val::List(v) => format!("[{}]", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
        }
    }
}

// ───────────────────────── section builder ─────────────────────────

pub(crate) struct Sec {
    kind: &'static str,
    title: String,
    intro: Option<String>,
    steps: Vec<Step>,
    table: Option<StepTable>,
    outcome: Option<String>,
    value: Option<Val>,
}

impl Sec {
    pub(crate) fn new(kind: &'static str, title: impl Into<String>) -> Sec {
        Sec { kind, title: title.into(), intro: None, steps: Vec::new(), table: None, outcome: None, value: None }
    }
    pub(crate) fn intro(mut self, t: impl Into<String>) -> Sec {
        self.intro = Some(t.into());
        self
    }
    pub(crate) fn tone(&mut self, tone: &'static str, t: impl Into<String>) {
        self.steps.push(Step { text: t.into(), tone });
    }
    pub(crate) fn line(&mut self, t: impl Into<String>) {
        self.tone("plain", t);
    }
    pub(crate) fn head(&mut self, t: impl Into<String>) {
        self.tone("head", t);
    }
    pub(crate) fn good(&mut self, t: impl Into<String>) {
        self.tone("good", t);
    }
    pub(crate) fn bad(&mut self, t: impl Into<String>) {
        self.tone("bad", t);
    }
    pub(crate) fn extend(&mut self, steps: Vec<Step>) {
        self.steps.extend(steps);
    }
    pub(crate) fn table(&mut self, headers: &[&str], rows: Vec<Vec<String>>) {
        self.table = Some(StepTable { headers: headers.iter().map(|h| h.to_string()).collect(), rows });
    }
    pub(crate) fn conclude(&mut self, text: impl Into<String>, v: Val) {
        self.outcome = Some(text.into());
        self.value = Some(v);
    }
}

pub(crate) fn finish(op: &str, expression: String, title: String, engine: Val, secs: Vec<Sec>) -> StepsIr {
    let mut verified = true;
    let sections = secs
        .into_iter()
        .map(|s| {
            let agrees = s.value.as_ref().map(|v| *v == engine);
            if agrees == Some(false) {
                verified = false;
            }
            StepSection {
                kind: s.kind,
                title: s.title,
                intro: s.intro,
                steps: s.steps,
                table: s.table,
                outcome: s.outcome,
                agrees,
            }
        })
        .collect();
    StepsIr { expression, operation: op.to_string(), title, result: engine.show(), sections, verified }
}

// ───────────────────────── formatting helpers ─────────────────────────

fn sup(n: u32) -> String {
    n.to_string()
        .chars()
        .map(|c| match c {
            '0' => '⁰',
            '1' => '¹',
            '2' => '²',
            '3' => '³',
            '4' => '⁴',
            '5' => '⁵',
            '6' => '⁶',
            '7' => '⁷',
            '8' => '⁸',
            '9' => '⁹',
            other => other,
        })
        .collect()
}

fn group(primes: &[u64]) -> Vec<(u64, u32)> {
    let mut out: Vec<(u64, u32)> = Vec::new();
    for &p in primes {
        match out.last_mut() {
            Some((q, e)) if *q == p => *e += 1,
            _ => out.push((p, 1)),
        }
    }
    out
}

/// `2³ × 3² × 5`
fn power_form(g: &[(u64, u32)]) -> String {
    if g.is_empty() {
        return "1".into();
    }
    g.iter()
        .map(|&(p, e)| if e > 1 { format!("{}{}", p, sup(e)) } else { p.to_string() })
        .collect::<Vec<_>>()
        .join(" × ")
}

/// `2 × 2 × 2 × 3 × 3 × 5`
fn expanded(g: &[(u64, u32)]) -> String {
    if g.is_empty() {
        return "1".into();
    }
    let mut parts = Vec::new();
    for &(p, e) in g {
        for _ in 0..e {
            parts.push(p.to_string());
        }
    }
    parts.join(" × ")
}

fn capped(items: &[String], cap: usize) -> String {
    if items.len() <= cap {
        items.join(", ")
    } else {
        let head = cap.saturating_sub(3).max(1);
        format!("{}, …, {} ({} in total)", items[..head].join(", "), items[items.len() - 2..].join(", "), items.len())
    }
}

fn nums<T: ToString>(v: &[T]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn isqrt(n: u64) -> u64 {
    let mut r = (n as f64).sqrt() as u64;
    while r.saturating_mul(r) > n {
        r -= 1;
    }
    while (r + 1).saturating_mul(r + 1) <= n {
        r += 1;
    }
    r
}

fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}

fn powmod(mut base: u64, mut exp: u64, m: u64) -> u64 {
    let mut acc = 1u64 % m;
    base %= m;
    while exp > 0 {
        if exp & 1 == 1 {
            acc = mulmod(acc, base, m);
        }
        base = mulmod(base, base, m);
        exp >>= 1;
    }
    acc
}

fn smallest_factor(n: u64) -> u64 {
    algo::prime_factors(n).first().copied().unwrap_or(n)
}

// ───────────────────────── shared building blocks ─────────────────────────

#[derive(Default)]
struct Pending {
    first: Vec<u64>,
    count: u64,
    last: u64,
}

impl Pending {
    fn push(&mut self, d: u64) {
        if self.first.len() < 8 {
            self.first.push(d);
        }
        self.count += 1;
        self.last = d;
    }

    fn flush(&mut self, m: u64, steps: &mut Vec<Step>) {
        if self.count == 0 {
            return;
        }
        let text = if self.count <= 8 {
            let verb = if self.count == 1 { "does not divide" } else { "do not divide" };
            format!("{} {} {}", nums(&self.first).join(", "), verb, m)
        } else {
            format!(
                "{}, …, {} ({} candidates) do not divide {}",
                nums(&self.first[..6]).join(", "),
                self.last,
                self.count,
                m
            )
        };
        steps.push(Step { text, tone: "plain" });
        *self = Pending::default();
    }
}

/// Prime factorization of `n ≥ 2` with a narrated trial-division run
/// (or an honest note when `n` is too big to do by hand).
fn factor_steps(n: u64) -> (Vec<(u64, u32)>, Vec<Step>) {
    let mut steps: Vec<Step> = Vec::new();
    let say = |steps: &mut Vec<Step>, tone: &'static str, t: String| steps.push(Step { text: t, tone });

    if n > TRIAL_LIMIT {
        let g = group(&algo::prime_factors(n));
        say(
            &mut steps,
            "plain",
            format!(
                "{} is too large to factor by hand (trial division would need up to about {} divisions), so the engine uses a Miller–Rabin primality test plus Pollard's rho.",
                n,
                isqrt(n)
            ),
        );
        say(&mut steps, "good", format!("Prime factors found: {}", expanded(&g)));
        return (g, steps);
    }

    say(
        &mut steps,
        "plain",
        format!("Divide {} by 2, then by odd numbers, for as long as d × d ≤ what is left.", n),
    );
    let mut m = n;
    let mut d = 2u64;
    let mut out: Vec<u64> = Vec::new();
    let mut pend = Pending::default();
    while d * d <= m {
        if m % d == 0 {
            pend.flush(m, &mut steps);
            while m % d == 0 {
                say(&mut steps, "plain", format!("{} ÷ {} = {}", m, d, m / d));
                m /= d;
                out.push(d);
            }
        } else {
            pend.push(d);
        }
        d = if d == 2 { 3 } else { d + 2 };
    }
    pend.flush(m, &mut steps);
    if m > 1 {
        if out.is_empty() {
            say(&mut steps, "good", format!("No divisor up to √{} divides {}, so {} is prime.", n, n, n));
        } else {
            say(
                &mut steps,
                "good",
                format!("What is left, {}, has no divisor d with d × d ≤ {}, so {} is prime.", m, m, m),
            );
        }
        out.push(m);
    }
    (group(&out), steps)
}

/// Fill `s` with the Euclidean algorithm for `(ua, ub)`, both > 0. Returns the GCD.
fn euclid_body(s: &mut Sec, ua: u64, ub: u64) -> u64 {
    let (mut x, mut y) = if ua >= ub { (ua, ub) } else { (ub, ua) };
    if ua < ub {
        s.line(format!("Put the larger number first: a = {}, b = {}.", x, y));
    }
    let mut rows = Vec::new();
    let mut i = 1;
    while y != 0 {
        let (q, r) = (x / y, x % y);
        s.line(format!("{} = {} × {} + {}", x, q, y, r));
        rows.push(vec![i.to_string(), x.to_string(), y.to_string(), q.to_string(), r.to_string()]);
        x = y;
        y = r;
        i += 1;
    }
    s.table(&["Step", "a", "b", "q = a ÷ b", "r = a mod b"], rows);
    x
}

// ───────────────────────── GCD ─────────────────────────

fn gcd_prime_check(ua: u64, ub: u64) -> Sec {
    let mut s = Sec::new("check", "Quick check: is either number prime?")
        .intro("A prime has exactly two divisors, 1 and itself. That gives a shortcut.");
    for n in [ua, ub] {
        if n == 1 {
            s.line("1 is neither prime nor composite; its only divisor is 1.");
        } else if algo::is_prime(n) {
            s.line(format!("{} is prime.", n));
        } else {
            s.line(format!("{} is not prime (it is divisible by {}).", n, smallest_factor(n)));
        }
    }
    let (pa, pb) = (algo::is_prime(ua), algo::is_prime(ub));
    if ua == 1 || ub == 1 {
        s.good("One of the numbers is 1, and 1 only divides itself → GCD = 1.");
        s.conclude("GCD = 1 (shortcut)", Val::Num(1));
    } else if pa || pb {
        let (p, other) = if pa { (ua, ub) } else { (ub, ua) };
        let r = other % p;
        s.line(format!("Test the prime {} against the other number: {} mod {} = {}.", p, other, p, r));
        if r != 0 {
            s.good(format!(
                "{} is prime, so its only divisors are 1 and {}. {} does not divide {}, so the only common divisor is 1 → GCD = 1.",
                p, p, p, other
            ));
            s.conclude("GCD = 1 (shortcut)", Val::Num(1));
        } else {
            s.good(format!(
                "{} divides {}, and no common divisor can be larger than the prime {} itself → GCD = {}.",
                p, other, p, p
            ));
            s.conclude(format!("GCD = {} (shortcut)", p), Val::Num(p as i64));
        }
    } else {
        s.line("Neither number is prime, so there is no shortcut. Use the full methods below.");
    }
    if s.value.is_some() {
        s.line("Let's confirm that with the full methods below.");
    }
    s
}

fn gcd_euclid(ua: u64, ub: u64) -> Sec {
    let mut s = Sec::new("euclid", "Euclidean algorithm").intro(
        "Divide a by b and keep the remainder; repeat with (b, remainder). It works because GCD(a, b) = GCD(b, a mod b). The last non-zero remainder is the GCD.",
    );
    let g = euclid_body(&mut s, ua, ub);
    s.conclude(format!("The remainder reached 0; the last non-zero remainder is {}, so GCD = {}.", g, g), Val::Num(g as i64));
    s
}

fn gcd_factorization(ua: u64, ub: u64) -> Sec {
    let mut s = Sec::new("factorization", "Prime factorization")
        .intro("Factor both numbers, then keep every prime they share, each with the smaller exponent.");
    let mut groups = Vec::new();
    for n in [ua, ub] {
        s.head(format!("Factor {}", n));
        if n == 1 {
            s.line("1 has no prime factors (it is the empty product).");
            groups.push(Vec::new());
        } else {
            let (g, st) = factor_steps(n);
            s.extend(st);
            s.good(format!("{} = {}", n, power_form(&g)));
            groups.push(g);
        }
    }
    let mut primes: Vec<u64> = groups.iter().flatten().map(|&(p, _)| p).collect();
    primes.sort_unstable();
    primes.dedup();
    let exp_of = |g: &Vec<(u64, u32)>, p: u64| g.iter().find(|&&(q, _)| q == p).map(|&(_, e)| e).unwrap_or(0);
    let mut rows = Vec::new();
    let mut product = 1u64;
    let mut parts = Vec::new();
    for &p in &primes {
        let (ea, eb) = (exp_of(&groups[0], p), exp_of(&groups[1], p));
        let m = ea.min(eb);
        let contrib = p.pow(m);
        if m > 0 {
            product *= contrib;
            parts.push(contrib.to_string());
        }
        rows.push(vec![p.to_string(), ea.to_string(), eb.to_string(), m.to_string(), contrib.to_string()]);
    }
    if !rows.is_empty() {
        s.table(&["Prime", format!("Exponent in {}", ua).as_str(), format!("Exponent in {}", ub).as_str(), "Smaller", "Contributes"], rows);
    }
    if parts.is_empty() {
        s.conclude("No prime in common, so GCD = 1 (the empty product).", Val::Num(1));
    } else {
        s.conclude(format!("GCD = {} = {}", parts.join(" × "), product), Val::Num(product as i64));
    }
    s
}

fn gcd_listing(ua: u64, ub: u64) -> Sec {
    let mut s = Sec::new("listing", "List the factors")
        .intro("Write down every divisor of each number; the largest one they share is the GCD.");
    let da = algo::divisors(ua as i64);
    let db = algo::divisors(ub as i64);
    s.line(format!("Divisors of {}: {}", ua, capped(&nums(&da), LIST_CAP)));
    s.line(format!("Divisors of {}: {}", ub, capped(&nums(&db), LIST_CAP)));
    let common: Vec<i64> = da.iter().copied().filter(|d| db.binary_search(d).is_ok()).collect();
    s.line(format!("Common divisors: {}", capped(&nums(&common), LIST_CAP)));
    let g = common.last().copied().unwrap_or(1);
    s.conclude(format!("The greatest common divisor is {}.", g), Val::Num(g));
    s
}

fn gcd_secs(a: i64, b: i64) -> (String, Val, Vec<Sec>) {
    let (ua, ub) = (a.unsigned_abs(), b.unsigned_abs());
    let g = algo::gcd(a, b);
    let title = format!("Greatest common divisor of {} and {}", a, b);
    let mut secs = Vec::new();
    if a < 0 || b < 0 {
        let mut s = Sec::new("note", "Signs are ignored");
        s.line(format!("GCD({}, {}) = GCD({}, {}).", a, b, ua, ub));
        secs.push(s);
    }
    if ua == 0 || ub == 0 {
        let mut s = Sec::new("note", "Zero is a special case");
        if ua == 0 && ub == 0 {
            s.line("Every number divides 0, so there is no greatest common divisor; by convention GCD(0, 0) = 0.");
        } else {
            let n = ua.max(ub);
            s.line(format!("Every number divides 0, so the common divisors of 0 and {} are exactly the divisors of {}.", n, n));
            s.line(format!("The greatest divisor of {} is {} itself.", n, n));
        }
        s.conclude(format!("GCD = {}", g), Val::Num(g));
        secs.push(s);
        return (title, Val::Num(g), secs);
    }
    secs.push(gcd_prime_check(ua, ub));
    secs.push(gcd_euclid(ua, ub));
    secs.push(gcd_factorization(ua, ub));
    secs.push(gcd_listing(ua, ub));
    (title, Val::Num(g), secs)
}

// ───────────────────────── LCM ─────────────────────────

fn lcm_secs(a: i64, b: i64, l: i64) -> (String, Val, Vec<Sec>) {
    let (ua, ub) = (a.unsigned_abs(), b.unsigned_abs());
    let title = format!("Least common multiple of {} and {}", a, b);
    let mut secs = Vec::new();
    if a < 0 || b < 0 {
        let mut s = Sec::new("note", "Signs are ignored");
        s.line(format!("LCM({}, {}) = LCM({}, {}).", a, b, ua, ub));
        secs.push(s);
    }
    if ua == 0 || ub == 0 {
        let mut s = Sec::new("note", "Zero is a special case");
        s.line("The only common multiple of 0 and any number is 0, so by convention LCM = 0.");
        s.conclude("LCM = 0", Val::Num(0));
        secs.push(s);
        return (title, Val::Num(0), secs);
    }
    let (big, small) = (ua.max(ub), ua.min(ub));

    // 1. one divides the other?
    let mut s = Sec::new("check", "Quick check: does one number divide the other?")
        .intro("If the smaller number divides the larger, the larger one is already a multiple of both.");
    s.line(format!("{} mod {} = {}", big, small, big % small));
    if big % small == 0 {
        s.good(format!("{} divides {}, so the LCM is the larger number, {}.", small, big, big));
        s.conclude(format!("LCM = {} (shortcut)", big), Val::Num(big as i64));
        s.line("Let's confirm that with the full methods below.");
    } else {
        s.line("Neither divides the other, so there is no shortcut.");
    }
    secs.push(s);

    // 2. prime factorization, larger exponent
    let mut s = Sec::new("factorization", "Prime factorization")
        .intro("Factor both numbers, then take every prime that appears in either, with the larger exponent.");
    let mut groups = Vec::new();
    for n in [ua, ub] {
        s.head(format!("Factor {}", n));
        if n == 1 {
            s.line("1 has no prime factors (it is the empty product).");
            groups.push(Vec::new());
        } else {
            let (g, st) = factor_steps(n);
            s.extend(st);
            s.good(format!("{} = {}", n, power_form(&g)));
            groups.push(g);
        }
    }
    let mut primes: Vec<u64> = groups.iter().flatten().map(|&(p, _)| p).collect();
    primes.sort_unstable();
    primes.dedup();
    let exp_of = |g: &Vec<(u64, u32)>, p: u64| g.iter().find(|&&(q, _)| q == p).map(|&(_, e)| e).unwrap_or(0);
    let mut rows = Vec::new();
    let mut product: u128 = 1;
    let mut parts = Vec::new();
    for &p in &primes {
        let (ea, eb) = (exp_of(&groups[0], p), exp_of(&groups[1], p));
        let m = ea.max(eb);
        let contrib = (p as u128).pow(m);
        product *= contrib;
        parts.push(contrib.to_string());
        rows.push(vec![p.to_string(), ea.to_string(), eb.to_string(), m.to_string(), contrib.to_string()]);
    }
    if !rows.is_empty() {
        s.table(&["Prime", format!("Exponent in {}", ua).as_str(), format!("Exponent in {}", ub).as_str(), "Larger", "Contributes"], rows);
    }
    s.conclude(format!("LCM = {} = {}", parts.join(" × "), product), Val::Num(product as i64));
    secs.push(s);

    // 3. via GCD
    let mut s = Sec::new("euclid", "Using the GCD").intro(
        "Because GCD(a, b) × LCM(a, b) = a × b, find the GCD with the Euclidean algorithm and divide.",
    );
    let g = euclid_body(&mut s, ua, ub);
    s.line(format!("GCD = {}", g));
    s.line(format!("LCM = {} × {} ÷ {} = {} ÷ {}", ua, ub, g, ua as u128 * ub as u128, g));
    s.conclude(format!("LCM = {}", (ua as u128 * ub as u128 / g as u128)), Val::Num((ua as u128 * ub as u128 / g as u128) as i64));
    secs.push(s);

    // 4. list multiples of the larger number until the smaller divides one
    let g = algo::gcd_u64(ua, ub);
    let needed = small / g;
    let mut s = Sec::new("listing", "List the multiples")
        .intro(format!("Walk through the multiples of {} until one is also a multiple of {}.", big, small));
    if needed <= 30 {
        let mut rows = Vec::new();
        for k in 1..=needed {
            let m = big * k;
            rows.push(vec![format!("{} × {}", k, big), m.to_string(), (m % small).to_string(), if m % small == 0 { "yes" } else { "no" }.to_string()]);
        }
        s.table(&["Multiple", "Value", &format!("mod {}", small), &format!("Divisible by {}?", small)], rows);
        s.conclude(
            format!("The first multiple of {} that {} divides is {} → LCM = {}.", big, small, big * needed, big * needed),
            Val::Num((big * needed) as i64),
        );
    } else {
        s.line(format!("That would take {} multiples, too many to write out; the methods above already give the answer.", needed));
    }
    secs.push(s);
    let _ = l;
    (title, Val::Num(l), secs)
}

// ───────────────────────── TOTIENT ─────────────────────────

fn totient_secs(n: u64) -> (String, Val, Vec<Sec>) {
    let phi = algo::totient(n);
    let title = format!("Euler's totient φ({})", n);
    let mut secs = Vec::new();
    if n == 1 {
        let mut s = Sec::new("note", "Special case");
        s.line("The only number from 1 to 1 is 1, and GCD(1, 1) = 1, so φ(1) = 1 by convention.");
        s.conclude("φ(1) = 1", Val::Num(1));
        secs.push(s);
        return (title, Val::Num(1), secs);
    }
    // 1. prime?
    let mut s = Sec::new("check", "Quick check: is n prime?")
        .intro("φ(n) counts the numbers from 1 to n that share no factor with n. If n is prime, every number below it qualifies.");
    if algo::is_prime(n) {
        s.good(format!("{} is prime, so φ({}) = {} − 1 = {}.", n, n, n, n - 1));
        s.conclude(format!("φ({}) = {} (shortcut)", n, n - 1), Val::Num((n - 1) as i64));
        s.line("Let's confirm that with the full methods below.");
    } else {
        s.line(format!("{} is not prime (it is divisible by {}), so use its prime factors.", n, smallest_factor(n)));
    }
    secs.push(s);

    // 2 + 3. factorization & product formula
    let (g, st) = factor_steps(n);
    let mut s = Sec::new("factorization", "Prime factorization").intro("The formula only needs the distinct primes dividing n.");
    s.extend(st);
    s.good(format!("{} = {}", n, power_form(&g)));
    secs.push(s);

    let mut s = Sec::new("formula", "Euler's product formula")
        .intro("φ(n) = n × (1 − 1/p₁) × (1 − 1/p₂) × … over the distinct primes p of n. Each factor means “÷ p, then × (p − 1)”.");
    let mut cur = n;
    s.line(format!("Start with φ = {}", cur));
    for &(p, _) in &g {
        let next = cur / p * (p - 1);
        s.line(format!("× (1 − 1/{}): {} ÷ {} × {} = {}", p, cur, p, p - 1, next));
        cur = next;
    }
    s.conclude(format!("φ({}) = {}", n, cur), Val::Num(cur as i64));
    secs.push(s);

    // 4. counting
    if n <= 1_000_000 {
        let mut s = Sec::new("listing", "Count the coprime numbers").intro(format!(
            "Check every k from 1 to {}: keep k when GCD(k, {}) = 1.",
            n, n
        ));
        let coprime: Vec<u64> = (1..=n).filter(|&k| algo::gcd_u64(k, n) == 1).collect();
        s.line(format!("Numbers coprime to {}: {}", n, capped(&nums(&coprime), LIST_CAP)));
        s.conclude(format!("{} numbers qualify, so φ({}) = {}.", coprime.len(), n, coprime.len()), Val::Num(coprime.len() as i64));
        secs.push(s);
    }
    (title, Val::Num(phi as i64), secs)
}

// ───────────────────────── IS_PRIME ─────────────────────────

fn is_prime_secs(n: u64) -> (String, Val, Vec<Sec>) {
    let truth = algo::is_prime(n);
    let title = format!("Is {} prime?", n);
    let mut secs = Vec::new();

    let mut s = Sec::new("check", "Quick checks").intro("A prime is a number greater than 1 whose only divisors are 1 and itself.");
    if n < 2 {
        s.bad(format!("{} is less than 2, and primes start at 2.", n));
        s.conclude("Not prime", Val::Bool(false));
        secs.push(s);
        return (title, Val::Bool(false), secs);
    }
    if [2u64, 3, 5, 7].contains(&n) {
        s.good(format!("{} is one of the smallest primes.", n));
        s.conclude("Prime", Val::Bool(true));
        secs.push(s);
        return (title, Val::Bool(true), secs);
    }
    let mut found = None;
    for p in [2u64, 3, 5, 7] {
        s.line(format!("{} mod {} = {}", n, p, n % p));
        if n % p == 0 {
            found = Some(p);
            break;
        }
    }
    match found {
        Some(p) => {
            s.bad(format!("{} divides {}, and {} > {}, so {} is composite.", p, n, n, p, n));
            s.conclude("Not prime", Val::Bool(false));
        }
        None => s.line("None of 2, 3, 5, 7 divides it, so keep testing."),
    }
    secs.push(s);

    if n <= TRIAL_LIMIT {
        let r = isqrt(n);
        let mut s = Sec::new("trial", "Trial division up to √n").intro(format!(
            "If n had a divisor, it would have one no larger than √{} ≈ {}. Test 2, then the odd numbers.",
            n, r
        ));
        let mut rows = Vec::new();
        let mut tested = 0u64;
        let mut hit = None;
        let mut d = 2u64;
        while d <= r {
            tested += 1;
            let rem = n % d;
            if rem == 0 {
                hit = Some(d);
                if rows.len() >= 12 {
                    rows.push(vec!["…".into(), "…".into(), format!("({} more candidates, none divide)", tested - 1 - 12)]);
                }
                rows.push(vec![d.to_string(), rem.to_string(), "divides!".into()]);
                break;
            }
            if rows.len() < 12 {
                rows.push(vec![d.to_string(), rem.to_string(), "no".into()]);
            }
            d = if d == 2 { 3 } else { d + 2 };
        }
        if hit.is_none() && tested > 12 {
            rows.push(vec!["…".into(), "…".into(), format!("({} more candidates, none divide)", tested - 12)]);
        }
        s.table(&["d", &format!("{} mod d", n), "divides?"], rows);
        match hit {
            Some(d) => {
                s.bad(format!("{} = {} × {}, so {} is composite.", n, d, n / d, n));
                s.conclude("Not prime", Val::Bool(false));
            }
            None => {
                s.good(format!("No d ≤ {} divides {}, so {} is prime.", r, n, n));
                s.conclude("Prime", Val::Bool(true));
            }
        }
        secs.push(s);
    } else {
        let mut s = Sec::new("witness", "Miller–Rabin test").intro(format!(
            "{} is too large for trial division. Write n − 1 = 2ˢ × d with d odd, then check witnesses a: a pass means “a^d ≡ 1 or a^(2ʳd) ≡ −1 (mod n)”; a single failure proves n composite.",
            n
        ));
        let nm1 = n - 1;
        let sx = nm1.trailing_zeros();
        let d = nm1 >> sx;
        s.line(format!("n − 1 = {} = 2^{} × {}", nm1, sx, d));
        let mut rows = Vec::new();
        let mut composite = false;
        for a in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
            if a % n == 0 {
                continue;
            }
            let mut x = powmod(a, d, n);
            let first = x;
            let mut pass = x == 1 || x == nm1;
            let mut r = 1;
            while !pass && r < sx {
                x = mulmod(x, x, n);
                if x == nm1 {
                    pass = true;
                }
                r += 1;
            }
            rows.push(vec![a.to_string(), format!("{}^{} mod n = {}", a, d, first), if pass { "passes" } else { "WITNESS: composite" }.to_string()]);
            if !pass {
                composite = true;
                break;
            }
        }
        s.table(&["Base a", "Computation", "Verdict"], rows);
        if composite {
            s.bad("A witness was found, so n is composite.");
            s.conclude("Not prime", Val::Bool(false));
        } else {
            s.good("Every base passes. For all n below 3.3 × 10²⁴ (every 64-bit number) these 12 bases are a proof, so n is prime.");
            s.conclude("Prime", Val::Bool(true));
        }
        secs.push(s);
    }
    (title, Val::Bool(truth), secs)
}

// ───────────────────────── FACTORS ─────────────────────────

fn factors_secs(n: u64) -> (String, Val, Vec<Sec>) {
    let primes = algo::prime_factors(n);
    let list: Vec<i64> = primes.iter().map(|&p| p as i64).collect();
    let title = format!("Prime factors of {}", n);
    let mut secs = Vec::new();
    if n == 1 {
        let mut s = Sec::new("note", "Special case");
        s.line("1 is the empty product: it has no prime factors.");
        s.conclude("[]", Val::List(Vec::new()));
        secs.push(s);
        return (title, Val::List(list), secs);
    }
    let mut s = Sec::new("check", "Quick check: is n already prime?");
    if algo::is_prime(n) {
        s.good(format!("{} is prime, so its only prime factor is itself.", n));
        s.conclude(format!("[{}]", n), Val::List(vec![n as i64]));
    } else {
        s.line(format!("{} is not prime (it is divisible by {}), so break it down.", n, smallest_factor(n)));
    }
    secs.push(s);

    let (g, st) = factor_steps(n);
    let mut s = Sec::new("trial", "Trial division").intro("Pull out the smallest prime factor again and again until nothing is left.");
    s.extend(st);
    s.conclude(format!("{} = {}", n, power_form(&g)), Val::List(list.clone()));
    secs.push(s);

    let mut s = Sec::new("verify", "Check by multiplying back");
    let prod: u128 = primes.iter().map(|&p| p as u128).product();
    s.line(format!("{} = {}", expanded(&g), prod));
    if prod == n as u128 {
        s.good(format!("The product is {} ✓", n));
    } else {
        s.bad(format!("The product is {}, not {}.", prod, n));
    }
    s.conclude(format!("[{}]", nums(&list).join(", ")), Val::List(list.clone()));
    secs.push(s);
    (title, Val::List(list), secs)
}

// ───────────────────────── DIVISORS ─────────────────────────

fn divisors_secs(n: u64) -> (String, Val, Vec<Sec>) {
    let divs = algo::divisors(n as i64);
    let title = format!("Divisors of {}", n);
    let mut secs = Vec::new();
    if n == 1 {
        let mut s = Sec::new("note", "Special case");
        s.line("1 is divisible only by 1.");
        s.conclude("[1]", Val::List(vec![1]));
        secs.push(s);
        return (title, Val::List(divs), secs);
    }
    // 1. factorization
    let (g, st) = factor_steps(n);
    let mut s = Sec::new("factorization", "Prime factorization").intro("Every divisor is built from the primes of n, each used between 0 and its full exponent times.");
    s.extend(st);
    s.good(format!("{} = {}", n, power_form(&g)));
    secs.push(s);

    // 2. count
    let mut s = Sec::new("formula", "How many divisors?")
        .intro("For each prime p with exponent e there are e + 1 choices (p⁰, p¹, …, pᵉ). Multiply the choices.");
    let mut rows = Vec::new();
    let mut count = 1u64;
    for &(p, e) in &g {
        let pows: Vec<String> = (0..=e).map(|i| p.pow(i).to_string()).collect();
        rows.push(vec![p.to_string(), e.to_string(), (e + 1).to_string(), pows.join(", ")]);
        count *= e as u64 + 1;
    }
    s.table(&["Prime", "Exponent e", "Choices e + 1", "Powers"], rows);
    s.line(format!("{} = {}", g.iter().map(|&(_, e)| format!("({}+1)", e)).collect::<Vec<_>>().join(" × "), count));
    s.good(format!("{} has {} divisors.", n, count));
    secs.push(s);

    // 3. combine
    let mut s = Sec::new("multiply", "Combine the prime powers")
        .intro("Start with [1]; for each prime, multiply every divisor so far by each power of that prime.");
    let mut cur: Vec<u64> = vec![1];
    s.line("Start: [1]");
    for &(p, e) in &g {
        let mut next = Vec::new();
        for &base in &cur {
            for i in 0..=e {
                next.push(base * p.pow(i));
            }
        }
        next.sort_unstable();
        s.line(format!("After using {}: {}", power_form(&[(p, e)]), capped(&nums(&next), LIST_CAP)));
        cur = next;
    }
    let combined: Vec<i64> = cur.iter().map(|&d| d as i64).collect();
    s.conclude(format!("{} divisors: {}", combined.len(), capped(&nums(&combined), LIST_CAP)), Val::List(combined));
    secs.push(s);

    // 4. pairing
    if n <= TRIAL_LIMIT {
        let mut s = Sec::new("listing", "Pair up the divisors").intro(format!(
            "Divisors come in pairs (d, {} ÷ d), so only d ≤ √{} ≈ {} need testing.",
            n,
            n,
            isqrt(n)
        ));
        let mut small = Vec::new();
        let mut large = Vec::new();
        let mut rows = Vec::new();
        let mut d = 1u64;
        while d * d <= n {
            if n % d == 0 {
                small.push(d as i64);
                if d * d != n {
                    large.push((n / d) as i64);
                }
                if rows.len() < 40 {
                    rows.push(vec![d.to_string(), (n / d).to_string()]);
                }
            }
            d += 1;
        }
        if small.len() + large.len() > 0 && small.len() > rows.len() {
            rows.push(vec!["…".into(), "…".into()]);
        }
        s.table(&["d", &format!("{} ÷ d", n)], rows);
        large.reverse();
        let mut all = small;
        all.extend(large);
        s.conclude(format!("{} divisors: {}", all.len(), capped(&nums(&all), LIST_CAP)), Val::List(all));
        secs.push(s);
    }
    (title, Val::List(divs), secs)
}

// ───────────────────────── SIEVE ─────────────────────────

fn sieve_secs(n: u64) -> (String, Val, Vec<Sec>) {
    let primes: Vec<i64> = algo::sieve(n).into_iter().map(|p| p as i64).collect();
    let title = format!("Sieve of Eratosthenes up to {}", n);
    let mut secs = Vec::new();
    if n < 2 {
        let mut s = Sec::new("note", "Nothing to sieve");
        s.line(format!("There are no numbers from 2 to {}, so there are no primes.", n));
        s.conclude("[]", Val::List(Vec::new()));
        secs.push(s);
        return (title, Val::List(primes), secs);
    }
    let r = isqrt(n);
    let mut s = Sec::new("note", "Setup").intro("Write 2, 3, …, n. Repeatedly take the smallest number not crossed out (it must be prime) and cross out its multiples.");
    s.line(format!("Numbers: 2 … {}", n));
    s.line(format!("A composite number ≤ {} has a prime factor ≤ √{} ≈ {}, so only primes up to {} need to be used.", n, n, r, r));
    secs.push(s);

    let mut crossed = vec![false; n as usize + 1];
    let mut s = Sec::new("rounds", "Cross out multiples").intro("Each round starts at p², because smaller multiples of p were already crossed out by smaller primes.");
    let mut rows = Vec::new();
    for p in 2..=r {
        if crossed[p as usize] {
            s.line(format!("{} is already crossed out, so it is not prime: skip.", p));
            continue;
        }
        let mut newly = Vec::new();
        let mut m = p * p;
        while m <= n {
            if !crossed[m as usize] {
                crossed[m as usize] = true;
                newly.push(m);
            }
            m += p;
        }
        let left = (2..=n).filter(|&k| !crossed[k as usize]).count();
        s.head(format!("p = {}", p));
        if newly.is_empty() {
            s.line(format!("Multiples of {} from {}² = {} are all crossed out already.", p, p, p * p));
        } else {
            s.line(format!("Cross out from {}² = {} in steps of {}: {} ({} new)", p, p * p, p, capped(&nums(&newly), 24), newly.len()));
        }
        rows.push(vec![p.to_string(), (p * p).to_string(), newly.len().to_string(), left.to_string()]);
    }
    s.table(&["Prime p", "Start p²", "Newly crossed out", "Numbers left"], rows);
    secs.push(s);

    let left: Vec<i64> = (2..=n).filter(|&k| !crossed[k as usize]).map(|k| k as i64).collect();
    let mut s = Sec::new("verify", "What is left");
    s.line(format!("Uncrossed numbers: {}", capped(&nums(&left), 48)));
    s.conclude(format!("{} primes up to {}", left.len(), n), Val::List(left));
    secs.push(s);
    (title, Val::List(primes), secs)
}

// ───────────────────────── FACT / PERM / COMB ─────────────────────────

fn fact_secs(n: i64) -> (String, Val, Vec<Sec>) {
    let f = algo::factorial_checked(n).unwrap_or(0);
    let title = format!("Factorial {}!", n);
    let mut secs = Vec::new();
    let mut s = Sec::new("multiply", "Multiply 1 × 2 × … × n").intro("n! is the product of all whole numbers from 1 up to n; the empty product is 1, so 0! = 1.");
    if n == 0 {
        s.line("0! is the empty product, so 0! = 1.");
        s.conclude("0! = 1", Val::Num(1));
    } else {
        let mut rows = Vec::new();
        let mut acc: i64 = 1;
        for k in 1..=n {
            let prev = acc;
            acc *= k;
            rows.push(vec![k.to_string(), format!("{} × {}", prev, k), acc.to_string()]);
        }
        s.table(&["k", "Previous × k", &format!("{}!", "k")], rows);
        let chain = (1..=n).map(|k| k.to_string()).collect::<Vec<_>>().join(" × ");
        s.line(format!("{}! = {} = {}", n, chain, acc));
        s.conclude(format!("{}! = {}", n, acc), Val::Num(acc));
    }
    secs.push(s);
    if n >= 2 {
        let mut s = Sec::new("factorization", "Prime-power decomposition (Legendre)")
            .intro("The exponent of a prime p in n! is ⌊n/p⌋ + ⌊n/p²⌋ + ⌊n/p³⌋ + … (each term counts multiples of p, p², …).");
        let mut rows = Vec::new();
        let mut product: i128 = 1;
        let mut parts = Vec::new();
        for p in 2..=n {
            if !algo::is_prime(p as u64) {
                continue;
            }
            let mut terms = Vec::new();
            let mut q = p;
            let mut e = 0;
            while q <= n {
                terms.push(format!("⌊{}/{}⌋ = {}", n, q, n / q));
                e += n / q;
                q *= p;
            }
            product *= (p as i128).pow(e as u32);
            parts.push(format!("{}{}", p, if e > 1 { sup(e as u32) } else { String::new() }));
            rows.push(vec![p.to_string(), terms.join(" + "), e.to_string()]);
        }
        s.table(&["Prime p", "Terms", "Exponent"], rows);
        s.line(format!("{}! = {}", n, parts.join(" × ")));
        s.conclude(format!("= {}", product), Val::Num(product as i64));
        secs.push(s);
    }
    (title, Val::Num(f), secs)
}

fn perm_secs(n: i64, k: i64) -> (String, Val, Vec<Sec>) {
    let p = algo::perm_checked(n, k).unwrap_or(0);
    let title = format!("Permutations P({}, {})", n, k);
    let mut secs = Vec::new();
    let mut s = Sec::new("multiply", "Multiply k descending numbers").intro(format!(
        "P(n, k) counts ordered choices of k items from n: n choices for the first, n − 1 for the second, … so P(n, k) = n × (n − 1) × … × (n − k + 1) ({} factors).",
        k
    ));
    if k == 0 {
        s.line("k = 0 means choosing nothing: there is exactly one way (the empty product).");
        s.conclude("P = 1", Val::Num(1));
    } else {
        let mut rows = Vec::new();
        let mut acc: i128 = 1;
        for i in 0..k {
            let f = n - i;
            let prev = acc;
            acc *= f as i128;
            rows.push(vec![(i + 1).to_string(), f.to_string(), format!("{} × {}", prev, f), acc.to_string()]);
        }
        s.table(&["#", "Factor n − i", "Running product", "= "], rows);
        let chain = (0..k).map(|i| (n - i).to_string()).collect::<Vec<_>>().join(" × ");
        s.line(format!("P({}, {}) = {} = {}", n, k, chain, acc));
        s.conclude(format!("P({}, {}) = {}", n, k, acc), Val::Num(acc as i64));
    }
    secs.push(s);
    if n <= algo::FACT_MAX_N {
        let (a, b) = (algo::factorial_checked(n).unwrap_or(0), algo::factorial_checked(n - k).unwrap_or(1));
        let mut s = Sec::new("formula", "Using factorials").intro("P(n, k) = n! ÷ (n − k)!");
        s.line(format!("{}! = {}", n, a));
        s.line(format!("{}! = {}", n - k, b));
        s.line(format!("{} ÷ {} = {}", a, b, a / b));
        s.conclude(format!("P({}, {}) = {}", n, k, a / b), Val::Num(a / b));
        secs.push(s);
    }
    (title, Val::Num(p), secs)
}

fn comb_secs(n: i64, k: i64) -> (String, Val, Vec<Sec>) {
    let c = algo::comb_checked(n, k).unwrap_or(0);
    let title = format!("Combinations C({}, {})", n, k);
    let mut secs = Vec::new();
    let kk = k.min(n - k);

    let mut s = Sec::new("check", "Symmetry shortcut").intro("Choosing k items to take is the same as choosing n − k to leave behind: C(n, k) = C(n, n − k). Use the smaller one.");
    if kk != k {
        s.line(format!("n − k = {} is smaller than k = {}, so C({}, {}) = C({}, {}).", n - k, k, n, k, n, kk));
    } else {
        s.line(format!("k = {} is already ≤ n − k = {}.", k, n - k));
    }
    if kk == 0 {
        s.good("Choosing 0 items (or all of them) can be done in exactly 1 way.");
        s.conclude("C = 1", Val::Num(1));
    }
    secs.push(s);

    let mut s = Sec::new("multiply", "Multiplicative formula").intro(format!(
        "C(n, k) = (n × (n−1) × … × (n−k+1)) ÷ k!. Multiply and divide one step at a time ({} steps); every intermediate result is a whole number.",
        kk
    ));
    let mut acc: u128 = 1;
    let mut rows = Vec::new();
    for i in 1..=kk {
        let f = (n - kk + i) as u128;
        let prev = acc;
        acc = acc * f / i as u128;
        rows.push(vec![i.to_string(), f.to_string(), format!("{} × {} = {}", prev, f, prev * f), format!("÷ {}", i), acc.to_string()]);
    }
    if !rows.is_empty() {
        s.table(&["i", "Factor", "Multiply", "Divide", "Result"], rows);
    } else {
        s.line("No steps needed: the product is empty.");
    }
    s.conclude(format!("C({}, {}) = {}", n, k, acc), Val::Num(acc as i64));
    secs.push(s);

    if n <= algo::FACT_MAX_N {
        let f = |x| algo::factorial_checked(x).unwrap_or(1);
        let (a, b, d) = (f(n), f(k), f(n - k));
        let mut s = Sec::new("formula", "Factorial formula").intro("C(n, k) = n! ÷ (k! × (n − k)!)");
        s.line(format!("{}! = {}, {}! = {}, {}! = {}", n, a, k, b, n - k, d));
        s.line(format!("{} ÷ ({} × {}) = {} ÷ {} = {}", a, b, d, a, b * d, a / (b * d)));
        s.conclude(format!("C({}, {}) = {}", n, k, a / (b * d)), Val::Num(a / (b * d)));
        secs.push(s);
    }

    if n >= 2 && k > 0 && k < n {
        let x = algo::comb_checked(n - 1, k - 1).unwrap_or(0);
        let y = algo::comb_checked(n - 1, k).unwrap_or(0);
        let mut s = Sec::new("verify", "Pascal's rule").intro("Each entry of Pascal's triangle is the sum of the two above it: C(n, k) = C(n−1, k−1) + C(n−1, k).");
        s.line(format!("C({}, {}) + C({}, {}) = {} + {} = {}", n - 1, k - 1, n - 1, k, x, y, x + y));
        s.conclude(format!("C({}, {}) = {}", n, k, x + y), Val::Num(x + y));
        secs.push(s);
    }
    (title, Val::Num(c), secs)
}

// ───────────────────────── entry point ─────────────────────────

/// Which builtins have a worked solution.
pub fn supports(func: Builtin) -> bool {
    matches!(
        func,
        Builtin::IsPrime
            | Builtin::Factors
            | Builtin::Divisors
            | Builtin::Gcd
            | Builtin::Lcm
            | Builtin::Totient
            | Builtin::Sieve
            | Builtin::Fact
            | Builtin::Perm
            | Builtin::Comb
            | Builtin::Egcd
            | Builtin::ModInv
            | Builtin::ModPow
    )
}

/// Build the worked solution for `func(args)`. `expression` is the source
/// text shown in the card header. Errors describe an argument problem.
pub fn explain(func: Builtin, args: &[i64], expression: String) -> Result<StepsIr, String> {
    let need = |k: usize| -> Result<(), String> {
        if args.len() == k {
            Ok(())
        } else {
            Err(format!("{} takes {} argument(s).", func.name(), k))
        }
    };
    if args.iter().any(|&a| a == i64::MIN) {
        return Err("That number is too large to explain step by step.".into());
    }
    let (title, engine, secs) = match func {
        Builtin::Gcd => {
            need(2)?;
            gcd_secs(args[0], args[1])
        }
        Builtin::Lcm => {
            need(2)?;
            match algo::lcm_checked(args[0], args[1]) {
                Some(l) => lcm_secs(args[0], args[1], l),
                None => return Err(format!("LCM({}, {}) exceeds the 64-bit integer range.", args[0], args[1])),
            }
        }
        Builtin::Totient => {
            need(1)?;
            if args[0] < 1 {
                return Err(format!("TOTIENT needs a positive integer, got {}.", args[0]));
            }
            totient_secs(args[0] as u64)
        }
        Builtin::IsPrime => {
            need(1)?;
            if args[0] < 0 {
                return Err(format!("IS_PRIME needs a non-negative integer, got {}.", args[0]));
            }
            is_prime_secs(args[0] as u64)
        }
        Builtin::Factors => {
            need(1)?;
            if args[0] < 1 {
                return Err(format!("FACTORS needs a positive integer, got {}.", args[0]));
            }
            factors_secs(args[0] as u64)
        }
        Builtin::Divisors => {
            need(1)?;
            if args[0] < 1 {
                return Err(format!("DIVISORS needs a positive integer, got {}.", args[0]));
            }
            divisors_secs(args[0] as u64)
        }
        Builtin::Sieve => {
            need(1)?;
            if args[0] < 0 || args[0] > algo::SIEVE_MAX_N {
                return Err(format!("SIEVE requires 0 ≤ n ≤ {}, got {}.", algo::SIEVE_MAX_N, args[0]));
            }
            sieve_secs(args[0] as u64)
        }
        Builtin::Fact => {
            need(1)?;
            if args[0] < 0 || args[0] > algo::FACT_MAX_N {
                return Err(format!("FACT requires 0 ≤ n ≤ {}, got {}.", algo::FACT_MAX_N, args[0]));
            }
            fact_secs(args[0])
        }
        Builtin::Perm | Builtin::Comb => {
            need(2)?;
            let (n, k) = (args[0], args[1]);
            if n < 0 || k < 0 || k > n {
                return Err(format!("{} needs 0 ≤ k ≤ n, got n = {}, k = {}.", func.name(), n, k));
            }
            let ok = if func == Builtin::Perm { algo::perm_checked(n, k).is_some() } else { algo::comb_checked(n, k).is_some() };
            if !ok {
                return Err(format!("{}({}, {}) exceeds the 64-bit integer range.", func.name(), n, k));
            }
            if func == Builtin::Perm {
                perm_secs(n, k)
            } else {
                comb_secs(n, k)
            }
        }
        Builtin::Egcd => {
            need(2)?;
            egcd_secs(args[0], args[1])
        }
        Builtin::ModInv => {
            need(2)?;
            if args[1] < 2 {
                return Err(format!("MODINV needs a modulus of at least 2, got {}.", args[1]));
            }
            match algo::modinv(args[0], args[1]) {
                Some(x) => modinv_secs(args[0], args[1], x),
                None => {
                    return Err(format!(
                        "{} has no inverse modulo {}: GCD({}, {}) = {}, not 1.",
                        args[0], args[1], args[0], args[1], algo::gcd(args[0], args[1])
                    ))
                }
            }
        }
        Builtin::ModPow => {
            need(3)?;
            if args[1] < 0 || args[2] < 1 {
                return Err("MODPOW needs an exponent of at least 0 and a modulus of at least 1.".into());
            }
            modpow_secs(args[0], args[1] as u64, args[2])
        }
        _ => return Err(format!("STEPS has no worked solution for {}.", func.name())),
    };
    Ok(finish(func.name(), expression, title, engine, secs))
}


// ───────────────────────── EGCD / MODINV / MODPOW ─────────────────────────

fn egcd_secs(a: i64, b: i64) -> (String, Val, Vec<Sec>) {
    let (g, x, y) = algo::egcd(a, b);
    let title = format!("Extended Euclid on {} and {}", a, b);
    let mut secs = Vec::new();

    let mut s = Sec::new("method", "Euclid's algorithm, tracking x and y")
        .intro("Each row is a remainder r that we write as a·x + b·y. Row k+1 comes from subtracting q × (row k) from row k−1, where q is the whole-number quotient.");
    let (mut r0, mut r1) = (a as i128, b as i128);
    let (mut x0, mut x1) = (1i128, 0i128);
    let (mut y0, mut y1) = (0i128, 1i128);
    let mut rows = vec![
        vec!["0".into(), "–".into(), r0.to_string(), x0.to_string(), y0.to_string()],
        vec!["1".into(), "–".into(), r1.to_string(), x1.to_string(), y1.to_string()],
    ];
    let mut k = 1;
    while r1 != 0 {
        let q = r0 / r1;
        (r0, r1) = (r1, r0 - q * r1);
        (x0, x1) = (x1, x0 - q * x1);
        (y0, y1) = (y1, y0 - q * y1);
        k += 1;
        rows.push(vec![k.to_string(), q.to_string(), r1.to_string(), x1.to_string(), y1.to_string()]);
    }
    s.table(&["row", "q", "r", "x", "y"], rows);
    s.line(format!("The last non-zero remainder is {}, with x = {} and y = {}.", r0, x0, y0));
    if r0 < 0 {
        s.line("It came out negative, so flip all three signs to keep the GCD positive.");
    }
    s.conclude(format!("[g, x, y] = [{}, {}, {}]", g, x, y), Val::List(vec![g, x, y]));
    secs.push(s);

    let mut c = Sec::new("check", "Check the Bézout identity")
        .intro("The answer is right if a·x + b·y really equals g.");
    let lhs = a as i128 * x as i128 + b as i128 * y as i128;
    c.line(format!("{}·({}) + {}·({}) = {}", a, x, b, y, lhs));
    if lhs == g as i128 {
        c.good(format!("That equals the GCD, {}.", g));
    } else {
        c.bad(format!("That is not {}, so something is wrong.", g));
    }
    c.line(format!("Check against plain GCD: GCD({}, {}) = {}.", a, b, algo::gcd(a, b)));
    secs.push(c);

    let mut n = Sec::new("note", "Why this matters");
    n.line("If g = 1, then x is the inverse of a modulo b (use MODINV), and the same table solves equations like a·x ≡ c (mod b).");
    secs.push(n);
    (title, Val::List(vec![g, x, y]), secs)
}

fn modinv_secs(a: i64, m: i64, inv: i64) -> (String, Val, Vec<Sec>) {
    let title = format!("Inverse of {} modulo {}", a, m);
    let mut secs = Vec::new();
    let r = algo::modulo(a, m);
    if r != a {
        let mut n = Sec::new("note", "Reduce first");
        n.line(format!("{} ≡ {} (mod {}), so we look for the inverse of {}.", a, r, m, r));
        secs.push(n);
    }

    // 1. extended Euclid
    let (g, x, y) = algo::egcd(r, m);
    let mut s = Sec::new("method", "Extended Euclid")
        .intro(format!("Find x and y with {}·x + {}·y = 1. Then {}·x ≡ 1 (mod {}), because the {}·y part is a multiple of {}.", r, m, r, m, m, m));
    s.line(format!("Extended Euclid gives GCD = {}, x = {}, y = {}.", g, x, y));
    s.line(format!("Check: {}·({}) + {}·({}) = {}", r, x, m, y, r as i128 * x as i128 + m as i128 * y as i128));
    s.line(format!("x = {} is negative or too big, so reduce it mod {}: x mod {} = {}.", x, m, m, algo::modulo(x, m)));
    s.conclude(format!("inverse = {}", inv), Val::Num(algo::modulo(x, m)));
    secs.push(s);

    // 2. brute force
    if m <= 100_000 {
        let mut b = Sec::new("method", "Try every candidate")
            .intro(format!("Multiply {} by 1, 2, 3, … and stop at the first product that leaves remainder 1.", r));
        let mut found = None;
        let mut rows = Vec::new();
        for k in 1..m {
            let prod = (r as i128 * k as i128) % m as i128;
            if rows.len() < 30 {
                rows.push(vec![k.to_string(), format!("{} × {}", r, k), prod.to_string()]);
            }
            if prod == 1 {
                found = Some(k);
                break;
            }
        }
        b.table(&["k", "product", &format!("mod {}", m)], rows);
        if let Some(k) = found {
            b.good(format!("{}·{} leaves remainder 1.", r, k));
            b.conclude(format!("inverse = {}", k), Val::Num(k));
        }
        secs.push(b);
    }

    // 3. Fermat
    if algo::is_prime(m as u64) {
        let mut f = Sec::new("method", "Fermat's little theorem")
            .intro(format!("{} is prime, so a^(m−1) ≡ 1, which means a^(m−2) is the inverse of a.", m));
        let v = algo::modpow(r, (m - 2) as u64, m);
        f.line(format!("{}^{} mod {} = {}", r, m - 2, m, v));
        f.conclude(format!("inverse = {}", v), Val::Num(v));
        secs.push(f);
    }
    (title, Val::Num(inv), secs)
}

fn modpow_secs(base: i64, exp: u64, m: i64) -> (String, Val, Vec<Sec>) {
    let result = algo::modpow(base, exp, m);
    let title = format!("{}^{} mod {}", base, exp, m);
    let mut secs = Vec::new();

    let mut s = Sec::new("method", "Square and multiply")
        .intro("Write the exponent in binary. Walking its bits from the left, square the running result, and multiply by the base whenever the bit is 1. Reduce mod m every time so numbers stay small.");
    let mb = m as i128;
    let b0 = (base as i128).rem_euclid(mb);
    let mut acc: i128 = 1 % mb;
    let mut rows = Vec::new();
    let bits: Vec<char> = if exp == 0 { vec!['0'] } else { format!("{:b}", exp).chars().collect() };
    s.line(format!("{} in binary is {}.", exp, bits.iter().collect::<String>()));
    if b0 != base as i128 {
        s.line(format!("Start from the base reduced mod {}: {} ≡ {}.", m, base, b0));
    }
    for (i, bit) in bits.iter().enumerate() {
        let before = acc;
        acc = acc * acc % mb;
        let mut what = format!("{}² = {}", before, acc);
        if *bit == '1' {
            let sq = acc;
            acc = acc * b0 % mb;
            what = format!("{}² = {}, then × {} = {}", before, sq, b0, acc);
        }
        rows.push(vec![(i + 1).to_string(), bit.to_string(), format!("{}  (mod {})", what, m), acc.to_string()]);
    }
    s.table(&["step", "bit", "work", "result"], rows);
    s.conclude(format!("{}^{} ≡ {} (mod {})", base, exp, acc, m), Val::Num(acc as i64));
    secs.push(s);

    if exp <= 100_000 {
        let mut n = Sec::new("method", "Multiply one at a time")
            .intro(format!("The slow way: multiply by {} exactly {} times, reducing mod {} after each.", b0, exp, m));
        let mut v: i128 = 1 % mb;
        for _ in 0..exp {
            v = v * b0 % mb;
        }
        n.line(format!("After {} multiplications the running value is {}.", exp, v));
        n.conclude(format!("{}^{} ≡ {} (mod {})", base, exp, v, m), Val::Num(v as i64));
        secs.push(n);
    }
    (title, Val::Num(result), secs)
}
