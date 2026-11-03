use crate::app::state::AppState;
use crate::app::tray;
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;

#[derive(Clone, Serialize)]
struct DeviceAvailabilityEvent {
    available: bool,
}

/// How this instance was launched (from the command line).
pub struct LaunchOptions {
    /// Autostart: stay in the tray.
    pub start_hidden: bool,
    /// The in-app updater relaunched this version because the install failed.
    pub update_failed: bool,
}

pub fn run_setup(
    app: &mut tauri::App<tauri::Wry>,
    launch: LaunchOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    // Runs after the single-instance guard, so a second launch never rotates
    // the running instance's session log or opens the device.
    if let Err(error) = crate::app::session_log::initialize_session_logging() {
        eprintln!("Failed to initialize session logging: {}", error);
    }
    if let Err(error) = crate::app::session_log::initialize_app_logger() {
        eprintln!("Failed to initialize app logger: {}", error);
    }
    app.manage(crate::app::state::create_app_state());

    configure_shell(app, &launch)?;
    refresh_autostart_entry(app);
    start_device_availability_watch(app);

    if launch.update_failed {
        crate::app::commands::update::show_update_failed_dialog(app.handle());
    }
    Ok(())
}

/// Tray icon, close-to-tray and the start-hidden decision. The main window
/// is created hidden (`visible: false`) and shown here unless this launch
/// should stay in the tray.
fn configure_shell(
    app: &mut tauri::App<tauri::Wry>,
    launch: &LaunchOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    tray::build_tray(app)?;
    tray::intercept_main_window_close(app);

    let tray_available = tauri::async_runtime::block_on(tray::is_tray_available());
    log::info!(
        "System tray available: {}; start hidden: {}",
        tray_available,
        launch.start_hidden
    );
    if launch.start_hidden && !tray_available {
        log::warn!(
            "Start hidden requested but no system tray is available; showing the main window so the app stays reachable"
        );
    }
    if !launch.start_hidden || !tray_available {
        tray::show_main_window(app.handle());
    }
    Ok(())
}

/// Autostart entries written before v1.0.9 launch without `--start-hidden`.
/// Re-registering an enabled entry rewrites it with the current arguments.
fn refresh_autostart_entry(app: &tauri::App<tauri::Wry>) {
    use tauri_plugin_autostart::ManagerExt;

    let autolaunch = app.autolaunch();
    match autolaunch.is_enabled() {
        Ok(true) => match autolaunch.enable() {
            Ok(()) => log::info!("Autostart entry refreshed (launches with --start-hidden)"),
            Err(error) => log::warn!("Failed to refresh the autostart entry: {}", error),
        },
        Ok(false) => {}
        Err(error) => log::warn!("Failed to check the autostart entry: {}", error),
    }
}

fn start_device_availability_watch(app: &tauri::App<tauri::Wry>) {
    let app_handle = app.handle().clone();

    thread::spawn(move || {
        let mut last_available: Option<bool> = None;

        loop {
            let (available, connected) = {
                let app_state = app_handle.state::<AppState>();
                let manager = app_state.device_manager.lock().unwrap();
                let available = manager.is_g6_interface_available().unwrap_or(false);
                let connected = manager.is_connected();
                (available, connected)
            };

            if last_available != Some(available) {
                let _ =
                    app_handle.emit("device-availability", DeviceAvailabilityEvent { available });
                tray::sync_output_state(&app_handle);
                last_available = Some(available);
            }

            if available && !connected {
                let _ = app_handle.emit(
                    "device-availability",
                    DeviceAvailabilityEvent { available: true },
                );
            }

            thread::sleep(Duration::from_millis(500));
        }
    });
}
