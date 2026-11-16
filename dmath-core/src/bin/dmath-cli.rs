//! dmath-cli — command line consumer of the DMath Core library.
//!
//! Usage:
//!   dmath-cli <file.dm>        run a program file
//!   cat program.dm | dmath-cli run from stdin
//!   dmath-cli --compact ...    emit compact JSON
//!
//! Prints the Visualization IR envelope as JSON on stdout.
//! Exit code 0 = all statements executed, 1 = at least one error.

use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let compact = args.iter().any(|a| a == "--compact");
    let path = args.iter().find(|a| !a.starts_with("--"));

    let src = match path {
        Some(p) => match std::fs::read_to_string(p) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("dmath-cli: cannot read '{}': {}", p, e);
                std::process::exit(2);
            }
        },
        None => {
            let mut s = String::new();
            if std::io::stdin().read_to_string(&mut s).is_err() {
                eprintln!("dmath-cli: cannot read stdin");
                std::process::exit(2);
            }
            s
        }
    };

    let source = src.clone();
    let envelope = std::panic::catch_unwind(move || dmath_core::run_source(&source))
        .unwrap_or_else(|_| {
            let mut env = dmath_core::Envelope::empty();
            env.ok = false;
            env.diagnostics.push(
                dmath_core::token::Diagnostic::error(1, 1, "Internal engine error (panic).")
                    .into(),
            );
            env
        });

    let json = if compact {
        serde_json::to_string(&envelope).unwrap()
    } else {
        serde_json::to_string_pretty(&envelope).unwrap()
    };
    println!("{}", json);

    if !envelope.ok {
        std::process::exit(1);
    }
}
