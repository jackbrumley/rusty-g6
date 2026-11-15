use crate::app::settings::AppSettings;
use crate::g6_device::G6DeviceManager;
#[cfg(target_os = "linux")]
use crate::g6_mixer::RecordingSourceWatch;
use std::sync::Mutex;

pub struct AppState {
    pub device_manager: Mutex<G6DeviceManager>,
    pub settings: Mutex<AppSettings>,
    /// Keeps the stored recording source selected (started in bootstrap).
    #[cfg(target_os = "linux")]
    pub recording_source: RecordingSourceWatch,
}

pub fn create_app_state(settings: AppSettings) -> AppState {
    let device_manager = G6DeviceManager::new().expect("Failed to initialize G6 Device Manager");
    AppState {
        device_manager: Mutex::new(device_manager),
        #[cfg(target_os = "linux")]
        recording_source: RecordingSourceWatch::new(settings.recording_source),
        settings: Mutex::new(settings),
    }
}
