mod g6_device;
mod g6_protocol_v2; // Unified protocol abstraction layer
mod g6_spec;

use g6_device::G6DeviceManager;
use g6_spec::{EffectState, G6Settings, OutputDevice, ProtocolConsoleMessage, ScoutModeState};
use log::info;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};

// Global protocol console storage
static PROTOCOL_CONSOLE: Mutex<Vec<ProtocolConsoleMessage>> = Mutex::new(Vec::new());

// Application state
struct AppState {
    device_manager: Mutex<G6DeviceManager>,
}

// Tauri Commands

#[tauri::command]
fn connect_device(app: AppHandle, state: State<AppState>) -> Result<String, String> {
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
    manager.connect().map_err(|e| {
        eprintln!("Connection error: {}", e);
        e.to_string()
    })?;

    // Start event listener for live updates
    let app_clone = app.clone();
    manager.start_listener(move || {
        // Emit event to frontend to trigger state refresh
        if let Err(e) = app_clone.emit("device-update", ()) {
            eprintln!("Failed to emit device update: {}", e);
        }
    });

    // Use enhanced synchronization that reads device state first
    manager
        .synchronize_with_device()
        .map(|_| "Connected and synchronized successfully".to_string())
        .map_err(|e| {
            eprintln!("Failed to synchronize with device: {}", e);
            // Device is connected but sync failed - still report success
            // but mention the issue
            format!("Connected but synchronization failed: {}", e)
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
fn toggle_output(app: AppHandle, state: State<AppState>) -> Result<String, String> {
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
fn set_output(
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
fn set_surround(
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
            Ok(format!(
                "Surround set to {:?} with value {}",
                enabled, value
            ))
        }
        Err(e) => {
            log_to_console(
                &app,
                "error",
                format!("❌ Set surround failed: {}", e),
                None,
            );
            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn set_crystalizer(
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
            Ok(format!(
                "Crystalizer set to {:?} with value {}",
                enabled, value
            ))
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
fn set_bass(
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
fn set_smart_volume(
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
            Ok(format!(
                "Smart Volume set to {:?} with value {}",
                enabled, value
            ))
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
fn set_dialog_plus(
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
            Ok(format!(
                "Dialog Plus set to {:?} with value {}",
                enabled, value
            ))
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
fn set_sbx_mode(
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
fn set_scout_mode(
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
fn set_microphone_boost(
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

#[tauri::command]
fn read_device_state(state: State<AppState>) -> Result<G6Settings, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.read_device_state().map_err(|e| e.to_string())
}

#[tauri::command]
fn synchronize_device(state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .synchronize_with_device()
        .map(|_| "Device synchronized successfully".to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_usb_devices(state: State<AppState>) -> Result<Vec<String>, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn clear_terminal(message: Option<String>) -> Result<String, String> {
    // Use ANSI escape codes to clear the terminal
    // This works on Windows 10+, Linux, and macOS
    print!("\x1b[2J\x1b[H");

    // Print a visible log separator marker with optional custom message
    info!("============================================================");
    if let Some(msg) = message {
        if !msg.trim().is_empty() {
            info!("LOG SEPARATOR: {}", msg.trim());
        } else {
            info!("LOG SEPARATOR - User requested terminal break");
        }
    } else {
        info!("LOG SEPARATOR - User requested terminal break");
    }
    info!("============================================================");

    Ok("Log separator added".to_string())
}

// Protocol Console Helper
fn log_to_console(app: &AppHandle, level: &str, text: String, details: Option<String>) {
    let msg = ProtocolConsoleMessage::new(level, text, details);
    PROTOCOL_CONSOLE.lock().unwrap().push(msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &msg) {
        eprintln!("Failed to emit console update: {}", e);
    }
}

// Protocol Console Commands

#[tauri::command]
fn get_protocol_console_messages() -> Vec<ProtocolConsoleMessage> {
    PROTOCOL_CONSOLE.lock().unwrap().clone()
}

#[tauri::command]
fn clear_protocol_console() -> Result<String, String> {
    PROTOCOL_CONSOLE.lock().unwrap().clear();
    Ok("Console cleared".to_string())
}

#[tauri::command]
fn test_protocol_v2(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    use crate::g6_protocol_v2::{build_firmware_query_ascii, G6ResponseParser};

    let manager = state.device_manager.lock().unwrap();

    // Check if connected
    if !manager.is_connected() {
        return Err("Device not connected".to_string());
    }

    eprintln!("=== TESTING PROTOCOL V2 ===");

    // Build firmware query command using v2
    let command = build_firmware_query_ascii();

    // Log command to console
    let cmd_hex: String = command
        .iter()
        .take(20)
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    let cmd_msg = ProtocolConsoleMessage::new(
        "command",
        "Firmware Query (ASCII Mode) - Protocol V2".to_string(),
        Some(format!("Command bytes: {}", cmd_hex)),
    );
    PROTOCOL_CONSOLE.lock().unwrap().push(cmd_msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &cmd_msg) {
        eprintln!("Failed to emit console update: {}", e);
    }

    eprintln!("Command (V2): {}", cmd_hex);

    // Send command to device
    let response = match manager.send_raw_command(&command) {
        Ok(resp) => resp,
        Err(e) => {
            let err_msg = ProtocolConsoleMessage::new(
                "error",
                format!("Failed to send command: {}", e),
                None,
            );
            PROTOCOL_CONSOLE.lock().unwrap().push(err_msg.clone());
            if let Err(e) = app.emit("protocol-console-update", &err_msg) {
                eprintln!("Failed to emit console update: {}", e);
            }
            return Err(format!("Failed to send command: {}", e));
        }
    };

    // Log raw response
    let resp_hex: String = response
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ");
    let resp_ascii: String = response
        .iter()
        .filter(|&&b| b >= 0x20 && b <= 0x7E)
        .map(|&b| b as char)
        .collect();

    let resp_msg = ProtocolConsoleMessage::new(
        "response",
        "Device response received".to_string(),
        Some(format!("Hex: {}\nASCII: {}", resp_hex, resp_ascii)),
    );
    PROTOCOL_CONSOLE.lock().unwrap().push(resp_msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &resp_msg) {
        eprintln!("Failed to emit console update: {}", e);
    }

    // Parse response using V2
    let (parsed, debug_info) = G6ResponseParser::parse(&response);

    // Log parsed result
    let parse_level = if parsed.is_ok() { "info" } else { "error" };
    let parse_text = match &parsed {
        Ok(p) => format!("✅ Parse SUCCESS: {:?}", p),
        Err(e) => format!("❌ Parse FAILED: {}", e),
    };

    let parse_msg =
        ProtocolConsoleMessage::new(parse_level, parse_text, Some(debug_info.to_readable_text()));
    PROTOCOL_CONSOLE.lock().unwrap().push(parse_msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &parse_msg) {
        eprintln!("Failed to emit console update: {}", e);
    }

    match parsed {
        Ok(_) => Ok("Protocol V2 test complete! Check console for full details.".to_string()),
        Err(e) => Ok(format!(
            "Test sent but parsing failed: {}. Check console for details.",
            e
        )),
    }
}

#[tauri::command]
fn test_output_toggle_v2(app: AppHandle, state: State<AppState>) -> Result<String, String> {
    use crate::g6_protocol_v2::build_toggle_output_simple;

    let manager = state.device_manager.lock().unwrap();

    if !manager.is_connected() {
        return Err("Device not connected".to_string());
    }

    let current_output = manager.get_settings().output;

    // Log start
    let start_msg = ProtocolConsoleMessage::new(
        "command",
        format!("🔄 Testing V2 Output Toggle from {:?}", current_output),
        Some("Simple 2-command version (routing + commit)".to_string()),
    );
    PROTOCOL_CONSOLE.lock().unwrap().push(start_msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &start_msg) {
        eprintln!("Failed to emit console update: {}", e);
    }

    // Build commands using V2
    let commands = build_toggle_output_simple(current_output);

    let info_msg = ProtocolConsoleMessage::new(
        "info",
        format!("Built {} commands (V2 minimal approach)", commands.len()),
        None,
    );
    PROTOCOL_CONSOLE.lock().unwrap().push(info_msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &info_msg) {
        eprintln!("Failed to emit console update: {}", e);
    }

    // Send each command and log
    for (i, cmd) in commands.iter().enumerate() {
        let cmd_hex: String = cmd
            .iter()
            .take(20)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let cmd_desc = if i == 0 {
            "Step 1: Set output routing"
        } else {
            "Step 2: Commit change"
        };

        let cmd_msg = ProtocolConsoleMessage::new(
            "command",
            cmd_desc.to_string(),
            Some(format!("Hex: {}", cmd_hex)),
        );
        PROTOCOL_CONSOLE.lock().unwrap().push(cmd_msg.clone());
        if let Err(e) = app.emit("protocol-console-update", &cmd_msg) {
            eprintln!("Failed to emit console update: {}", e);
        }

        // Send command
        let response = match manager.send_raw_command(cmd) {
            Ok(resp) => resp,
            Err(e) => {
                let err_msg = ProtocolConsoleMessage::new(
                    "error",
                    format!("❌ Step {} failed: {}", i + 1, e),
                    None,
                );
                PROTOCOL_CONSOLE.lock().unwrap().push(err_msg.clone());
                if let Err(e) = app.emit("protocol-console-update", &err_msg) {
                    eprintln!("Failed to emit console update: {}", e);
                }
                return Err(format!("Failed at step {}: {}", i + 1, e));
            }
        };

        // Log response
        let resp_hex: String = response
            .iter()
            .take(20)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");

        let resp_msg = ProtocolConsoleMessage::new(
            "response",
            format!("Response {}/{}", i + 1, commands.len()),
            Some(format!("Hex: {}", resp_hex)),
        );
        PROTOCOL_CONSOLE.lock().unwrap().push(resp_msg.clone());
        if let Err(e) = app.emit("protocol-console-update", &resp_msg) {
            eprintln!("Failed to emit console update: {}", e);
        }
    }

    // Success message
    let success_msg = ProtocolConsoleMessage::new(
        "info",
        "✅ V2 output toggle test complete!".to_string(),
        Some("Check if output actually switched. Try Read State to verify.".to_string()),
    );
    PROTOCOL_CONSOLE.lock().unwrap().push(success_msg.clone());
    if let Err(e) = app.emit("protocol-console-update", &success_msg) {
        eprintln!("Failed to emit console update: {}", e);
    }

    Ok(format!(
        "V2 toggle test sent ({} commands). Check Protocol Console for details.",
        commands.len()
    ))
}

fn create_tray_menu(app: &tauri::AppHandle) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&show, &quit])
}

#[tauri::command]
fn configure_microphone() -> Result<String, String> {
    use std::process::Command;

    eprintln!("=== Configuring G6 Microphone ===");

    // Try different possible card names
    let card_names = vec!["Sound BlasterX G6", "G6", "SoundBlasterXG6"];

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
                            .args(&[
                                "-c",
                                card_name,
                                "cset",
                                "name=PCM Capture Source",
                                "External Mic",
                            ])
                            .output()
                        {
                            if output.status.success() {
                                eprintln!("✓ PCM Capture Source set to External Mic");
                                return Ok(format!(
                                    "Microphone configured successfully on '{}'",
                                    card_name
                                ));
                            }
                        }
                    }
                }
            }
            Ok(_output) => {
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
    // Initialize logging with default level "info"
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Create device manager
    let device_manager = G6DeviceManager::new().expect("Failed to initialize G6 Device Manager");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            device_manager: Mutex::new(device_manager),
        })
        .setup(|app| {
            // Create tray menu
            let menu = create_tray_menu(app.handle())?;

            // Create tray icon
            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Rusty G6")
                .on_menu_event(|app_handle, event| match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if let tauri::tray::MouseButton::Left = button {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Handle window close event to hide instead of exit
            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent the window from closing
                        api.prevent_close();
                        // Hide the window instead
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_device,
            disconnect_device,
            is_device_connected,
            get_device_settings,
            read_device_state,
            synchronize_device,
            toggle_output,
            set_output,
            set_surround,
            set_crystalizer,
            set_bass,
            set_smart_volume,
            set_dialog_plus,
            set_sbx_mode,
            set_scout_mode,
            set_microphone_boost,
            list_usb_devices,
            get_app_version,
            configure_microphone,
            clear_terminal,
            get_protocol_console_messages,
            clear_protocol_console,
            test_protocol_v2,
            test_output_toggle_v2,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
