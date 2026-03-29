pub mod alsa_audio;
pub mod detection;
pub mod wayland;
pub mod x11; // Shared ALSA implementation

use crate::platform::traits::PlatformBackend;
use std::sync::Arc;

pub fn initialize() -> Arc<dyn PlatformBackend> {
    match detection::detect_display_server() {
        detection::LinuxDisplayServer::Wayland => Arc::new(wayland::WaylandBackend::new()),
        detection::LinuxDisplayServer::X11 => Arc::new(x11::X11Backend::new()),
        detection::LinuxDisplayServer::Unknown => {
            eprintln!("Warning: Unknown Linux display server. Defaulting to X11 backend.");
            Arc::new(x11::X11Backend::new())
        }
    }
}
