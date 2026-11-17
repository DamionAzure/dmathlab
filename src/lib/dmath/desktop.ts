// Desktop shell features (native menu, file dialogs, window title).
// Everything here is implemented in src-tauri/src/desktop.rs.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface OpenedFile {
  path: string;
  contents: string;
}

export type MenuAction = "new" | "open" | "save" | "save_as" | "run" | "clear_output" | "clear_editor" | "reference";

export const openFileDialog = () => invoke<OpenedFile | null>("open_dialog");
export const readFile = (path: string) => invoke<OpenedFile>("read_file", { path });
export const saveFile = (path: string | null, contents: string) =>
  invoke<string | null>("save_file", { path, contents });
export const confirmDiscard = () => invoke<boolean>("confirm_discard");
export const startupFile = () => invoke<OpenedFile | null>("startup_file");
export const setDocState = (name: string | null, dirty: boolean) =>
  invoke<void>("set_doc_state", { name, dirty });

export function fileName(path: string | null): string | null {
  return path ? (path.split(/[\\/]/).pop() ?? path) : null;
}

export interface DesktopHandlers {
  onMenu(action: MenuAction): void;
  onOpenPath(path: string): void;
}

/** Subscribe to native menu clicks and files dropped on the window. */
export async function listenDesktop(h: DesktopHandlers): Promise<UnlistenFn> {
  const offs = await Promise.all([
    listen<MenuAction>("menu", (e) => h.onMenu(e.payload)),
    listen<string>("menu-open-path", (e) => h.onOpenPath(e.payload)),
    listen<{ paths: string[] }>("tauri://drag-drop", (e) => {
      const p = e.payload.paths.find((x) => /\.(dm|txt)$/i.test(x));
      if (p) h.onOpenPath(p);
    }),
  ]);
  return () => offs.forEach((off) => off());
}
