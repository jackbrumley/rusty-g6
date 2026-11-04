//! The Logs block in Settings -> Debug (App Standard section 4): copy and
//! open the session log, open the log folder, system info, log level.

use crate::app::session_log;
use crate::app::settings;
use crate::app::state::AppState;
use crate::app::system_info::SystemInfo;
use std::path::Path;
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn copy_session_log(app: AppHandle) -> Result<(), String> {
    let text = session_log::get_session_log_text()?;
    app.clipboard()
        .write_text(text)
        .map_err(|error| error.to_string())?;
    log::info!("Session log copied to the clipboard");
    Ok(())
}

#[tauri::command]
pub fn open_session_log() -> Result<(), String> {
    open_with_system(&session_log::resolve_session_log_path()?)
}

#[tauri::command]
pub fn open_log_folder() -> Result<(), String> {
    open_with_system(&crate::paths::debug_dir()?)
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    SystemInfo::collect()
}

#[tauri::command]
pub fn get_log_level(state: State<AppState>) -> String {
    state.settings.lock().unwrap().log_level.clone()
}

#[tauri::command]
pub fn set_log_level(state: State<AppState>, level: String) -> Result<String, String> {
    let filter = session_log::parse_log_level(&level)
        .ok_or_else(|| format!("Unknown log level '{level}'"))?;
    let mut current = state.settings.lock().unwrap();
    let mut updated = current.clone();
    updated.log_level = level.clone();
    settings::save(&updated)?;
    *current = updated;
    log::set_max_level(filter);
    log::info!("Setting changed: log_level={}", level);
    Ok(level)
}

#[tauri::command]
pub fn log_ui_event(message: String) {
    log::info!("[UI] {}", message);
}

fn open_with_system(path: &Path) -> Result<(), String> {
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(|error| error.to_string())
}
