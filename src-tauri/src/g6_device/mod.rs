/// SoundBlaster X G6 USB Device Communication
/// Handles USB HID communication with the G6 device
use crate::g6_spec::*;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// The G6 accepted the handle but did not answer: state queries returned no
/// valid data, or a write was not acknowledged. Seen after a cold boot, when
/// the G6 takes HID output reports on the control pipe but sends nothing on
/// its interrupt IN endpoint until it is re-enumerated (docs/TODO.md).
#[derive(Debug)]
pub struct DeviceUnresponsive(pub String);

impl std::fmt::Display for DeviceUnresponsive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "G6 is not responding: {}", self.0)
    }
}

impl std::error::Error for DeviceUnresponsive {}

mod connection;
mod listener;
mod setters;
mod state_read;
mod transport;

fn is_unresponsive(error: &anyhow::Error) -> bool {
    error.downcast_ref::<DeviceUnresponsive>().is_some()
}

/// G6 Device Manager
pub struct G6DeviceManager {
    api: Arc<Mutex<HidApi>>,
    device: Arc<Mutex<Option<HidDevice>>>,
    current_settings: Arc<Mutex<G6Settings>>,
    command_active: Arc<AtomicBool>,
    /// Whether `current_settings` reflects the device: set by a state read
    /// that parsed data or by a change event from the device, cleared on
    /// (re)connect, disconnect and unacknowledged writes. Toggles and the
    /// tray label only trust the cached output while this is set.
    state_confirmed: Arc<AtomicBool>,
    /// The event listener thread is started once and follows whatever handle
    /// `device` holds, so reconnects never stack up extra readers.
    listener_started: AtomicBool,
    /// A USB reset was tried for the current unresponsive episode; cleared
    /// by the next successful read so a dead device is never reset in a loop.
    usb_reset_attempted: AtomicBool,
}

impl G6DeviceManager {
    /// Create a new G6 Device Manager
    pub fn new() -> Result<Self> {
        let api = HidApi::new().context("Failed to initialize HID API")?;

        Ok(Self {
            api: Arc::new(Mutex::new(api)),
            device: Arc::new(Mutex::new(None)),
            current_settings: Arc::new(Mutex::new(G6Settings::default())),
            command_active: Arc::new(AtomicBool::new(false)),
            state_confirmed: Arc::new(AtomicBool::new(false)),
            listener_started: AtomicBool::new(false),
            usb_reset_attempted: AtomicBool::new(false),
        })
    }

    /// Check if a G6 device is connected
    pub fn is_connected(&self) -> bool {
        self.device.lock().unwrap().is_some()
    }

    /// Whether the cached settings were confirmed by the device since the
    /// handle was opened (see `state_confirmed`).
    pub fn is_state_confirmed(&self) -> bool {
        self.is_connected() && self.state_confirmed.load(Ordering::SeqCst)
    }

    /// Get current settings
    pub fn get_settings(&self) -> G6Settings {
        self.current_settings.lock().unwrap().clone()
    }
}

impl Default for G6DeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create G6DeviceManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresponsive_errors_are_recognised_through_context() {
        let error: anyhow::Error = DeviceUnresponsive("silent".to_string()).into();
        let error = error.context("while reading");
        assert!(is_unresponsive(&error));
        assert!(!is_unresponsive(&anyhow::anyhow!("Device not connected")));
    }
}
