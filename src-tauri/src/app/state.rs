use crate::g6_device::G6DeviceManager;
use std::sync::Mutex;

pub struct AppState {
    pub device_manager: Mutex<G6DeviceManager>,
}

pub fn create_app_state() -> AppState {
    let device_manager = G6DeviceManager::new().expect("Failed to initialize G6 Device Manager");
    AppState {
        device_manager: Mutex::new(device_manager),
    }
}
