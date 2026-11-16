//! Desktop-only "app feel": native menu bar, open/save of `.dm` files, recent
//! files, unsaved-changes protection and a live window title.
//!
//! The UI owns the document text; Rust owns dialogs, disk access and menus.

use std::{fs, path::PathBuf, sync::Mutex};

use serde::Serialize;
use tauri::{
    menu::{Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

const MAX_RECENT: usize = 8;
const MAX_FILE_BYTES: u64 = 5 * 1024 * 1024;
const FILTER_NAME: &str = "DMath program";
const FILTER_EXT: &[&str] = &["dm", "txt"];

pub struct DocState {
    recents: Mutex<Vec<String>>,
    dirty: Mutex<bool>,
    name: Mutex<String>,
}

impl DocState {
    pub fn new() -> Self {
        Self {
            recents: Mutex::new(Vec::new()),
            dirty: Mutex::new(false),
            name: Mutex::new("Untitled".into()),
        }
    }
}

#[derive(Serialize)]
pub struct OpenedFile {
    path: String,
    contents: String,
}

fn file_name(path: &str) -> String {
    PathBuf::from(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

// ── recent files ────────────────────────────────────────────────────────────

fn recent_store(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_config_dir().ok()?;
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join("recent-files.json"))
}

fn load_recents(app: &AppHandle) -> Vec<String> {
    recent_store(app)
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
}

fn persist_recents(app: &AppHandle, list: &[String]) {
    if let Some(p) = recent_store(app) {
        if let Ok(json) = serde_json::to_string(list) {
            let _ = fs::write(p, json);
        }
    }
}

fn set_recents(app: &AppHandle, state: &DocState, list: Vec<String>) {
    persist_recents(app, &list);
    if let Ok(menu) = build_menu(app, &list) {
        let _ = app.set_menu(menu);
    }
    *state.recents.lock().unwrap() = list;
}

fn add_recent(app: &AppHandle, state: &DocState, path: &str) {
    let mut list = state.recents.lock().unwrap().clone();
    list.retain(|p| p != path);
    list.insert(0, path.to_string());
    list.truncate(MAX_RECENT);
    set_recents(app, state, list);
}

// ── menu ────────────────────────────────────────────────────────────────────

fn build_menu(app: &AppHandle, recents: &[String]) -> tauri::Result<Menu<tauri::Wry>> {
    let item = |id: &str, label: &str, accel: Option<&str>| {
        let b = MenuItemBuilder::with_id(id, label);
        match accel {
            Some(a) => b.accelerator(a).build(app),
            None => b.build(app),
        }
    };

    let mut recent_menu = SubmenuBuilder::new(app, "Open Recent");
    if recents.is_empty() {
        recent_menu = recent_menu.item(
            &MenuItemBuilder::with_id("no_recent", "No recent files")
                .enabled(false)
                .build(app)?,
        );
    } else {
        for (i, p) in recents.iter().enumerate() {
            recent_menu =
                recent_menu.item(&MenuItemBuilder::with_id(format!("recent:{i}"), file_name(p)).build(app)?);
        }
        recent_menu = recent_menu
            .separator()
            .item(&item("clear_recent", "Clear Recent Files", None)?);
    }
    let recent_menu = recent_menu.build()?;

    let quit = item("quit", "Quit DMath Lab", Some("CmdOrCtrl+Q"))?;

    let file = SubmenuBuilder::new(app, "File")
        .item(&item("new", "New", Some("CmdOrCtrl+N"))?)
        .item(&item("open", "Open…", Some("CmdOrCtrl+O"))?)
        .item(&recent_menu)
        .separator()
        .item(&item("save", "Save", Some("CmdOrCtrl+S"))?)
        .item(&item("save_as", "Save As…", Some("CmdOrCtrl+Shift+S"))?)
        .separator()
        .item(&quit)
        .build()?;

    let edit = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let run = SubmenuBuilder::new(app, "Run")
        .item(&item("run", "Run Program (Ctrl+Enter)", None)?)
        .separator()
        .item(&item("clear_output", "Clear Output", None)?)
        .item(&item("clear_editor", "Clear Editor", None)?)
        .build()?;

    let window = SubmenuBuilder::new(app, "Window")
        .minimize()
        .maximize()
        .build()?;

    let help = SubmenuBuilder::new(app, "Help").about(None).build()?;

    let mut menu = MenuBuilder::new(app);
    if cfg!(target_os = "macos") {
        // macOS: the first submenu is the application menu.
        let app_menu = SubmenuBuilder::new(app, "DMath Lab")
            .about(None)
            .separator()
            .hide()
            .hide_others()
            .separator()
            .item(&item("quit", "Quit DMath Lab", Some("CmdOrCtrl+Q"))?)
            .build()?;
        menu = menu.item(&app_menu);
    }
    menu.item(&file)
        .item(&edit)
        .item(&run)
        .item(&window)
        .item(&help)
        .build()
}

pub fn on_menu_event(app: &AppHandle, id: &str) {
    let state = app.state::<DocState>();
    if let Some(idx) = id.strip_prefix("recent:").and_then(|i| i.parse::<usize>().ok()) {
        let path = state.recents.lock().unwrap().get(idx).cloned();
        if let Some(p) = path {
            let _ = app.emit("menu-open-path", p);
        }
        return;
    }
    match id {
        "clear_recent" => set_recents(app, &state, Vec::new()),
        "quit" => {
            // Go through the normal close path so unsaved changes are protected.
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.close();
            }
        }
        "new" | "open" | "save" | "save_as" | "run" | "clear_output" | "clear_editor" => {
            let _ = app.emit("menu", id);
        }
        _ => {}
    }
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<DocState>();
    let recents = load_recents(app);
    *state.recents.lock().unwrap() = recents.clone();
    app.set_menu(build_menu(app, &recents)?)?;
    Ok(())
}

// ── window title + close protection ─────────────────────────────────────────

fn refresh_title(app: &AppHandle, state: &DocState) {
    if let Some(w) = app.get_webview_window("main") {
        let name = state.name.lock().unwrap().clone();
        let dirty = *state.dirty.lock().unwrap();
        let _ = w.set_title(&format!("{}{} — DMath Lab", if dirty { "• " } else { "" }, name));
    }
}

pub fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        let app = window.app_handle().clone();
        let dirty = *app.state::<DocState>().dirty.lock().unwrap();
        if dirty {
            api.prevent_close();
            let window = window.clone();
            app.dialog()
                .message("You have unsaved changes. Quit without saving?")
                .title("Unsaved changes")
                .kind(MessageDialogKind::Warning)
                .buttons(MessageDialogButtons::OkCancelCustom("Quit".into(), "Cancel".into()))
                .show(move |quit| {
                    if quit {
                        *window.app_handle().state::<DocState>().dirty.lock().unwrap() = false;
                        let _ = window.destroy();
                    }
                });
        }
    }
}

// ── commands ────────────────────────────────────────────────────────────────

fn read_checked(path: &str) -> Result<String, String> {
    let meta = fs::metadata(path).map_err(|e| format!("Cannot open {}: {e}", file_name(path)))?;
    if meta.len() > MAX_FILE_BYTES {
        return Err(format!("{} is too large (limit 5 MB).", file_name(path)));
    }
    let bytes = fs::read(path).map_err(|e| format!("Cannot read {}: {e}", file_name(path)))?;
    String::from_utf8(bytes).map_err(|_| format!("{} is not a UTF-8 text file.", file_name(path)))
}

#[tauri::command]
pub async fn open_dialog(app: AppHandle, state: State<'_, DocState>) -> Result<Option<OpenedFile>, String> {
    let picked = app
        .dialog()
        .file()
        .add_filter(FILTER_NAME, FILTER_EXT)
        .blocking_pick_file();
    let Some(picked) = picked else { return Ok(None) };
    let path = picked.into_path().map_err(|e| e.to_string())?;
    let path = path.to_string_lossy().into_owned();
    let contents = read_checked(&path)?;
    add_recent(&app, &state, &path);
    Ok(Some(OpenedFile { path, contents }))
}

#[tauri::command]
pub async fn read_file(app: AppHandle, state: State<'_, DocState>, path: String) -> Result<OpenedFile, String> {
    let contents = match read_checked(&path) {
        Ok(c) => c,
        Err(e) => {
            // Drop dead entries from the recent list.
            let mut list = state.recents.lock().unwrap().clone();
            if list.contains(&path) {
                list.retain(|p| p != &path);
                set_recents(&app, &state, list);
            }
            return Err(e);
        }
    };
    add_recent(&app, &state, &path);
    Ok(OpenedFile { path, contents })
}

/// Save to `path`, or ask where to save when `path` is `None`.
/// Returns the final path, or `None` if the user cancelled.
#[tauri::command]
pub async fn save_file(
    app: AppHandle,
    state: State<'_, DocState>,
    path: Option<String>,
    contents: String,
) -> Result<Option<String>, String> {
    let target = match path {
        Some(p) => p,
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter(FILTER_NAME, FILTER_EXT)
                .set_file_name("program.dm")
                .blocking_save_file();
            let Some(picked) = picked else { return Ok(None) };
            let mut p = picked.into_path().map_err(|e| e.to_string())?;
            if p.extension().is_none() {
                p.set_extension("dm");
            }
            p.to_string_lossy().into_owned()
        }
    };
    fs::write(&target, contents).map_err(|e| format!("Cannot save {}: {e}", file_name(&target)))?;
    add_recent(&app, &state, &target);
    Ok(Some(target))
}

#[tauri::command]
pub async fn confirm_discard(app: AppHandle) -> bool {
    app.dialog()
        .message("You have unsaved changes. Discard them?")
        .title("Unsaved changes")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("Discard".into(), "Cancel".into()))
        .blocking_show()
}

#[tauri::command]
pub fn set_doc_state(app: AppHandle, state: State<'_, DocState>, name: Option<String>, dirty: bool) {
    *state.name.lock().unwrap() = name.unwrap_or_else(|| "Untitled".into());
    *state.dirty.lock().unwrap() = dirty;
    refresh_title(&app, &state);
}

/// A `.dm` path passed on the command line (e.g. double-clicking a file).
#[tauri::command]
pub async fn startup_file(app: AppHandle, state: State<'_, DocState>) -> Result<Option<OpenedFile>, String> {
    let arg = std::env::args()
        .skip(1)
        .find(|a| !a.starts_with('-') && a.to_lowercase().ends_with(".dm"));
    let Some(path) = arg else { return Ok(None) };
    let contents = read_checked(&path)?;
    add_recent(&app, &state, &path);
    Ok(Some(OpenedFile { path, contents }))
}
