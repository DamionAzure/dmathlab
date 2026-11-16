// Bridge to the native DMath Core.
//
// The computation engine is the Rust crate `dmath-core`, linked straight into
// the Tauri shell (src-tauri). The UI calls it through Tauri commands:
//   core_version()      -> string       sanity check / "core ready" signal
//   run_dsl(source)     -> DMathEnvelope run a program, get the dmath.viz.v1 JSON

import { invoke } from "@tauri-apps/api/core";
import type { DMathEnvelope } from "./types";

export interface DMathEngine {
  version: string;
  run(source: string): Promise<DMathEnvelope>;
}

let enginePromise: Promise<DMathEngine> | null = null;

export function loadEngine(): Promise<DMathEngine> {
  if (!enginePromise) {
    enginePromise = connect().catch((e) => {
      enginePromise = null; // allow retry
      throw e;
    });
  }
  return enginePromise;
}

async function connect(): Promise<DMathEngine> {
  if (!("__TAURI_INTERNALS__" in window)) {
    throw new Error(
      "Not running inside the DMath Lab desktop app. Start it with `npm run tauri dev`.",
    );
  }
  const version = await invoke<string>("core_version");
  return {
    version,
    run: (source: string) => invoke<DMathEnvelope>("run_dsl", { source }),
  };
}
