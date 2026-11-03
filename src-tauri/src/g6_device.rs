/// SoundBlaster X G6 USB Device Communication
/// Handles USB HID communication with the G6 device
use crate::g6_protocol_v2;
use crate::g6_spec::*;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

fn is_unresponsive(error: &anyhow::Error) -> bool {
    error.downcast_ref::<DeviceUnresponsive>().is_some()
}

/// A state read stops after this many queries in a row get no packet at
/// all: a healthy G6 answers each query within a few milliseconds, so
/// waiting out the remaining queries (500 ms each) only delays recovery.
const MAX_SILENT_QUERIES: u32 = 2;

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

    /// Check if the G6 USB interface required for communication is currently available.
    pub fn is_g6_interface_available(&self) -> Result<bool> {
        const G6_INTERFACE: i32 = 4;

        let api = HidApi::new().context("Failed to refresh HID API for availability check")?;
        for device_info in api.device_list() {
            if device_info.vendor_id() == USB_VENDOR_ID
                && device_info.product_id() == USB_PRODUCT_ID
                && device_info.interface_number() == G6_INTERFACE
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Connect to the G6 device
    pub fn connect(&self) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY_MS: u64 = 500;
        const G6_INTERFACE: i32 = 4;

        if self.is_connected() {
            info!("Connect requested while already connected; reusing current handle");
            return Ok(());
        }

        let mut last_error = None;

        for attempt in 1..=MAX_RETRIES {
            if attempt > 1 {
                info!(
                    "Connection attempt {}/{} (waiting {}ms for device enumeration)...",
                    attempt, MAX_RETRIES, RETRY_DELAY_MS
                );
                thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
            }

            // Refresh HidApi to get current device paths (critical after suspend/resume)
            let new_api = match HidApi::new() {
                Ok(api) => api,
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to refresh HID API: {}", e));
                    continue;
                }
            };
            *self.api.lock().unwrap() = new_api;

            let api = self.api.lock().unwrap();

            info!("Attempting to connect to SoundBlaster X G6...");
            info!(
                "Looking for device with VID: {:04x}, PID: {:04x}",
                USB_VENDOR_ID, USB_PRODUCT_ID
            );

            // Enumerate all devices to find the correct interface
            let mut device_found = false;
            let mut target_path: Option<std::ffi::CString> = None;

            for device_info in api.device_list() {
                info!(
                    "Found device: VID={:04x}, PID={:04x}, Interface={}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info.interface_number()
                );

                if device_info.vendor_id() == USB_VENDOR_ID
                    && device_info.product_id() == USB_PRODUCT_ID
                {
                    device_found = true;

                    if device_info.interface_number() == G6_INTERFACE {
                        info!("Found correct interface ({})", G6_INTERFACE);
                        target_path = Some(device_info.path().to_owned());
                        break;
                    }
                }
            }

            if !device_found {
                last_error = Some(anyhow::anyhow!(
                    "G6 device not found. Is it plugged in? VID={:04x}, PID={:04x}",
                    USB_VENDOR_ID,
                    USB_PRODUCT_ID
                ));
                continue;
            }

            let path = match target_path {
                Some(p) => p,
                None => {
                    last_error = Some(anyhow::anyhow!(
                        "G6 device found but required interface {} is not available. \
                         The device has multiple interfaces but we need the 4th one.",
                        G6_INTERFACE
                    ));
                    continue;
                }
            };

            // Try to open the device by path
            match api.open_path(&path) {
                Ok(device) => {
                    info!(
                        "Successfully connected to G6 device via interface {} (attempt {})",
                        G6_INTERFACE, attempt
                    );
                    let manufacturer = device
                        .get_manufacturer_string()
                        .unwrap_or(Some("Unknown".to_string()))
                        .unwrap_or("Unknown".to_string());
                    let product = device
                        .get_product_string()
                        .unwrap_or(Some("Unknown".to_string()))
                        .unwrap_or("Unknown".to_string());

                    info!("Device: {} {}", manufacturer, product);

                    *self.device.lock().unwrap() = Some(device);
                    self.state_confirmed.store(false, Ordering::SeqCst);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Failed to open G6 device: {}", e));
                    error!("Attempt {}/{} failed: {}", attempt, MAX_RETRIES, e);
                }
            }
        }

        // All retries exhausted
        let error = last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown connection error"));
        error!(
            "Failed to connect after {} attempts: {}",
            MAX_RETRIES, error
        );
        Err(error)
    }

    /// Disconnect from the G6 device
    pub fn disconnect(&self) {
        *self.device.lock().unwrap() = None;
        self.state_confirmed.store(false, Ordering::SeqCst);
        info!("Disconnected from G6 device");
    }

    /// Send commands to the G6 device (DATA + COMMIT pair). Returns whether
    /// the device acknowledged the batch with at least one response packet.
    /// An unacknowledged batch is logged and marks the cached state as
    /// unconfirmed, so the next toggle re-reads the device first.
    fn send_commands(&self, commands: Vec<Vec<u8>>) -> Result<bool> {
        // Signal listener to pause
        self.command_active.store(true, Ordering::SeqCst);

        // Ensure strictly scoped lock
        let result = (|| -> Result<bool> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;
            let mut acknowledged = false;

            for (_i, command) in commands.iter().enumerate() {
                // Helper logging
                let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
                let desc = g6_protocol_v2::describe_packet(command);

                // Color Scheme: Green for TX
                info!("\x1b[32m[TX] {}\x1b[0m | \x1b[35m{}\x1b[0m", hex_str, desc);

                let mut data_with_report_id = vec![0x00];
                data_with_report_id.extend_from_slice(&command);

                device
                    .write(&data_with_report_id)
                    .map_err(|e| anyhow::anyhow!("Failed to write to device: {}", e))?;

                // Wait for device to process
                std::thread::sleep(Duration::from_millis(20));

                // Try to read acknowledgment
                let mut buffer = vec![0u8; 65];
                if let Ok(bytes) = device.read_timeout(&mut buffer, 50) {
                    if bytes > 0 {
                        let response = if buffer[0] == 0x00 && bytes > 1 {
                            buffer[1..bytes].to_vec()
                        } else {
                            buffer[0..bytes].to_vec()
                        };

                        if response.len() >= 2 && response[0] == 0x5a {
                            acknowledged = true;
                            let rx_hex: String =
                                response.iter().map(|b| format!("{:02x}", b)).collect();
                            let rx_desc = g6_protocol_v2::describe_packet(&response);
                            info!(
                                "\x1b[33m[RX-ACK] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                                rx_hex, rx_desc
                            );
                        }
                    }
                }
            }
            Ok(acknowledged)
        })();

        // Resume listener
        self.command_active.store(false, Ordering::SeqCst);

        if let Ok(false) = result {
            warn!(
                "UNCONFIRMED WRITE: the G6 sent no acknowledgement for {} command(s); \
                 cached state is no longer trusted",
                commands.len()
            );
            self.state_confirmed.store(false, Ordering::SeqCst);
        }

        result
    }

    /// Send a single raw command and return the response (for Protocol V2 testing)
    pub fn send_raw_command(&self, command: &[u8]) -> Result<Vec<u8>> {
        self.command_active.store(true, Ordering::SeqCst);

        let result = (|| -> Result<Vec<u8>> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;

            // Log TX
            let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
            info!("\x1b[32m[TX-RAW] {}\x1b[0m", hex_str);

            // Prepend report ID
            let mut data_with_report_id = vec![0x00];
            data_with_report_id.extend_from_slice(command);

            // Send command
            device
                .write(&data_with_report_id)
                .context("Failed to write raw command to device")?;

            // Read response with timeout (100ms)
            let mut buffer = vec![0u8; 512];
            let bytes_read = device
                .read_timeout(&mut buffer, 100)
                .context("Failed to read raw response from device")?;

            // Remove report ID if present
            let response = if bytes_read > 0 && buffer[0] == 0x00 && bytes_read > 1 {
                buffer[1..bytes_read].to_vec()
            } else {
                buffer[0..bytes_read].to_vec()
            };

            // Log RX
            let rx_hex: String = response.iter().map(|b| format!("{:02x}", b)).collect();
            info!("\x1b[33m[RX-RAW] {}\x1b[0m", rx_hex);

            Ok(response)
        })();

        self.command_active.store(false, Ordering::SeqCst);
        result
    }

    /// Send read commands and collect responses with robust filtering
    fn send_read_commands(&self, commands: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        self.command_active.store(true, Ordering::SeqCst);

        let result = (|| -> Result<Vec<Vec<u8>>> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;

            let mut responses = Vec::new();
            let mut consecutive_silent = 0;
            info!("Sending {} read commands to G6 device", commands.len());

            for (i, command) in commands.iter().enumerate() {
                if command.len() < 2 {
                    continue;
                }
                let cmd_type = command[1];

                // Log TX
                let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
                let desc = g6_protocol_v2::describe_packet(command);
                info!(
                    "\x1b[32m[TX-READ] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                    hex_str, desc
                );

                let mut data_with_report_id = vec![0x00];
                data_with_report_id.extend_from_slice(&command);

                if let Err(e) = device.write(&data_with_report_id) {
                    error!("  ✗ Write failed for command {}: {}", i + 1, e);
                    responses.push(Vec::new());
                    continue;
                }

                let mut matched_response = None;
                let mut all_received = Vec::new();
                let mut packets_seen = 0;
                let start_time = std::time::Instant::now();

                while start_time.elapsed().as_millis() < 500 {
                    // 500ms total timeout per command
                    let mut buffer = vec![0u8; 65];
                    match device.read_timeout(&mut buffer, 100) {
                        // 100ms per read
                        Ok(bytes_read) if bytes_read > 0 => {
                            packets_seen += 1;
                            let response = if buffer[0] == 0x00 && bytes_read > 1 {
                                buffer[1..bytes_read].to_vec()
                            } else {
                                buffer[0..bytes_read].to_vec()
                            };

                            if response.len() < 2 || response[0] != 0x5a {
                                continue;
                            }

                            // Log ALL RX packets with detailed info
                            let rx_hex: String =
                                response.iter().map(|b| format!("{:02x}", b)).collect();
                            let rx_desc = g6_protocol_v2::describe_packet(&response);

                            // Check if this matches our expected response
                            let matches = response[1] == cmd_type;
                            let match_status = if matches { "✓ MATCH" } else { "✗ NO MATCH" };

                            info!(
                                "\x1b[33m[RX-READ] {}\x1b[0m | \x1b[35m{}\x1b[0m | Expected: 0x{:02x}, Got: 0x{:02x} {}",
                                rx_hex, rx_desc, cmd_type, response[1], match_status
                            );

                            all_received.push(response.clone());

                            if matches {
                                matched_response = Some(response);
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                if let Some(resp) = matched_response {
                    info!(
                        "  ✓ Found matching response for command {} (0x{:02x})",
                        i + 1,
                        cmd_type
                    );
                    responses.push(resp);
                } else {
                    error!("  ✗ No matching response for command {} (0x{:02x}). Received {} packet(s) total.",
                        i + 1, cmd_type, all_received.len());
                    // No fallback used - return empty if not matched
                    responses.push(Vec::new());
                }

                if packets_seen == 0 {
                    consecutive_silent += 1;
                    if consecutive_silent >= MAX_SILENT_QUERIES {
                        return Err(DeviceUnresponsive(format!(
                            "{} state queries in a row got no packet at all \
                             (the HID handle is open and writes succeed, but nothing comes back)",
                            consecutive_silent
                        ))
                        .into());
                    }
                } else {
                    consecutive_silent = 0;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(responses)
        })();

        self.command_active.store(false, Ordering::SeqCst);
        result
    }

    /// Start the event listener thread. It runs for the life of the app and
    /// reads from whatever handle is current, so calling this again after a
    /// reconnect is a no-op.
    pub fn start_listener<F>(&self, on_event: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if self.listener_started.swap(true, Ordering::SeqCst) {
            return;
        }

        let device_arc = self.device.clone();
        let settings_arc = self.current_settings.clone();
        let active_arc = self.command_active.clone();
        let confirmed_arc = self.state_confirmed.clone();

        thread::spawn(move || {
            info!("Device Event Listener Started");
            loop {
                // If main thread is sending commands, back off
                if active_arc.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                let mut buffer = vec![0u8; 65];
                let read_result = {
                    // Try to acquire lock - if connection drops, lock might fail?
                    match device_arc.lock() {
                        Ok(guard) => {
                            if let Some(dev) = guard.as_ref() {
                                dev.read_timeout(&mut buffer, 100)
                            } else {
                                // Device disconnected
                                Ok(0)
                            }
                        }
                        Err(_) => Ok(0), // Poisoned lock
                    }
                };

                match read_result {
                    Ok(bytes) if bytes > 0 => {
                        let packet = if buffer[0] == 0x00 && bytes > 1 {
                            &buffer[1..bytes]
                        } else {
                            &buffer[0..bytes]
                        };

                        // Parse packet using the unified event parser
                        if packet.len() > 2 && packet[0] == 0x5a {
                            // Get high-precision timestamp
                            let timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap();
                            let millis = timestamp.as_millis();
                            let micros = timestamp.as_micros() % 1000;

                            // Log all incoming events with full hex dump (up to 20 bytes)
                            let hex_str: String = packet
                                .iter()
                                .take(20)
                                .map(|b| format!("{:02x}", b))
                                .collect();
                            let desc = g6_protocol_v2::describe_packet(packet);

                            // Enhanced logging with timestamp
                            info!(
                                "\x1b[33m[RX-EVENT @{}.{:03}µs] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                                millis, micros, hex_str, desc
                            );

                            // Parse events using the protocol V2 parser
                            let events = crate::g6_protocol_v2::G6EventParser::parse(packet);

                            // Process each detected event
                            for event in &events {
                                if let Ok(mut settings) = settings_arc.lock() {
                                    use crate::g6_protocol_v2::DeviceEvent;

                                    match event {
                                        // Output switching
                                        DeviceEvent::OutputChanged(output) => {
                                            settings.output = *output;
                                            confirmed_arc.store(true, Ordering::SeqCst);
                                            info!("Detected {:?}", event);
                                        }

                                        // Gaming modes
                                        DeviceEvent::SbxModeChanged(state) => {
                                            settings.sbx_enabled = *state;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::ScoutModeChanged(state) => {
                                            settings.scout_mode = *state;
                                            info!("Detected {:?}", event);
                                        }

                                        // Audio effects - toggles
                                        DeviceEvent::SurroundToggled(state) => {
                                            settings.surround_enabled = *state;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::CrystalizerToggled(state) => {
                                            settings.crystalizer_enabled = *state;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::BassToggled(state) => {
                                            settings.bass_enabled = *state;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::SmartVolumeToggled(state) => {
                                            settings.smart_volume_enabled = *state;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::DialogPlusToggled(state) => {
                                            settings.dialog_plus_enabled = *state;
                                            info!("Detected {:?}", event);
                                        }

                                        // Audio effects - values
                                        DeviceEvent::SurroundValueChanged(value) => {
                                            settings.surround_value = *value;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::CrystalizerValueChanged(value) => {
                                            settings.crystalizer_value = *value;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::BassValueChanged(value) => {
                                            settings.bass_value = *value;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::SmartVolumeValueChanged(value) => {
                                            settings.smart_volume_value = *value;
                                            info!("Detected {:?}", event);
                                        }
                                        DeviceEvent::DialogPlusValueChanged(value) => {
                                            settings.dialog_plus_value = *value;
                                            info!("Detected {:?}", event);
                                        }

                                        // Digital filter
                                        DeviceEvent::DigitalFilterChanged(filter) => {
                                            settings.digital_filter = Some(*filter);
                                            info!("Detected {:?}", event);
                                        }

                                        // Audio config
                                        DeviceEvent::AudioConfigChanged(config) => {
                                            settings.audio_config = Some(*config);
                                            info!("Detected {:?}", event);
                                        }

                                        // Microphone boost
                                        DeviceEvent::MicrophoneBoostChanged(db_value) => {
                                            settings.microphone_boost = *db_value;
                                            info!("Detected {:?}", event);
                                        }
                                    }
                                }
                            }

                            // Notify UI if any events were processed
                            if !events.is_empty() {
                                on_event();
                            }
                        }
                    }
                    Ok(_) => {} // No data
                    Err(e) => {
                        error!("Read error in listener (possibly disconnected): {}", e);
                        // Clear the device on error to trigger reconnection flow
                        *device_arc.lock().unwrap() = None;
                        confirmed_arc.store(false, Ordering::SeqCst);
                        // Notify UI of disconnect
                        on_event();
                        // Sleep a bit longer before next attempt
                        thread::sleep(Duration::from_millis(500));
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    /// Toggle output between speakers and headphones
    /// Now using Protocol V2 (clean 2-command version)
    pub fn toggle_output(&self) -> Result<()> {
        // The toggle target comes from the cached output, so it must be the
        // device's real output. Without this check a device that never
        // answered left the cache at its default (Headphones) and every click
        // re-sent "switch to Speakers": one relay click, then nothing.
        if !self.state_confirmed.load(Ordering::SeqCst) {
            info!("Output state not confirmed by the device; reading it before toggling");
            self.read_device_state()
                .context("Cannot toggle output: the current output is unknown")?;
        }

        let current = self.current_settings.lock().unwrap().output;
        let target = toggle_target(current);

        // Use V2 protocol - just 2 commands instead of 30!
        let commands = crate::g6_protocol_v2::build_toggle_output_simple(current);

        let acknowledged = self.send_commands(commands)?;
        info!("Output toggle command sent: {:?} → {:?}", current, target);
        if !acknowledged {
            self.verify_unacknowledged_output_write(target)?;
        }

        // State will be updated by listener when device confirms the change
        Ok(())
    }

    /// An output write got no acknowledgement: re-read the device (healing
    /// the handle if needed) to learn whether the switch happened.
    fn verify_unacknowledged_output_write(&self, target: OutputDevice) -> Result<()> {
        warn!(
            "Output switch to {:?} was not acknowledged; re-reading device state",
            target
        );
        let settings = self.read_device_state()?;
        if settings.output == target {
            info!(
                "Device reports {:?} after re-read; the switch took effect",
                target
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Output switch to {:?} was not applied (device reports {:?})",
                target,
                settings.output
            ))
        }
    }

    /// Set output device
    /// Now using Protocol V2 (clean 2-command version)
    pub fn set_output(&self, output: OutputDevice) -> Result<()> {
        // Use V2 protocol - just 2 commands!
        let commands = vec![
            crate::g6_protocol_v2::build_set_output(output),
            crate::g6_protocol_v2::build_commit_output(),
        ];

        let acknowledged = self.send_commands(commands)?;

        // State will be updated by listener when device confirms
        info!("Set output command sent using V2: {:?}", output);
        if !acknowledged {
            self.verify_unacknowledged_output_write(output)?;
        }
        Ok(())
    }

    /// Set surround sound
    /// Now using Protocol V2
    pub fn set_surround(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = crate::g6_protocol_v2::build_set_surround_toggle(enabled_bool);
        let value_commands = crate::g6_protocol_v2::build_set_surround_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.surround_enabled = enabled;
        settings.surround_value = value;

        info!(
            "Surround set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set crystalizer
    /// Now using Protocol V2
    pub fn set_crystalizer(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = crate::g6_protocol_v2::build_set_crystalizer_toggle(enabled_bool);
        let value_commands = crate::g6_protocol_v2::build_set_crystalizer_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.crystalizer_enabled = enabled;
        settings.crystalizer_value = value;

        info!(
            "Crystalizer set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set bass
    /// Now using Protocol V2
    pub fn set_bass(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // IMPORTANT: Official software sends ONLY the toggle command when enabling/disabling
        // The value command should be sent separately only when adjusting the slider
        // For now, we only send the toggle command to match official behavior
        let toggle_commands = crate::g6_protocol_v2::build_set_bass_toggle(enabled_bool);
        self.send_commands(toggle_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.bass_enabled = enabled;
        settings.bass_value = value;

        info!(
            "Bass set to {:?} using V2 (value {} stored locally)",
            enabled, value
        );
        Ok(())
    }

    /// Set smart volume
    /// Now using Protocol V2
    pub fn set_smart_volume(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = crate::g6_protocol_v2::build_set_smart_volume_toggle(enabled_bool);
        let value_commands = crate::g6_protocol_v2::build_set_smart_volume_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.smart_volume_enabled = enabled;
        settings.smart_volume_value = value;

        info!(
            "Smart Volume set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set dialog plus
    /// Now using Protocol V2
    pub fn set_dialog_plus(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = crate::g6_protocol_v2::build_set_dialog_plus_toggle(enabled_bool);
        let value_commands = crate::g6_protocol_v2::build_set_dialog_plus_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.dialog_plus_enabled = enabled;
        settings.dialog_plus_value = value;

        info!(
            "Dialog Plus set to {:?} with value {} using V2",
            enabled, value
        );
        Ok(())
    }

    /// Set SBX Master Mode
    /// Now using Protocol V2
    pub fn set_sbx_mode(&self, enabled: EffectState) -> Result<()> {
        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - Gaming family (0x26)
        let commands = crate::g6_protocol_v2::build_set_sbx_mode(enabled_bool);

        self.send_commands(commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.sbx_enabled = enabled;

        info!("SBX Mode set to {:?} using V2", enabled);
        Ok(())
    }

    /// Set Scout Mode
    /// Now using Protocol V2
    pub fn set_scout_mode(&self, enabled: ScoutModeState) -> Result<()> {
        let enabled_bool = matches!(enabled, ScoutModeState::Enabled);

        // Use V2 protocol - Gaming family (0x26)
        let commands = crate::g6_protocol_v2::build_set_scout_mode(enabled_bool);

        self.send_commands(commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.scout_mode = enabled;

        info!("Scout Mode set to {:?} using V2", enabled);
        Ok(())
    }

    /// Set Microphone Boost
    /// Now using Protocol V2 - discovered from packet capture
    /// Valid values: 0, 10, 20, or 30 dB
    pub fn set_microphone_boost(&self, db_value: u8) -> Result<()> {
        // Validate: only 0, 10, 20, or 30 dB allowed
        if ![0, 10, 20, 30].contains(&db_value) {
            return Err(anyhow::anyhow!(
                "Invalid microphone boost value: {}dB. Valid values: 0, 10, 20, 30",
                db_value
            ));
        }

        // Use V2 protocol - AudioConfig family (0x3c)
        let commands = crate::g6_protocol_v2::build_set_microphone_boost(db_value);

        self.send_commands(commands)?;

        info!("Microphone Boost set to {}dB using V2", db_value);
        Ok(())
    }

    /// Get current settings
    pub fn get_settings(&self) -> G6Settings {
        self.current_settings.lock().unwrap().clone()
    }

    /// Read current device state from hardware and update the cached
    /// settings. Fails unless at least one response was parsed.
    ///
    /// When the device does not answer, the handle is healed before giving
    /// up, escalating like a manual replug would:
    /// 1. close and reopen the HID handle, read again;
    /// 2. (Linux) reset the G6 on the USB bus, reconnect, read again.
    ///
    /// Every step is logged so the next occurrence is diagnosable.
    pub fn read_device_state(&self) -> Result<G6Settings> {
        if !self.is_connected() {
            return Err(anyhow::anyhow!("Device not connected"));
        }

        let error = match self.read_state_once() {
            Ok(settings) => return Ok(settings),
            Err(error) if is_unresponsive(&error) => error,
            Err(error) => return Err(error),
        };

        warn!(
            "SELF-HEAL: {}. Reopening the HID handle (software replug)...",
            error
        );
        self.disconnect();
        self.connect()
            .context("Self-heal: reopening the G6 HID handle failed")?;
        let error = match self.read_state_once() {
            Ok(settings) => {
                info!("SELF-HEAL: device answered after reopening the HID handle");
                return Ok(settings);
            }
            Err(error) if is_unresponsive(&error) => error,
            Err(error) => return Err(error),
        };

        self.reset_usb_and_read(error)
    }

    #[cfg(target_os = "linux")]
    fn reset_usb_and_read(&self, error: anyhow::Error) -> Result<G6Settings> {
        if self.usb_reset_attempted.swap(true, Ordering::SeqCst) {
            error!(
                "SELF-HEAL FAILED: {} after reopening the handle, and a USB reset was already tried. \
                 Unplug and replug the G6.",
                error
            );
            return Err(error.context(
                "The G6 is still not responding after a USB reset. Unplug and replug it",
            ));
        }

        warn!(
            "SELF-HEAL: {} after reopening the handle. Resetting the G6 on the USB bus...",
            error
        );
        self.disconnect();
        reset_usb_device().context("Self-heal: USB reset failed")?;
        info!("SELF-HEAL: USB reset done; reconnecting");
        self.connect()
            .context("Self-heal: reconnecting after the USB reset failed")?;
        match self.read_state_once() {
            Ok(settings) => {
                info!("SELF-HEAL: device answered after the USB reset");
                Ok(settings)
            }
            Err(error) => {
                error!(
                    "SELF-HEAL FAILED: {} even after a USB reset. Unplug and replug the G6.",
                    error
                );
                Err(error.context(
                    "The G6 is still not responding after a USB reset. Unplug and replug it",
                ))
            }
        }
    }

    /// Windows: the G6 is bound to the HID driver, so libusb cannot reset it.
    #[cfg(not(target_os = "linux"))]
    fn reset_usb_and_read(&self, error: anyhow::Error) -> Result<G6Settings> {
        error!(
            "SELF-HEAL FAILED: {} after reopening the handle. Unplug and replug the G6.",
            error
        );
        Err(error.context("The G6 is not responding. Unplug and replug it"))
    }

    /// One state read on the current handle, without recovery.
    fn read_state_once(&self) -> Result<G6Settings> {
        info!("Reading complete device state from G6...");

        // Build all read commands using V2 protocol
        let commands = crate::g6_protocol_v2::build_read_all_state_commands();

        // Send commands and get responses
        let responses = self.send_read_commands(commands)?;

        info!("Processing {} read responses...", responses.len());

        let mut settings = self.current_settings.lock().unwrap();
        let parsed = apply_read_responses(&mut settings, &responses);
        if parsed == 0 {
            return Err(DeviceUnresponsive(format!(
                "none of the {} state queries got a usable response",
                responses.len()
            ))
            .into());
        }

        settings.last_read_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        );
        self.state_confirmed.store(true, Ordering::SeqCst);
        self.usb_reset_attempted.store(false, Ordering::SeqCst);

        info!(
            "Device state read complete ({}/{} responses parsed)",
            parsed,
            responses.len()
        );

        // Return updated settings
        Ok(settings.clone())
    }

    /// Synchronize with device state on startup
    pub fn synchronize_with_device(&self) -> Result<()> {
        info!("Synchronizing with device state...");

        match self.read_device_state() {
            Ok(device_settings) => {
                info!("Device synchronization successful");

                // Log interesting findings
                if let Some(firmware) = &device_settings.firmware_info {
                    info!("Device firmware: {}", firmware.version);
                }

                if let Some(eq) = &device_settings.equalizer {
                    info!(
                        "EQ bands detected: {}, enabled: {:?}",
                        eq.bands.len(),
                        eq.enabled
                    );
                }

                if let Some(ext) = &device_settings.extended_params {
                    let non_null_params = [
                        ext.param_0x0a,
                        ext.param_0x0b,
                        ext.param_0x0c,
                        ext.param_0x0d,
                        ext.param_0x0e,
                        ext.param_0x0f,
                        ext.param_0x10,
                        ext.param_0x11,
                        ext.param_0x12,
                        ext.param_0x13,
                        ext.param_0x14,
                        ext.param_0x1a,
                        ext.param_0x1b,
                        ext.param_0x1c,
                        ext.param_0x1d,
                    ]
                    .iter()
                    .filter(|p| p.is_some())
                    .count();

                    info!("Extended parameters detected: {}/15", non_null_params);
                }

                Ok(())
            }
            Err(e) => {
                error!("Device synchronization failed: {}.", e);
                // Do NOT apply defaults on failure per user request.
                // Just leave internal state as default but don't write to device.
                Ok(())
            }
        }
    }

    /// List all connected HID devices (for debugging)
    pub fn list_devices(&self) -> Result<Vec<String>> {
        let api = self.api.lock().unwrap();
        let devices = api.device_list();

        let device_list: Vec<String> = devices
            .map(|dev| {
                format!(
                    "VID: {:04x}, PID: {:04x}, Interface: {}, Manufacturer: {}, Product: {}",
                    dev.vendor_id(),
                    dev.product_id(),
                    dev.interface_number(),
                    dev.manufacturer_string().unwrap_or("Unknown"),
                    dev.product_string().unwrap_or("Unknown")
                )
            })
            .collect();

        Ok(device_list)
    }
}

/// The output a toggle switches to.
fn toggle_target(current: OutputDevice) -> OutputDevice {
    match current {
        OutputDevice::Headphones => OutputDevice::Speakers,
        OutputDevice::Speakers => OutputDevice::Headphones,
    }
}

/// Applies state-read responses to `settings`. Returns how many responses
/// carried data that was parsed; empty (unanswered) responses count as none.
fn apply_read_responses(settings: &mut G6Settings, responses: &[Vec<u8>]) -> usize {
    use crate::g6_protocol_v2::{DeviceEvent, G6EventParser, G6ResponseParser, ParsedResponse};

    let mut parsed = 0;
    for response in responses {
        if response.len() < 2 {
            continue;
        }

        match response[1] {
            0x07 => {
                // Firmware query
                if let (Ok(ParsedResponse::FirmwareInfo(info)), _) =
                    G6ResponseParser::parse(response)
                {
                    info!("Firmware: {}", info.version);
                    settings.firmware_info = Some(info);
                    parsed += 1;
                }
            }
            0x2c => {
                // Output routing
                if let (Ok(ParsedResponse::OutputDevice(output)), _) =
                    G6ResponseParser::parse(response)
                {
                    info!("Output: {:?}", output);
                    settings.output = output;
                    parsed += 1;
                }
            }
            _ => {
                let mut applied = false;
                for event in G6EventParser::parse(response) {
                    match event {
                        DeviceEvent::SbxModeChanged(state) => {
                            settings.sbx_enabled = state;
                            info!("SBX Mode: {:?}", state);
                        }
                        DeviceEvent::ScoutModeChanged(state) => {
                            settings.scout_mode = state;
                            info!("Scout Mode: {:?}", state);
                        }
                        DeviceEvent::SurroundToggled(state) => {
                            settings.surround_enabled = state;
                            info!("Surround enabled: {:?}", state);
                        }
                        DeviceEvent::SurroundValueChanged(value) => {
                            settings.surround_value = value;
                            info!("Surround value: {}", value);
                        }
                        DeviceEvent::CrystalizerToggled(state) => {
                            settings.crystalizer_enabled = state;
                            info!("Crystalizer enabled: {:?}", state);
                        }
                        DeviceEvent::CrystalizerValueChanged(value) => {
                            settings.crystalizer_value = value;
                            info!("Crystalizer value: {}", value);
                        }
                        DeviceEvent::BassToggled(state) => {
                            settings.bass_enabled = state;
                            info!("Bass enabled: {:?}", state);
                        }
                        DeviceEvent::BassValueChanged(value) => {
                            settings.bass_value = value;
                            info!("Bass value: {}", value);
                        }
                        DeviceEvent::SmartVolumeToggled(state) => {
                            settings.smart_volume_enabled = state;
                            info!("Smart Volume enabled: {:?}", state);
                        }
                        DeviceEvent::SmartVolumeValueChanged(value) => {
                            settings.smart_volume_value = value;
                            info!("Smart Volume value: {}", value);
                        }
                        DeviceEvent::DialogPlusToggled(state) => {
                            settings.dialog_plus_enabled = state;
                            info!("Dialog Plus enabled: {:?}", state);
                        }
                        DeviceEvent::DialogPlusValueChanged(value) => {
                            settings.dialog_plus_value = value;
                            info!("Dialog Plus value: {}", value);
                        }
                        DeviceEvent::DigitalFilterChanged(filter) => {
                            settings.digital_filter = Some(filter);
                            info!("Digital filter: {:?}", filter);
                        }
                        DeviceEvent::AudioConfigChanged(config) => {
                            settings.audio_config = Some(config);
                            info!("Audio config: {:?}", config);
                        }
                        _ => continue, // Ignore other events
                    }
                    applied = true;
                }
                if applied {
                    parsed += 1;
                }
            }
        }
    }
    parsed
}

/// Resets the G6 on the USB bus (USBDEVFS_RESET through libusb): the same
/// re-enumeration a physical replug causes, without the power cycle. Needs
/// write access to the G6's usbfs node, which the udev rule grants.
#[cfg(target_os = "linux")]
fn reset_usb_device() -> Result<()> {
    let devices = rusb::devices().context("Failed to list USB devices")?;
    for device in devices.iter() {
        let Ok(descriptor) = device.device_descriptor() else {
            continue;
        };
        if descriptor.vendor_id() != USB_VENDOR_ID || descriptor.product_id() != USB_PRODUCT_ID {
            continue;
        }
        info!(
            "Resetting G6 at USB bus {:03} device {:03}",
            device.bus_number(),
            device.address()
        );
        let handle = device
            .open()
            .context("Failed to open the G6 USB device (check the udev rule)")?;
        return match handle.reset() {
            // NotFound: the device re-enumerated as part of the reset.
            Ok(()) | Err(rusb::Error::NotFound) => Ok(()),
            Err(error) => Err(anyhow::anyhow!("USB reset failed: {}", error)),
        };
    }
    Err(anyhow::anyhow!("G6 not found on the USB bus"))
}

impl Default for G6DeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create G6DeviceManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    /// Responses captured from a healthy G6 (session log, 2026-11-03).
    fn healthy_responses() -> Vec<Vec<u8>> {
        vec![
            hex("5a0710322e312e3235303930332e313332340000"),
            hex("5a2c0501020000000000"),
            hex("5a260b08ffff05000000"),
            hex("5a1108010096000000803f00000000"),
        ]
    }

    #[test]
    fn unanswered_read_parses_nothing() {
        // After a cold boot every query came back empty; that must not count
        // as a successful read.
        let mut settings = G6Settings::default();
        let responses = vec![Vec::new(); 14];
        assert_eq!(apply_read_responses(&mut settings, &responses), 0);
        assert!(settings.firmware_info.is_none());
    }

    #[test]
    fn healthy_read_applies_firmware_output_and_modes() {
        let mut settings = G6Settings::default();
        let parsed = apply_read_responses(&mut settings, &healthy_responses());
        assert_eq!(parsed, 4);
        assert_eq!(settings.firmware_info.unwrap().version, "2.1.250903.1324");
        assert_eq!(settings.output, OutputDevice::Speakers);
        assert_eq!(settings.sbx_enabled, EffectState::Enabled);
        assert_eq!(settings.surround_enabled, EffectState::Enabled);
    }

    #[test]
    fn toggle_target_flips_output() {
        assert_eq!(
            toggle_target(OutputDevice::Speakers),
            OutputDevice::Headphones
        );
        assert_eq!(
            toggle_target(OutputDevice::Headphones),
            OutputDevice::Speakers
        );
    }

    #[test]
    fn unresponsive_errors_are_recognised_through_context() {
        let error: anyhow::Error = DeviceUnresponsive("silent".to_string()).into();
        let error = error.context("while reading");
        assert!(is_unresponsive(&error));
        assert!(!is_unresponsive(&anyhow::anyhow!("Device not connected")));
    }
}
