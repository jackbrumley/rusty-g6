use crate::g6_device::G6DeviceManager;
use crate::platform;
use crate::platform::traits::PlatformBackend;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub device_manager: Mutex<G6DeviceManager>,
    pub platform_backend: Arc<dyn PlatformBackend>,
    pub tray_available: Mutex<bool>,
}

pub fn build_app_state() -> AppState {
    let device_manager = G6DeviceManager::new().expect("Failed to initialize G6 Device Manager");
    let platform_backend = platform::initialize();

    AppState {
        device_manager: Mutex::new(device_manager),
        platform_backend,
        tray_available: Mutex::new(false),
    }
}
