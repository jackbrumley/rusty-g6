mod app;
mod g6_device;
mod g6_protocol_v2;
mod g6_spec;

pub use app::bootstrap::LaunchOptions;
use app::commands::*;
#[cfg(target_os = "linux")]
pub use app::linux_env::configure_linux_session_environment;
use tauri::Manager;

/// `second_launch_shows_window` decides, from a second launch's arguments,
/// whether the running instance should show its window (`main.rs` owns the
/// command line).
pub fn run(launch: LaunchOptions, second_launch_shows_window: fn(&[String]) -> bool) {
    // Process-local setup (the Linux environment) happens in `main`.
    // Everything that touches the user's files, session log or the device
    // runs in setup, after the single-instance guard.
    tauri::Builder::default()
        // Must be the first plugin: a second launch hands its arguments to the
        // running instance and exits here, before it can rotate that
        // instance's session log or open the device.
        .plugin(tauri_plugin_single_instance::init(
            move |app, args, _cwd| {
                app::tray::handle_second_launch(app, second_launch_shows_window(&args));
            },
        ))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("rusty-g6")
                .arg("--start-hidden")
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| app::bootstrap::run_setup(app, launch))
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
            get_session_log_text,
            open_session_log,
            log_ui_event,
            check_for_updates,
            install_update,
            minimize_to_tray_or_taskbar,
            close_main_window,
            get_app_settings,
            set_close_to_tray,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // The single shutdown owner: every exit (tray Quit, close with
            // close-to-tray off, update install, OS logout) passes through
            // here, so the device handle is always released.
            if let tauri::RunEvent::Exit = event {
                // Absent only when setup failed before managing the state.
                if let Some(state) = app_handle.try_state::<app::state::AppState>() {
                    if let Ok(manager) = state.device_manager.lock() {
                        manager.disconnect();
                    }
                }
            }
        });
}
