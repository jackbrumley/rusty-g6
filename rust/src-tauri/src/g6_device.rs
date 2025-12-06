use crate::g6_protocol;
/// SoundBlaster X G6 USB Device Communication
/// Handles USB HID communication with the G6 device
use crate::g6_spec::*;
use anyhow::{Context, Result};
use hidapi::{HidApi, HidDevice};
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// G6 Device Manager
pub struct G6DeviceManager {
    api: Arc<Mutex<HidApi>>,
    device: Arc<Mutex<Option<HidDevice>>>,
    current_settings: Arc<Mutex<G6Settings>>,
    command_active: Arc<AtomicBool>,
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
        })
    }

    /// Check if a G6 device is connected
    pub fn is_connected(&self) -> bool {
        self.device.lock().unwrap().is_some()
    }

    /// Connect to the G6 device
    pub fn connect(&self) -> Result<()> {
        let api = self.api.lock().unwrap();

        info!("Attempting to connect to SoundBlaster X G6...");
        info!(
            "Looking for device with VID: {:04x}, PID: {:04x}",
            USB_VENDOR_ID, USB_PRODUCT_ID
        );

        // The G6 has 4 interfaces (2 Audio and 2 HID), we need interface 4
        // This is critical - the other interfaces ignore commands!
        const G6_INTERFACE: i32 = 4;

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
            error!(
                "No G6 device found with VID={:04x}, PID={:04x}",
                USB_VENDOR_ID, USB_PRODUCT_ID
            );
            return Err(anyhow::anyhow!(
                "G6 device not found. Is it plugged in? VID={:04x}, PID={:04x}",
                USB_VENDOR_ID,
                USB_PRODUCT_ID
            ));
        }

        let path = target_path.ok_or_else(|| {
            error!(
                "G6 device found but interface {} not available",
                G6_INTERFACE
            );
            anyhow::anyhow!(
                "G6 device found but required interface {} is not available. \
                 The device has multiple interfaces but we need the 4th one.",
                G6_INTERFACE
            )
        })?;

        // Open the device by path (not by VID/PID which defaults to interface 0)
        match api.open_path(&path) {
            Ok(device) => {
                info!(
                    "Successfully connected to G6 device via interface {}",
                    G6_INTERFACE
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
                Ok(())
            }
            Err(e) => {
                error!("Failed to open G6 device: {}", e);
                Err(anyhow::anyhow!("Failed to open G6 device: {}", e))
            }
        }
    }

    /// Disconnect from the G6 device
    pub fn disconnect(&self) {
        *self.device.lock().unwrap() = None;
        info!("Disconnected from G6 device");
    }

    /// Send commands to the G6 device (DATA + COMMIT pair)
    fn send_commands(&self, commands: Vec<Vec<u8>>) -> Result<()> {
        // Signal listener to pause
        self.command_active.store(true, Ordering::SeqCst);

        // Ensure strictly scoped lock
        let result = (|| -> Result<()> {
            let device_guard = self.device.lock().unwrap();
            let device = device_guard.as_ref().context("Device not connected")?;

            for (i, command) in commands.iter().enumerate() {
                // Helper logging
                let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
                let desc = g6_protocol::describe_packet(command);

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
                            let rx_hex: String =
                                response.iter().map(|b| format!("{:02x}", b)).collect();
                            let rx_desc = g6_protocol::describe_packet(&response);
                            info!(
                                "\x1b[33m[RX-ACK] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                                rx_hex, rx_desc
                            );
                        }
                    }
                }
            }
            Ok(())
        })();

        // Resume listener
        self.command_active.store(false, Ordering::SeqCst);

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
            info!("Sending {} read commands to G6 device", commands.len());

            for (i, command) in commands.iter().enumerate() {
                if command.len() < 2 {
                    continue;
                }
                let cmd_type = command[1];

                // Log TX
                let hex_str: String = command.iter().map(|b| format!("{:02x}", b)).collect();
                let desc = g6_protocol::describe_packet(command);
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
                let start_time = std::time::Instant::now();

                while start_time.elapsed().as_millis() < 500 {
                    // 500ms total timeout per command
                    let mut buffer = vec![0u8; 65];
                    match device.read_timeout(&mut buffer, 100) {
                        // 100ms per read
                        Ok(bytes_read) if bytes_read > 0 => {
                            let response = if buffer[0] == 0x00 && bytes_read > 1 {
                                buffer[1..bytes_read].to_vec()
                            } else {
                                buffer[0..bytes_read].to_vec()
                            };

                            if response.len() < 2 || response[0] != 0x5a {
                                continue;
                            }

                            // Log RX
                            let rx_hex: String =
                                response.iter().map(|b| format!("{:02x}", b)).collect();
                            let rx_desc = g6_protocol::describe_packet(&response);
                            info!(
                                "\x1b[33m[RX-READ] {}\x1b[0m | \x1b[35m{}\x1b[0m",
                                rx_hex, rx_desc
                            );

                            if response[1] == cmd_type {
                                matched_response = Some(response);
                                break;
                            }
                            // Ignore stray events here as needed
                        }
                        _ => {}
                    }
                }

                if let Some(resp) = matched_response {
                    responses.push(resp);
                } else {
                    responses.push(Vec::new());
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Ok(responses)
        })();

        self.command_active.store(false, Ordering::SeqCst);
        result
    }

    /// Start the event listener thread
    pub fn start_listener<F>(&self, on_event: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let device_arc = self.device.clone();
        let settings_arc = self.current_settings.clone();
        let active_arc = self.command_active.clone();

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

                        // Parse packet and update settings if scout mode detected
                        if packet.len() > 2 && packet[0] == 0x5a {
                            // Log all incoming events
                            let hex_str: String =
                                packet.iter().map(|b| format!("{:02x}", b)).collect();
                            let desc = g6_protocol::describe_packet(packet);

                            // Color Scheme: Yellow for RX
                            info!("\x1b[33m[RX] {}\x1b[0m | \x1b[35m{}\x1b[0m", hex_str, desc);

                            // Check for Output Change event (0x2c family)
                            if packet[1] == 0x2c {
                                let mut new_output = None;
                                // Scan for status byte (0x02=Speakers, 0x04=Headphones)
                                // Skip index 3 (often header/command byte) - start from index 4
                                for i in 4..std::cmp::min(12, packet.len()) {
                                    match packet[i] {
                                        0x04 => {
                                            new_output = Some(OutputDevice::Headphones);
                                            break;
                                        }
                                        0x02 => {
                                            new_output = Some(OutputDevice::Speakers);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }

                                if let Some(output) = new_output {
                                    if let Ok(mut settings) = settings_arc.lock() {
                                        if settings.output != output {
                                            settings.output = output;
                                            info!("Detected Output change: {:?}", output);
                                        }
                                    }
                                }
                            }

                            // Check for Scout Mode event (0x26 family)
                            if packet[1] == 0x26 {
                                // Heuristic: Look for Feature 0x02 and Values 0x01 (On) or 0x00 (Off)

                                let mut new_state = None;

                                // Search packet for Feature 0x02 followed closely by 0x01 or 0x00
                                for i in 2..packet.len() - 2 {
                                    if packet[i] == 0x02 {
                                        // If 0x02 is followed by 0x00 then 0x01 (Enabled)
                                        if i + 2 < packet.len()
                                            && packet[i + 1] == 0x00
                                            && packet[i + 2] == 0x01
                                        {
                                            new_state = Some(ScoutModeState::Enabled);
                                            break;
                                        }
                                        // If 0x02 is followed by 0x00 then 0x00 (Disabled)
                                        if i + 2 < packet.len()
                                            && packet[i + 1] == 0x00
                                            && packet[i + 2] == 0x00
                                        {
                                            new_state = Some(ScoutModeState::Disabled);
                                            break;
                                        }
                                    }
                                }

                                if let Some(state) = new_state {
                                    if let Ok(mut settings) = settings_arc.lock() {
                                        settings.scout_mode = state;
                                        info!("Detected Scout Mode change: {:?}", state);
                                    }
                                }
                            }

                            on_event();
                        }
                    }
                    _ => {}
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    /// Toggle output between speakers and headphones
    /// Now using Protocol V2 (clean 2-command version)
    pub fn toggle_output(&self) -> Result<()> {
        let current = self.current_settings.lock().unwrap().output;

        // Use V2 protocol - just 2 commands instead of 30!
        let commands = crate::g6_protocol_v2::build_toggle_output_simple(current);

        self.send_commands(commands)?;

        // State will be updated by listener when device confirms
        info!(
            "Output toggle command sent using V2 (current: {:?})",
            current
        );
        Ok(())
    }

    /// Set output device
    /// Now using Protocol V2 (clean 2-command version)
    pub fn set_output(&self, output: OutputDevice) -> Result<()> {
        // Use V2 protocol - just 2 commands!
        let commands = vec![
            crate::g6_protocol_v2::build_set_output(output),
            crate::g6_protocol_v2::build_commit_output(),
        ];

        self.send_commands(commands)?;

        // State will be updated by listener when device confirms
        info!("Set output command sent using V2: {:?}", output);
        Ok(())
    }

    /// Set surround sound
    pub fn set_surround(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_surround_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_surround_slider(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.surround_enabled = enabled;
        settings.surround_value = value;

        info!("Surround set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set crystalizer
    pub fn set_crystalizer(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_crystalizer_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_crystalizer_slider(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.crystalizer_enabled = enabled;
        settings.crystalizer_value = value;

        info!("Crystalizer set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set bass
    /// Now using Protocol V2
    pub fn set_bass(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);

        // Use V2 protocol - cleaner command builders
        let toggle_commands = crate::g6_protocol_v2::build_set_bass_toggle(enabled_bool);
        let value_commands = crate::g6_protocol_v2::build_set_bass_value(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(value_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.bass_enabled = enabled;
        settings.bass_value = value;

        info!("Bass set to {:?} with value {} using V2", enabled, value);
        Ok(())
    }

    /// Set smart volume
    pub fn set_smart_volume(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_smart_volume_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_smart_volume_slider(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.smart_volume_enabled = enabled;
        settings.smart_volume_value = value;

        info!("Smart Volume set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set dialog plus
    pub fn set_dialog_plus(&self, enabled: EffectState, value: u8) -> Result<()> {
        validate_effect_value(value)?;

        let enabled_bool = matches!(enabled, EffectState::Enabled);
        let toggle_commands = g6_protocol::build_dialog_plus_toggle(enabled_bool);
        let slider_commands = g6_protocol::build_dialog_plus_slider(value);

        self.send_commands(toggle_commands)?;
        self.send_commands(slider_commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.dialog_plus_enabled = enabled;
        settings.dialog_plus_value = value;

        info!("Dialog Plus set to {:?} with value {}", enabled, value);
        Ok(())
    }

    /// Set SBX Master Mode
    pub fn set_sbx_mode(&self, enabled: EffectState) -> Result<()> {
        let commands = match enabled {
            EffectState::Enabled => g6_protocol::build_sbx_mode_enable(),
            EffectState::Disabled => g6_protocol::build_sbx_mode_disable(),
        };

        self.send_commands(commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.sbx_enabled = enabled;

        info!("SBX Mode set to {:?}", enabled);
        Ok(())
    }

    /// Set Scout Mode
    pub fn set_scout_mode(&self, enabled: ScoutModeState) -> Result<()> {
        let commands = match enabled {
            ScoutModeState::Enabled => g6_protocol::build_scout_mode_enable(),
            ScoutModeState::Disabled => g6_protocol::build_scout_mode_disable(),
        };

        self.send_commands(commands)?;

        let mut settings = self.current_settings.lock().unwrap();
        settings.scout_mode = enabled;

        info!("Scout Mode set to {:?}", enabled);
        Ok(())
    }

    /// Get current settings
    pub fn get_settings(&self) -> G6Settings {
        self.current_settings.lock().unwrap().clone()
    }

    /// Read current device state from hardware
    pub fn read_device_state(&self) -> Result<G6Settings> {
        if !self.is_connected() {
            return Err(anyhow::anyhow!("Device not connected"));
        }

        info!("Reading complete device state from G6...");

        // Build all read commands
        let commands = g6_protocol::build_read_all_state_commands();

        // Send commands and collect responses
        let responses = self.send_read_commands(commands)?;

        // Parse responses into device settings
        let mut settings = g6_protocol::parse_device_state_responses(&responses)
            .map_err(|e| anyhow::anyhow!("Failed to parse device state: {}", e))?;

        // Preserve SBX Mode state since we can't read it yet
        // This prevents "Read State" from accidentally disabling SBX in the UI
        let current_sbx = self.current_settings.lock().unwrap().sbx_enabled;
        settings.sbx_enabled = current_sbx;

        // Preserve Scout Mode state
        let current_scout = self.current_settings.lock().unwrap().scout_mode;
        settings.scout_mode = current_scout;

        info!(
            "Device state read successfully: {} effects, firmware: {:?}",
            responses.len(),
            settings.firmware_info.as_ref().map(|f| &f.version)
        );

        // Update our internal state
        *self.current_settings.lock().unwrap() = settings.clone();

        Ok(settings)
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

impl Default for G6DeviceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create G6DeviceManager")
    }
}
