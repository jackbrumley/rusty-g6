use crate::app::protocol_console::log_to_console;
use crate::app::state::AppState;
use crate::g6_spec::{EffectState, OutputDevice, ScoutModeState};
use log::info;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn toggle_output(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!("User initiated: Toggle Output");

    let manager = state.device_manager.lock().unwrap();
    let current = manager.get_settings().output;

    log_to_console(
        &app,
        "command",
        format!("🔄 Toggle Output (V2 Protocol) from {:?}", current),
        Some("Using 2-command sequence: routing + commit".to_string()),
    );

    match manager.toggle_output() {
        Ok(_) => {
            let new_settings = manager.get_settings();
            log_to_console(
                &app,
                "info",
                format!("✅ Output toggled to {:?}", new_settings.output),
                None,
            );
            Ok(format!("Output toggled to {:?}", new_settings.output))
        }
        Err(e) => {
            log_to_console(&app, "error", format!("❌ Toggle failed: {}", e), None);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_output(
    app: AppHandle,
    state: State<AppState>,
    output: OutputDevice,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!("User initiated: Set Output to {:?}", output);

    log_to_console(
        &app,
        "command",
        format!("📡 Set Output (V2 Protocol) to {:?}", output),
        Some("Using 2-command sequence: routing + commit".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_output(output) {
        Ok(_) => {
            log_to_console(&app, "info", format!("✅ Output set to {:?}", output), None);
            Ok(format!("Output set to {:?}", output))
        }
        Err(e) => {
            log_to_console(&app, "error", format!("❌ Set output failed: {}", e), None);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_surround(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!(
        "User initiated: Set Surround to {:?} (Value: {})",
        enabled, value
    );

    log_to_console(
        &app,
        "command",
        format!(
            "🔊 Set Surround (V2 Protocol): {:?}, Value: {}",
            enabled, value
        ),
        Some("Using DATA + COMMIT command sequence".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_surround(enabled, value) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Surround set to {:?} with value {}", enabled, value),
                None,
            );
            Ok(format!("Surround set to {:?} with value {}", enabled, value))
        }
        Err(e) => {
            log_to_console(&app, "error", format!("❌ Set surround failed: {}", e), None);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_crystalizer(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!(
        "User initiated: Set Crystalizer to {:?} (Value: {})",
        enabled, value
    );

    log_to_console(
        &app,
        "command",
        format!(
            "💎 Set Crystalizer (V2 Protocol): {:?}, Value: {}",
            enabled, value
        ),
        Some("Using DATA + COMMIT command sequence".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_crystalizer(enabled, value) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Crystalizer set to {:?} with value {}", enabled, value),
                None,
            );
            Ok(format!("Crystalizer set to {:?} with value {}", enabled, value))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set crystalizer failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_bass(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!(
        "User initiated: Set Bass to {:?} (Value: {})",
        enabled, value
    );

    log_to_console(
        &app,
        "command",
        format!("🎵 Set Bass (V2 Protocol): {:?}, Value: {}", enabled, value),
        Some("Using DATA + COMMIT command sequence".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_bass(enabled, value) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Bass set to {:?} with value {}", enabled, value),
                None,
            );
            Ok(format!("Bass set to {:?} with value {}", enabled, value))
        }
        Err(e) => {
            log_to_console(&app, "error", format!("❌ Set bass failed: {}", e), None);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_smart_volume(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!(
        "User initiated: Set Smart Volume to {:?} (Value: {})",
        enabled, value
    );

    log_to_console(
        &app,
        "command",
        format!(
            "🔉 Set Smart Volume (V2 Protocol): {:?}, Value: {}",
            enabled, value
        ),
        Some("Using DATA + COMMIT command sequence".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_smart_volume(enabled, value) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Smart Volume set to {:?} with value {}", enabled, value),
                None,
            );
            Ok(format!("Smart Volume set to {:?} with value {}", enabled, value))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set smart volume failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_dialog_plus(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!(
        "User initiated: Set Dialog Plus to {:?} (Value: {})",
        enabled, value
    );

    log_to_console(
        &app,
        "command",
        format!(
            "🗣️ Set Dialog Plus (V2 Protocol): {:?}, Value: {}",
            enabled, value
        ),
        Some("Using DATA + COMMIT command sequence".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_dialog_plus(enabled, value) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Dialog Plus set to {:?} with value {}", enabled, value),
                None,
            );
            Ok(format!("Dialog Plus set to {:?} with value {}", enabled, value))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set dialog plus failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_sbx_mode(
    app: AppHandle,
    state: State<AppState>,
    enabled: EffectState,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!("User initiated: Set SBX Mode to {:?}", enabled);

    log_to_console(
        &app,
        "command",
        format!("🎚️ Set SBX Mode (V2 Protocol): {:?}", enabled),
        Some("Master audio effects switch - DATA + COMMIT".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_sbx_mode(enabled) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ SBX Mode set to {:?}", enabled),
                None,
            );
            Ok(format!("SBX Mode set to {:?}", enabled))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set SBX mode failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_scout_mode(
    app: AppHandle,
    state: State<AppState>,
    enabled: ScoutModeState,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!("User initiated: Set Scout Mode to {:?}", enabled);

    log_to_console(
        &app,
        "command",
        format!("🎯 Set Scout Mode (V2 Protocol): {:?}", enabled),
        Some("Gaming audio enhancement - DATA + COMMIT".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_scout_mode(enabled) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Scout Mode set to {:?}", enabled),
                None,
            );
            Ok(format!("Scout Mode set to {:?}", enabled))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set scout mode failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn set_microphone_boost(
    app: AppHandle,
    state: State<AppState>,
    db_value: u8,
) -> Result<String, String> {
    info!("------------------------------------------------------------");
    info!("User initiated: Set Microphone Boost to {}dB", db_value);

    log_to_console(
        &app,
        "command",
        format!("🎤 Set Microphone Boost (V2 Protocol): {}dB", db_value),
        Some("Microphone input gain - DATA + COMMIT (0x3c family)".to_string()),
    );

    let manager = state.device_manager.lock().unwrap();
    match manager.set_microphone_boost(db_value) {
        Ok(_) => {
            log_to_console(
                &app,
                "info",
                format!("✅ Microphone Boost set to {}dB", db_value),
                None,
            );
            Ok(format!("Microphone Boost set to {}dB", db_value))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set microphone boost failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}
