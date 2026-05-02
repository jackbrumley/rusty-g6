use crate::app::state::AppState;
use crate::g6_spec::{EffectState, G6Settings, OutputDevice};
use serde::{Deserialize, Serialize};
use tauri::{Manager, State};

const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/jackbrumley/rusty-g6/releases/latest";
const GITHUB_RELEASES_LATEST_URL: &str = "https://github.com/jackbrumley/rusty-g6/releases/latest";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_url: String,
    notes_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubLatestRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
}

#[tauri::command]
pub fn connect_device(state: State<AppState>) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();

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

    manager
        .connect()
        .map(|_| "Connected successfully".to_string())
        .map_err(|e| {
            eprintln!("Connection error: {}", e);
            e.to_string()
        })
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
pub fn toggle_output(state: State<AppState>) -> Result<String, String> {
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
pub fn set_output(state: State<AppState>, output: OutputDevice) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_output(output)
        .map(|_| format!("Output set to {:?}", output))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_surround(
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_surround(enabled, value)
        .map(|_| format!("Surround set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_crystalizer(
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_crystalizer(enabled, value)
        .map(|_| format!("Crystalizer set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_bass(state: State<AppState>, enabled: EffectState, value: u8) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_bass(enabled, value)
        .map(|_| format!("Bass set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_smart_volume(
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_smart_volume(enabled, value)
        .map(|_| format!("Smart Volume set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_dialog_plus(
    state: State<AppState>,
    enabled: EffectState,
    value: u8,
) -> Result<String, String> {
    let manager = state.device_manager.lock().unwrap();
    manager
        .set_dialog_plus(enabled, value)
        .map(|_| format!("Dialog Plus set to {:?} with value {}", enabled, value))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_usb_devices(state: State<AppState>) -> Result<Vec<String>, String> {
    let manager = state.device_manager.lock().unwrap();
    manager.list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_device_state(state: State<AppState>) -> Result<String, String> {
    eprintln!("=== Read Device State Called ===");
    let manager = state.device_manager.lock().unwrap();

    match manager.read_device_state() {
        Ok(response) => {
            let hex_string = response
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("Device state response: {}", hex_string);
            Ok(hex_string)
        }
        Err(e) => {
            eprintln!("Read device state error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn read_full_device_state(state: State<AppState>) -> Result<Vec<(String, String)>, String> {
    eprintln!("=== Read Full Device State Called ===");
    let manager = state.device_manager.lock().unwrap();

    match manager.read_full_device_state() {
        Ok(results) => {
            let formatted: Vec<(String, String)> = results
                .iter()
                .map(|(name, data)| {
                    let hex_string = data
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<Vec<_>>()
                        .join(" ");
                    (name.clone(), hex_string)
                })
                .collect();

            eprintln!("Read {} command responses", formatted.len());
            Ok(formatted)
        }
        Err(e) => {
            eprintln!("Read full device state error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub fn setup_microphone(state: State<AppState>) -> Result<String, String> {
    eprintln!("=== Setup Microphone Called ===");
    state.platform_backend.setup_microphone()
}

#[tauri::command]
pub fn get_microphone_status(state: State<AppState>) -> Result<String, String> {
    state.platform_backend.get_microphone_status()
}

#[tauri::command]
pub fn minimize_to_tray_or_taskbar(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> Result<String, String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    let tray_available = *state.tray_available.lock().unwrap();
    if tray_available {
        window.hide().map_err(|error| error.to_string())?;
        Ok("tray".to_string())
    } else {
        window.minimize().map_err(|error| error.to_string())?;
        Ok("taskbar".to_string())
    }
}

#[tauri::command]
pub fn quit_application(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();

    let response = reqwest::Client::builder()
        .build()
        .map_err(|error| format!("Failed to build update client: {error}"))?
        .get(GITHUB_LATEST_RELEASE_API_URL)
        .header(reqwest::header::USER_AGENT, "rusty-g6-update-checker")
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| format!("Failed to fetch latest release: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Latest release request failed with status {}",
            response.status()
        ));
    }

    let latest_release: GitHubLatestRelease = response
        .json()
        .await
        .map_err(|error| format!("Failed to decode latest release response: {error}"))?;

    let latest_version = normalize_version(&latest_release.tag_name)
        .ok_or_else(|| format!("Invalid latest release version tag: {}", latest_release.tag_name))?;

    let parsed_current = parse_version(&current_version)
        .ok_or_else(|| format!("Invalid current app version: {current_version}"))?;
    let parsed_latest = parse_version(&latest_version)
        .ok_or_else(|| format!("Invalid latest app version: {latest_version}"))?;

    let update_available = parsed_latest > parsed_current;

    Ok(UpdateCheckResult {
        current_version,
        latest_version,
        update_available,
        release_url: latest_release.html_url,
        notes_url: latest_release
            .body
            .map(|_| GITHUB_RELEASES_LATEST_URL.to_string()),
    })
}

fn normalize_version(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches('v');
    let parsed = parse_version(trimmed)?;
    Some(format!("{}.{}.{}", parsed.0, parsed.1, parsed.2))
}

fn parse_version(value: &str) -> Option<(u64, u64, u64)> {
    let sanitized = value.trim();
    if sanitized.is_empty() {
        return None;
    }

    let core = sanitized.split_once('-').map_or(sanitized, |(left, _)| left);
    let mut segments = core.split('.');

    let major = segments.next()?.parse::<u64>().ok()?;
    let minor = segments.next()?.parse::<u64>().ok()?;
    let patch = segments.next()?.parse::<u64>().ok()?;

    if segments.next().is_some() {
        return None;
    }

    Some((major, minor, patch))
}
