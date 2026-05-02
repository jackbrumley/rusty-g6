mod app;
mod g6_device;
mod g6_protocol_v2;
mod g6_spec;

use app::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(app::state::create_app_state())
        .setup(app::bootstrap::configure_shell)
        .invoke_handler(tauri::generate_handler![
            connect_device,
            disconnect_device,
            quit_application,
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
            setup_udev_rules,
            check_usb_permissions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
