# DMath Lab — DMath Visualizer for the Desktop

Tauri 2 desktop app. The React UI in `src/` talks to the **native Rust core** (`dmath-core/`)
through two Tauri commands — there is no WebAssembly anywhere.

```
React UI  --invoke("run_dsl")-->  src-tauri (Rust)  -->  dmath-core (lexer → parser → semantic → interpreter)
          <-- dmath.viz.v1 JSON --
```

## Setup

Requires Node.js (or Bun) and the Rust toolchain with Tauri 2 dependencies
(on Linux: `webkit2gtk-4.1`, `libgtk-3-dev`, … — see https://tauri.app/start/prerequisites/).

```
cd dmath-lab
npm install
npm run tauri dev
```

The first Rust build compiles all Tauri dependencies and takes several minutes.
A production bundle: `npm run tauri build`.

## How it works

- **Frontend** — Vite 6 + React 19 + Tailwind 4. Entry: `src/main.tsx` → `src/components/dmath/dmath-app.tsx`.
- **Engine bridge** — `src/lib/dmath/engine.ts` calls `core_version` and `run_dsl` via `@tauri-apps/api`.
- **Native shell** — `src-tauri/src/lib.rs` links `dmath-core` as a normal Cargo dependency. `run_dsl` runs on a
  blocking thread and converts core panics into errors instead of crashing the app.
- The UI only works inside the Tauri window (`npm run tauri dev`), not in a plain browser tab.
- The old native DSL prototype lives in `legacy-native-prototype/` (not compiled).

## UI components

- `dmath-app.tsx` — app shell: header with engine status, example picker
  (16 example programs in 7 groups), DSL editor card, Run / Clear controls, output,
  visualization and trace panels.
- `code-editor.tsx` — syntax-highlighted DSL editor (tokenizer in `lib/dmath/highlight.ts`).
- `output-panel.tsx` — `DISPLAY` results, diagnostics with line numbers, property-check
  reports (delegates to `viz/properties-view.tsx`).
- `trace-panel.tsx` — collapsible step-by-step execution trace.
- **10 visualization views** (`viz/`): `truth-table-view`, `venn-view`, `equivalence-view`,
  `digraph-view`, `hasse-view`, `matrix-view`, `graph-view`, `sieve-view`
  (Sieve of Eratosthenes grid), `pascal-view` (Pascal's triangle with the highlighted
  C(n, k) cell) and `properties-view` (rendered inside the Output panel by
  `CHECK PROPERTIES`).
- shadcn primitives under `components/ui/`: `badge`, `button`, `card`, `select`
  (Radix), plus `lib/utils.ts` (`cn`).

## Testing the core

```
cd dmath-core && cargo test      # 133 tests
cargo run --bin dmath-cli ...    # run programs from the terminal
```

## Layout

```
src/
  components/dmath/        React UI (copied 1:1 from the main project)
  components/dmath/viz/    the 10 visualization views
  components/ui/           shadcn primitives used by the UI
  lib/dmath/               engine bridge, types, examples, highlighter
  lib/utils.ts             cn() helper
  styles.css               Tailwind 4 globals (copied from the main app/globals.css)
  main.tsx                 React entry point
dmath-core/                the Rust engine (plain library, also has a CLI)
src-tauri/                 Tauri 2 shell (window 1400x900, CSP, capabilities)
docs/GRAMMAR.md            DSL grammar reference
examples/demo.dm           example DSL program
```

## Step-by-step solutions (`AS STEPS`)
```
G = GCD(48, 18)
VISUALIZE G AS STEPS
```
Works for IS_PRIME, FACTORS, DIVISORS, GCD, LCM, TOTIENT, SIEVE, FACT, PERM and COMB, directly on a call or through a
variable. Each operation is solved by several independent methods (GCD: prime shortcut → Euclidean algorithm → prime
factorization → listing factors), and every method that computes its own answer is checked against the engine; the card shows
"All N methods agree". Code: `dmath-core/src/explain.rs`, tests: `dmath-core/tests/steps_tests.rs`, UI:
`src/components/dmath/viz/steps-view.tsx`. Numbers above 10¹² are not factored by hand (the card says so and uses
Pollard's rho / Miller–Rabin instead). Graph algorithms (BFS, Dijkstra, Kruskal…) do not have step views yet.

## Look & feel
Bundled Inter + JetBrains Mono fonts (identical on every OS, works offline), sticky blurred header, softer cards, icon tiles,
and a wider layout that uses big monitors. Styling lives in `src/styles.css` (bottom section) and `dmath-app.tsx`.

## Desktop features
Native menu bar (File / Edit / Run / Window / Help), open & save `.dm` files (Ctrl+O / Ctrl+S / Ctrl+Shift+S),
recent files, drag-and-drop a `.dm` file onto the window, unsaved-changes protection (new, open, examples, quit),
live window title (`• name — DMath Lab`), and remembered window size/position. Code: `src-tauri/src/desktop.rs`
(desktop only; Android builds skip it). `.dm` double-click works on Windows/Linux; macOS file-open events are not wired yet.

## Releases (fully automatic)
Bump the version (identical in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`), then:

```
git tag v0.1.0 && git push --tags
```

(or Actions → Release → Run workflow). `.github/workflows/release.yml` builds Windows (.exe/.msi), macOS
(universal .dmg), Linux (.deb/.AppImage/.rpm) and a signed Android .apk in parallel, attaches all of them to a
draft release, and publishes it only if every build succeeded. Android signing setup: `ANDROID.md`.
`ci.yml` runs type-check, build and the core tests on every push.
