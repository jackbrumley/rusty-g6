use crate::app::settings::{self, AppSettings};
use crate::app::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_app_settings(state: State<AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_close_to_tray(state: State<AppState>, enabled: bool) -> Result<AppSettings, String> {
    let mut current = state.settings.lock().unwrap();
    let mut updated = current.clone();
    updated.close_to_tray = enabled;
    settings::save(&updated)?;
    *current = updated.clone();
    log::info!("Setting changed: close_to_tray={}", enabled);
    Ok(updated)
}
