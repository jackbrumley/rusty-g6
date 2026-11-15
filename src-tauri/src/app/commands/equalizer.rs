//! Graphic EQ commands: the toggle, the preamp and the ten bands.
use crate::app::protocol_console::log_to_console;
use crate::app::state::AppState;
use crate::g6_spec::EffectState;
use log::{error, info};
use tauri::{AppHandle, State};

#[tauri::command(async)]
pub fn set_eq_enabled(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
) -> Result<(), String> {
    info!("User initiated: Set EQ to {:?}", enabled);
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_eq_enabled(enabled)
        .map_err(|e| report_failure(&app, "Set EQ", e))
}

/// Returns the gain the device was set to (snapped to the 0.5 dB step).
#[tauri::command(async)]
pub fn set_eq_preamp(app: AppHandle, state: State<AppState>, gain_db: f32) -> Result<f32, String> {
    info!("User initiated: Set EQ preamp to {} dB", gain_db);
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_eq_preamp(gain_db)
        .map_err(|e| report_failure(&app, "Set EQ preamp", e))
}

/// `band` is 0 (31 Hz) to 9 (16 kHz). Returns the gain the device was set to
/// (snapped to the 0.5 dB step).
#[tauri::command(async)]
pub fn set_eq_band(
    app: AppHandle,
    state: State<AppState>,
    band: usize,
    gain_db: f32,
) -> Result<f32, String> {
    info!("User initiated: Set EQ band {} to {} dB", band, gain_db);
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_eq_band(band, gain_db)
        .map_err(|e| report_failure(&app, "Set EQ band", e))
}

/// Set all ten bands at once (a preset). Returns the gains the device was
/// set to (snapped to the 0.5 dB step).
#[tauri::command(async)]
pub fn set_eq_bands(
    app: AppHandle,
    state: State<AppState>,
    gains_db: [f32; 10],
) -> Result<[f32; 10], String> {
    info!("User initiated: Set EQ bands to {:?} dB", gains_db);
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_eq_bands(gains_db)
        .map_err(|e| report_failure(&app, "Set EQ bands", e))
}

fn report_failure(app: &AppHandle, action: &str, error: anyhow::Error) -> String {
    error!("{} failed: {:#}", action, error);
    log_to_console(app, "error", format!("❌ {action} failed: {error}"), None);
    error.to_string()
}
