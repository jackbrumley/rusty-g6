use crate::app::settings::AppSettings;
use crate::g6_device::G6DeviceManager;
#[cfg(target_os = "linux")]
use crate::g6_mixer::MixerWatch;
use std::sync::Mutex;

pub struct AppState {
    pub device_manager: Mutex<G6DeviceManager>,
    pub settings: Mutex<AppSettings>,
    /// Keeps the stored recording source and mic level applied (started in
    /// bootstrap).
    #[cfg(target_os = "linux")]
    pub mixer: MixerWatch,
}

pub fn create_app_state(settings: AppSettings) -> AppState {
    let device_manager = G6DeviceManager::new().expect("Failed to initialize G6 Device Manager");
    AppState {
        device_manager: Mutex::new(device_manager),
        #[cfg(target_os = "linux")]
        mixer: MixerWatch::new(
            settings.recording_source,
            settings.mic_level_db,
            settings.mic_monitoring,
        ),
        settings: Mutex::new(settings),
    }
}
