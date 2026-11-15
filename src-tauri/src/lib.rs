mod app;
mod g6_device;
#[cfg(target_os = "linux")]
mod g6_mixer;
mod g6_protocol;
mod g6_spec;
mod paths;

pub use app::bootstrap::LaunchOptions;
use app::commands::*;
#[cfg(target_os = "linux")]
pub use app::linux_env::configure_linux_session_environment;
pub use app::session_log::install_panic_hook;
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
            toggle_output,
            set_sbx_enabled,
            set_scout_mode_enabled,
            set_effect_enabled,
            set_effect_level,
            set_microphone_boost,
            set_mic_switch,
            set_noise_reduction_level,
            set_dac_filter,
            set_eq_enabled,
            set_eq_preamp,
            set_eq_band,
            set_eq_bands,
            get_app_version,
            get_recording_source,
            set_recording_source,
            get_mic_level,
            set_mic_level,
            setup_udev_rules,
            check_usb_permissions,
            copy_session_log,
            open_session_log,
            open_log_folder,
            get_system_info,
            get_log_level,
            set_log_level,
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
