mod g6_spec;
mod g6_device;
mod g6_protocol;

use g6_device::G6DeviceManager;
use g6_spec::{G6Settings, OutputDevice, EffectState};
use std::sync::Mutex;
use tauri::State;

// Application state
struct AppState {
    device_manager: Mutex<G6DeviceManager>,
}

// Tauri Commands

#[tauri::command]
fn connect_device(state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    
    // List all devices first for debugging
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
    
    // Connect to the device
    manager.connect()
        .map_err(|e| {
            eprintln!("Connection error: {}", e);
            e.to_string()
        })?;
    
    // Apply all saved settings to the device
    manager.apply_all_settings()
        .map(|_| "Connected and settings applied successfully".to_string())
        .map_err(|e| {
            eprintln!("Failed to apply settings: {}", e);
            // Device is connected but settings failed - still report success
            // but mention the issue
            format!("Connected but failed to apply settings: {}", e)
        })
}

#[tauri::command]
fn disconnect_device(state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.disconnect();
    Ok("Disconnected successfully".to_string())
}

#[tauri::command]
fn is_device_connected(state: State<AppState>) -> bool {
    let manager = state.device_manager.lock().unwrap();
    manager.is_connected()
}

#[tauri::command]
fn get_device_settings(state: State<AppState>) -> Result<G6Settings, String> {
    let manager = state.device_manager.lock().unwrap();
    Ok(manager.get_settings())
}

#[tauri::command]
fn toggle_output(state: State<AppState>) -> Result<String, String> {
    eprintln!("=== Toggle Output Called ===");
    let manager = state.device_manager.lock().unwrap();
    
    let current_settings = manager.get_settings();
    eprintln!("Current output: {:?}", current_settings.output);
    
    match manager.toggle_output() {
        Ok(_) => {
            let new_settings = manager.get_settings();
            eprintln!("Output toggled successfully to: {:?}", new_settings.output);
            Ok(format!("Output toggled to {:?}", new_settings.output))
        }
        Err(e) => {
            eprintln!("Toggle output error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn set_output(state: State<AppState>, output: OutputDevice) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.set_output(output)
        .map(|_| format!("Output set to {:?}", output))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_surround(state: State<AppState>, enabled: EffectState, value: u8) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.set_surround(enabled, value)
        .map(|_| format!("Surround set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_crystalizer(state: State<AppState>, enabled: EffectState, value: u8) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.set_crystalizer(enabled, value)
        .map(|_| format!("Crystalizer set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_bass(state: State<AppState>, enabled: EffectState, value: u8) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.set_bass(enabled, value)
        .map(|_| format!("Bass set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_smart_volume(state: State<AppState>, enabled: EffectState, value: u8) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.set_smart_volume(enabled, value)
        .map(|_| format!("Smart Volume set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_dialog_plus(state: State<AppState>, enabled: EffectState, value: u8) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.set_dialog_plus(enabled, value)
        .map(|_| format!("Dialog Plus set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_usb_devices(state: State<AppState>) -> Result<Vec<String>, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.list_devices()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn configure_microphone() -> Result<String, String> {
    use std::process::Command;
    
    eprintln!("=== Configuring G6 Microphone ===");
    
    // Try different possible card names
    let card_names = vec![
        "Sound BlasterX G6",
        "G6",
        "SoundBlasterXG6",
    ];
    
    let mut last_error = String::new();
    
    for card_name in &card_names {
        eprintln!("Trying card name: {}", card_name);
        
        // Try to set Line In capture
        match Command::new("amixer")
            .args(&["-c", card_name, "sset", "Line In", "cap"])
            .output()
        {
            Ok(output) if output.status.success() => {
                eprintln!("✓ Line In capture enabled");
                
                // Set External Mic capture
                if let Ok(output) = Command::new("amixer")
                    .args(&["-c", card_name, "sset", "External Mic", "cap"])
                    .output()
                {
                    if output.status.success() {
                        eprintln!("✓ External Mic capture enabled");
                        
                        // Set PCM Capture Source to External Mic
                        if let Ok(output) = Command::new("amixer")
                            .args(&["-c", card_name, "cset", "name=PCM Capture Source", "External Mic"])
                            .output()
                        {
                            if output.status.success() {
                                eprintln!("✓ PCM Capture Source set to External Mic");
                                return Ok(format!("Microphone configured successfully on '{}'", card_name));
                            }
                        }
                    }
                }
            }
            Ok(output) => {
                last_error = format!("Card '{}' found but configuration failed", card_name);
                eprintln!("{}", last_error);
            }
            Err(e) => {
                last_error = format!("Error with card '{}': {}", card_name, e);
                eprintln!("{}", last_error);
            }
        }
    }
    
    // If we got here, all attempts failed
    Err(format!(
        "Failed to configure microphone. Make sure 'amixer' is installed and the G6 is connected. Last error: {}",
        last_error
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    env_logger::init();
    
    // Create device manager
    let device_manager = G6DeviceManager::new()
        .expect("Failed to initialize G6 Device Manager");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            device_manager: Mutex::new(device_manager),
        })
        .invoke_handler(tauri::generate_handler![
            connect_device,
            disconnect_device,
            is_device_connected,
            get_device_settings,
            toggle_output,
            set_output,
            set_surround,
            set_crystalizer,
            set_bass,
            set_smart_volume,
            set_dialog_plus,
            list_usb_devices,
            get_app_version,
            configure_microphone,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
