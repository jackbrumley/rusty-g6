use crate::platform::linux::alsa_audio;
use crate::platform::traits::{AudioSetup, WindowManagement};
use tauri::WebviewWindow;

pub struct WaylandBackend;

impl WaylandBackend {
    pub fn new() -> Self {
        Self
    }
}

impl AudioSetup for WaylandBackend {
    fn setup_microphone(&self) -> Result<String, String> {
        match alsa_audio::setup_g6_microphone() {
            Ok(result) => {
                if result.success {
                    Ok(result.message)
                } else {
                    Err(result.message)
                }
            }
            Err(e) => Err(format!("Failed to setup microphone via ALSA: {}", e)),
        }
    }

    fn get_microphone_status(&self) -> Result<String, String> {
        match alsa_audio::get_mic_status() {
            Ok(status) => Ok(status),
            Err(e) => Err(format!("Failed to get microphone status: {}", e)),
        }
    }
}

impl WindowManagement for WaylandBackend {
    fn setup_window(&self, _window: &WebviewWindow) {
        // Wayland specific window setup if needed in the future
        // e.g., gtk layer shell, setting unset environment variables
    }
}
