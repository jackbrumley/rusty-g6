use crate::app::state::AppState;
use crate::g6_spec::G6Settings;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn connect_device(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();

    if manager.is_connected() {
        return Ok("Already connected".to_string());
    }

    match manager.list_devices() {
        Ok(devices) => {
            eprintln!("=== USB Devices Found ===");
            for device in &devices {
                eprintln!("{}", device);
            }
            eprintln!("========================");
        }
        Err(e) => eprintln!("Failed to list devices: {}", e),
    }

    manager.connect().map_err(|e| {
        eprintln!("Connection error: {}", e);
        e.to_string()
    })?;

    let app_clone = app.clone();
    manager.start_listener(move || {
        if let Err(e) = app_clone.emit("device-update", ()) {
            eprintln!("Failed to emit device update: {}", e);
        }
    });

    Ok("Connected successfully".to_string())
}

#[tauri::command]
pub fn disconnect_device(state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.disconnect();
    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
pub fn is_device_connected(state: State<AppState>) -> bool {
    let manager = state.device_manager.lock().unwrap();
    manager.is_connected()
}

#[tauri::command]
pub fn get_device_settings(state: State<AppState>) -> Result<G6Settings, String> {
    let manager = state.device_manager.lock().unwrap();
    Ok(manager.get_settings())
}

#[tauri::command]
pub fn read_device_state(state: State<AppState>) -> Result<G6Settings, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.read_device_state().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn synchronize_device(state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .synchronize_with_device()
        .map(|_| "Device synchronized successfully".to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_usb_devices(state: State<AppState>) -> Result<Vec<String>, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.list_devices().map_err(|e| e.to_string())
}
