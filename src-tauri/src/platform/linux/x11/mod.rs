use crate::platform::linux::alsa_audio;
use crate::platform::traits::{AudioSetup, WindowManagement};
use tauri::WebviewWindow;

pub struct X11Backend;

impl X11Backend {
    pub fn new() -> Self {
        Self
    }
}

impl AudioSetup for X11Backend {
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

impl WindowManagement for X11Backend {
    fn setup_window(&self, _window: &WebviewWindow) {
        // X11 specific window setup if needed in the future
    }
}
