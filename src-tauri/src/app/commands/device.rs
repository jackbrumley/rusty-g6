use crate::app::state::AppState;
use crate::g6_spec::G6Settings;
use tauri::{AppHandle, Emitter, State};

/// The device connected, disconnected or changed: refresh the UI (it
/// re-reads the cached settings on "device-update") and the tray's output
/// item.
pub fn notify_device_changed(app: &AppHandle) {
    if let Err(error) = app.emit("device-update", ()) {
        log::error!("Failed to notify the UI of a device change: {error}");
    }
    crate::app::tray::sync_output_state(app);
}

#[tauri::command(async)]
pub fn connect_device(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();

    if manager.is_connected() {
        return Ok("Already connected".to_string());
    }

    manager.connect().map_err(|error| {
        log::error!("Connecting to the G6 failed: {error:#}");
        format!("{error:#}")
    })?;

    let app_clone = app.clone();
    manager.start_listener(move || notify_device_changed(&app_clone));
    crate::app::tray::sync_output_state(&app);

    Ok("Connected successfully".to_string())
}

#[tauri::command(async)]
pub fn disconnect_device(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.disconnect();
    crate::app::tray::sync_output_state(&app);
    Ok("Disconnected successfully".to_string())
}

#[tauri::command(async)]
pub fn is_device_connected(state: State<AppState>) -> bool {
    let manager = state.device_manager.lock().unwrap();
    manager.is_connected()
}

#[tauri::command(async)]
pub fn get_device_settings(state: State<AppState>) -> Result<G6Settings, String> {
    let manager = state.device_manager.lock().unwrap();
    Ok(manager.get_settings())
}

/// Device commands run off the main thread (`async`): a read of an
/// unresponsive G6 takes seconds while it heals the handle, and a command on
/// the main thread would freeze the window meanwhile.
#[tauri::command(async)]
pub fn read_device_state(app: AppHandle, state: State<AppState>) -> Result<G6Settings, String> {
    let result = {
        let manager = state.device_manager.lock().unwrap();
        manager.read_device_state().map_err(|e| format!("{:#}", e))
    };
    // A read may reopen or reset the device; tell the UI either way.
    notify_device_changed(&app);
    result
}
