use crate::app::settings::{self, AppSettings};
use crate::app::state::AppState;
use crate::g6_spec::RecordingSource;
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

#[tauri::command]
pub fn get_recording_source(state: State<AppState>) -> RecordingSource {
    state.settings.lock().unwrap().recording_source
}

/// Store the G6 recording source and select it now (Linux). Rusty G6 keeps
/// it selected from then on, also after reboots and replugs.
#[tauri::command(async)]
pub fn set_recording_source(
    state: State<AppState>,
    source: RecordingSource,
) -> Result<AppSettings, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (&state, source);
        Err("The recording source is set by Windows on this platform".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        let updated = {
            let mut current = state.settings.lock().unwrap();
            let mut updated = current.clone();
            updated.recording_source = source;
            settings::save(&updated)?;
            *current = updated.clone();
            updated
        };
        log::info!("Setting changed: recording_source={:?}", source);
        state.recording_source.set(source)?;
        Ok(updated)
    }
}
