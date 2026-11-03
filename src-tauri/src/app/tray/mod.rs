//! Owns the tray icon, tray availability, and every way the main window is
//! shown, parked or quit (Release and Update Standard, "App Shell"). The tray
//! backends differ per platform (Tauri's tray on Windows, a
//! StatusNotifierItem on Linux) but share one menu, one click behavior and one
//! set of actions defined here:
//! - left click opens the main window;
//! - right click shows the menu (Open Rusty G6 / output switch / Quit).

#[cfg(target_os = "linux")]
mod status_notifier;
#[cfg(not(target_os = "linux"))]
mod system_tray;

use crate::app::state::AppState;
use crate::g6_spec::OutputDevice;
use serde::Serialize;
use tauri::Manager;

/// Where the main window went when it was taken off screen without quitting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowParking {
    Tray,
    Taskbar,
}

/// What the tray's output item can offer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrayOutputState {
    /// No G6 connected: the item is shown disabled.
    Disconnected,
    /// Connected, but the device has not confirmed its output yet; switching
    /// reads the device first.
    Unknown,
    /// The output the device reported.
    Current(OutputDevice),
}

/// The tray menu, in display order. Both tray backends build their menu from
/// this list and route selections through `handle_menu_item`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TrayMenuItem {
    Open,
    /// Rusty G6's app-specific item: switch between speakers and headphones.
    SwitchOutput,
    Quit,
}

impl TrayMenuItem {
    const ALL: [Self; 3] = [Self::Open, Self::SwitchOutput, Self::Quit];

    fn label(self, output: TrayOutputState) -> &'static str {
        match self {
            Self::Open => "Open Rusty G6",
            Self::SwitchOutput => match output {
                TrayOutputState::Disconnected => "No G6 connected",
                TrayOutputState::Unknown => "Switch Speakers/Headphones",
                TrayOutputState::Current(OutputDevice::Speakers) => "Switch to Headphones",
                TrayOutputState::Current(OutputDevice::Headphones) => "Switch to Speakers",
            },
            Self::Quit => "Quit",
        }
    }

    fn enabled(self, output: TrayOutputState) -> bool {
        !matches!(
            (self, output),
            (Self::SwitchOutput, TrayOutputState::Disconnected)
        )
    }
}

fn handle_menu_item(app_handle: &tauri::AppHandle, item: TrayMenuItem) {
    match item {
        TrayMenuItem::Open => show_main_window(app_handle),
        TrayMenuItem::SwitchOutput => {
            let app_handle = app_handle.clone();
            // Device I/O blocks; keep it off the tray and UI threads.
            tauri::async_runtime::spawn_blocking(move || {
                log::info!("Tray: switch output requested");
                if let Err(error) = crate::app::commands::audio::perform_toggle_output(&app_handle)
                {
                    log::warn!("Tray: switching output failed: {}", error);
                }
                crate::app::commands::device::notify_device_changed(&app_handle);
            });
        }
        TrayMenuItem::Quit => quit(app_handle),
    }
}

/// Creates the tray icon for this platform.
pub fn build_tray(app: &tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    let icon = app
        .default_window_icon()
        .cloned()
        .map(tauri::image::Image::to_owned)
        .ok_or("Rusty G6 has no bundled icon to show in the system tray")?;

    #[cfg(target_os = "linux")]
    status_notifier::spawn(app.handle(), &icon);

    #[cfg(not(target_os = "linux"))]
    system_tray::build(app, icon)?;

    Ok(())
}

/// Brings the tray's output item in line with the device. Called whenever
/// the device connects, disconnects or reports a change.
pub fn sync_output_state(app_handle: &tauri::AppHandle) {
    let app_handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let reader = app_handle.clone();
        let state = tauri::async_runtime::spawn_blocking(move || {
            let app_state = reader.state::<AppState>();
            let manager = app_state.device_manager.lock().unwrap();
            if !manager.is_connected() {
                TrayOutputState::Disconnected
            } else if !manager.is_state_confirmed() {
                TrayOutputState::Unknown
            } else {
                TrayOutputState::Current(manager.get_settings().output)
            }
        })
        .await;
        let Ok(state) = state else {
            return;
        };

        #[cfg(target_os = "linux")]
        status_notifier::set_output_state(state).await;

        #[cfg(not(target_os = "linux"))]
        system_tray::set_output_state(&app_handle, state);
    });
}

/// Routes every close request (title-bar button, Alt+F4, taskbar "Close
/// window") through `close_main_window` so the `close_to_tray` setting is
/// honored no matter how the window was closed.
pub fn intercept_main_window_close(app: &tauri::App<tauri::Wry>) {
    let Some(window) = app.get_webview_window("main") else {
        log::warn!("Main window not found; close-to-tray handling not installed");
        return;
    };
    let app_handle = app.handle().clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = close_main_window(&app_handle).await {
                    log::warn!("Failed to close main window: {}", error);
                }
            });
        }
    });
}

/// Whether a system tray host is present to receive the tray icon. Windows
/// always provides one; Linux depends on a StatusNotifier watcher (absent on
/// stock GNOME without the AppIndicator extension).
pub async fn is_tray_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        status_notifier::is_watcher_available().await
    }

    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

/// Takes the main window off screen without quitting: hidden to the tray
/// when one exists, otherwise minimized to the taskbar so it stays reachable.
pub async fn park_main_window(app_handle: &tauri::AppHandle) -> Result<WindowParking, String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    if is_tray_available().await {
        window.hide().map_err(|error| error.to_string())?;
        return Ok(WindowParking::Tray);
    }

    window.minimize().map_err(|error| error.to_string())?;
    Ok(WindowParking::Taskbar)
}

/// Closes the main window according to the `close_to_tray` setting. Returns
/// where the window was parked, or `None` when the application is quitting.
pub async fn close_main_window(
    app_handle: &tauri::AppHandle,
) -> Result<Option<WindowParking>, String> {
    let close_to_tray = app_handle
        .state::<AppState>()
        .settings
        .lock()
        .unwrap()
        .close_to_tray;

    if !close_to_tray {
        quit(app_handle);
        return Ok(None);
    }

    let parking = park_main_window(app_handle).await?;
    if parking == WindowParking::Taskbar {
        log::warn!(
            "Close to tray requested but no system tray is available; minimized to taskbar instead"
        );
    }
    Ok(Some(parking))
}

/// The single path for bringing Rusty G6 forward: tray click, tray menu and a
/// second launch all end here.
pub fn show_main_window(app_handle: &tauri::AppHandle) {
    let Some(window) = app_handle.get_webview_window("main") else {
        log::warn!("Cannot show the main window: window not found");
        return;
    };
    if let Err(error) = window.unminimize() {
        log::warn!("Failed to unminimize the main window: {}", error);
    }
    if let Err(error) = window.show() {
        log::warn!("Failed to show the main window: {}", error);
    }
    if let Err(error) = window.set_focus() {
        log::warn!("Failed to focus the main window: {}", error);
    }
}

/// A second launch (start menu, desktop file, autostart) hands its arguments
/// to this instance and exits. Bring the window forward unless that launch
/// asked to start hidden.
pub fn handle_second_launch(app_handle: &tauri::AppHandle, show_window: bool) {
    if show_window {
        log::info!("Second launch detected; showing the main window");
        show_main_window(app_handle);
    } else {
        log::info!("Second launch detected with --start-hidden; leaving the window as is");
    }
}

/// Exits the app; the `RunEvent::Exit` handler releases its resources.
pub fn quit(app_handle: &tauri::AppHandle) {
    app_handle.exit(0);
}

#[cfg(test)]
mod tests {
    use super::{TrayMenuItem, TrayOutputState};
    use crate::g6_spec::OutputDevice;

    #[test]
    fn menu_lists_open_first_and_quit_last() {
        assert_eq!(TrayMenuItem::ALL.first(), Some(&TrayMenuItem::Open));
        assert_eq!(TrayMenuItem::ALL.last(), Some(&TrayMenuItem::Quit));
    }

    #[test]
    fn output_item_offers_the_other_output() {
        let item = TrayMenuItem::SwitchOutput;
        let speakers = TrayOutputState::Current(OutputDevice::Speakers);
        let headphones = TrayOutputState::Current(OutputDevice::Headphones);
        assert_eq!(item.label(speakers), "Switch to Headphones");
        assert_eq!(item.label(headphones), "Switch to Speakers");
        assert!(item.enabled(speakers) && item.enabled(TrayOutputState::Unknown));
    }

    #[test]
    fn output_item_is_disabled_without_a_device() {
        let item = TrayMenuItem::SwitchOutput;
        assert!(!item.enabled(TrayOutputState::Disconnected));
        assert_eq!(item.label(TrayOutputState::Disconnected), "No G6 connected");
        assert!(TrayMenuItem::Open.enabled(TrayOutputState::Disconnected));
        assert!(TrayMenuItem::Quit.enabled(TrayOutputState::Disconnected));
    }
}
