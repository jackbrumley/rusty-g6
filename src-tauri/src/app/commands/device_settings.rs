//! Commands that change a device setting. They run off the main thread
//! (`async`): device I/O blocks, and an unresponsive G6 takes seconds to heal.
use crate::app::commands::device::notify_device_changed;
use crate::app::state::AppState;
use crate::g6_device::G6DeviceManager;
use crate::g6_spec::{DecoderMode, DeviceFlag, DigitalFilter, MicSwitch, RgbColor, SbxEffect};
use tauri::{AppHandle, Manager};

/// Run one setting change: log it, report a failure as the command's error
/// (and in the session log), and tell the UI and tray the cached settings
/// changed. The one path for every setting command, the UI's and the tray's.
pub fn change_device_setting(
    app: &AppHandle,
    change: &str,
    action: impl FnOnce(&G6DeviceManager) -> anyhow::Result<()>,
) -> Result<(), String> {
    log::info!("User requested: {change}");
    let result = {
        let state = app.state::<AppState>();
        let manager = state.device_manager.lock().unwrap();
        action(&manager)
    };
    notify_device_changed(app);
    result.map_err(|error| {
        log::error!("{change} failed: {error:#}");
        format!("{error:#}")
    })
}

#[tauri::command(async)]
pub fn toggle_output(app: AppHandle) -> Result<(), String> {
    change_device_setting(&app, "Switch output", |device| device.toggle_output())
}

#[tauri::command(async)]
pub fn set_sbx_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("SBX enabled = {enabled}"), |device| {
        device.set_sbx_enabled(enabled)
    })
}

#[tauri::command(async)]
pub fn set_scout_mode_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("Scout Mode enabled = {enabled}"), |device| {
        device.set_scout_mode_enabled(enabled)
    })
}

#[tauri::command(async)]
pub fn set_effect_enabled(app: AppHandle, effect: SbxEffect, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("{effect:?} enabled = {enabled}"), |device| {
        device.set_effect_enabled(effect, enabled)
    })
}

#[tauri::command(async)]
pub fn set_effect_level(app: AppHandle, effect: SbxEffect, level: u8) -> Result<(), String> {
    change_device_setting(&app, &format!("{effect:?} level = {level}"), |device| {
        device.set_effect_level(effect, level)
    })
}

#[tauri::command(async)]
pub fn set_microphone_boost(app: AppHandle, db_value: u8) -> Result<(), String> {
    change_device_setting(&app, &format!("Mic boost = {db_value} dB"), |device| {
        device.set_microphone_boost(db_value)
    })
}

#[tauri::command(async)]
pub fn set_mic_switch(app: AppHandle, switch: MicSwitch, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("{switch:?} enabled = {enabled}"), |device| {
        device.set_mic_switch(switch, enabled)
    })
}

/// All eight mic EQ bands at once (a preset), lowest band first.
#[tauri::command(async)]
pub fn set_mic_eq_bands(app: AppHandle, gains_db: [f32; 8]) -> Result<(), String> {
    change_device_setting(&app, &format!("Mic EQ bands = {gains_db:?} dB"), |device| {
        device.set_mic_eq_bands(gains_db)
    })
}

#[tauri::command(async)]
pub fn set_noise_reduction_level(app: AppHandle, level: u8) -> Result<(), String> {
    change_device_setting(
        &app,
        &format!("Noise reduction level = {level}"),
        |device| device.set_noise_reduction_level(level),
    )
}

#[tauri::command(async)]
pub fn set_device_flag(app: AppHandle, flag: DeviceFlag, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("{flag:?} enabled = {enabled}"), |device| {
        device.set_device_flag(flag, enabled)
    })
}

#[tauri::command(async)]
pub fn set_decoder_mode(app: AppHandle, mode: DecoderMode) -> Result<(), String> {
    change_device_setting(&app, &format!("Decoder mode = {mode:?}"), |device| {
        device.set_decoder_mode(mode)
    })
}

#[tauri::command(async)]
pub fn set_lighting_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("Lighting enabled = {enabled}"), |device| {
        device.set_lighting_enabled(enabled)
    })
}

#[tauri::command(async)]
pub fn set_lighting_color(app: AppHandle, color: RgbColor) -> Result<(), String> {
    change_device_setting(&app, &format!("Lighting colour = {color:?}"), |device| {
        device.set_lighting_color(color)
    })
}

#[tauri::command(async)]
pub fn set_dac_filter(app: AppHandle, filter: DigitalFilter) -> Result<(), String> {
    change_device_setting(&app, &format!("DAC filter = {filter:?}"), |device| {
        device.set_dac_filter(filter)
    })
}

#[tauri::command(async)]
pub fn set_eq_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    change_device_setting(&app, &format!("EQ enabled = {enabled}"), |device| {
        device.set_eq_enabled(enabled)
    })
}

#[tauri::command(async)]
pub fn set_eq_preamp(app: AppHandle, gain_db: f32) -> Result<(), String> {
    change_device_setting(&app, &format!("EQ preamp = {gain_db} dB"), |device| {
        device.set_eq_preamp(gain_db)
    })
}

/// `band` is 0 (31 Hz) to 9 (16 kHz).
#[tauri::command(async)]
pub fn set_eq_band(app: AppHandle, band: usize, gain_db: f32) -> Result<(), String> {
    change_device_setting(&app, &format!("EQ band {band} = {gain_db} dB"), |device| {
        device.set_eq_band(band, gain_db)
    })
}

/// All ten bands at once (a preset).
#[tauri::command(async)]
pub fn set_eq_bands(app: AppHandle, gains_db: [f32; 10]) -> Result<(), String> {
    change_device_setting(&app, &format!("EQ bands = {gains_db:?} dB"), |device| {
        device.set_eq_bands(gains_db)
    })
}
