use crate::app::settings::{self, AppSettings};
use crate::app::state::AppState;
use crate::g6_spec::{MicLevel, MicMonitoring, MicMonitoringChoice, RecordingSource};
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
        state.mixer.set_recording_source(source)?;
        Ok(updated)
    }
}

/// The G6's current mic level and range (Linux).
#[tauri::command(async)]
pub fn get_mic_level(state: State<AppState>) -> Result<MicLevel, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = &state;
        Err("The mic level is set in Windows sound settings on this platform".to_string())
    }

    #[cfg(target_os = "linux")]
    state.mixer.mic_level()
}

/// Set the G6's mic level in dB and keep it (Linux). Returns what the
/// control took.
#[tauri::command(async)]
pub fn set_mic_level(state: State<AppState>, level_db: f32) -> Result<MicLevel, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (&state, level_db);
        Err("The mic level is set in Windows sound settings on this platform".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        let level = state.mixer.set_mic_level(level_db)?;
        let mut current = state.settings.lock().unwrap();
        let mut updated = current.clone();
        updated.mic_level_db = Some(level.level_db);
        settings::save(&updated)?;
        *current = updated;
        log::info!("Setting changed: mic_level_db={}", level.level_db);
        Ok(level)
    }
}

/// The G6's current mic monitoring state (Linux).
#[tauri::command(async)]
pub fn get_mic_monitoring(state: State<AppState>) -> Result<MicMonitoring, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = &state;
        Err("Mic monitoring is set in Windows sound settings on this platform".to_string())
    }

    #[cfg(target_os = "linux")]
    state.mixer.mic_monitoring()
}

/// Switch mic monitoring and set its level, and keep the choice (Linux).
#[tauri::command(async)]
pub fn set_mic_monitoring(
    state: State<AppState>,
    choice: MicMonitoringChoice,
) -> Result<MicMonitoring, String> {
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (&state, choice);
        Err("Mic monitoring is set in Windows sound settings on this platform".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        let monitoring = state.mixer.set_mic_monitoring(choice)?;
        let mut current = state.settings.lock().unwrap();
        let mut updated = current.clone();
        updated.mic_monitoring = Some(MicMonitoringChoice {
            enabled: monitoring.enabled,
            level_db: monitoring.level_db,
        });
        settings::save(&updated)?;
        *current = updated;
        log::info!(
            "Setting changed: mic_monitoring={:?}",
            current.mic_monitoring
        );
        Ok(monitoring)
    }
}
