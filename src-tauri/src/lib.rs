use dmath_core::Envelope;

#[cfg(desktop)]
mod desktop;

/// Engine version, used by the UI as its "core ready" check.
#[tauri::command]
fn core_version() -> String {
    format!("dmath-core {} ({})", env!("CARGO_PKG_VERSION"), dmath_core::CONTRACT_VERSION)
}

/// Run a DMath program natively. Executes on a blocking thread so a heavy
/// program never freezes the UI, and a panic inside the core is reported as an
/// error instead of taking the whole app down.
#[tauri::command]
async fn run_dsl(source: String) -> Result<Envelope, String> {
    tauri::async_runtime::spawn_blocking(move || {
        std::panic::catch_unwind(|| dmath_core::run_source(&source))
            .map_err(|_| "DMath Core panicked while running this program (internal bug).".to_string())
    })
    .await
    .map_err(|e| format!("DMath Core task failed: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    // Desktop: remembered window size/position, native menu, file dialogs.
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .manage(desktop::DocState::new())
        .setup(|app| {
            desktop::setup(app.handle())?;
            Ok(())
        })
        .on_menu_event(|app, event| desktop::on_menu_event(app, event.id().as_ref()))
        .on_window_event(|window, event| desktop::on_window_event(window, event))
        .invoke_handler(tauri::generate_handler![
            core_version,
            run_dsl,
            desktop::open_dialog,
            desktop::read_file,
            desktop::save_file,
            desktop::confirm_discard,
            desktop::set_doc_state,
            desktop::startup_file,
        ]);

    // Mobile: engine only (no menu bar / file dialogs).
    #[cfg(mobile)]
    let builder = builder.invoke_handler(tauri::generate_handler![core_version, run_dsl]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
