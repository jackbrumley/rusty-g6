use crate::app::state::AppState;
use serde::Serialize;
use std::thread;
use std::time::Duration;
use tauri::Emitter;
use tauri::Manager;

#[derive(Clone, Serialize)]
struct DeviceAvailabilityEvent {
    available: bool,
}

pub fn configure_shell(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
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
                let _ = app_handle.emit(
                    "device-availability",
                    DeviceAvailabilityEvent { available },
                );
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

    Ok(())
}
