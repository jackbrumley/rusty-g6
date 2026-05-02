mod app;
mod g6_device;
mod g6_protocol;
mod g6_spec;
pub mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app::state::build_app_state())
        .setup(app::bootstrap::setup)
        .invoke_handler(tauri::generate_handler![
            app::commands::connect_device,
            app::commands::disconnect_device,
            app::commands::is_device_connected,
            app::commands::get_device_settings,
            app::commands::toggle_output,
            app::commands::set_output,
            app::commands::set_surround,
            app::commands::set_crystalizer,
            app::commands::set_bass,
            app::commands::set_smart_volume,
            app::commands::set_dialog_plus,
            app::commands::list_usb_devices,
            app::commands::read_device_state,
            app::commands::read_full_device_state,
            app::commands::setup_microphone,
            app::commands::get_microphone_status,
            app::commands::minimize_to_tray_or_taskbar,
            app::commands::quit_application,
            app::commands::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
