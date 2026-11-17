// Runs every example in src/lib/dmath/reference.ts through the real engine
// (dmath-cli) and fails if any of them does not run cleanly.
//   cargo build --manifest-path dmath-core/Cargo.toml --bin dmath-cli
//   node scripts/check-reference.mjs
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const src = readFileSync("src/lib/dmath/reference.ts", "utf8")
  .replace(/export interface[\s\S]*?\n}\n/g, "")
  .replace(/: RefSection\[\]/, "")
  .replace(/export const/g, "const");
const REFERENCE = new Function(`${src}; return REFERENCE;`)();
const cli = process.env.DMATH_CLI ?? "dmath-core/target/debug/dmath-cli";
const dir = mkdtempSync(join(tmpdir(), "dmref-"));
let bad = 0;
let n = 0;
for (const sec of REFERENCE) {
  for (const e of sec.entries) {
    n++;
    const f = join(dir, "x.dm");
    writeFileSync(f, e.example);
    let raw;
    try {
      raw = execFileSync(cli, [f], { encoding: "utf8" });
    } catch (err) {
      raw = err.stdout; // the CLI exits 1 when the program has an error
    }
    const out = JSON.parse(raw);
    if (!out.ok) {
      bad++;
      console.error(`FAIL ${e.name}:`, JSON.stringify(out.diagnostics?.[0]));
    }
  }
}
console.log(`${n} reference examples, ${bad} failing`);
process.exit(bad ? 1 : 0);
