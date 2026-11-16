//! `VISUALIZE … AS STEPS`: every worked solution must agree with the engine,
//! method by method, over wide input ranges and edge cases.

use dmath_core::explain::explain;
use dmath_core::ir::{StepsIr, VizIr};
use dmath_core::token::Builtin;
use dmath_core::{algo, run_source};

fn ex(func: Builtin, args: &[i64]) -> StepsIr {
    let ir = explain(func, args, format!("{}{:?}", func.name(), args)).unwrap_or_else(|e| panic!("{} {:?}: {e}", func.name(), args));
    assert!(ir.verified, "{} {:?}: methods DISAGREE: {:#?}", func.name(), args, ir.sections.iter().map(|s| (&s.title, s.agrees, &s.outcome)).collect::<Vec<_>>());
    // every section that computed something must have agreed
    for s in &ir.sections {
        assert_ne!(s.agrees, Some(false), "{} {:?}: section '{}' disagrees", func.name(), args, s.title);
    }
    ir
}

#[test]
fn gcd_all_methods_agree_over_a_grid() {
    for a in -40..=60 {
        for b in -40..=60 {
            let ir = ex(Builtin::Gcd, &[a, b]);
            assert_eq!(ir.result, algo::gcd(a, b).to_string(), "GCD({a},{b})");
        }
    }
}

#[test]
fn gcd_shortcuts_and_methods_are_present() {
    // neither prime: 4 methods
    let ir = ex(Builtin::Gcd, &[48, 18]);
    assert_eq!(ir.result, "6");
    let kinds: Vec<_> = ir.sections.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, ["check", "euclid", "factorization", "listing"]);
    assert!(ir.sections[0].outcome.is_none(), "no shortcut for 48, 18");
    // prime vs other, not dividing -> shortcut 1
    let ir = ex(Builtin::Gcd, &[17, 51 + 1]);
    assert_eq!(ir.result, "1");
    assert_eq!(ir.sections[0].outcome.as_deref(), Some("GCD = 1 (shortcut)"));
    // prime that divides the other -> shortcut p
    let ir = ex(Builtin::Gcd, &[17, 51]);
    assert_eq!(ir.result, "17");
    assert_eq!(ir.sections[0].outcome.as_deref(), Some("GCD = 17 (shortcut)"));
    // euclid table for 48,18: 48=2*18+12, 18=1*12+6, 12=2*6+0
    let ir = ex(Builtin::Gcd, &[48, 18]);
    let t = ir.sections[1].table.as_ref().unwrap();
    assert_eq!(t.rows.len(), 3);
    assert_eq!(t.rows[0], ["1", "48", "18", "2", "12"]);
    assert_eq!(t.rows[2], ["3", "12", "6", "2", "0"]);
}

#[test]
fn gcd_zero_negative_and_big() {
    assert_eq!(ex(Builtin::Gcd, &[0, 0]).result, "0");
    assert_eq!(ex(Builtin::Gcd, &[0, 9]).result, "9");
    assert_eq!(ex(Builtin::Gcd, &[-48, 18]).result, "6");
    // larger than the trial-division limit: factor narration falls back honestly
    let ir = ex(Builtin::Gcd, &[9_223_372_036_854_775_807, 4_611_686_018_427_387_903]);
    assert_eq!(ir.result, algo::gcd(9_223_372_036_854_775_807, 4_611_686_018_427_387_903).to_string());
    ex(Builtin::Gcd, &[963_761_198_400, 481_880_599_200]); // 6720 divisors
}

#[test]
fn lcm_agrees() {
    for a in -30..=40 {
        for b in -30..=40 {
            let ir = ex(Builtin::Lcm, &[a, b]);
            assert_eq!(ir.result, algo::lcm_checked(a, b).unwrap().to_string(), "LCM({a},{b})");
        }
    }
    assert_eq!(ex(Builtin::Lcm, &[4, 6]).result, "12");
    assert!(explain(Builtin::Lcm, &[i64::MAX, i64::MAX - 1], "x".into()).is_err(), "overflow must be an error, not wrong math");
}

#[test]
fn totient_agrees() {
    for n in 1..=700 {
        let ir = ex(Builtin::Totient, &[n]);
        assert_eq!(ir.result, algo::totient(n as u64).to_string(), "φ({n})");
    }
    ex(Builtin::Totient, &[999_999]);
    ex(Builtin::Totient, &[1_000_000]);
    ex(Builtin::Totient, &[2_000_001]); // above the counting cap: formula only
}

#[test]
fn is_prime_agrees_including_huge() {
    for n in 0..=600 {
        let ir = ex(Builtin::IsPrime, &[n]);
        assert_eq!(ir.result, if algo::is_prime(n as u64) { "TRUE" } else { "FALSE" }, "IS_PRIME({n})");
    }
    for n in [
        1_000_000_007,
        999_999_999_989,               // largest prime below 10^12 (trial division)
        999_999_999_999,               // composite
        1_000_000_000_039,             // first prime above 10^12 (Miller-Rabin path)
        1_000_000_000_001,             // composite, above the limit
        9_223_372_036_854_775_783,     // largest prime below 2^63
        9_223_372_036_854_775_807,     // 2^63 - 1, composite
        3_215_031_751,                 // strong pseudoprime to bases 2,3,5,7
    ] {
        let ir = ex(Builtin::IsPrime, &[n]);
        assert_eq!(ir.result, if algo::is_prime(n as u64) { "TRUE" } else { "FALSE" }, "IS_PRIME({n})");
    }
}

#[test]
fn factors_and_divisors_agree() {
    for n in 1..=500 {
        let f = ex(Builtin::Factors, &[n]);
        let expect = format!("[{}]", algo::prime_factors(n as u64).iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
        assert_eq!(f.result, expect, "FACTORS({n})");
        let d = ex(Builtin::Divisors, &[n]);
        let expect = format!("[{}]", algo::divisors(n).iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
        assert_eq!(d.result, expect, "DIVISORS({n})");
    }
    for n in [360, 963_761_198_400, 999_999_999_989, 1_000_000_000_000, 4_611_686_018_427_387_904 / 2, 9_223_372_036_854_775_807] {
        ex(Builtin::Factors, &[n]);
    }
    ex(Builtin::Divisors, &[963_761_198_400]);
    ex(Builtin::Divisors, &[9_223_372_036_854_775_807]);
}

#[test]
fn sieve_agrees() {
    for n in 0..=130 {
        let ir = ex(Builtin::Sieve, &[n]);
        let expect = format!("[{}]", algo::sieve(n as u64).iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
        assert_eq!(ir.result, expect, "SIEVE({n})");
    }
    ex(Builtin::Sieve, &[100_000]);
    assert!(explain(Builtin::Sieve, &[100_001], "x".into()).is_err());
    assert!(explain(Builtin::Sieve, &[-1], "x".into()).is_err());
}

#[test]
fn combinatorics_agrees() {
    for n in 0..=20 {
        assert_eq!(ex(Builtin::Fact, &[n]).result, algo::factorial_checked(n).unwrap().to_string(), "FACT({n})");
    }
    assert!(explain(Builtin::Fact, &[21], "x".into()).is_err());
    for n in 0..=40 {
        for k in 0..=n {
            if let Some(p) = algo::perm_checked(n, k) {
                assert_eq!(ex(Builtin::Perm, &[n, k]).result, p.to_string(), "PERM({n},{k})");
            } else {
                assert!(explain(Builtin::Perm, &[n, k], "x".into()).is_err());
            }
            if let Some(c) = algo::comb_checked(n, k) {
                assert_eq!(ex(Builtin::Comb, &[n, k]).result, c.to_string(), "COMB({n},{k})");
            }
        }
    }
    assert!(explain(Builtin::Comb, &[5, 6], "x".into()).is_err());
    assert!(explain(Builtin::Perm, &[-1, 0], "x".into()).is_err());
}

#[test]
fn argument_errors_are_clean() {
    for (f, a) in [
        (Builtin::Factors, vec![0]),
        (Builtin::Divisors, vec![-5]),
        (Builtin::Totient, vec![0]),
        (Builtin::IsPrime, vec![-1]),
        (Builtin::Gcd, vec![i64::MIN, 4]),
    ] {
        assert!(explain(f, &a, "x".into()).is_err(), "{:?} {:?}", f.name(), a);
    }
}

// ── through the real language ─────────────────────────────────────────────

fn steps_of(src: &str) -> StepsIr {
    let env = run_source(src);
    assert!(env.ok, "program failed: {:?}", env.diagnostics.iter().map(|d| &d.message).collect::<Vec<_>>());
    env.visualizations
        .into_iter()
        .find_map(|v| if let VizIr::Steps(s) = v { Some(s) } else { None })
        .expect("a steps visualization")
}

#[test]
fn language_surface() {
    // via a variable
    let s = steps_of("G = GCD(48, 18)\nVISUALIZE G AS STEPS\n");
    assert_eq!((s.operation.as_str(), s.result.as_str(), s.verified), ("GCD", "6", true));
    // directly on a call
    let s = steps_of("VISUALIZE LCM(4, 6) AS STEPS\n");
    assert_eq!(s.result, "12");
    // through a chain of variables, and with computed arguments
    let s = steps_of("N = LCM(4, 6)\nF = FACTORS(N)\nG = F\nVISUALIZE G AS STEPS\n");
    assert_eq!(s.result, "[2, 2, 3]"); // N = 12, via a variable and a copy
    // every supported function
    for src in [
        "X = IS_PRIME(97)", "X = FACTORS(360)", "X = DIVISORS(36)", "X = GCD(12, 18)", "X = LCM(4, 6)",
        "X = TOTIENT(36)", "X = SIEVE(50)", "X = FACT(6)", "X = PERM(5, 2)", "X = COMB(6, 3)",
    ] {
        let s = steps_of(&format!("{src}\nVISUALIZE X AS STEPS\n"));
        assert!(s.verified, "{src}");
        assert!(s.sections.len() >= 2, "{src}");
    }
}

#[test]
fn language_errors_are_diagnostics_not_panics() {
    for src in [
        "A = {1, 2}\nVISUALIZE A AS STEPS\n",              // not a steppable call
        "X = 5\nVISUALIZE X AS STEPS\n",                    // plain number, no provenance
        "X = FACTORS(0)\nVISUALIZE X AS STEPS\n",           // engine error propagates
        "VISUALIZE FACT(25) AS STEPS\n",                    // out of range
        "VISUALIZE GCD(1, 2) AS TRUTH_TABLE\n",             // other kinds still reject it
    ] {
        let env = run_source(src);
        assert!(!env.ok, "should fail: {src}");
        assert!(!env.diagnostics.is_empty(), "{src}");
    }
    // reassignment drops provenance
    let env = run_source("X = GCD(4, 6)\nX = 7\nVISUALIZE X AS STEPS\n");
    assert!(!env.ok);
}
